use crate::metadata::metadata_for_document;
use crate::model::{DocumentEntry, DocumentKind, RequiredDocument};
use chrono::NaiveDate;
use std::collections::HashMap;

pub type GroupKey = (String, NaiveDate);

pub fn matched_groups_for_entry(
    entry: &DocumentEntry,
    required_groups: &HashMap<GroupKey, Vec<RequiredDocument>>,
) -> Vec<GroupKey> {
    let metadata = metadata_for_document(&entry.path).unwrap_or_default();

    if metadata.is_none() {
        if entry.kind == DocumentKind::MatchedPdf {
            if let Some(key) = entry.key() {
                if required_groups.contains_key(&key) {
                    return vec![key];
                }
            }
        }
        return vec![];
    }

    let metadata = metadata.unwrap();
    let entry_account = entry.account_path.to_string_lossy().into_owned();

    let metadata_groups: Vec<GroupKey> = metadata
        .covers
        .iter()
        .filter_map(|cover| {
            let date = cover.posting_date?;
            let account = cover
                .account_path
                .clone()
                .unwrap_or_else(|| entry_account.clone());
            Some((account, date))
        })
        .filter(|key| required_groups.contains_key(key))
        .collect();

    if !metadata_groups.is_empty() {
        return metadata_groups;
    }

    if matches!(
        entry.kind,
        DocumentKind::MatchedPdf | DocumentKind::MissingMetadata
    ) {
        if let Some(key) = entry.key() {
            if required_groups.contains_key(&key) {
                return vec![key];
            }
        }
    }

    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn nd(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn make_entry(path: PathBuf, account: &str, match_date: Option<NaiveDate>) -> DocumentEntry {
        DocumentEntry {
            path,
            account_path: account.split(':').collect(),
            match_date,
            rest_name: None,
            kind: crate::model::DocumentKind::MatchedPdf,
        }
    }

    #[test]
    fn metadata_cover_can_match_descendant_account() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("liabilities/health-insurance");
        std::fs::create_dir_all(&dir).unwrap();
        let invoice = dir.join("2025-01-01.pdf");
        std::fs::write(&invoice, b"pdf").unwrap();
        std::fs::write(dir.join("2025-01-01.document.yml"),
            "covers:\n  - date: 2025-01-01\n    account: liabilities:health-insurance:2023\n    amount: 69.26\n    currency: EUR\n").unwrap();

        let required_groups: HashMap<GroupKey, Vec<crate::model::RequiredDocument>> = [(
            (
                "liabilities/health-insurance/2023".to_string(),
                nd(2025, 1, 1),
            ),
            vec![],
        )]
        .into_iter()
        .collect();
        let entry = make_entry(
            invoice,
            "liabilities:health-insurance",
            Some(nd(2025, 1, 1)),
        );

        let groups = matched_groups_for_entry(&entry, &required_groups);
        assert_eq!(
            groups,
            vec![(
                "liabilities/health-insurance/2023".to_string(),
                nd(2025, 1, 1)
            )]
        );
    }

    #[test]
    fn without_metadata_parent_folder_does_not_cover_descendant() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("liabilities/health-insurance");
        std::fs::create_dir_all(&dir).unwrap();
        let invoice = dir.join("2025-01-01.pdf");
        std::fs::write(&invoice, b"pdf").unwrap();

        let required_groups: HashMap<GroupKey, Vec<crate::model::RequiredDocument>> = [(
            (
                "liabilities/health-insurance/2023".to_string(),
                nd(2025, 1, 1),
            ),
            vec![],
        )]
        .into_iter()
        .collect();
        let entry = make_entry(
            invoice,
            "liabilities:health-insurance",
            Some(nd(2025, 1, 1)),
        );

        assert_eq!(matched_groups_for_entry(&entry, &required_groups), vec![]);
    }

    #[test]
    fn cover_with_no_account_defaults_to_entry_account() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/business/software/ai/anthropic");
        std::fs::create_dir_all(&dir).unwrap();
        let invoice = dir.join("2026-03-11-invoice.pdf");
        std::fs::write(&invoice, b"pdf").unwrap();
        std::fs::write(
            dir.join("2026-03-11-invoice.document.yml"),
            "covers:\n  - amount: 19.00\n    currency: USD\n",
        )
        .unwrap();

        let key = (
            "expenses/business/software/ai/anthropic".to_string(),
            nd(2026, 3, 11),
        );
        let required_groups: HashMap<GroupKey, Vec<crate::model::RequiredDocument>> =
            [(key.clone(), vec![])].into_iter().collect();
        let entry = make_entry(
            invoice,
            "expenses:business:software:ai:anthropic",
            Some(nd(2026, 3, 11)),
        );

        assert_eq!(
            matched_groups_for_entry(&entry, &required_groups),
            vec![key]
        );
    }
}
