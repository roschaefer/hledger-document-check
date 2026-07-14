use chrono::NaiveDate;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::document_paths::metadata_path_for_document;
use crate::metadata::metadata_for_document;
use crate::model::{
    DocumentEntry, DocumentKind, RequiredDocument, WrongAccountCover, METADATA_SUFFIX,
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

/// Finds `covers[].account` (or the top-level `account:` shorthand) overrides
/// that don't correspond to any required transaction. Such an override is
/// silently ignored by `matched_groups_for_entry` (it falls back to matching
/// by the file's own directory/filename instead), so the document still gets
/// matched correctly and nothing else surfaces the mistake — the declared
/// account is simply wrong and worth flagging on its own.
pub fn find_wrong_account_metadata(
    required_groups: &HashMap<GroupKey, Vec<RequiredDocument>>,
    matched_entries: &[DocumentEntry],
) -> Vec<WrongAccountCover> {
    let mut results = Vec::new();

    for entry in matched_entries {
        if !matches!(
            entry.kind,
            DocumentKind::MatchedPdf | DocumentKind::MissingMetadata
        ) {
            continue;
        }
        let Ok(Some(metadata)) = metadata_for_document(&entry.path) else {
            continue;
        };
        let entry_account = entry.account_path.to_string_lossy().into_owned();

        for cover in &metadata.covers {
            let Some(declared_account) = &cover.account_path else {
                continue; // no explicit override, nothing to verify
            };
            if declared_account == &entry_account {
                continue; // same as the file's own location, so declared/inferred keys would be identical
            }
            let Some(posting_date) = cover.posting_date else {
                continue;
            };

            let declared_key = (declared_account.clone(), posting_date);
            if required_groups.contains_key(&declared_key) {
                continue; // the declared account genuinely has a matching transaction
            }

            let inferred_key = (entry_account.clone(), posting_date);
            let suggested_account = required_groups
                .contains_key(&inferred_key)
                .then(|| entry_account.clone());

            results.push(WrongAccountCover {
                path: sidecar_path(&entry.path),
                declared_account: declared_account.clone(),
                posting_date,
                suggested_account,
            });
        }
    }

    results.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then_with(|| a.posting_date.cmp(&b.posting_date))
            .then_with(|| a.declared_account.cmp(&b.declared_account))
    });
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn nd(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
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

    fn make_entry(path: std::path::PathBuf, account: &str, match_date: NaiveDate) -> DocumentEntry {
        DocumentEntry {
            path,
            account_path: account.split('/').collect(),
            match_date: Some(match_date),
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
    fn flags_declared_account_with_no_matching_transaction_and_suggests_the_real_one() {
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
            nd(2025, 4, 3),
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

        let results = find_wrong_account_metadata(&required_groups, &[entry]);

        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0].declared_account,
            "expenses/business/hosting/wasabi"
        );
        assert_eq!(
            results[0].suggested_account.as_deref(),
            Some("expenses/business/hosting/wasabi/beforereversecharge")
        );
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

        let entry = make_entry(pdf, "expenses/insurance/health", nd(2026, 1, 1));
        let required_groups: HashMap<GroupKey, Vec<RequiredDocument>> = [(
            ("expenses/insurance/health/base".to_string(), nd(2026, 1, 1)),
            vec![make_required(
                "expenses:insurance:health:base",
                nd(2026, 1, 1),
            )],
        )]
        .into_iter()
        .collect();

        let results = find_wrong_account_metadata(&required_groups, &[entry]);

        assert!(results.is_empty());
    }

    #[test]
    fn flags_wrong_account_without_a_suggestion_when_neither_key_resolves() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/business/software");
        std::fs::create_dir_all(&dir).unwrap();
        let pdf = write(&dir, "2026-01-01-suite.pdf", "suite invoice");
        write(
            &dir,
            "2026-01-01-suite.document.yml",
            "covers:\n  - date: 2026-01-01\n    account: expenses:business:other\n    amount: 92.00\n",
        );

        let entry = make_entry(pdf, "expenses/business/software", nd(2026, 1, 1));
        let required_groups: HashMap<GroupKey, Vec<RequiredDocument>> = HashMap::new();

        let results = find_wrong_account_metadata(&required_groups, &[entry]);

        assert_eq!(results.len(), 1);
        assert!(results[0].suggested_account.is_none());
    }

    #[test]
    fn does_not_flag_covers_without_an_explicit_account() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/business/software");
        std::fs::create_dir_all(&dir).unwrap();
        let pdf = write(&dir, "2026-01-01-suite.pdf", "suite invoice");
        write(
            &dir,
            "2026-01-01-suite.document.yml",
            "amount: 92.00\ncurrency: EUR\n",
        );

        let entry = make_entry(pdf, "expenses/business/software", nd(2026, 1, 1));
        let required_groups: HashMap<GroupKey, Vec<RequiredDocument>> = HashMap::new();

        let results = find_wrong_account_metadata(&required_groups, &[entry]);

        assert!(results.is_empty());
    }
}
