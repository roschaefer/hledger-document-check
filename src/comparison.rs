use std::collections::HashMap;
use std::path::PathBuf;

use crate::document_tree::scan_document_tree;
use crate::journal::iter_required_documents;
use crate::matching::{matched_groups_for_entry, GroupKey};
use crate::metadata::metadata_for_document;
use crate::model::{DocumentEntry, DocumentKind, DocumentTree, RequiredDocument};

pub struct JournalTree {
    pub required_groups: HashMap<GroupKey, Vec<RequiredDocument>>,
}

pub struct DocumentJournalDiff {
    pub journal: JournalTree,
    pub documents: DocumentTree,
    pub matched_groups_by_entry: HashMap<DocumentEntry, Vec<GroupKey>>,
}

impl DocumentJournalDiff {
    pub fn covered_groups(&self) -> std::collections::HashSet<GroupKey> {
        self.matched_groups_by_entry
            .values()
            .flatten()
            .cloned()
            .collect()
    }

    pub fn missing_transactions(&self) -> Vec<RequiredDocument> {
        let covered = self.covered_groups();
        let mut missing = Vec::new();
        let mut keys: Vec<&GroupKey> = self.journal.required_groups.keys().collect();
        keys.sort();
        for key in keys {
            if covered.contains(key) {
                continue;
            }
            missing.extend(self.journal.required_groups[key].iter().cloned());
        }
        missing
    }

    pub fn unmatched_entries(&self) -> Vec<&DocumentEntry> {
        self.documents
            .matched_entries
            .iter()
            .filter(|e| {
                self.matched_groups_by_entry
                    .get(*e)
                    .map(|v| v.is_empty())
                    .unwrap_or(true)
            })
            .collect()
    }

    pub fn ambiguous_transaction_groups(&self) -> HashMap<GroupKey, usize> {
        let metadata_cover_counts = metadata_cover_counts(&self.documents.matched_entries);
        self.journal
            .required_groups
            .iter()
            .filter(|(key, txns)| {
                txns.len() > 1 && metadata_cover_counts.get(key).copied().unwrap_or(0) < txns.len()
            })
            .map(|(key, txns)| (key.clone(), txns.len()))
            .collect()
    }

    pub fn documents_by_transaction_index(&self) -> HashMap<i64, Vec<PathBuf>> {
        let mut docs_by_index: HashMap<i64, std::collections::HashSet<PathBuf>> = HashMap::new();
        for (entry, group_keys) in &self.matched_groups_by_entry {
            if entry.kind != DocumentKind::MatchedPdf {
                continue;
            }
            for group_key in group_keys {
                if let Some(txns) = self.journal.required_groups.get(group_key) {
                    for txn in txns {
                        let resolved = entry
                            .path
                            .canonicalize()
                            .unwrap_or_else(|_| entry.path.clone());
                        docs_by_index
                            .entry(txn.transaction_index)
                            .or_default()
                            .insert(resolved);
                    }
                }
            }
        }
        docs_by_index
            .into_iter()
            .map(|(k, v)| {
                let mut paths: Vec<PathBuf> = v.into_iter().collect();
                paths.sort();
                (k, paths)
            })
            .collect()
    }
}

pub fn build_journal_tree(
    transactions: Vec<serde_json::Value>,
    required_tag_prefixes: &[String],
) -> JournalTree {
    let required_documents = iter_required_documents(&transactions, required_tag_prefixes);
    let mut required_groups: HashMap<GroupKey, Vec<RequiredDocument>> = HashMap::new();
    for doc in &required_documents {
        let key = (
            doc.account_path().to_string_lossy().into_owned(),
            doc.posting_date,
        );
        required_groups.entry(key).or_default().push(doc.clone());
    }
    JournalTree { required_groups }
}

pub fn build_document_journal_diff(
    journal: JournalTree,
    documents: DocumentTree,
) -> DocumentJournalDiff {
    let matched_groups_by_entry: HashMap<DocumentEntry, Vec<GroupKey>> = documents
        .matched_entries
        .iter()
        .map(|entry| {
            let groups = matched_groups_for_entry(entry, &journal.required_groups);
            (entry.clone(), groups)
        })
        .collect();

    DocumentJournalDiff {
        journal,
        documents,
        matched_groups_by_entry,
    }
}

pub fn load_document_journal_diff(
    transactions: Vec<serde_json::Value>,
    document_root: &std::path::Path,
    required_tag_prefixes: &[String],
) -> DocumentJournalDiff {
    let journal = build_journal_tree(transactions, required_tag_prefixes);
    let documents = scan_document_tree(document_root);
    build_document_journal_diff(journal, documents)
}

fn metadata_cover_counts(entries: &[DocumentEntry]) -> HashMap<GroupKey, usize> {
    let mut counts: HashMap<GroupKey, usize> = HashMap::new();
    for entry in entries {
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
            let Some(posting_date) = cover.posting_date else {
                continue;
            };
            let account = cover
                .account_path
                .clone()
                .unwrap_or_else(|| entry_account.clone());
            *counts.entry((account, posting_date)).or_insert(0) += 1;
        }
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use tempfile::TempDir;

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

    #[test]
    fn indexes_matched_documents_by_transaction_index() {
        let tmp = TempDir::new().unwrap();
        let account = "expenses:business:hosting:aws";
        let dir = tmp.path().join("expenses/business/hosting/aws");
        std::fs::create_dir_all(&dir).unwrap();
        let document = dir.join("2026-01-01-invoice57.pdf");
        std::fs::write(&document, b"pdf").unwrap();

        let required = make_required(account, nd(2026, 1, 1), 100.0);
        let group_key = ("expenses/business/hosting/aws".to_string(), nd(2026, 1, 1));
        let journal = JournalTree {
            required_groups: [(group_key.clone(), vec![required])].into_iter().collect(),
        };
        let documents = crate::document_tree::scan_document_tree(tmp.path());
        let diff = build_document_journal_diff(journal, documents);

        assert_eq!(
            diff.covered_groups(),
            std::collections::HashSet::from([group_key])
        );
        assert!(diff.missing_transactions().is_empty());
        assert!(diff.unmatched_entries().is_empty());
        let by_index = diff.documents_by_transaction_index();
        assert_eq!(by_index[&1], vec![document.canonicalize().unwrap()]);
    }
}
