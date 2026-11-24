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

#[derive(Debug, Clone)]
pub struct AttributedCover {
    pub entry: DocumentEntry,
    pub cover: crate::model::DocumentCover,
    pub transaction: RequiredDocument,
}

pub fn attributed_groups(
    required_groups: &HashMap<GroupKey, Vec<RequiredDocument>>,
    entries: &[DocumentEntry],
) -> HashMap<GroupKey, Vec<AttributedCover>> {
    let mut result = HashMap::new();
    for (key, transactions) in required_groups {
        if transactions.len() < 2 {
            continue;
        }
        let mut covers = Vec::new();
        let mut complete = true;
        for entry in entries {
            if !matched_groups_for_entry(entry, required_groups).contains(key) {
                continue;
            }
            let Ok(Some(metadata)) = metadata_for_document(&entry.path) else {
                complete = false;
                break;
            };
            let account = entry.account_path.to_string_lossy();
            let matching: Vec<_> = metadata
                .covers
                .into_iter()
                .filter(|cover| {
                    cover.posting_date == Some(key.1)
                        && cover.account_path.as_deref().unwrap_or(&account) == key.0
                })
                .collect();
            if matching.is_empty() {
                complete = false;
                break;
            }
            covers.extend(matching.into_iter().map(|cover| (entry.clone(), cover)));
        }
        if !complete || covers.len() != transactions.len() {
            continue;
        }
        covers.sort_by(|(a, ac), (b, bc)| (&a.path, &ac.location).cmp(&(&b.path, &bc.location)));
        let mut candidates: Vec<Vec<usize>> = covers
            .iter()
            .map(|(_, cover)| {
                transactions
                    .iter()
                    .enumerate()
                    .filter_map(|(index, txn)| {
                        let currency_matches = match (&cover.currency, &txn.commodity) {
                            (Some(a), Some(b)) => a == b,
                            _ => true,
                        };
                        let identity_matches = match cover
                            .description
                            .as_deref()
                            .filter(|s| !s.trim().is_empty())
                        {
                            Some(description) => description
                                .trim()
                                .eq_ignore_ascii_case(txn.description.trim()),
                            None => {
                                match (cover.amount, txn.amount, &cover.currency, &txn.commodity) {
                                    (Some(a), Some(b), Some(_), Some(_)) => {
                                        (a - b).abs() <= crate::model::MONEY_TOLERANCE
                                    }
                                    _ => false,
                                }
                            }
                        };
                        (currency_matches && identity_matches).then_some(index)
                    })
                    .collect()
            })
            .collect();
        for ((_, cover), candidates) in covers.iter().zip(&mut candidates) {
            if candidates.len() > 1 && cover.description.is_some() {
                candidates.retain(|&index| match (cover.amount, transactions[index].amount) {
                    (Some(a), Some(b)) => (a - b).abs() <= crate::model::MONEY_TOLERANCE,
                    _ => true,
                });
            }
        }
        let Some(assignment) = perfect_assignment(&candidates, None) else {
            continue;
        };
        if assignment
            .iter()
            .enumerate()
            .any(|(cover, &txn)| perfect_assignment(&candidates, Some((cover, txn))).is_some())
        {
            continue;
        }
        result.insert(
            key.clone(),
            covers
                .into_iter()
                .zip(assignment)
                .map(|((entry, cover), index)| AttributedCover {
                    entry,
                    cover,
                    transaction: transactions[index].clone(),
                })
                .collect(),
        );
    }
    result
}

fn perfect_assignment(
    candidates: &[Vec<usize>],
    forbidden: Option<(usize, usize)>,
) -> Option<Vec<usize>> {
    fn augment(
        cover: usize,
        candidates: &[Vec<usize>],
        forbidden: Option<(usize, usize)>,
        seen: &mut [bool],
        owners: &mut [Option<usize>],
    ) -> bool {
        for &txn in &candidates[cover] {
            if forbidden == Some((cover, txn)) || seen[txn] {
                continue;
            }
            seen[txn] = true;
            if owners[txn].is_none()
                || augment(owners[txn].unwrap(), candidates, forbidden, seen, owners)
            {
                owners[txn] = Some(cover);
                return true;
            }
        }
        false
    }
    let mut owners = vec![None; candidates.len()];
    for cover in 0..candidates.len() {
        if !augment(
            cover,
            candidates,
            forbidden,
            &mut vec![false; candidates.len()],
            &mut owners,
        ) {
            return None;
        }
    }
    let mut assignment = vec![0; candidates.len()];
    for (txn, cover) in owners.into_iter().enumerate() {
        assignment[cover?] = txn;
    }
    Some(assignment)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn equal_descriptions_use_amounts_to_distinguish_transactions() {
        let tmp = TempDir::new().unwrap();
        let date = nd(2026, 1, 1);
        let mut entries = Vec::new();
        let mut transactions = Vec::new();
        for (index, amount) in [(1, 4.0), (2, 6.0)] {
            let path = tmp.path().join(format!("2026-01-01-{index}.pdf"));
            std::fs::write(&path, b"pdf").unwrap();
            std::fs::write(
                path.with_extension("document.yml"),
                format!("description: Shared\namount: {amount}\ncurrency: EUR\n"),
            )
            .unwrap();
            entries.push(make_entry(path, "expenses:domains", Some(date)));
            transactions.push(RequiredDocument {
                transaction_date: date.to_string(),
                description: "Shared".into(),
                comment: String::new(),
                account: "expenses:domains".into(),
                posting_date: date,
                amount: Some(amount),
                commodity: Some("EUR".into()),
                transaction_index: index,
            });
        }
        let key = ("expenses/domains".into(), date);
        let groups = attributed_groups(&HashMap::from([(key.clone(), transactions)]), &entries);
        assert_eq!(groups[&key][0].transaction.transaction_index, 1);
        assert_eq!(groups[&key][1].transaction.transaction_index, 2);
    }

    #[test]
    fn assignment_uses_all_constraints_instead_of_greedy_order() {
        let candidates = vec![vec![0, 1], vec![1]];
        assert_eq!(perfect_assignment(&candidates, None), Some(vec![0, 1]));
        assert_eq!(perfect_assignment(&candidates, Some((0, 0))), None);
        assert_eq!(perfect_assignment(&candidates, Some((1, 1))), None);
    }

    #[test]
    fn equal_candidates_have_an_alternative_assignment() {
        let candidates = vec![vec![0, 1], vec![0, 1]];
        let assignment = perfect_assignment(&candidates, None).unwrap();
        assert!(perfect_assignment(&candidates, Some((0, assignment[0]))).is_some());
    }

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
