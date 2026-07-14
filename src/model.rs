use chrono::NaiveDate;
use std::path::PathBuf;

pub const PDF_SUFFIX: &str = ".pdf";
pub const METADATA_SUFFIX: &str = ".document.yml";
pub const CONFIG_FILENAME: &str = "hledger-document-check.toml";
pub const UNBOOKED_DIR: &str = "unbooked";
pub const MONEY_TOLERANCE: f64 = 0.01;

#[derive(Debug, Clone, PartialEq)]
pub struct RequiredDocument {
    pub transaction_date: String,
    pub description: String,
    pub comment: String,
    pub account: String,
    pub posting_date: NaiveDate,
    pub amount: Option<f64>,
    pub commodity: Option<String>,
    pub transaction_index: i64,
}

impl RequiredDocument {
    pub fn account_path(&self) -> PathBuf {
        self.account.split(':').collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DocumentKind {
    MatchedPdf,
    UnbookedPdf,
    MissingMetadata,
}

impl DocumentKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            DocumentKind::MatchedPdf => "matched_pdf",
            DocumentKind::UnbookedPdf => "unbooked_pdf",
            DocumentKind::MissingMetadata => "missing_metadata",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocumentEntry {
    pub path: PathBuf,
    pub account_path: PathBuf,
    pub match_date: Option<NaiveDate>,
    pub rest_name: Option<String>,
    pub kind: DocumentKind,
}

impl DocumentEntry {
    pub fn key(&self) -> Option<(String, NaiveDate)> {
        self.match_date
            .map(|d| (self.account_path.to_string_lossy().into_owned(), d))
    }
}

#[derive(Debug, Clone)]
pub struct TreeIssue {
    pub path: PathBuf,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct DuplicateFileGroup {
    pub paths: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct WrongAccountCover {
    pub path: PathBuf,
    pub declared_account: String,
    pub posting_date: NaiveDate,
    /// The account the file's own location implies, when that account does
    /// have a matching required transaction (a likely intended value).
    pub suggested_account: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PdfAmount {
    pub amount: f64,
    pub currency: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AmountMismatch {
    pub document_paths: Vec<PathBuf>,
    pub transaction_keys: Vec<(String, NaiveDate)>,
    pub document_total: f64,
    pub transaction_total: f64,
    pub document_label: String,
    pub transaction_label: String,
}

impl AmountMismatch {
    pub fn new(
        document_paths: Vec<PathBuf>,
        transaction_keys: Vec<(String, NaiveDate)>,
        document_total: f64,
        transaction_total: f64,
    ) -> Self {
        Self {
            document_paths,
            transaction_keys,
            document_total,
            transaction_total,
            document_label: "document total".to_string(),
            transaction_label: "transaction total".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AmountAuditSkip {
    pub reason: String,
    pub document_paths: Vec<PathBuf>,
    pub transaction_keys: Vec<(String, NaiveDate)>,
}

#[derive(Debug, Clone, Default)]
pub struct AmountAudit {
    pub mismatches: Vec<AmountMismatch>,
    pub skipped: Vec<AmountAuditSkip>,
    pub checked_groups: usize,
    pub skipped_missing_document_amount_groups: usize,
    pub skipped_mixed_document_currency_groups: usize,
    pub skipped_missing_transaction_amount_groups: usize,
    pub skipped_mixed_transaction_currency_groups: usize,
    pub skipped_currency_mismatch_groups: usize,
}

impl AmountAudit {
    pub fn skipped_groups(&self) -> usize {
        self.skipped_missing_document_amount_groups
            + self.skipped_mixed_document_currency_groups
            + self.skipped_missing_transaction_amount_groups
            + self.skipped_mixed_transaction_currency_groups
            + self.skipped_currency_mismatch_groups
    }
}

#[derive(Debug, Clone)]
pub struct DocumentCover {
    pub posting_date: Option<NaiveDate>,
    pub account_path: Option<String>,
    pub amount: Option<f64>,
    pub currency: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DocumentMetadata {
    pub covers: Vec<DocumentCover>,
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub due_date: Option<NaiveDate>,
}

#[derive(Debug, Default)]
pub struct DocumentTree {
    pub matched_entries: Vec<DocumentEntry>,
    pub unbooked_entries: Vec<DocumentEntry>,
    pub issues: Vec<TreeIssue>,
}
