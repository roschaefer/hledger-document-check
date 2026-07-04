use chrono::NaiveDate;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::amount_audit::find_amount_mismatches;
use crate::amounts::document_amount_for_document;
use crate::comparison::{load_document_journal_diff, DocumentJournalDiff};
use crate::document_paths::{
    account_dir_for_document, matched_name, metadata_path_for_document, support_dir_for_document,
};
use crate::duplicates::find_duplicate_files;
use crate::journal::{load_transactions, validate_document_duties};
use crate::matching::GroupKey;
use crate::metadata::metadata_for_document;
use crate::model::{
    AmountAuditSkip, AmountMismatch, DocumentEntry, DocumentKind, DuplicateFileGroup, PdfAmount,
    RequiredDocument, TreeIssue, MONEY_TOLERANCE,
};

pub const CHECK_INVALID_CONFIGURATION: &str = "invalid-configuration";
pub const CHECK_MISSING_DOCUMENT_COVERAGE: &str = "missing-document-coverage";
pub const CHECK_UNBOOKED_DOCUMENTS: &str = "unbooked-documents";
pub const CHECK_OVERDUE_UNBOOKED_DOCUMENTS: &str = "overdue-unbooked-documents";
pub const CHECK_UNMATCHED_DOCUMENTS: &str = "unmatched-documents";
pub const CHECK_UNEXPECTED_FILES: &str = "unexpected-files";
pub const CHECK_DUPLICATE_FILES: &str = "duplicate-files";
pub const CHECK_AMOUNT_MISMATCHES: &str = "amount-mismatches";
pub const CHECK_AMOUNT_AUDIT_SKIPS: &str = "amount-audit-skips";
pub const CHECK_MISSING_DOCUMENT_PLACEHOLDERS: &str = "missing-document-placeholders";
pub const CHECK_AMBIGUOUS_TRANSACTION_GROUPS: &str = "ambiguous-transaction-groups";

pub const ALL_CHECKS: &[&str] = &[
    CHECK_INVALID_CONFIGURATION,
    CHECK_MISSING_DOCUMENT_COVERAGE,
    CHECK_UNBOOKED_DOCUMENTS,
    CHECK_OVERDUE_UNBOOKED_DOCUMENTS,
    CHECK_UNMATCHED_DOCUMENTS,
    CHECK_UNEXPECTED_FILES,
    CHECK_DUPLICATE_FILES,
    CHECK_AMOUNT_MISMATCHES,
    CHECK_AMOUNT_AUDIT_SKIPS,
    CHECK_MISSING_DOCUMENT_PLACEHOLDERS,
    CHECK_AMBIGUOUS_TRANSACTION_GROUPS,
];

pub struct CheckArgs {
    pub journal: Option<PathBuf>,
    pub documents: PathBuf,
    pub require_document_for_tag_prefixes: Vec<String>,
    pub fail_on: Vec<String>,
    pub warn_on: Vec<String>,
    pub ignore_checks: Vec<String>,
    pub today: Option<NaiveDate>,
    pub overdue_after_days: u32,
}

fn build_check_policy(args: &CheckArgs) -> Result<HashMap<String, String>, String> {
    let mut policy: HashMap<String, String> = HashMap::new();
    for check in [
        CHECK_INVALID_CONFIGURATION,
        CHECK_MISSING_DOCUMENT_COVERAGE,
        CHECK_OVERDUE_UNBOOKED_DOCUMENTS,
        CHECK_UNMATCHED_DOCUMENTS,
        CHECK_UNEXPECTED_FILES,
        CHECK_DUPLICATE_FILES,
        CHECK_AMOUNT_MISMATCHES,
    ] {
        policy.insert(check.to_string(), "fail".to_string());
    }
    for check in [CHECK_UNBOOKED_DOCUMENTS, CHECK_AMBIGUOUS_TRANSACTION_GROUPS] {
        policy.insert(check.to_string(), "warn".to_string());
    }
    for check in [
        CHECK_AMOUNT_AUDIT_SKIPS,
        CHECK_MISSING_DOCUMENT_PLACEHOLDERS,
    ] {
        policy.insert(check.to_string(), "ignore".to_string());
    }

    for (level, names) in [
        ("fail", &args.fail_on),
        ("warn", &args.warn_on),
        ("ignore", &args.ignore_checks),
    ] {
        for name in names {
            if !ALL_CHECKS.contains(&name.as_str()) {
                return Err(format!("unknown check: {name}"));
            }
            policy.insert(name.clone(), level.to_string());
        }
    }
    Ok(policy)
}

fn should_report(policy: &HashMap<String, String>, check_name: &str) -> bool {
    policy
        .get(check_name)
        .map(|l| l != "ignore")
        .unwrap_or(true)
}

fn is_failure(policy: &HashMap<String, String>, check_name: &str, has_items: bool) -> bool {
    has_items && policy.get(check_name).map(|l| l == "fail").unwrap_or(false)
}

fn display_path(path: &Path) -> String {
    match path.canonicalize() {
        Ok(abs) => {
            if let Ok(cwd) = std::env::current_dir() {
                if let Ok(rel) = abs.strip_prefix(&cwd) {
                    return rel.to_string_lossy().into_owned();
                }
            }
            abs.to_string_lossy().into_owned()
        }
        Err(_) => path.to_string_lossy().into_owned(),
    }
}

fn format_amount(amount: Option<f64>, commodity: Option<&str>) -> String {
    match amount {
        None => "unknown".to_string(),
        Some(a) => match commodity {
            Some(c) => format!("{a:.2} {c}"),
            None => format!("{a:.2}"),
        },
    }
}

fn format_pdf_amount(amount: Option<&PdfAmount>) -> String {
    match amount {
        None => "unknown".to_string(),
        Some(a) => format_amount(Some(a.amount), a.currency.as_deref()),
    }
}

fn format_key(account: &str, date: &NaiveDate) -> String {
    format!("{account} @ {}", date.format("%Y-%m-%d"))
}

struct OverdueUnbookedDocument {
    path: PathBuf,
    due_date: NaiveDate,
    days_overdue: i64,
}

fn find_overdue_unbooked_documents(
    unbooked_entries: &[DocumentEntry],
    today: NaiveDate,
    overdue_after_days: i64,
) -> Vec<OverdueUnbookedDocument> {
    let mut overdue = Vec::new();
    for entry in unbooked_entries {
        let Ok(Some(meta)) = metadata_for_document(&entry.path) else {
            continue;
        };
        let Some(due_date) = meta.due_date else {
            continue;
        };
        let days_overdue = (today - due_date).num_days();
        if days_overdue > overdue_after_days {
            overdue.push(OverdueUnbookedDocument {
                path: entry.path.clone(),
                due_date,
                days_overdue,
            });
        }
    }
    overdue.sort_by(|a, b| {
        a.due_date
            .cmp(&b.due_date)
            .then_with(|| display_path(&a.path).cmp(&display_path(&b.path)))
    });
    overdue
}

struct SuggestedMove {
    source: PathBuf,
    target: PathBuf,
    transaction: RequiredDocument,
    metadata_source: Option<PathBuf>,
    metadata_target: Option<PathBuf>,
    support_source: Option<PathBuf>,
    support_target: Option<PathBuf>,
}

fn find_suggested_moves(
    missing_transactions: &[RequiredDocument],
    unbooked_entries: &[DocumentEntry],
) -> Vec<SuggestedMove> {
    let amount_cache: HashMap<PathBuf, Option<PdfAmount>> = unbooked_entries
        .iter()
        .map(|e| (e.path.clone(), document_amount_for_document(&e.path)))
        .collect();

    let mut candidates_by_transaction: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut candidates_by_entry: HashMap<usize, Vec<usize>> = HashMap::new();

    for (ti, txn) in missing_transactions.iter().enumerate() {
        let (Some(txn_amount), Some(txn_commodity)) = (&txn.amount, &txn.commodity) else {
            continue;
        };
        for (ei, entry) in unbooked_entries.iter().enumerate() {
            if entry.account_path != txn.account_path() {
                continue;
            }
            let Some(doc_amount) = amount_cache.get(&entry.path).and_then(|a| a.as_ref()) else {
                continue;
            };
            if doc_amount.currency.as_deref() != Some(txn_commodity.as_str()) {
                continue;
            }
            if (doc_amount.amount - txn_amount).abs() > MONEY_TOLERANCE {
                continue;
            }
            let target = account_dir_for_document(&entry.path).join(matched_name(
                txn.posting_date,
                entry.path.file_name().unwrap().to_str().unwrap(),
            ));
            if target.exists() {
                continue;
            }
            candidates_by_transaction.entry(ti).or_default().push(ei);
            candidates_by_entry.entry(ei).or_default().push(ti);
        }
    }

    let mut suggestions: Vec<SuggestedMove> = Vec::new();
    for (ti, entries) in &candidates_by_transaction {
        if entries.len() != 1 {
            continue;
        }
        let ei = entries[0];
        if candidates_by_entry.get(&ei).map(|v| v.len()).unwrap_or(0) != 1 {
            continue;
        }
        let entry = &unbooked_entries[ei];
        let txn = &missing_transactions[*ti];
        let file_name = entry.path.file_name().unwrap().to_str().unwrap();
        let target =
            account_dir_for_document(&entry.path).join(matched_name(txn.posting_date, file_name));
        let support_source = support_dir_for_document(&entry.path);
        let support_target = support_dir_for_document(&target);
        let metadata_source = metadata_path_for_document(&entry.path);
        let metadata_target = metadata_path_for_document(&target);

        if metadata_source.exists() && metadata_target.exists() {
            continue;
        }
        if support_source.exists() && support_target.exists() {
            continue;
        }

        let meta_exists = metadata_source.exists();
        let supp_exists = support_source.exists();
        suggestions.push(SuggestedMove {
            source: entry.path.clone(),
            target: target.clone(),
            transaction: txn.clone(),
            metadata_source: if meta_exists {
                Some(metadata_source)
            } else {
                None
            },
            metadata_target: if meta_exists {
                Some(metadata_target)
            } else {
                None
            },
            support_source: if supp_exists {
                Some(support_source)
            } else {
                None
            },
            support_target: if supp_exists {
                Some(support_target)
            } else {
                None
            },
        });
    }

    suggestions.sort_by(|a, b| {
        a.transaction
            .posting_date
            .cmp(&b.transaction.posting_date)
            .then_with(|| display_path(&a.source).cmp(&display_path(&b.source)))
    });
    suggestions
}

fn shell_quote(s: &str) -> String {
    // Mirror Python shlex.quote: only quote if the string contains unsafe characters.
    // Safe chars: word chars, @%+=:,./-
    let needs_quoting = s.is_empty()
        || s.chars().any(|c| !matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '@' | '%' | '+' | '=' | ':' | ',' | '.' | '/' | '-' | '_'));
    if needs_quoting {
        format!("'{}'", s.replace('\'', "'\"'\"'"))
    } else {
        s.to_string()
    }
}

fn command_path(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .into_owned()
}

pub fn run_check(args: CheckArgs) -> i32 {
    let policy = match build_check_policy(&args) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("ERROR: {e}");
            return 2;
        }
    };

    if args.overdue_after_days == 0 {
        // Technically 0 is valid; only reject if negative (but u32 can't be negative)
    }

    let transactions = match load_transactions(args.journal.as_deref()) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("ERROR: {e}");
            return 2;
        }
    };

    let duty_errors = validate_document_duties(&transactions);
    if !duty_errors.is_empty() {
        if should_report(&policy, CHECK_INVALID_CONFIGURATION) {
            println!("ERROR: invalid document_check configuration:\n");
            for error in &duty_errors {
                println!("{error}");
            }
            println!("\nEnsure document_check and any required document_check_counter_account tags are configured correctly.");
        }
        return if is_failure(&policy, CHECK_INVALID_CONFIGURATION, true) {
            2
        } else {
            0
        };
    }

    let today = args
        .today
        .unwrap_or_else(|| chrono::Local::now().date_naive());

    let diff = load_document_journal_diff(
        transactions,
        &args.documents,
        &args.require_document_for_tag_prefixes,
    );

    let required_groups = &diff.journal.required_groups;
    let covered_groups = diff.covered_groups();

    let missing = find_missing_document_coverage(&diff);
    let unmatched_entries: Vec<&DocumentEntry> = diff.unmatched_entries();
    let ambiguous_groups = diff.ambiguous_transaction_groups();

    let duplicate_files = find_duplicate_files(&args.documents);
    let amount_audit = find_amount_mismatches(required_groups, &diff.documents.matched_entries);

    let unbooked_entries = &diff.documents.unbooked_entries;
    let overdue_unbooked =
        find_overdue_unbooked_documents(unbooked_entries, today, args.overdue_after_days as i64);

    let missing_document_count = diff
        .documents
        .matched_entries
        .iter()
        .filter(|e| e.kind == DocumentKind::MissingMetadata)
        .count();

    // Print sections
    if should_report(&policy, CHECK_MISSING_DOCUMENT_COVERAGE) {
        print_missing_document_coverage(&missing);
    }

    let unmatched_amounts: HashMap<PathBuf, Option<PdfAmount>> = unmatched_entries
        .iter()
        .map(|e| (e.path.clone(), document_amount_for_document(&e.path)))
        .collect();

    if should_report(&policy, CHECK_UNBOOKED_DOCUMENTS) {
        let unbooked_amounts: HashMap<PathBuf, Option<PdfAmount>> = unbooked_entries
            .iter()
            .map(|e| (e.path.clone(), document_amount_for_document(&e.path)))
            .collect();
        print_document_entries("Unbooked Documents", unbooked_entries, &unbooked_amounts);
        let suggestions = find_suggested_moves(&diff.missing_transactions(), unbooked_entries);
        print_suggested_moves(&suggestions);
    }

    if should_report(&policy, CHECK_OVERDUE_UNBOOKED_DOCUMENTS) {
        print_overdue_unbooked_documents(&overdue_unbooked, args.overdue_after_days);
    }

    if should_report(&policy, CHECK_UNMATCHED_DOCUMENTS) {
        let unmatched_vec: Vec<&DocumentEntry> = unmatched_entries.clone();
        print_document_entries(
            "Unmatched Documents",
            &unmatched_vec.into_iter().cloned().collect::<Vec<_>>(),
            &unmatched_amounts,
        );
    }

    if should_report(&policy, CHECK_UNEXPECTED_FILES) {
        let issues: Vec<&TreeIssue> = diff.documents.issues.iter().collect();
        print_issues("Unexpected Files", &issues);
    }

    if should_report(&policy, CHECK_DUPLICATE_FILES) {
        print_duplicates(&duplicate_files);
    }

    if should_report(&policy, CHECK_AMOUNT_MISMATCHES) {
        print_amount_mismatches(&amount_audit.mismatches);
    }

    if should_report(&policy, CHECK_AMOUNT_AUDIT_SKIPS) {
        print_amount_audit_skips(&amount_audit.skipped);
    }

    if should_report(&policy, CHECK_AMBIGUOUS_TRANSACTION_GROUPS) {
        print_ambiguous(&ambiguous_groups, required_groups);
    }

    let matched_group_count = required_groups
        .keys()
        .filter(|k| covered_groups.contains(*k))
        .count();
    let total_groups = required_groups.len();

    println!("\nSummary:");
    println!("  Coverage:");
    println!("    {matched_group_count}/{total_groups} transaction groups covered");
    println!("    {missing_document_count} missing-document placeholders");
    println!("  Open Items:");
    println!("    {} missing document coverage", missing.len());
    println!("    {} unbooked documents", unbooked_entries.len());
    println!("    {} overdue unbooked documents", overdue_unbooked.len());
    println!("    {} unmatched documents", unmatched_entries.len());
    println!("    {} unexpected files", diff.documents.issues.len());
    println!("    {} duplicate groups", duplicate_files.len());
    println!("  Amount Audit:");
    println!("    {} amount mismatches", amount_audit.mismatches.len());
    println!("    {} linked groups checked", amount_audit.checked_groups);
    println!(
        "    {} linked groups skipped",
        amount_audit.skipped_groups()
    );
    println!("    Skip Reasons:");
    println!(
        "      {} document amount unreadable",
        amount_audit.skipped_missing_document_amount_groups
    );
    println!(
        "      {} document currencies unclear",
        amount_audit.skipped_mixed_document_currency_groups
    );
    println!(
        "      {} transaction amount missing",
        amount_audit.skipped_missing_transaction_amount_groups
    );
    println!(
        "      {} transaction currencies mixed",
        amount_audit.skipped_mixed_transaction_currency_groups
    );
    println!(
        "      {} document/transaction currency mismatch",
        amount_audit.skipped_currency_mismatch_groups
    );

    let has_failure = [
        is_failure(
            &policy,
            CHECK_MISSING_DOCUMENT_COVERAGE,
            !missing.is_empty(),
        ),
        is_failure(
            &policy,
            CHECK_UNBOOKED_DOCUMENTS,
            !unbooked_entries.is_empty(),
        ),
        is_failure(
            &policy,
            CHECK_OVERDUE_UNBOOKED_DOCUMENTS,
            !overdue_unbooked.is_empty(),
        ),
        is_failure(
            &policy,
            CHECK_UNMATCHED_DOCUMENTS,
            !unmatched_entries.is_empty(),
        ),
        is_failure(
            &policy,
            CHECK_UNEXPECTED_FILES,
            !diff.documents.issues.is_empty(),
        ),
        is_failure(&policy, CHECK_DUPLICATE_FILES, !duplicate_files.is_empty()),
        is_failure(
            &policy,
            CHECK_AMOUNT_MISMATCHES,
            !amount_audit.mismatches.is_empty(),
        ),
        is_failure(
            &policy,
            CHECK_AMOUNT_AUDIT_SKIPS,
            !amount_audit.skipped.is_empty(),
        ),
        is_failure(
            &policy,
            CHECK_MISSING_DOCUMENT_PLACEHOLDERS,
            missing_document_count > 0,
        ),
        is_failure(
            &policy,
            CHECK_AMBIGUOUS_TRANSACTION_GROUPS,
            !ambiguous_groups.is_empty(),
        ),
    ]
    .iter()
    .any(|&f| f);

    if !has_failure {
        println!("OK.");
    }

    if has_failure {
        1
    } else {
        0
    }
}

// --- Print helpers ---

struct MissingDocument {
    transaction_date: String,
    description: String,
    account: String,
    amount: Option<f64>,
    commodity: Option<String>,
}

fn find_missing_document_coverage(diff: &DocumentJournalDiff) -> Vec<MissingDocument> {
    diff.missing_transactions()
        .into_iter()
        .map(|txn| MissingDocument {
            transaction_date: txn.transaction_date.clone(),
            description: txn.description.clone(),
            account: txn.account.clone(),
            amount: txn.amount,
            commodity: txn.commodity.clone(),
        })
        .collect()
}

fn print_missing_document_coverage(missing: &[MissingDocument]) {
    if missing.is_empty() {
        return;
    }
    println!("\nMissing Document Coverage ({}):", missing.len());
    for item in missing {
        println!("  {}  {}", item.transaction_date, item.description);
        println!("    account: {}", item.account);
        println!(
            "    amount: {}",
            format_amount(item.amount, item.commodity.as_deref())
        );
    }
}

fn print_document_entries(
    header: &str,
    entries: &[DocumentEntry],
    amount_by_path: &HashMap<PathBuf, Option<PdfAmount>>,
) {
    if entries.is_empty() {
        return;
    }
    println!("\n{header} ({}):", entries.len());
    for entry in entries {
        println!("  {}", display_path(&entry.path));
        println!(
            "    amount: {}",
            format_pdf_amount(amount_by_path.get(&entry.path).and_then(|a| a.as_ref()))
        );
    }
}

fn print_overdue_unbooked_documents(items: &[OverdueUnbookedDocument], overdue_after_days: u32) {
    if items.is_empty() {
        return;
    }
    println!("\nOverdue Unbooked Documents ({}):", items.len());
    println!("  Unbooked documents whose due_date is more than {overdue_after_days} days ago.");
    for item in items {
        println!("  {}", display_path(&item.path));
        println!("    due_date: {}", item.due_date.format("%Y-%m-%d"));
        println!("    days overdue: {}", item.days_overdue);
    }
}

fn print_issues(header: &str, items: &[&TreeIssue]) {
    if items.is_empty() {
        return;
    }
    println!("\n{header} ({}):", items.len());
    for issue in items {
        println!("  {}  [{}]", display_path(&issue.path), issue.reason);
    }
}

fn print_duplicates(groups: &[DuplicateFileGroup]) {
    if groups.is_empty() {
        return;
    }
    println!("\nExact Duplicate Files ({}):", groups.len());
    for group in groups {
        for path in &group.paths {
            println!("  {}", display_path(path));
        }
        println!("    exact file equality");
    }
}

fn print_amount_mismatches(items: &[AmountMismatch]) {
    if items.is_empty() {
        return;
    }
    println!("\nAmount Mismatches ({}):", items.len());
    for item in items {
        let accounts: Vec<String> = {
            let mut a: Vec<String> = item
                .transaction_keys
                .iter()
                .map(|(acc, _)| acc.clone())
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();
            a.sort();
            a
        };
        let dates: Vec<String> = {
            let mut d: Vec<String> = item
                .transaction_keys
                .iter()
                .map(|(_, date)| date.format("%Y-%m-%d").to_string())
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();
            d.sort();
            d
        };
        println!(
            "  {} {:.2} != {} {:.2}",
            item.document_label,
            item.document_total,
            item.transaction_label,
            item.transaction_total
        );
        println!(
            "    accounts/dates: {} @ {}",
            accounts.join(", "),
            dates.join(", ")
        );
        for path in &item.document_paths {
            println!("    {}", display_path(path));
        }
    }
}

fn print_suggested_moves(suggestions: &[SuggestedMove]) {
    if suggestions.is_empty() {
        return;
    }
    println!("\nSuggested Moves ({}):", suggestions.len());
    println!("  High-confidence unbooked document matches by account, amount, and currency.");
    for s in suggestions {
        println!(
            "  {}  {}",
            s.transaction.transaction_date, s.transaction.description
        );
        println!("    account: {}", s.transaction.account);
        println!(
            "    amount: {}",
            format_amount(s.transaction.amount, s.transaction.commodity.as_deref())
        );
        println!(
            "    mv {} {}",
            shell_quote(&command_path(&s.source)),
            shell_quote(&command_path(&s.target))
        );
        if let (Some(ms), Some(mt)) = (&s.metadata_source, &s.metadata_target) {
            println!(
                "    mv {} {}",
                shell_quote(&command_path(ms)),
                shell_quote(&command_path(mt))
            );
        }
        if let (Some(ss), Some(st)) = (&s.support_source, &s.support_target) {
            println!(
                "    mv {} {}",
                shell_quote(&command_path(ss)),
                shell_quote(&command_path(st))
            );
        }
    }
}

fn print_ambiguous(
    ambiguous_groups: &HashMap<GroupKey, usize>,
    required_groups: &HashMap<GroupKey, Vec<RequiredDocument>>,
) {
    if ambiguous_groups.is_empty() {
        return;
    }
    println!(
        "\nAmbiguous Transaction Groups ({}):",
        ambiguous_groups.len()
    );
    println!("  Date-prefixed filenames can only match by account and date.");
    println!(
        "  These groups have multiple required transactions on the same date in the same account:"
    );
    let mut sorted: Vec<(&GroupKey, &usize)> = ambiguous_groups.iter().collect();
    sorted.sort_by_key(|(k, _)| *k);
    for ((account, date), count) in sorted {
        println!("  {}  [{count} transactions]", format_key(account, date));
        if let Some(txns) = required_groups.get(&(account.clone(), *date)) {
            for txn in txns {
                println!(
                    "    {}  {}  {}",
                    txn.transaction_date,
                    format_amount(txn.amount, txn.commodity.as_deref()),
                    txn.description
                );
            }
        }
    }
}

const SKIP_REASON_LABELS: &[(&str, &str)] = &[
    ("document_amount_unreadable", "document amount unreadable"),
    ("document_currencies_unclear", "document currencies unclear"),
    ("transaction_amount_missing", "transaction amount missing"),
    (
        "transaction_currencies_mixed",
        "transaction currencies mixed",
    ),
    (
        "currency_mismatch",
        "document/transaction currency mismatch",
    ),
];

fn print_amount_audit_skips(items: &[AmountAuditSkip]) {
    if items.is_empty() {
        return;
    }
    println!("\nAmount Audit Skips ({}):", items.len());
    let mut grouped: HashMap<&str, Vec<&AmountAuditSkip>> = HashMap::new();
    for item in items {
        grouped.entry(item.reason.as_str()).or_default().push(item);
    }
    let mut reasons: Vec<&str> = grouped.keys().copied().collect();
    reasons.sort();
    for reason in reasons {
        let label = SKIP_REASON_LABELS
            .iter()
            .find(|(k, _)| *k == reason)
            .map(|(_, v)| *v)
            .unwrap_or(reason);
        println!("  {label}");
        for item in &grouped[reason] {
            for path in &item.document_paths {
                println!("    {}", display_path(path));
            }
            let accounts: Vec<String> = {
                let mut a: Vec<String> = item
                    .transaction_keys
                    .iter()
                    .map(|(acc, _)| acc.clone())
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect();
                a.sort();
                a
            };
            let dates: Vec<String> = {
                let mut d: Vec<String> = item
                    .transaction_keys
                    .iter()
                    .map(|(_, date)| date.format("%Y-%m-%d").to_string())
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect();
                d.sort();
                d
            };
            println!(
                "      accounts/dates: {} @ {}",
                accounts.join(", "),
                dates.join(", ")
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn nd(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn make_required(account: &str, date: NaiveDate, amount: f64) -> RequiredDocument {
        RequiredDocument {
            transaction_date: "2026-01-02".to_string(),
            description: "Example transaction".to_string(),
            comment: "".to_string(),
            account: account.to_string(),
            posting_date: date,
            amount: Some(amount),
            commodity: Some("EUR".to_string()),
            transaction_index: 1,
        }
    }

    fn make_unbooked(path: PathBuf, account: &str) -> DocumentEntry {
        DocumentEntry {
            path,
            account_path: account.split(':').collect(),
            match_date: None,
            rest_name: None,
            kind: DocumentKind::UnbookedPdf,
        }
    }

    // --- find_overdue_unbooked_documents ---

    #[test]
    fn finds_overdue_unbooked_invoice_with_due_date() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp
            .path()
            .join("income/business/freelance/customer/unbooked");
        std::fs::create_dir_all(&dir).unwrap();
        let unbooked = dir.join("invoice.pdf");
        std::fs::write(&unbooked, b"pdf").unwrap();
        std::fs::write(dir.join("invoice.document.yml"), "due_date: 2026-04-15\n").unwrap();

        let entry = make_unbooked(unbooked.clone(), "income:business:freelance:customer");
        let overdue = find_overdue_unbooked_documents(&[entry], nd(2026, 5, 1), 14);

        assert_eq!(overdue.len(), 1);
        assert_eq!(overdue[0].path, unbooked);
        assert_eq!(overdue[0].due_date, nd(2026, 4, 15));
        assert_eq!(overdue[0].days_overdue, 16);
    }

    #[test]
    fn ignores_unbooked_documents_without_due_date() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp
            .path()
            .join("income/business/freelance/customer/unbooked");
        std::fs::create_dir_all(&dir).unwrap();
        let unbooked = dir.join("invoice.pdf");
        std::fs::write(&unbooked, b"pdf").unwrap();

        let entry = make_unbooked(unbooked, "income:business:freelance:customer");
        let overdue = find_overdue_unbooked_documents(&[entry], nd(2026, 5, 1), 14);

        assert!(overdue.is_empty());
    }

    // --- find_suggested_moves ---

    #[test]
    fn suggests_unique_unbooked_document_with_matching_amount() {
        let tmp = TempDir::new().unwrap();
        let account = "expenses:business:hosting:aws";
        let dir = tmp.path().join("expenses/business/hosting/aws");
        let unbooked_dir = dir.join("unbooked");
        std::fs::create_dir_all(&unbooked_dir).unwrap();
        let unbooked = unbooked_dir.join("invoice.pdf");
        std::fs::write(&unbooked, b"pdf").unwrap();
        std::fs::write(
            unbooked_dir.join("invoice.document.yml"),
            "due_date: 2026-01-31\namount: 12.34\ncurrency: EUR\n",
        )
        .unwrap();
        let support_dir = unbooked_dir.join("invoice");
        std::fs::create_dir_all(&support_dir).unwrap();
        std::fs::write(support_dir.join("usage.csv"), b"line,item\n").unwrap();

        let entry = make_unbooked(unbooked.clone(), account);
        let txn = make_required(account, nd(2026, 1, 2), 12.34);
        let suggestions = find_suggested_moves(&[txn], &[entry]);

        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].source, unbooked);
        assert_eq!(suggestions[0].target, dir.join("2026-01-02-invoice.pdf"));
        assert_eq!(
            suggestions[0].metadata_source,
            Some(unbooked_dir.join("invoice.document.yml"))
        );
        assert_eq!(
            suggestions[0].metadata_target,
            Some(dir.join("2026-01-02-invoice.document.yml"))
        );
        assert_eq!(suggestions[0].support_source, Some(support_dir.clone()));
        assert_eq!(
            suggestions[0].support_target,
            Some(dir.join("2026-01-02-invoice"))
        );
    }

    #[test]
    fn suggests_move_using_metadata_cover_amount() {
        let tmp = TempDir::new().unwrap();
        let account = "expenses:business:transport:train:flixtrain";
        let dir = tmp
            .path()
            .join("expenses/business/transport/train/flixtrain");
        let unbooked_dir = dir.join("unbooked");
        std::fs::create_dir_all(&unbooked_dir).unwrap();
        let unbooked = unbooked_dir.join("booking.pdf");
        std::fs::write(&unbooked, b"pdf").unwrap();
        std::fs::write(
            unbooked_dir.join("booking.document.yml"),
            "covers:\n  - amount: 14.99\n    currency: EUR\n",
        )
        .unwrap();

        let entry = make_unbooked(unbooked.clone(), account);
        let txn = make_required(account, nd(2026, 6, 10), 14.99);
        let suggestions = find_suggested_moves(&[txn], &[entry]);

        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].source, unbooked);
        assert_eq!(suggestions[0].target, dir.join("2026-06-10-booking.pdf"));
    }

    #[test]
    fn does_not_suggest_when_multiple_unbooked_documents_match() {
        let tmp = TempDir::new().unwrap();
        let account = "expenses:business:hosting:aws";
        let unbooked_dir = tmp.path().join("expenses/business/hosting/aws/unbooked");
        std::fs::create_dir_all(&unbooked_dir).unwrap();
        let first = unbooked_dir.join("invoice-a.pdf");
        let second = unbooked_dir.join("invoice-b.pdf");
        std::fs::write(&first, b"pdf-a").unwrap();
        std::fs::write(&second, b"pdf-b").unwrap();
        std::fs::write(
            unbooked_dir.join("invoice-a.document.yml"),
            "amount: 12.34\ncurrency: EUR\n",
        )
        .unwrap();
        std::fs::write(
            unbooked_dir.join("invoice-b.document.yml"),
            "amount: 12.34\ncurrency: EUR\n",
        )
        .unwrap();

        let entries = vec![
            make_unbooked(first, account),
            make_unbooked(second, account),
        ];
        let txn = make_required(account, nd(2026, 1, 2), 12.34);
        let suggestions = find_suggested_moves(&[txn], &entries);

        assert!(suggestions.is_empty());
    }
}
