use chrono::NaiveDate;
use std::fmt;
use std::path::Path;

use crate::document_paths::{metadata_path_for_document, parse_metadata_filename};
use crate::model::{DocumentCover, DocumentMetadata, PdfAmount, METADATA_SUFFIX};

#[derive(Debug)]
pub struct DocumentMetadataError(pub String);

impl fmt::Display for DocumentMetadataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for DocumentMetadataError {}

pub(crate) fn yaml_to_string(v: &serde_yaml::Value) -> Option<String> {
    match v {
        serde_yaml::Value::String(s) => Some(s.clone()),
        serde_yaml::Value::Null => None,
        serde_yaml::Value::Bool(b) => Some(b.to_string()),
        serde_yaml::Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

/// Parse a YAML value as a date. Handles both YAML native timestamps and ISO strings.
pub(crate) fn parse_yaml_date(
    v: &serde_yaml::Value,
) -> Result<Option<NaiveDate>, DocumentMetadataError> {
    match v {
        serde_yaml::Value::Null => Ok(None),
        serde_yaml::Value::String(s) => {
            // Strip time component if present (e.g. "2024-01-15T00:00:00Z" → "2024-01-15")
            let date_part = s.split('T').next().unwrap_or(s).trim();
            NaiveDate::parse_from_str(date_part, "%Y-%m-%d")
                .map(Some)
                .map_err(|_| DocumentMetadataError(format!("invalid date: {s:?}")))
        }
        _ => Err(DocumentMetadataError(format!("invalid date: {v:?}"))),
    }
}

fn parse_yaml_amount(v: &serde_yaml::Value) -> Result<Option<f64>, DocumentMetadataError> {
    match v {
        serde_yaml::Value::Null => Ok(None),
        serde_yaml::Value::String(s) if s.is_empty() => Ok(None),
        serde_yaml::Value::String(s) => s
            .parse::<f64>()
            .map(|f| Some(f.abs()))
            .map_err(|_| DocumentMetadataError(format!("invalid amount: {s:?}"))),
        serde_yaml::Value::Number(n) => Ok(n.as_f64().map(|f| f.abs())),
        other => Err(DocumentMetadataError(format!("invalid amount: {other:?}"))),
    }
}

const ALLOWED_TOP_LEVEL_KEYS: &[&str] = &[
    "date",
    "account",
    "amount",
    "currency",
    "description",
    "document_type",
    "due_date",
    "notes",
    "covers",
];

const ALLOWED_COVER_KEYS: &[&str] = &["date", "account", "amount", "currency", "description"];
// "description" remains allowed so existing metadata files don't break validation.

fn validate_keys(
    mapping: &serde_yaml::Mapping,
    allowed: &[&str],
    context: &str,
) -> Result<(), DocumentMetadataError> {
    for key in mapping.keys() {
        let k = key
            .as_str()
            .ok_or_else(|| DocumentMetadataError(format!("{context}: non-string key")))?;
        if !allowed.contains(&k) {
            return Err(DocumentMetadataError(format!(
                "{context}: additional properties are not allowed ('{k}' was unexpected)"
            )));
        }
    }
    Ok(())
}

pub fn load_document_metadata(path: &Path) -> Result<DocumentMetadata, DocumentMetadataError> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| DocumentMetadataError(format!("cannot read {}: {e}", path.display())))?;

    let raw: serde_yaml::Value = serde_yaml::from_str(&text)
        .map_err(|e| DocumentMetadataError(format!("invalid YAML: {e}")))?;

    let mapping = match &raw {
        serde_yaml::Value::Null => {
            // Empty file is valid — treat as empty mapping
            return Ok(DocumentMetadata {
                covers: vec![],
                amount: None,
                currency: None,
                due_date: None,
            });
        }
        serde_yaml::Value::Mapping(m) => m,
        _ => {
            return Err(DocumentMetadataError(
                "metadata root must be a mapping".into(),
            ))
        }
    };

    validate_keys(mapping, ALLOWED_TOP_LEVEL_KEYS, "metadata root")?;

    let get = |key: &str| -> Option<&serde_yaml::Value> {
        mapping.get(serde_yaml::Value::String(key.to_string()))
    };

    let (default_date, _) = parse_metadata_filename(path);

    let top_level_date = if let Some(v) = get("date") {
        parse_yaml_date(v)?
    } else {
        None
    };

    let top_level_amount = if let Some(v) = get("amount") {
        parse_yaml_amount(v)?
    } else {
        None
    };

    let top_level_currency = get("currency").and_then(yaml_to_string);
    let top_level_account = get("account").and_then(yaml_to_string);

    let due_date = if let Some(v) = get("due_date") {
        parse_yaml_date(v)?
    } else {
        None
    };

    // Build covers
    let mut covers: Vec<DocumentCover> = Vec::new();

    if let Some(covers_val) = get("covers") {
        let arr = covers_val
            .as_sequence()
            .ok_or_else(|| DocumentMetadataError("covers must be an array".into()))?;

        for (i, item) in arr.iter().enumerate() {
            let cover_mapping = item
                .as_mapping()
                .ok_or_else(|| DocumentMetadataError(format!("covers[{i}]: must be a mapping")))?;

            validate_keys(cover_mapping, ALLOWED_COVER_KEYS, &format!("covers[{i}]"))?;

            let get_c = |key: &str| -> Option<&serde_yaml::Value> {
                cover_mapping.get(serde_yaml::Value::String(key.to_string()))
            };

            let posting_date: Option<NaiveDate> = if let Some(v) = get_c("date") {
                match parse_yaml_date(v)? {
                    Some(d) => Some(d),
                    None => default_date.or(top_level_date),
                }
            } else {
                top_level_date.or(default_date)
            };

            let row_account = get_c("account").and_then(yaml_to_string);
            let account_path = row_account.as_deref().map(|a| a.replace(':', "/"));
            let row_currency = get_c("currency")
                .and_then(yaml_to_string)
                .or_else(|| top_level_currency.clone());
            let amount = if let Some(v) = get_c("amount") {
                parse_yaml_amount(v)?
            } else {
                None
            };

            covers.push(DocumentCover {
                location: format!("covers[{i}]"),
                posting_date,
                account_path,
                amount,
                currency: row_currency,
            });
        }
    } else {
        // No `covers` array. If top-level cover-like fields exist, synthesize a single cover.
        let has_cover_fields = top_level_date.is_some()
            || top_level_account.is_some()
            || top_level_amount.is_some()
            || get("description").is_some();

        if has_cover_fields {
            let posting_date = top_level_date.or(default_date);
            let account_path = top_level_account.as_deref().map(|a| a.replace(':', "/"));
            covers.push(DocumentCover {
                location: "top-level".to_string(),
                posting_date,
                account_path,
                amount: top_level_amount,
                currency: top_level_currency.clone(),
            });
        }
    }

    Ok(DocumentMetadata {
        covers,
        amount: top_level_amount,
        currency: top_level_currency,
        due_date,
    })
}

pub fn metadata_for_document(
    path: &Path,
) -> Result<Option<DocumentMetadata>, DocumentMetadataError> {
    let metadata_path = if path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.ends_with(METADATA_SUFFIX))
        .unwrap_or(false)
    {
        path.to_path_buf()
    } else {
        metadata_path_for_document(path)
    };

    if !metadata_path.exists() {
        return Ok(None);
    }
    load_document_metadata(&metadata_path).map(Some)
}

pub fn metadata_amount_for_document(
    path: &Path,
) -> Result<Option<PdfAmount>, DocumentMetadataError> {
    let metadata_path = if path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.ends_with(METADATA_SUFFIX))
        .unwrap_or(false)
    {
        path.to_path_buf()
    } else {
        metadata_path_for_document(path)
    };

    if !metadata_path.exists() {
        return Ok(None);
    }

    let meta = load_document_metadata(&metadata_path)?;

    if let Some(amt) = meta.amount {
        return Ok(Some(PdfAmount {
            amount: amt,
            currency: meta.currency.clone(),
        }));
    }

    let cover_amounts: Vec<(f64, Option<String>)> = meta
        .covers
        .iter()
        .filter_map(|c| c.amount.map(|a| (a, c.currency.clone())))
        .collect();

    if cover_amounts.is_empty() {
        return Ok(None);
    }

    let currencies: std::collections::HashSet<&str> = cover_amounts
        .iter()
        .filter_map(|(_, c)| c.as_deref())
        .collect();

    if currencies.len() > 1 {
        return Ok(None);
    }

    let total: f64 = cover_amounts.iter().map(|(a, _)| a).sum();
    let currency = cover_amounts
        .iter()
        .find_map(|(_, c)| c.clone())
        .or_else(|| meta.currency.clone());

    Ok(Some(PdfAmount {
        amount: total,
        currency,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write(dir: &std::path::Path, name: &str, text: &str) -> std::path::PathBuf {
        let p = dir.join(name);
        std::fs::write(&p, text).unwrap();
        p
    }

    // --- load_document_metadata ---

    #[test]
    fn metadata_cover_defaults_date_from_filename() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/business/software/ai/anthropic");
        std::fs::create_dir_all(&dir).unwrap();
        let meta = write(
            &dir,
            "2026-03-11-invoice.document.yml",
            "covers:\n  - amount: 19.00\n    currency: USD\n",
        );
        let loaded = load_document_metadata(&meta).unwrap();
        assert_eq!(
            loaded.covers[0].posting_date,
            Some(NaiveDate::from_ymd_opt(2026, 3, 11).unwrap())
        );
        assert!(loaded.covers[0].account_path.is_none());
    }

    #[test]
    fn metadata_cover_can_explicitly_set_account() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("liabilities/health-insurance");
        std::fs::create_dir_all(&dir).unwrap();
        let meta = write(&dir, "2025-01-01.document.yml",
            "covers:\n  - date: 2025-01-01\n    account: liabilities:health-insurance:2023\n    amount: 69.26\n    currency: EUR\n");
        let loaded = load_document_metadata(&meta).unwrap();
        assert_eq!(
            loaded.covers[0].account_path.as_deref(),
            Some("liabilities/health-insurance/2023")
        );
    }

    #[test]
    fn metadata_stores_document_total_and_cover_amount() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/business/software");
        std::fs::create_dir_all(&dir).unwrap();
        let meta = write(&dir, "2026-01-01-suite.document.yml",
            "amount: 100.00\ncurrency: EUR\ncovers:\n  - date: 2026-01-01\n    account: expenses:business:software\n    amount: 60.00\n");
        let loaded = load_document_metadata(&meta).unwrap();
        assert_eq!(loaded.amount, Some(100.0));
        assert_eq!(loaded.currency.as_deref(), Some("EUR"));
        assert_eq!(loaded.covers[0].amount, Some(60.0));
        assert_eq!(loaded.covers[0].currency.as_deref(), Some("EUR"));
    }

    #[test]
    fn metadata_top_level_shorthand_synthesises_cover() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/business/hosting/aws");
        std::fs::create_dir_all(&dir).unwrap();
        let meta = write(&dir, "2022-06-03-aws.document.yml",
            "date: 2022-06-03\naccount: Expenses:Business:Hosting:AWS\namount: 2.99\ncurrency: EUR\n");
        let loaded = load_document_metadata(&meta).unwrap();
        assert_eq!(loaded.covers.len(), 1);
        assert_eq!(
            loaded.covers[0].posting_date,
            Some(NaiveDate::from_ymd_opt(2022, 6, 3).unwrap())
        );
        assert_eq!(
            loaded.covers[0].account_path.as_deref(),
            Some("Expenses/Business/Hosting/AWS")
        );
        assert_eq!(loaded.covers[0].amount, Some(2.99));
        assert_eq!(loaded.covers[0].currency.as_deref(), Some("EUR"));
    }

    #[test]
    fn metadata_top_level_amount_alone_synthesises_cover() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/business/domains");
        std::fs::create_dir_all(&dir).unwrap();
        let meta = write(
            &dir,
            "2026-03-01-domain-a.document.yml",
            "amount: 4.00\ncurrency: EUR\n",
        );
        let loaded = load_document_metadata(&meta).unwrap();
        assert_eq!(loaded.covers.len(), 1);
        assert_eq!(
            loaded.covers[0].posting_date,
            Some(NaiveDate::from_ymd_opt(2026, 3, 1).unwrap())
        );
        assert!(loaded.covers[0].account_path.is_none());
        assert_eq!(loaded.covers[0].amount, Some(4.00));
        assert_eq!(loaded.covers[0].currency.as_deref(), Some("EUR"));
    }

    #[test]
    fn metadata_due_date_is_parsed() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp
            .path()
            .join("income/business/freelance/customer/unbooked");
        std::fs::create_dir_all(&dir).unwrap();
        let meta = write(&dir, "invoice.document.yml", "due_date: 2026-04-15\n");
        let loaded = load_document_metadata(&meta).unwrap();
        assert_eq!(
            loaded.due_date,
            Some(NaiveDate::from_ymd_opt(2026, 4, 15).unwrap())
        );
    }

    #[test]
    fn metadata_rejects_unknown_fields() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/business/software");
        std::fs::create_dir_all(&dir).unwrap();
        let meta = write(
            &dir,
            "2026-01-01-suite.document.yml",
            "amount: 100.00\nunexpected: true\n",
        );
        assert!(load_document_metadata(&meta).is_err());
    }

    #[test]
    fn metadata_rejects_covers_that_is_not_a_sequence() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("expenses/business/software");
        std::fs::create_dir_all(&dir).unwrap();
        let meta = write(
            &dir,
            "2026-01-01-suite.document.yml",
            "covers:\n  amount: 100.00\n",
        );
        let err = load_document_metadata(&meta).unwrap_err();
        assert!(err.to_string().contains("covers"));
    }
}
