use chrono::NaiveDate;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::amounts::document_amount_for_document;
use crate::matching::matched_groups_for_entry;
use crate::metadata::metadata_for_document;
use crate::model::{
    AmountAudit, AmountAuditSkip, AmountMismatch, DocumentEntry, DocumentKind, RequiredDocument,
    MONEY_TOLERANCE,
};

type GroupKey = (String, NaiveDate);
/// metadata_amounts[(file, group)] = Vec<(amount, currency)>
type MetadataAmounts = HashMap<(PathBuf, GroupKey), Vec<(f64, Option<String>)>>;

pub fn find_amount_mismatches(
    required_groups: &HashMap<GroupKey, Vec<RequiredDocument>>,
    matched_entries: &[DocumentEntry],
) -> AmountAudit {
    let mut file_to_groups: HashMap<PathBuf, HashSet<GroupKey>> = HashMap::new();
    let mut group_to_files: HashMap<GroupKey, HashSet<PathBuf>> = HashMap::new();
    let mut metadata_amounts: MetadataAmounts = HashMap::new();
    // metadata_document_totals[file] = Set<(amount, currency)>
    let mut metadata_document_totals: HashMap<PathBuf, HashSet<(u64, Option<String>)>> =
        HashMap::new(); // store f64 as bits
    let mut file_kinds: HashMap<PathBuf, HashSet<String>> = HashMap::new();

    for entry in matched_entries {
        let metadata = metadata_for_document(&entry.path).unwrap_or_default();

        if entry.kind != DocumentKind::MatchedPdf && metadata.is_none() {
            continue;
        }

        let matching_group_keys = matched_groups_for_entry(entry, required_groups);
        if matching_group_keys.is_empty() {
            continue;
        }

        let real_path = entry
            .path
            .canonicalize()
            .unwrap_or_else(|_| entry.path.clone());
        file_kinds
            .entry(real_path.clone())
            .or_default()
            .insert(entry.kind.as_str().to_string());

        for group_key in &matching_group_keys {
            file_to_groups
                .entry(real_path.clone())
                .or_default()
                .insert(group_key.clone());
            group_to_files
                .entry(group_key.clone())
                .or_default()
                .insert(real_path.clone());
        }

        if let Some(ref meta) = metadata {
            let entry_account = entry.account_path.to_string_lossy().into_owned();
            for cover in &meta.covers {
                let Some(posting_date) = cover.posting_date else {
                    continue;
                };
                let account = cover
                    .account_path
                    .clone()
                    .unwrap_or_else(|| entry_account.clone());
                let group_key = (account, posting_date);
                if matching_group_keys.contains(&group_key) {
                    if let Some(cover_amount) = cover.amount {
                        metadata_amounts
                            .entry((real_path.clone(), group_key.clone()))
                            .or_default()
                            .push((cover_amount, cover.currency.clone()));
                    }
                    if let Some(doc_total) = meta.amount {
                        metadata_document_totals
                            .entry(real_path.clone())
                            .or_default()
                            .insert((doc_total.to_bits(), meta.currency.clone()));
                    }
                }
            }
        }
    }

    let mut visited_files: HashSet<PathBuf> = HashSet::new();
    let mut visited_groups: HashSet<GroupKey> = HashSet::new();
    let mut mismatches: Vec<AmountMismatch> = Vec::new();
    let mut skipped: Vec<AmountAuditSkip> = Vec::new();
    let mut checked_groups = 0usize;
    let mut skipped_missing_document_amount_groups = 0usize;
    let mut skipped_mixed_document_currency_groups = 0usize;
    let mut skipped_missing_transaction_amount_groups = 0usize;
    let mut skipped_mixed_transaction_currency_groups = 0usize;
    let mut skipped_currency_mismatch_groups = 0usize;

    let mut sorted_files: Vec<PathBuf> = file_to_groups.keys().cloned().collect();
    sorted_files.sort();

    for start_file in sorted_files {
        if visited_files.contains(&start_file) {
            continue;
        }

        // BFS/DFS to find connected component
        let mut component_files: HashSet<PathBuf> = HashSet::new();
        let mut component_groups: HashSet<GroupKey> = HashSet::new();

        enum Node {
            File(PathBuf),
            Group(GroupKey),
        }
        let mut stack: Vec<Node> = vec![Node::File(start_file.clone())];

        while let Some(node) = stack.pop() {
            match node {
                Node::File(fp) => {
                    if visited_files.contains(&fp) {
                        continue;
                    }
                    visited_files.insert(fp.clone());
                    component_files.insert(fp.clone());
                    if let Some(groups) = file_to_groups.get(&fp) {
                        for gk in groups {
                            if !visited_groups.contains(gk) {
                                stack.push(Node::Group(gk.clone()));
                            }
                        }
                    }
                }
                Node::Group(gk) => {
                    if visited_groups.contains(&gk) {
                        continue;
                    }
                    visited_groups.insert(gk.clone());
                    component_groups.insert(gk.clone());
                    if let Some(files) = group_to_files.get(&gk) {
                        for fp in files {
                            if !visited_files.contains(fp) {
                                stack.push(Node::File(fp.clone()));
                            }
                        }
                    }
                }
            }
        }

        if component_files.is_empty() || component_groups.is_empty() {
            continue;
        }

        let mut sorted_comp_files: Vec<PathBuf> = component_files.iter().cloned().collect();
        sorted_comp_files.sort();
        let mut sorted_comp_groups: Vec<GroupKey> = component_groups.iter().cloned().collect();
        sorted_comp_groups.sort();

        // Collect metadata amounts for this component
        let metadata_component_amounts: Vec<(f64, Option<String>)> = sorted_comp_files
            .iter()
            .flat_map(|fp| {
                sorted_comp_groups.iter().flat_map(|gk| {
                    metadata_amounts
                        .get(&(fp.clone(), gk.clone()))
                        .cloned()
                        .unwrap_or_default()
                        .into_iter()
                })
            })
            .collect();

        let metadata_group_keys: HashSet<GroupKey> = component_files
            .iter()
            .flat_map(|fp| {
                component_groups
                    .iter()
                    .filter(|gk| metadata_amounts.contains_key(&(fp.clone(), (*gk).clone())))
            })
            .cloned()
            .collect();

        let using_metadata_amounts =
            !metadata_component_amounts.is_empty() && metadata_group_keys == component_groups;

        let (document_amounts, pdf_currencies): (Vec<f64>, HashSet<String>) =
            if using_metadata_amounts {
                let amounts: Vec<f64> =
                    metadata_component_amounts.iter().map(|(a, _)| *a).collect();
                let currencies: HashSet<String> = metadata_component_amounts
                    .iter()
                    .filter_map(|(_, c)| c.clone())
                    .collect();
                (amounts, currencies)
            } else {
                let real_doc_files: Vec<PathBuf> = sorted_comp_files
                    .iter()
                    .filter(|fp| {
                        file_kinds
                            .get(*fp)
                            .map(|kinds| kinds.contains("matched_pdf"))
                            .unwrap_or(false)
                    })
                    .cloned()
                    .collect();

                if real_doc_files.is_empty() {
                    continue;
                }

                let mut amounts = Vec::new();
                let mut currencies = HashSet::new();
                let mut missing_pdf_amount = false;

                for fp in &real_doc_files {
                    match document_amount_for_document(fp) {
                        Some(pa) if pa.currency.is_some() => {
                            amounts.push(pa.amount);
                            currencies.insert(pa.currency.unwrap());
                        }
                        _ => {
                            missing_pdf_amount = true;
                            break;
                        }
                    }
                }

                if missing_pdf_amount || currencies.len() != 1 {
                    if missing_pdf_amount {
                        skipped_missing_document_amount_groups += 1;
                        skipped.push(AmountAuditSkip {
                            reason: "document_amount_unreadable".to_string(),
                            document_paths: sorted_comp_files.clone(),
                            transaction_keys: sorted_comp_groups.clone(),
                        });
                    } else {
                        skipped_mixed_document_currency_groups += 1;
                        skipped.push(AmountAuditSkip {
                            reason: "document_currencies_unclear".to_string(),
                            document_paths: sorted_comp_files.clone(),
                            transaction_keys: sorted_comp_groups.clone(),
                        });
                    }
                    continue;
                }

                (amounts, currencies)
            };

        if pdf_currencies.len() != 1 {
            skipped_mixed_document_currency_groups += 1;
            skipped.push(AmountAuditSkip {
                reason: "document_currencies_unclear".to_string(),
                document_paths: sorted_comp_files.clone(),
                transaction_keys: sorted_comp_groups.clone(),
            });
            continue;
        }

        // Check document total vs allocated amounts (when metadata has explicit totals)
        for fp in &sorted_comp_files {
            let Some(doc_totals) = metadata_document_totals.get(fp) else {
                continue;
            };
            let total_currencies: HashSet<&Option<String>> =
                doc_totals.iter().map(|(_, c)| c).collect();
            if doc_totals.len() != 1 || total_currencies.len() > 1 {
                skipped_mixed_document_currency_groups += 1;
                skipped.push(AmountAuditSkip {
                    reason: "document_currencies_unclear".to_string(),
                    document_paths: vec![fp.clone()],
                    transaction_keys: sorted_comp_groups.clone(),
                });
                continue;
            }
            let (declared_total_bits, _) = doc_totals.iter().next().unwrap();
            let declared_total = f64::from_bits(*declared_total_bits);
            let allocated_total: f64 = sorted_comp_groups
                .iter()
                .flat_map(|gk| {
                    metadata_amounts
                        .get(&(fp.clone(), gk.clone()))
                        .into_iter()
                        .flatten()
                        .map(|(a, _)| *a)
                })
                .sum();

            if (allocated_total - declared_total).abs() > MONEY_TOLERANCE {
                mismatches.push(AmountMismatch {
                    document_paths: vec![fp.clone()],
                    transaction_keys: sorted_comp_groups.clone(),
                    document_total: allocated_total,
                    transaction_total: declared_total,
                    document_label: "allocated document amounts".to_string(),
                    transaction_label: "declared document total".to_string(),
                });
            }
        }

        // Check document amounts vs transaction amounts
        let mut transaction_total = 0.0f64;
        let mut transaction_currencies: HashSet<String> = HashSet::new();
        let mut missing_transaction_amount = false;

        'outer: for gk in &sorted_comp_groups {
            if let Some(txns) = required_groups.get(gk) {
                for txn in txns {
                    match (&txn.amount, &txn.commodity) {
                        (Some(a), Some(c)) => {
                            transaction_total += a;
                            transaction_currencies.insert(c.clone());
                        }
                        _ => {
                            missing_transaction_amount = true;
                            break 'outer;
                        }
                    }
                }
            }
        }

        if missing_transaction_amount || transaction_currencies.len() != 1 {
            if missing_transaction_amount {
                skipped_missing_transaction_amount_groups += 1;
                skipped.push(AmountAuditSkip {
                    reason: "transaction_amount_missing".to_string(),
                    document_paths: sorted_comp_files.clone(),
                    transaction_keys: sorted_comp_groups.clone(),
                });
            } else {
                skipped_mixed_transaction_currency_groups += 1;
                skipped.push(AmountAuditSkip {
                    reason: "transaction_currencies_mixed".to_string(),
                    document_paths: sorted_comp_files.clone(),
                    transaction_keys: sorted_comp_groups.clone(),
                });
            }
            continue;
        }

        let transaction_currency = transaction_currencies.iter().next().unwrap();
        let pdf_currency = pdf_currencies.iter().next().unwrap();

        if transaction_currency != pdf_currency {
            skipped_currency_mismatch_groups += 1;
            skipped.push(AmountAuditSkip {
                reason: "currency_mismatch".to_string(),
                document_paths: sorted_comp_files.clone(),
                transaction_keys: sorted_comp_groups.clone(),
            });
            continue;
        }

        checked_groups += 1;
        let document_total: f64 = document_amounts.iter().sum();
        if (document_total - transaction_total).abs() > MONEY_TOLERANCE {
            mismatches.push(AmountMismatch::new(
                sorted_comp_files.clone(),
                sorted_comp_groups.clone(),
                document_total,
                transaction_total,
            ));
        }
    }

    AmountAudit {
        mismatches,
        skipped,
        checked_groups,
        skipped_missing_document_amount_groups,
        skipped_mixed_document_currency_groups,
        skipped_missing_transaction_amount_groups,
        skipped_mixed_transaction_currency_groups,
        skipped_currency_mismatch_groups,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    const ACCOUNT: &str = "expenses:business:hosting:aws";

    fn nd(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn make_required(account: &str, date: NaiveDate, amount: f64) -> RequiredDocument {
        RequiredDocument {
            transaction_date: "2026-01-01".to_string(),
            description: "Example transaction".to_string(),
            comment: "".to_string(),
            account: account.to_string(),
            posting_date: date,
            amount: Some(amount),
            commodity: Some("EUR".to_string()),
            transaction_index: 1,
        }
    }

    fn make_entry(path: PathBuf, account: &str, match_date: NaiveDate) -> DocumentEntry {
        DocumentEntry {
            path,
            account_path: account.split(':').collect(),
            match_date: Some(match_date),
            rest_name: None,
            kind: DocumentKind::MatchedPdf,
        }
    }

    fn group_key(account_path: &str, date: NaiveDate) -> GroupKey {
        (account_path.to_string(), date)
    }

    #[test]
    fn one_invoice_covers_multiple_transactions() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/business/hosting/aws");
        std::fs::create_dir_all(&dir).unwrap();
        let invoice = dir.join("2026-01-01-invoice57.pdf");
        std::fs::write(&invoice, b"pdf").unwrap();
        std::fs::write(dir.join("2026-01-01-invoice57.document.yml"),
            "covers:\n  - date: 2026-01-01\n    account: expenses:business:hosting:aws\n    amount: 60.00\n    currency: EUR\n  - date: 2026-02-01\n    account: expenses:business:hosting:aws\n    amount: 40.00\n    currency: EUR\n").unwrap();

        let gk1 = group_key("expenses/business/hosting/aws", nd(2026, 1, 1));
        let gk2 = group_key("expenses/business/hosting/aws", nd(2026, 2, 1));
        let required_groups = HashMap::from([
            (
                gk1.clone(),
                vec![make_required(ACCOUNT, nd(2026, 1, 1), 60.0)],
            ),
            (
                gk2.clone(),
                vec![make_required(ACCOUNT, nd(2026, 2, 1), 40.0)],
            ),
        ]);
        let entries = vec![make_entry(invoice, ACCOUNT, nd(2026, 1, 1))];
        let audit = find_amount_mismatches(&required_groups, &entries);

        assert!(audit.mismatches.is_empty());
        assert_eq!(audit.checked_groups, 1);
        assert_eq!(audit.skipped_groups(), 0);
    }

    #[test]
    fn document_level_metadata_amount_is_used_without_pdf_read() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/business/hosting/aws");
        std::fs::create_dir_all(&dir).unwrap();
        let invoice = dir.join("2026-01-01-invoice57.pdf");
        std::fs::write(&invoice, b"pdf").unwrap();
        std::fs::write(
            dir.join("2026-01-01-invoice57.document.yml"),
            "amount: 100.00\ncurrency: EUR\n",
        )
        .unwrap();

        let gk = group_key("expenses/business/hosting/aws", nd(2026, 1, 1));
        let required_groups =
            HashMap::from([(gk, vec![make_required(ACCOUNT, nd(2026, 1, 1), 100.0)])]);
        let entries = vec![make_entry(invoice, ACCOUNT, nd(2026, 1, 1))];
        let audit = find_amount_mismatches(&required_groups, &entries);

        assert!(audit.mismatches.is_empty());
        assert_eq!(audit.checked_groups, 1);
    }

    #[test]
    fn multiple_files_can_cover_one_transaction() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("income/business/freelance/customer");
        std::fs::create_dir_all(&dir).unwrap();
        let invoice_a = dir.join("2026-01-15-invoice57.pdf");
        let invoice_b = dir.join("2026-01-15-invoice58.pdf");
        std::fs::write(&invoice_a, b"pdf-a").unwrap();
        std::fs::write(&invoice_b, b"pdf-b").unwrap();
        std::fs::write(
            dir.join("2026-01-15-invoice57.document.yml"),
            "amount: 30.00\ncurrency: EUR\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("2026-01-15-invoice58.document.yml"),
            "amount: 70.00\ncurrency: EUR\n",
        )
        .unwrap();

        let gk = group_key("income/business/freelance/customer", nd(2026, 1, 15));
        let required_groups = HashMap::from([(
            gk,
            vec![make_required(
                "income:business:freelance:customer",
                nd(2026, 1, 15),
                100.0,
            )],
        )]);
        let entries = vec![
            make_entry(
                invoice_a,
                "income:business:freelance:customer",
                nd(2026, 1, 15),
            ),
            make_entry(
                invoice_b,
                "income:business:freelance:customer",
                nd(2026, 1, 15),
            ),
        ];
        let audit = find_amount_mismatches(&required_groups, &entries);

        assert!(audit.mismatches.is_empty());
        assert_eq!(audit.checked_groups, 1);
    }

    #[test]
    fn missing_document_placeholder_without_amount_is_not_audited() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/business/hosting/aws");
        std::fs::create_dir_all(&dir).unwrap();
        let placeholder = dir.join("2026-01-01-missing.document.yml");
        std::fs::write(
            &placeholder,
            "covers:\n  - date: 2026-01-01\n    account: expenses:business:hosting:aws\n",
        )
        .unwrap();

        let gk = group_key("expenses/business/hosting/aws", nd(2026, 1, 1));
        let required_groups =
            HashMap::from([(gk, vec![make_required(ACCOUNT, nd(2026, 1, 1), 100.0)])]);
        let entries = vec![DocumentEntry {
            path: placeholder,
            account_path: ACCOUNT.split(':').collect(),
            match_date: Some(nd(2026, 1, 1)),
            rest_name: None,
            kind: DocumentKind::MissingMetadata,
        }];
        let audit = find_amount_mismatches(&required_groups, &entries);

        assert!(audit.mismatches.is_empty());
        assert_eq!(audit.checked_groups, 0);
        assert_eq!(audit.skipped_groups(), 0);
    }

    #[test]
    fn reports_amount_mismatch_for_connected_component() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/business/hosting/aws");
        std::fs::create_dir_all(&dir).unwrap();
        let invoice = dir.join("2026-01-01-invoice57.pdf");
        std::fs::write(&invoice, b"pdf").unwrap();
        std::fs::write(dir.join("2026-01-01-invoice57.document.yml"),
            "covers:\n  - date: 2026-01-01\n    account: expenses:business:hosting:aws\n    amount: 100.00\n    currency: EUR\n  - date: 2026-02-01\n    account: expenses:business:hosting:aws\n    amount: 0.00\n    currency: EUR\n").unwrap();

        let required_groups = HashMap::from([
            (
                group_key("expenses/business/hosting/aws", nd(2026, 1, 1)),
                vec![make_required(ACCOUNT, nd(2026, 1, 1), 30.0)],
            ),
            (
                group_key("expenses/business/hosting/aws", nd(2026, 2, 1)),
                vec![make_required(ACCOUNT, nd(2026, 2, 1), 40.0)],
            ),
        ]);
        let entries = vec![make_entry(invoice, ACCOUNT, nd(2026, 1, 1))];
        let audit = find_amount_mismatches(&required_groups, &entries);

        assert_eq!(audit.mismatches.len(), 1);
        assert!((audit.mismatches[0].document_total - 100.0).abs() < 0.001);
        assert!((audit.mismatches[0].transaction_total - 70.0).abs() < 0.001);
    }

    #[test]
    fn zero_document_amount_from_metadata_is_a_mismatch_not_unreadable() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/business/education/vuemastery");
        std::fs::create_dir_all(&dir).unwrap();
        let invoice = dir.join("2026-01-01-invoice57.pdf");
        std::fs::write(&invoice, b"pdf").unwrap();
        std::fs::write(
            dir.join("2026-01-01-invoice57.document.yml"),
            "amount: 0.00\ncurrency: USD\n",
        )
        .unwrap();

        let required_groups = HashMap::from([(
            group_key("expenses/business/education/vuemastery", nd(2026, 1, 1)),
            vec![RequiredDocument {
                transaction_date: "2026-01-01".to_string(),
                description: "Example transaction".to_string(),
                comment: "".to_string(),
                account: "expenses:business:education:vuemastery".to_string(),
                posting_date: nd(2026, 1, 1),
                amount: Some(19.0),
                commodity: Some("USD".to_string()),
                transaction_index: 1,
            }],
        )]);
        let entries = vec![make_entry(
            invoice,
            "expenses:business:education:vuemastery",
            nd(2026, 1, 1),
        )];
        let audit = find_amount_mismatches(&required_groups, &entries);

        assert_eq!(audit.checked_groups, 1);
        assert_eq!(audit.skipped_groups(), 0);
        assert_eq!(audit.mismatches.len(), 1);
        assert!((audit.mismatches[0].document_total - 0.0).abs() < 0.001);
        assert!((audit.mismatches[0].transaction_total - 19.0).abs() < 0.001);
    }

    #[test]
    fn skips_currency_mismatches() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/business/hosting/aws");
        std::fs::create_dir_all(&dir).unwrap();
        let invoice = dir.join("2026-01-01-invoice57.pdf");
        std::fs::write(&invoice, b"pdf").unwrap();
        std::fs::write(
            dir.join("2026-01-01-invoice57.document.yml"),
            "amount: 100.00\ncurrency: USD\n",
        )
        .unwrap();

        let gk = group_key("expenses/business/hosting/aws", nd(2026, 1, 1));
        let required_groups =
            HashMap::from([(gk, vec![make_required(ACCOUNT, nd(2026, 1, 1), 100.0)])]);
        let entries = vec![make_entry(invoice, ACCOUNT, nd(2026, 1, 1))];
        let audit = find_amount_mismatches(&required_groups, &entries);

        assert!(audit.mismatches.is_empty());
        assert_eq!(audit.checked_groups, 0);
        assert_eq!(audit.skipped_groups(), 1);
        assert_eq!(audit.skipped_currency_mismatch_groups, 1);
    }

    #[test]
    fn metadata_cover_amounts_cover_descendant_transaction_groups() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("liabilities/health-insurance");
        std::fs::create_dir_all(&dir).unwrap();
        let invoice = dir.join("2025-01-01-health-insurance.pdf");
        std::fs::write(&invoice, b"pdf").unwrap();
        std::fs::write(dir.join("2025-01-01-health-insurance.document.yml"),
            "covers:\n  - date: 2025-01-01\n    account: liabilities:health-insurance:2023\n    amount: 40.00\n    currency: EUR\n  - date: 2025-01-01\n    account: liabilities:health-insurance:2024\n    amount: 60.00\n    currency: EUR\n").unwrap();

        let required_groups = HashMap::from([
            (
                group_key("liabilities/health-insurance/2023", nd(2025, 1, 1)),
                vec![make_required(
                    "liabilities:health-insurance:2023",
                    nd(2025, 1, 1),
                    40.0,
                )],
            ),
            (
                group_key("liabilities/health-insurance/2024", nd(2025, 1, 1)),
                vec![make_required(
                    "liabilities:health-insurance:2024",
                    nd(2025, 1, 1),
                    60.0,
                )],
            ),
        ]);
        let entries = vec![make_entry(
            invoice,
            "liabilities:health-insurance",
            nd(2025, 1, 1),
        )];
        let audit = find_amount_mismatches(&required_groups, &entries);

        assert!(audit.mismatches.is_empty());
        assert_eq!(audit.checked_groups, 1);
    }

    #[test]
    fn document_total_mismatch_reported_for_split_cover_amounts() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/business/hosting/aws");
        std::fs::create_dir_all(&dir).unwrap();
        let invoice = dir.join("2026-01-01-invoice57.pdf");
        std::fs::write(&invoice, b"pdf").unwrap();
        std::fs::write(dir.join("2026-01-01-invoice57.document.yml"),
            "amount: 100.00\ncurrency: EUR\ncovers:\n  - date: 2026-01-01\n    account: expenses:business:hosting:aws\n    amount: 60.00\n  - date: 2026-02-01\n    account: expenses:business:hosting:aws\n    amount: 30.00\n").unwrap();

        let required_groups = HashMap::from([
            (
                group_key("expenses/business/hosting/aws", nd(2026, 1, 1)),
                vec![make_required(ACCOUNT, nd(2026, 1, 1), 60.0)],
            ),
            (
                group_key("expenses/business/hosting/aws", nd(2026, 2, 1)),
                vec![make_required(ACCOUNT, nd(2026, 2, 1), 30.0)],
            ),
        ]);
        let entries = vec![make_entry(invoice, ACCOUNT, nd(2026, 1, 1))];
        let audit = find_amount_mismatches(&required_groups, &entries);

        assert_eq!(audit.mismatches.len(), 1);
        assert!((audit.mismatches[0].document_total - 90.0).abs() < 0.001);
        assert!((audit.mismatches[0].transaction_total - 100.0).abs() < 0.001);
    }
}
