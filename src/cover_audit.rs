use chrono::NaiveDate;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::document_paths::metadata_path_for_document;
use crate::metadata::metadata_for_document;
use crate::model::{
    DocumentEntry, DocumentKind, RequiredDocument, UnresolvableCover, METADATA_SUFFIX,
};

type GroupKey = (String, NaiveDate);

/// The sidecar path for a document entry — the file to actually edit. For
/// `MatchedPdf` entries, `entry.path` is the PDF; for `MissingMetadata`
/// entries, `entry.path` already is the sidecar.
fn sidecar_path(path: &Path) -> PathBuf {
    let is_metadata = path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.ends_with(METADATA_SUFFIX))
        .unwrap_or(false);
    if is_metadata {
        path.to_path_buf()
    } else {
        metadata_path_for_document(path)
    }
}

/// Finds `covers[]` entries whose (account, date) pair doesn't correspond to
/// any required transaction. Such a cover is silently dropped by
/// `matched_groups_for_entry` — if it was the file's only cover, or if other
/// covers on the same file still resolve, matching falls back to (or is
/// carried by) the file's own directory/filename identity instead, and the
/// mistake never surfaces anywhere else in the tool's output.
///
/// Two kinds of cover are skipped even when they fail to resolve, because
/// they're an `unmatched-documents` concern rather than a metadata mistake:
/// - a cover that asserts nothing beyond the file's own location (its
///   account equals the directory and its date equals the filename), and
/// - any cover on a document that has no resolved groups at all — i.e. the
///   whole entry is unmatched (`matched_groups_by_entry[entry]` is empty).
///   Reporting per-cover mistakes there would just duplicate the
///   file-level "this document isn't linked to anything" finding.
pub fn find_unresolvable_covers(
    required_groups: &HashMap<GroupKey, Vec<RequiredDocument>>,
    matched_groups_by_entry: &HashMap<DocumentEntry, Vec<GroupKey>>,
) -> Vec<UnresolvableCover> {
    let mut results = Vec::new();

    for (entry, groups) in matched_groups_by_entry {
        if !matches!(
            entry.kind,
            DocumentKind::MatchedPdf | DocumentKind::MissingMetadata
        ) {
            continue;
        }
        if groups.is_empty() {
            continue; // the whole document is unmatched; owned by unmatched-documents instead
        }
        let Ok(Some(metadata)) = metadata_for_document(&entry.path) else {
            continue;
        };
        let entry_account = entry.account_path.to_string_lossy().into_owned();

        for cover in &metadata.covers {
            let Some(declared_date) = cover.posting_date else {
                continue;
            };
            let declared_account = cover
                .account_path
                .clone()
                .unwrap_or_else(|| entry_account.clone());

            let declared_key = (declared_account.clone(), declared_date);
            if required_groups.contains_key(&declared_key) {
                continue; // resolves to a real transaction; nothing wrong
            }

            let is_trivial =
                declared_account == entry_account && entry.match_date == Some(declared_date);
            if is_trivial {
                continue; // asserts nothing beyond the file's own location
            }

            let suggested = entry.match_date.and_then(|inferred_date| {
                let inferred_key = (entry_account.clone(), inferred_date);
                required_groups
                    .contains_key(&inferred_key)
                    .then_some((entry_account.clone(), inferred_date))
            });

            results.push(UnresolvableCover {
                path: sidecar_path(&entry.path),
                location: cover.location.clone(),
                declared_account,
                declared_date,
                suggested,
            });
        }
    }

    results.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then_with(|| a.declared_date.cmp(&b.declared_date))
            .then_with(|| a.declared_account.cmp(&b.declared_account))
    });
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matching::matched_groups_for_entry;
    use tempfile::TempDir;

    fn nd(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    /// Mirrors what `build_document_journal_diff` computes in production, so
    /// tests exercise the real resolution logic instead of hand-rolling it.
    fn matched_groups_by_entry(
        entries: &[DocumentEntry],
        required_groups: &HashMap<GroupKey, Vec<RequiredDocument>>,
    ) -> HashMap<DocumentEntry, Vec<GroupKey>> {
        entries
            .iter()
            .map(|e| (e.clone(), matched_groups_for_entry(e, required_groups)))
            .collect()
    }

    fn make_required(account: &str, date: NaiveDate) -> RequiredDocument {
        RequiredDocument {
            transaction_date: "2026-01-01".to_string(),
            description: "Example transaction".to_string(),
            comment: "".to_string(),
            account: account.to_string(),
            posting_date: date,
            amount: Some(1.0),
            commodity: Some("EUR".to_string()),
            transaction_index: 1,
        }
    }

    fn make_entry(
        path: std::path::PathBuf,
        account: &str,
        match_date: Option<NaiveDate>,
    ) -> DocumentEntry {
        DocumentEntry {
            path,
            account_path: account.split('/').collect(),
            match_date,
            rest_name: None,
            kind: DocumentKind::MatchedPdf,
        }
    }

    fn write(dir: &std::path::Path, name: &str, text: &str) -> std::path::PathBuf {
        let p = dir.join(name);
        std::fs::write(&p, text).unwrap();
        p
    }

    #[test]
    fn flags_wrong_account_and_suggests_the_real_one() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp
            .path()
            .join("expenses/business/hosting/wasabi/beforereversecharge");
        std::fs::create_dir_all(&dir).unwrap();
        let pdf = write(&dir, "2025-04-03.pdf", "wasabi invoice");
        write(
            &dir,
            "2025-04-03.document.yml",
            "covers:\n  - date: 2025-04-03\n    account: expenses:business:hosting:wasabi\n    amount: 8.94\n    currency: EUR\n",
        );

        let entry = make_entry(
            pdf,
            "expenses/business/hosting/wasabi/beforereversecharge",
            Some(nd(2025, 4, 3)),
        );
        let required_groups: HashMap<GroupKey, Vec<RequiredDocument>> = [(
            (
                "expenses/business/hosting/wasabi/beforereversecharge".to_string(),
                nd(2025, 4, 3),
            ),
            vec![make_required(
                "expenses:business:hosting:wasabi:beforereversecharge",
                nd(2025, 4, 3),
            )],
        )]
        .into_iter()
        .collect();

        let matched = matched_groups_by_entry(&[entry], &required_groups);
        let results = find_unresolvable_covers(&required_groups, &matched);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].location, "covers[0]");
        assert_eq!(
            results[0].declared_account,
            "expenses/business/hosting/wasabi"
        );
        assert_eq!(
            results[0].suggested,
            Some((
                "expenses/business/hosting/wasabi/beforereversecharge".to_string(),
                nd(2025, 4, 3)
            ))
        );
    }

    #[test]
    fn labels_a_top_level_shorthand_mistake_as_top_level() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/business/hosting/aws");
        std::fs::create_dir_all(&dir).unwrap();
        let pdf = write(&dir, "2022-06-03-invoice.pdf", "aws invoice");
        write(
            &dir,
            "2022-06-03-invoice.document.yml",
            "date: 2022-06-04\namount: 2.99\ncurrency: EUR\n",
        );

        let entry = make_entry(pdf, "expenses/business/hosting/aws", Some(nd(2022, 6, 3)));
        let required_groups: HashMap<GroupKey, Vec<RequiredDocument>> = [(
            ("expenses/business/hosting/aws".to_string(), nd(2022, 6, 3)),
            vec![make_required(
                "expenses:business:hosting:aws",
                nd(2022, 6, 3),
            )],
        )]
        .into_iter()
        .collect();

        let matched = matched_groups_by_entry(&[entry], &required_groups);
        let results = find_unresolvable_covers(&required_groups, &matched);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].location, "top-level");
    }

    #[test]
    fn flags_wrong_date_and_suggests_the_real_one() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/business/hosting/mailbox");
        std::fs::create_dir_all(&dir).unwrap();
        let pdf = write(&dir, "2024-01-05-invoice-1234.pdf", "invoice");
        // filed under the invoice date, but the SEPA debit actually cleared later
        write(
            &dir,
            "2024-01-05-invoice-1234.document.yml",
            "covers:\n  - date: 2024-01-06\n    amount: 8.94\n    currency: EUR\n",
        );

        let entry = make_entry(
            pdf,
            "expenses/business/hosting/mailbox",
            Some(nd(2024, 1, 5)),
        );
        let required_groups: HashMap<GroupKey, Vec<RequiredDocument>> = [(
            (
                "expenses/business/hosting/mailbox".to_string(),
                nd(2024, 1, 5),
            ),
            vec![make_required(
                "expenses:business:hosting:mailbox",
                nd(2024, 1, 5),
            )],
        )]
        .into_iter()
        .collect();

        let matched = matched_groups_by_entry(&[entry], &required_groups);
        let results = find_unresolvable_covers(&required_groups, &matched);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].declared_date, nd(2024, 1, 6));
        assert_eq!(
            results[0].suggested,
            Some((
                "expenses/business/hosting/mailbox".to_string(),
                nd(2024, 1, 5)
            ))
        );
    }

    #[test]
    fn does_not_flag_a_legitimate_date_override_that_resolves() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/business/hosting/mailbox");
        std::fs::create_dir_all(&dir).unwrap();
        let pdf = write(&dir, "2024-01-05-invoice-1234.pdf", "invoice");
        write(
            &dir,
            "2024-01-05-invoice-1234.document.yml",
            "covers:\n  - date: 2024-01-20\n    amount: 8.94\n    currency: EUR\n",
        );

        let entry = make_entry(
            pdf,
            "expenses/business/hosting/mailbox",
            Some(nd(2024, 1, 5)),
        );
        let required_groups: HashMap<GroupKey, Vec<RequiredDocument>> = [(
            (
                "expenses/business/hosting/mailbox".to_string(),
                nd(2024, 1, 20),
            ),
            vec![make_required(
                "expenses:business:hosting:mailbox",
                nd(2024, 1, 20),
            )],
        )]
        .into_iter()
        .collect();

        let matched = matched_groups_by_entry(&[entry], &required_groups);
        let results = find_unresolvable_covers(&required_groups, &matched);

        assert!(results.is_empty());
    }

    #[test]
    fn does_not_flag_a_legitimate_account_override() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/insurance/health");
        std::fs::create_dir_all(&dir).unwrap();
        let pdf = write(&dir, "2026-01-01-annual-notice.pdf", "notice");
        write(
            &dir,
            "2026-01-01-annual-notice.document.yml",
            "covers:\n  - account: expenses:insurance:health:base\n    amount: 100.00\n",
        );

        let entry = make_entry(pdf, "expenses/insurance/health", Some(nd(2026, 1, 1)));
        let required_groups: HashMap<GroupKey, Vec<RequiredDocument>> = [(
            ("expenses/insurance/health/base".to_string(), nd(2026, 1, 1)),
            vec![make_required(
                "expenses:insurance:health:base",
                nd(2026, 1, 1),
            )],
        )]
        .into_iter()
        .collect();

        let matched = matched_groups_by_entry(&[entry], &required_groups);
        let results = find_unresolvable_covers(&required_groups, &matched);

        assert!(results.is_empty());
    }

    #[test]
    fn does_not_flag_a_wrong_cover_when_the_whole_entry_is_unmatched() {
        // A non-trivial wrong cover on a document that doesn't resolve at all
        // (via this cover or any fallback) used to also show up here,
        // duplicating the unmatched-documents finding for the same file.
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/business/software");
        std::fs::create_dir_all(&dir).unwrap();
        let pdf = write(&dir, "2026-01-01-suite.pdf", "suite invoice");
        write(
            &dir,
            "2026-01-01-suite.document.yml",
            "covers:\n  - date: 2026-01-01\n    account: expenses:business:other\n    amount: 92.00\n",
        );

        let entry = make_entry(pdf, "expenses/business/software", Some(nd(2026, 1, 1)));
        let required_groups: HashMap<GroupKey, Vec<RequiredDocument>> = HashMap::new();

        let matched = matched_groups_by_entry(&[entry], &required_groups);
        let results = find_unresolvable_covers(&required_groups, &matched);

        assert!(results.is_empty());
    }

    #[test]
    fn does_not_flag_covers_without_an_explicit_account_or_date() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/business/software");
        std::fs::create_dir_all(&dir).unwrap();
        let pdf = write(&dir, "2026-01-01-suite.pdf", "suite invoice");
        write(
            &dir,
            "2026-01-01-suite.document.yml",
            "amount: 92.00\ncurrency: EUR\n",
        );

        let entry = make_entry(pdf, "expenses/business/software", Some(nd(2026, 1, 1)));
        let required_groups: HashMap<GroupKey, Vec<RequiredDocument>> = HashMap::new();

        let matched = matched_groups_by_entry(&[entry], &required_groups);
        let results = find_unresolvable_covers(&required_groups, &matched);

        assert!(results.is_empty());
    }

    #[test]
    fn does_not_duplicate_unmatched_documents_for_a_trivial_cover() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/business/software");
        std::fs::create_dir_all(&dir).unwrap();
        let pdf = write(&dir, "2026-01-01-suite.pdf", "suite invoice");
        write(
            &dir,
            "2026-01-01-suite.document.yml",
            "covers:\n  - date: 2026-01-01\n    account: expenses:business:software\n    amount: 92.00\n",
        );

        // No required transaction exists at all; this document is simply unmatched.
        let entry = make_entry(pdf, "expenses/business/software", Some(nd(2026, 1, 1)));
        let required_groups: HashMap<GroupKey, Vec<RequiredDocument>> = HashMap::new();

        let matched = matched_groups_by_entry(&[entry], &required_groups);
        let results = find_unresolvable_covers(&required_groups, &matched);

        assert!(results.is_empty());
    }

    #[test]
    fn flags_one_wrong_cover_among_several_on_an_undated_file() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("income/business/freelance/customer");
        std::fs::create_dir_all(&dir).unwrap();
        let pdf = write(&dir, "customer-project.pdf", "invoice");
        write(
            &dir,
            "customer-project.document.yml",
            "covers:\n  - date: 2026-01-05\n    amount: 60.00\n  - date: 2026-02-05\n    amount: 40.00\n",
        );

        // second installment's real date was 2026-02-06, not 2026-02-05
        let entry = make_entry(pdf, "income/business/freelance/customer", None);
        let required_groups: HashMap<GroupKey, Vec<RequiredDocument>> = [(
            (
                "income/business/freelance/customer".to_string(),
                nd(2026, 1, 5),
            ),
            vec![make_required(
                "income:business:freelance:customer",
                nd(2026, 1, 5),
            )],
        )]
        .into_iter()
        .collect();

        let matched = matched_groups_by_entry(&[entry], &required_groups);
        let results = find_unresolvable_covers(&required_groups, &matched);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].location, "covers[1]");
        assert_eq!(results[0].declared_date, nd(2026, 2, 5));
        assert!(results[0].suggested.is_none()); // undated file has no fallback identity to suggest
    }
}
