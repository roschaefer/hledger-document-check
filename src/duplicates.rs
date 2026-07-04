use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::model::{DuplicateFileGroup, METADATA_SUFFIX};

fn iter_document_files(root: &Path) -> Vec<PathBuf> {
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
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            !name.starts_with('.') && !name.ends_with(METADATA_SUFFIX)
        })
        .collect();
    files.sort();
    files
}

fn file_digest(path: &Path) -> Option<Vec<u8>> {
    let mut hasher = Sha256::new();
    let mut file = std::fs::File::open(path).ok()?;
    std::io::copy(&mut file, &mut hasher).ok()?;
    Some(hasher.finalize().to_vec())
}

pub fn find_duplicate_files(root: &Path) -> Vec<DuplicateFileGroup> {
    let mut size_buckets: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    for path in iter_document_files(root) {
        if let Ok(meta) = std::fs::metadata(&path) {
            size_buckets.entry(meta.len()).or_default().push(path);
        }
    }

    let mut digest_buckets: HashMap<Vec<u8>, Vec<PathBuf>> = HashMap::new();
    for paths in size_buckets.values() {
        if paths.len() < 2 {
            continue;
        }
        for path in paths {
            if let Some(digest) = file_digest(path) {
                digest_buckets.entry(digest).or_default().push(path.clone());
            }
        }
    }

    let mut result: Vec<DuplicateFileGroup> = digest_buckets
        .into_values()
        .filter(|paths| paths.len() > 1)
        .map(|mut paths| {
            paths.sort();
            DuplicateFileGroup { paths }
        })
        .collect();
    result.sort_by(|a, b| a.paths.cmp(&b.paths));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn flags_distinct_copies() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/business/hosting/aws");
        std::fs::create_dir_all(&dir).unwrap();

        let original = dir.join("2026-01-01-invoice-a.pdf");
        let duplicate = dir.join("2026-02-01-invoice-b.pdf");
        let different = dir.join("2026-04-01-invoice-c.pdf");
        std::fs::write(&original, b"same-pdf-content").unwrap();
        std::fs::write(&duplicate, b"same-pdf-content").unwrap();
        std::fs::write(&different, b"different-pdf-content").unwrap();

        let groups = find_duplicate_files(tmp.path());

        assert_eq!(groups.len(), 1);
        let mut expected = vec![
            original.canonicalize().unwrap(),
            duplicate.canonicalize().unwrap(),
        ];
        expected.sort();
        assert_eq!(groups[0].paths, expected);
    }
}
