use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::document_paths::{
    account_dir_for_document, account_path_from_dir, document_path_for_metadata,
    is_missing_metadata, is_unbooked_document_path, metadata_path_for_document,
    parse_document_filename, parse_metadata_filename,
};
use crate::metadata::metadata_for_document;
use crate::model::{
    DocumentEntry, DocumentKind, DocumentTree, TreeIssue, CONFIG_FILENAME, METADATA_SUFFIX,
};

pub fn scan_document_tree(root: &Path) -> DocumentTree {
    let root = match root.canonicalize() {
        Ok(p) => p,
        Err(_) => root.to_path_buf(),
    };

    if !root.exists() {
        return DocumentTree::default();
    }

    let mut matched_entries: Vec<DocumentEntry> = Vec::new();
    let mut unbooked_entries: Vec<DocumentEntry> = Vec::new();
    let mut issues: Vec<TreeIssue> = Vec::new();

    // First pass: collect all paths sorted
    let mut all_paths: Vec<PathBuf> = Vec::new();
    for entry in walkdir::WalkDir::new(&root).sort_by_file_name() {
        match entry {
            Ok(e) => {
                if e.path() != root {
                    all_paths.push(e.path().to_path_buf());
                }
            }
            Err(e) => {
                let path = e
                    .path()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| root.clone());
                issues.push(TreeIssue {
                    path,
                    reason: format!("cannot read: {e}"),
                });
            }
        }
    }

    // Identify support directories: a directory named after the stem of a sibling PDF
    let mut support_dirs: HashSet<PathBuf> = HashSet::new();
    // Group files by parent directory
    let mut dirs_to_children: std::collections::HashMap<PathBuf, Vec<PathBuf>> =
        std::collections::HashMap::new();
    for path in &all_paths {
        if let Some(parent) = path.parent() {
            dirs_to_children
                .entry(parent.to_path_buf())
                .or_default()
                .push(path.clone());
        }
    }

    for children in dirs_to_children.values() {
        let pdf_stems: HashSet<String> = children
            .iter()
            .filter(|p| {
                p.is_file()
                    && p.extension()
                        .and_then(|e| e.to_str())
                        .map(|e| e == "pdf")
                        .unwrap_or(false)
            })
            .filter_map(|p| {
                p.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            })
            .collect();

        for child in children {
            if child.is_dir() {
                if let Some(name) = child.file_name().and_then(|n| n.to_str()) {
                    if pdf_stems.contains(name) {
                        support_dirs.insert(child.clone());
                    }
                }
            }
        }
    }

    fn in_support_tree(path: &Path, support_dirs: &HashSet<PathBuf>) -> bool {
        path.ancestors().skip(1).any(|a| support_dirs.contains(a))
    }

    let config_path = root.join(CONFIG_FILENAME);
    let mut invalid_metadata_paths: HashSet<PathBuf> = HashSet::new();

    for path in &all_paths {
        // Skip directories (we handle them implicitly via children)
        if path.is_dir() {
            continue;
        }

        // Skip support tree contents
        if in_support_tree(path, &support_dirs) {
            continue;
        }

        // Skip hidden files
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with('.'))
            .unwrap_or(false)
        {
            continue;
        }

        // Skip the config file itself
        if path == &config_path {
            continue;
        }

        // Skip symlinks
        if path.is_symlink() {
            issues.push(TreeIssue {
                path: path.clone(),
                reason: "symlink aliases are not supported; use document metadata".to_string(),
            });
            continue;
        }

        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Handle metadata files
        if name.ends_with(METADATA_SUFFIX) {
            if let Err(e) = metadata_for_document(path) {
                invalid_metadata_paths.insert(path.clone());
                issues.push(TreeIssue {
                    path: path.clone(),
                    reason: format!("invalid metadata: {e}"),
                });
                continue;
            }

            // Check if corresponding PDF exists
            let doc_path = document_path_for_metadata(path);
            if doc_path.exists() {
                continue;
            }

            // No PDF — must be a missing-document placeholder
            let (match_date, _) = parse_metadata_filename(path);
            if match_date.is_none() || !is_missing_metadata(path) {
                issues.push(TreeIssue {
                    path: path.clone(),
                    reason: "metadata file has no matching document PDF; use YYYY-MM-DD-missing.document.yml for missing documents".to_string(),
                });
                continue;
            }

            let account_path = account_path_from_dir(path.parent().unwrap_or(&root), &root);
            let (_, rest_name) = parse_metadata_filename(path);
            matched_entries.push(DocumentEntry {
                path: path.clone(),
                account_path,
                match_date,
                rest_name,
                kind: DocumentKind::MissingMetadata,
            });
            continue;
        }

        // Handle PDF files
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext == "pdf" {
            let account_path = account_path_from_dir(&account_dir_for_document(path), &root);

            let metadata_path = metadata_path_for_document(path);
            let has_valid_metadata =
                if metadata_path.exists() && !invalid_metadata_paths.contains(&metadata_path) {
                    match metadata_for_document(path) {
                        Ok(Some(_)) => true,
                        Ok(None) => false,
                        Err(e) => {
                            invalid_metadata_paths.insert(metadata_path.clone());
                            issues.push(TreeIssue {
                                path: metadata_path.clone(),
                                reason: format!("invalid metadata: {e}"),
                            });
                            false
                        }
                    }
                } else {
                    false
                };

            let (match_date, rest_name) = parse_document_filename(path);

            if match_date.is_none() {
                // Undated PDF
                if has_valid_metadata {
                    if let Ok(Some(meta)) = metadata_for_document(path) {
                        if meta.covers.iter().any(|c| c.posting_date.is_some()) {
                            matched_entries.push(DocumentEntry {
                                path: path.clone(),
                                account_path,
                                match_date: None,
                                rest_name: path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .map(|s| s.to_string()),
                                kind: DocumentKind::MatchedPdf,
                            });
                            continue;
                        }
                    }
                }
                if is_unbooked_document_path(path) {
                    unbooked_entries.push(DocumentEntry {
                        path: path.clone(),
                        account_path,
                        match_date: None,
                        rest_name: path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .map(|s| s.to_string()),
                        kind: DocumentKind::UnbookedPdf,
                    });
                } else {
                    issues.push(TreeIssue {
                        path: path.clone(),
                        reason: "undated document PDF must be filed in an unbooked/ subfolder or have matching metadata".to_string(),
                    });
                }
            } else {
                matched_entries.push(DocumentEntry {
                    path: path.clone(),
                    account_path,
                    match_date,
                    rest_name,
                    kind: DocumentKind::MatchedPdf,
                });
            }
            continue;
        }

        // Any other file is unexpected
        issues.push(TreeIssue {
            path: path.clone(),
            reason: "unexpected file outside document support directory".to_string(),
        });
    }

    // Check that each support dir has a corresponding PDF
    for support_dir in &support_dirs {
        if in_support_tree(support_dir, &support_dirs) {
            continue;
        }
        let name = support_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        let pdf = support_dir.with_file_name(format!("{name}.pdf"));
        if !pdf.exists() {
            issues.push(TreeIssue {
                path: support_dir.clone(),
                reason: "support directory has no matching document PDF".to_string(),
            });
        }
    }

    DocumentTree {
        matched_entries,
        unbooked_entries,
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write(dir: &std::path::Path, name: &str, content: &[u8]) -> PathBuf {
        let p = dir.join(name);
        std::fs::write(&p, content).unwrap();
        p
    }

    #[test]
    fn undated_pdf_outside_unbooked_is_an_issue() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/business/hosting/aws");
        std::fs::create_dir_all(&dir).unwrap();
        write(&dir, "invoice.pdf", b"pdf");

        let tree = scan_document_tree(tmp.path());

        assert!(tree.unbooked_entries.is_empty());
        assert_eq!(tree.issues.len(), 1);
        assert!(tree.issues[0].reason.contains("unbooked"));
    }

    #[test]
    fn missing_metadata_stands_in_for_absent_pdf() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/business/hosting/uberspace");
        std::fs::create_dir_all(&dir).unwrap();
        write(
            &dir,
            "2025-02-18-missing-rfmb.document.yml",
            b"covers: []\n",
        );

        let tree = scan_document_tree(tmp.path());

        assert!(tree.issues.is_empty());
        assert_eq!(tree.matched_entries.len(), 1);
        assert_eq!(tree.matched_entries[0].kind, DocumentKind::MissingMetadata);
    }

    #[test]
    fn empty_missing_metadata_stands_in_for_absent_pdf() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/business/hosting/uberspace");
        std::fs::create_dir_all(&dir).unwrap();
        write(&dir, "2025-02-18-missing.document.yml", b"");

        let tree = scan_document_tree(tmp.path());

        assert!(tree.issues.is_empty());
        assert_eq!(tree.matched_entries.len(), 1);
        assert_eq!(tree.matched_entries[0].kind, DocumentKind::MissingMetadata);
    }

    #[test]
    fn unpaired_metadata_without_missing_marker_is_an_issue() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/business/hosting/uberspace");
        std::fs::create_dir_all(&dir).unwrap();
        write(&dir, "2025-02-18-rfmb.document.yml", b"covers: []\n");

        let tree = scan_document_tree(tmp.path());

        assert_eq!(tree.issues.len(), 1);
        assert!(tree.issues[0].reason.contains("missing.document.yml"));
    }

    #[test]
    fn arbitrary_pdf_with_metadata_covers_is_matched() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("liabilities/health-insurance");
        std::fs::create_dir_all(&dir).unwrap();
        write(&dir, "for-2025.pdf", b"pdf");
        write(&dir, "for-2025.document.yml",
            b"covers:\n  - date: 2025-12-31\n    account: liabilities:health-insurance:2025\n    amount: 120.00\n    currency: EUR\n");

        let tree = scan_document_tree(tmp.path());

        assert!(tree.unbooked_entries.is_empty());
        assert_eq!(tree.matched_entries.len(), 1);
        assert!(tree.matched_entries[0].match_date.is_none());
    }

    #[test]
    fn support_dir_matches_pdf_stem_containing_a_dot() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/phone/service");
        std::fs::create_dir_all(&dir).unwrap();
        write(&dir, "2026-06-22-rechnung-FM.F26018179745.pdf", b"pdf");
        std::fs::create_dir_all(dir.join("2026-06-22-rechnung-FM.F26018179745")).unwrap();

        let tree = scan_document_tree(tmp.path());

        assert!(
            tree.issues.is_empty(),
            "support dir should match its PDF even though the stem contains a dot; \
             Path::with_extension(\"pdf\") splits on the last dot and would truncate \
             an invoice number like \"FM.F26018179745\" instead of appending \".pdf\" \
             to the full name. Got: {:?}",
            tree.issues
        );
    }

    #[test]
    fn invalid_metadata_is_reported_as_tree_issue() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/business/software");
        std::fs::create_dir_all(&dir).unwrap();
        write(&dir, "2026-01-01-suite.pdf", b"pdf");
        let meta = write(
            &dir,
            "2026-01-01-suite.document.yml",
            b"covers:\n  - amount:\n      nested: value\n",
        );

        let tree = scan_document_tree(tmp.path());

        assert_eq!(tree.issues.len(), 1);
        assert_eq!(tree.issues[0].path, meta.canonicalize().unwrap());
        assert!(tree.issues[0].reason.contains("invalid metadata"));
    }
}
