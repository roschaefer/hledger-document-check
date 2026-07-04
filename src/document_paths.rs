use chrono::NaiveDate;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::model::{METADATA_SUFFIX, PDF_SUFFIX, UNBOOKED_DIR};

static DATE_PREFIX_RE: OnceLock<Regex> = OnceLock::new();

fn date_prefix_re() -> &'static Regex {
    DATE_PREFIX_RE.get_or_init(|| {
        Regex::new(r"^(?P<date>\d{4}-\d{2}-\d{2})(?:(?:-|__)(?P<rest>.+))?$").unwrap()
    })
}

pub fn parse_document_filename(path: &Path) -> (Option<NaiveDate>, Option<String>) {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext != "pdf" {
        return (None, None);
    }
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    parse_dated_stem(stem)
}

pub fn parse_metadata_filename(path: &Path) -> (Option<NaiveDate>, Option<String>) {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if !name.ends_with(METADATA_SUFFIX) {
        return (None, None);
    }
    let stem = &name[..name.len() - METADATA_SUFFIX.len()];
    parse_dated_stem(stem)
}

pub fn parse_dated_stem(stem: &str) -> (Option<NaiveDate>, Option<String>) {
    let re = date_prefix_re();
    let Some(caps) = re.captures(stem) else {
        return (None, None);
    };
    let date_str = &caps["date"];
    let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") else {
        return (None, None);
    };
    let rest = caps.name("rest").map(|m| m.as_str().to_string());
    (Some(date), rest)
}

pub fn strip_match_prefix(name: &str) -> String {
    let path = Path::new(name);
    let (match_date, rest) = parse_document_filename(path);
    if match_date.is_none() {
        return name.to_string();
    }
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if let Some(r) = rest {
        format!("{r}.{ext}")
    } else {
        name.to_string()
    }
}

pub fn matched_name(match_date: NaiveDate, original_name: &str) -> String {
    let bare_name = strip_match_prefix(original_name);
    format!("{}-{bare_name}", match_date.format("%Y-%m-%d"))
}

pub fn account_path_from_dir(directory: &Path, root: &Path) -> PathBuf {
    directory
        .strip_prefix(root)
        .unwrap_or(directory)
        .to_path_buf()
}

pub fn is_unbooked_document_path(path: &Path) -> bool {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext != "pdf" {
        return false;
    }
    path.parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        == Some(UNBOOKED_DIR)
}

pub fn account_dir_for_document(path: &Path) -> PathBuf {
    if is_unbooked_document_path(path) {
        path.parent()
            .and_then(|p| p.parent())
            .unwrap_or(path)
            .to_path_buf()
    } else {
        path.parent().unwrap_or(path).to_path_buf()
    }
}

pub fn metadata_path_for_document(path: &Path) -> PathBuf {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let stem = if let Some(s) = name.strip_suffix(PDF_SUFFIX) {
        s
    } else {
        name
    };
    path.with_file_name(format!("{stem}{METADATA_SUFFIX}"))
}

pub fn document_path_for_metadata(path: &Path) -> PathBuf {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let stem = &name[..name.len() - METADATA_SUFFIX.len()];
    path.with_file_name(format!("{stem}{PDF_SUFFIX}"))
}

pub fn is_missing_metadata(path: &Path) -> bool {
    let (_, rest) = parse_metadata_filename(path);
    rest.as_deref()
        .map(|r| r == "missing" || r.starts_with("missing-"))
        .unwrap_or(false)
}

pub fn support_dir_for_document(path: &Path) -> PathBuf {
    path.with_extension("")
}
