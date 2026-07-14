use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::model::CONFIG_FILENAME;

pub const DEFAULT_CONFIG: &str = r#"[ledger]
# journal = "../ledger/hledger.journal"

[documents]
# root = "."

[requirements]
tag_prefixes = []
# tag_prefixes = ["tax_"]

[overdue]
after_days = 14

[checks]
invalid-configuration = "fail"
missing-document-coverage = "fail"
unbooked-documents = "warn"
overdue-unbooked-documents = "fail"
unmatched-documents = "fail"
unexpected-files = "fail"
duplicate-files = "fail"
amount-mismatches = "fail"
amount-audit-skips = "ignore"
missing-document-placeholders = "ignore"
ambiguous-transaction-groups = "warn"
redundant-metadata = "warn"
unresolvable-cover-metadata = "fail"

[enrich_journal]
# Prefix written into emitted document: tags.
# Use an absolute path when the consuming tool has no separate document-root setting.
# document_tag_root = "/absolute/path/to/documents"
"#;

#[derive(Debug, Default, Clone)]
pub struct DocumentCheckConfig {
    pub journal: Option<String>,
    pub documents: Option<String>,
    pub tag_prefixes: Vec<String>,
    pub overdue_after_days: Option<u32>,
    pub checks: std::collections::HashMap<String, String>,
    pub document_tag_root: Option<String>,
}

#[derive(Deserialize, Default)]
struct RawConfig {
    ledger: Option<RawLedger>,
    documents: Option<RawDocuments>,
    requirements: Option<RawRequirements>,
    overdue: Option<RawOverdue>,
    checks: Option<std::collections::HashMap<String, toml::Value>>,
    enrich_journal: Option<RawEnrichJournal>,
}

#[derive(Deserialize, Default)]
struct RawLedger {
    journal: Option<String>,
}

#[derive(Deserialize, Default)]
struct RawDocuments {
    root: Option<String>,
}

#[derive(Deserialize, Default)]
struct RawRequirements {
    tag_prefixes: Option<Vec<String>>,
}

#[derive(Deserialize, Default)]
struct RawOverdue {
    after_days: Option<u32>,
}

#[derive(Deserialize, Default)]
struct RawEnrichJournal {
    document_tag_root: Option<String>,
}

pub fn discover_config_path(
    documents: Option<&str>,
    explicit_config: Option<&str>,
) -> Option<PathBuf> {
    if let Some(path) = explicit_config {
        return Some(PathBuf::from(path));
    }
    let root = PathBuf::from(documents.unwrap_or("."));
    let candidate = root.join(CONFIG_FILENAME);
    if candidate.exists() {
        Some(candidate)
    } else {
        None
    }
}

pub fn load_config(path: Option<&Path>) -> Result<DocumentCheckConfig> {
    let Some(path) = path else {
        return Ok(DocumentCheckConfig::default());
    };

    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading config {}", path.display()))?;
    let raw: RawConfig =
        toml::from_str(&text).with_context(|| format!("parsing config {}", path.display()))?;

    let base_dir = path.parent().unwrap_or(Path::new("."));

    let journal = raw
        .ledger
        .as_ref()
        .and_then(|l| l.journal.as_deref())
        .map(|j| resolve_path_string(j, base_dir));

    let documents = raw
        .documents
        .as_ref()
        .and_then(|d| d.root.as_deref())
        .map(|r| resolve_path_string(r, base_dir));

    let tag_prefixes = raw
        .requirements
        .as_ref()
        .and_then(|r| r.tag_prefixes.clone())
        .unwrap_or_default();

    let overdue_after_days = raw.overdue.as_ref().and_then(|o| o.after_days);

    let document_tag_root = raw
        .enrich_journal
        .as_ref()
        .and_then(|e| e.document_tag_root.clone());

    let mut checks = std::collections::HashMap::new();
    for (name, value) in raw.checks.unwrap_or_default() {
        let level = match &value {
            toml::Value::String(s) => s.clone(),
            other => bail!("Config value checks.{name} must be a string, got {other}"),
        };
        if !["fail", "warn", "ignore"].contains(&level.as_str()) {
            bail!("Config value checks.{name} must be \"fail\", \"warn\", or \"ignore\"");
        }
        checks.insert(name, level);
    }

    Ok(DocumentCheckConfig {
        journal,
        documents,
        tag_prefixes,
        overdue_after_days,
        checks,
        document_tag_root,
    })
}

fn resolve_path_string(value: &str, base_dir: &Path) -> String {
    let p = Path::new(value);
    if p.is_absolute() {
        value.to_string()
    } else {
        base_dir
            .join(p)
            .canonicalize()
            .unwrap_or_else(|_| base_dir.join(p))
            .to_string_lossy()
            .into_owned()
    }
}

pub fn write_default_config(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating directory {}", parent.display()))?;
    }
    std::fs::write(path, DEFAULT_CONFIG)
        .with_context(|| format!("writing config to {}", path.display()))?;
    Ok(())
}
