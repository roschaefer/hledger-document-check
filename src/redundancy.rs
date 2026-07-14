use std::path::{Path, PathBuf};

use crate::document_paths::{
    account_dir_for_document, account_path_from_dir, document_path_for_metadata,
    parse_metadata_filename,
};
use crate::metadata::{parse_yaml_date, yaml_to_string};
use crate::model::{RedundantMetadataField, METADATA_SUFFIX};

fn iter_metadata_files(root: &Path) -> Vec<PathBuf> {
    let root = match root.canonicalize() {
        Ok(p) => p,
        Err(_) => root.to_path_buf(),
    };
    if !root.exists() {
        return vec![];
    }
    let mut files: Vec<PathBuf> = walkdir::WalkDir::new(&root)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.ends_with(METADATA_SUFFIX))
                .unwrap_or(false)
        })
        .collect();
    files.sort();
    files
}

/// The account a `.document.yml` sidecar's location implies, matching the same
/// directory-derived fallback that `matching.rs`/`comparison.rs` use for covers
/// with no explicit `account`. Computed from the sidecar's would-be PDF path so
/// the `unbooked/` directory is skipped the same way it is for real documents.
fn inferred_account(path: &Path, root: &Path) -> String {
    let document_shaped_path = document_path_for_metadata(path);
    let dir = account_dir_for_document(&document_shaped_path);
    account_path_from_dir(&dir, root)
        .to_string_lossy()
        .into_owned()
}

fn check_field(
    mapping: &serde_yaml::Mapping,
    location: &str,
    path: &Path,
    default_date: Option<chrono::NaiveDate>,
    default_account: &str,
    out: &mut Vec<RedundantMetadataField>,
) {
    let get = |key: &str| -> Option<&serde_yaml::Value> {
        mapping.get(serde_yaml::Value::String(key.to_string()))
    };

    if let Some(v) = get("date") {
        if let Ok(Some(date)) = parse_yaml_date(v) {
            if Some(date) == default_date {
                out.push(RedundantMetadataField {
                    path: path.to_path_buf(),
                    location: location.to_string(),
                    field: "date",
                    value: date.format("%Y-%m-%d").to_string(),
                });
            }
        }
    }

    if let Some(v) = get("account") {
        if let Some(account) = yaml_to_string(v) {
            if account.replace(':', "/") == default_account {
                out.push(RedundantMetadataField {
                    path: path.to_path_buf(),
                    location: location.to_string(),
                    field: "account",
                    value: account,
                });
            }
        }
    }
}

pub fn find_redundant_metadata(root: &Path) -> Vec<RedundantMetadataField> {
    let canonical_root = match root.canonicalize() {
        Ok(p) => p,
        Err(_) => root.to_path_buf(),
    };

    let mut results = Vec::new();

    for path in iter_metadata_files(root) {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(serde_yaml::Value::Mapping(mapping)) = serde_yaml::from_str(&text) else {
            // Invalid or empty YAML is already reported by the metadata/tree checks.
            continue;
        };

        let (default_date, _) = parse_metadata_filename(&path);
        let default_account = inferred_account(&path, &canonical_root);

        check_field(
            &mapping,
            "top-level",
            &path,
            default_date,
            &default_account,
            &mut results,
        );

        if let Some(covers_val) = mapping.get(serde_yaml::Value::String("covers".to_string())) {
            if let Some(seq) = covers_val.as_sequence() {
                for (i, item) in seq.iter().enumerate() {
                    if let Some(cover_mapping) = item.as_mapping() {
                        let location = format!("covers[{i}]");
                        check_field(
                            cover_mapping,
                            &location,
                            &path,
                            default_date,
                            &default_account,
                            &mut results,
                        );
                    }
                }
            }
        }
    }

    results.sort_by(|a, b| {
        // Stable sort: within the same file/location, keep date before account
        // (the order `check_field` reports them in).
        a.path
            .cmp(&b.path)
            .then_with(|| a.location.cmp(&b.location))
    });
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write(dir: &Path, name: &str, text: &str) -> PathBuf {
        let p = dir.join(name);
        std::fs::write(&p, text).unwrap();
        p
    }

    #[test]
    fn flags_redundant_top_level_shorthand() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("Expenses/Business/Hosting/Uberspace");
        std::fs::create_dir_all(&dir).unwrap();
        write(
            &dir,
            "2024-10-31-missing.document.yml",
            "date: 2024-10-31\naccount: Expenses:Business:Hosting:Uberspace\n",
        );

        let results = find_redundant_metadata(tmp.path());

        assert_eq!(results.len(), 2);
        assert!(results
            .iter()
            .any(|r| r.field == "date" && r.location == "top-level"));
        assert!(results
            .iter()
            .any(|r| r.field == "account" && r.location == "top-level"));
    }

    #[test]
    fn flags_redundant_covers_fields() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("income/business/freelance/customer");
        std::fs::create_dir_all(&dir).unwrap();
        write(
            &dir,
            "2026-03-01-invoice.document.yml",
            "covers:\n  - date: 2026-03-01\n    account: income:business:freelance:customer\n    amount: 40.00\n",
        );

        let results = find_redundant_metadata(tmp.path());

        assert_eq!(results.len(), 2);
        assert!(results
            .iter()
            .all(|r| r.location == "covers[0]" && (r.field == "date" || r.field == "account")));
    }

    #[test]
    fn does_not_flag_legitimate_account_override() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/insurance/health");
        std::fs::create_dir_all(&dir).unwrap();
        write(
            &dir,
            "2026-01-01-annual-notice.document.yml",
            "covers:\n  - account: expenses:insurance:health:base\n    amount: 100.00\n",
        );

        let results = find_redundant_metadata(tmp.path());

        assert!(results.is_empty());
    }

    #[test]
    fn does_not_flag_non_cover_fields() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/business/software");
        std::fs::create_dir_all(&dir).unwrap();
        write(
            &dir,
            "2026-01-01-suite.document.yml",
            "amount: 100.00\ncurrency: EUR\ndescription: Software Suite\n",
        );

        let results = find_redundant_metadata(tmp.path());

        assert!(results.is_empty());
    }

    #[test]
    fn accounts_for_unbooked_directory_when_checking_account() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/business/hosting/aws/unbooked");
        std::fs::create_dir_all(&dir).unwrap();
        write(
            &dir,
            "invoice.document.yml",
            "account: expenses:business:hosting:aws\n",
        );

        let results = find_redundant_metadata(tmp.path());

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].field, "account");
    }

    #[test]
    fn does_not_flag_different_date_or_account() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/business/hosting/aws");
        std::fs::create_dir_all(&dir).unwrap();
        write(
            &dir,
            "2026-01-01-invoice.document.yml",
            "covers:\n  - date: 2026-01-02\n    account: expenses:business:hosting:other\n",
        );

        let results = find_redundant_metadata(tmp.path());

        assert!(results.is_empty());
    }
}
