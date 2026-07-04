use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde_json::Value;
use std::path::Path;
use std::process::Command;

use crate::model::RequiredDocument;

pub fn load_transactions(journal_path: Option<&Path>) -> Result<Vec<Value>> {
    let mut cmd = Command::new("hledger");
    if let Some(path) = journal_path {
        cmd.arg("-f").arg(path);
    }
    cmd.args(["print", "--output-format", "json"]);

    let output = cmd
        .output()
        .context("running hledger print --output-format json")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("hledger failed: {stderr}");
    }
    let transactions: Vec<Value> =
        serde_json::from_slice(&output.stdout).context("parsing hledger JSON output")?;
    Ok(transactions)
}

pub fn posting_amount(posting: &Value) -> Option<f64> {
    let amounts = posting.get("pamount")?.as_array()?;
    if amounts.is_empty() {
        return None;
    }
    let quantity = amounts[0].get("aquantity")?;
    let fp = quantity.get("floatingPoint")?;
    let v = fp
        .as_f64()
        .or_else(|| fp.as_str().and_then(|s| s.parse().ok()))?;
    Some(v.abs())
}

pub fn posting_commodity(posting: &Value) -> Option<String> {
    let amounts = posting.get("pamount")?.as_array()?;
    if amounts.is_empty() {
        return None;
    }
    amounts[0]
        .get("acommodity")?
        .as_str()
        .map(|s| s.to_string())
}

pub fn posting_tags(posting: &Value) -> std::collections::HashMap<String, String> {
    let mut tags = std::collections::HashMap::new();
    let Some(ptags) = posting.get("ptags").and_then(|v| v.as_array()) else {
        return tags;
    };
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for tag in ptags {
        if let Some(arr) = tag.as_array() {
            if arr.len() >= 2 {
                let key = arr[0].as_str().unwrap_or("").to_string();
                let value = arr[1].as_str().unwrap_or("").to_string();
                if !seen.contains(&key) {
                    seen.insert(key.clone());
                    tags.insert(key, value);
                }
            }
        }
    }
    tags
}

fn account_matches_pattern(account: &str, pattern: &str) -> bool {
    account == pattern || account.starts_with(&format!("{pattern}:"))
}

fn counter_account_patterns(tags: &std::collections::HashMap<String, String>) -> Vec<String> {
    let mut patterns = Vec::new();
    for (key, value) in tags {
        if key == "document_check_counter_account"
            || key.starts_with("document_check_counter_account_")
        {
            let v = value.trim();
            if !v.is_empty() {
                patterns.push(v.to_string());
            }
        }
    }
    patterns
}

fn counter_account_matches(
    posting: &Value,
    transaction_postings: &[Value],
    tags: &std::collections::HashMap<String, String>,
) -> bool {
    let patterns = counter_account_patterns(tags);
    if patterns.is_empty() {
        return false;
    }
    for other in transaction_postings {
        if std::ptr::eq(other as *const Value, posting as *const Value) {
            continue;
        }
        let other_account = other.get("paccount").and_then(|v| v.as_str()).unwrap_or("");
        if patterns
            .iter()
            .any(|p| account_matches_pattern(other_account, p))
        {
            return true;
        }
    }
    false
}

fn has_required_prefix_tag(
    tags: &std::collections::HashMap<String, String>,
    required_tag_prefixes: &[String],
) -> bool {
    if required_tag_prefixes.is_empty() {
        return false;
    }
    tags.keys().any(|key| {
        required_tag_prefixes
            .iter()
            .any(|prefix| key.starts_with(prefix.as_str()))
    })
}

pub fn posting_requires_document(
    posting: &Value,
    transaction_postings: &[Value],
    required_tag_prefixes: &[String],
) -> bool {
    let tags = posting_tags(posting);
    match tags.get("document_check").map(String::as_str) {
        Some("required") => true,
        Some("exempt") => false,
        Some("counter_account") => counter_account_matches(posting, transaction_postings, &tags),
        None => has_required_prefix_tag(&tags, required_tag_prefixes),
        _ => false,
    }
}

pub fn validate_document_duties(transactions: &[Value]) -> Vec<String> {
    let mut errors = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for txn in transactions {
        let postings = txn
            .get("tpostings")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]);
        for posting in postings {
            let tags = posting_tags(posting);
            let account = posting
                .get("paccount")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if tags.get("document_check").map(String::as_str) == Some("counter_account")
                && counter_account_patterns(&tags).is_empty()
            {
                let key = format!("{account}::counter_account");
                if !seen.contains(&key) {
                    seen.insert(key);
                    errors.push(format!(
                            "  {account}: document_check:counter_account requires document_check_counter_account"
                        ));
                }
            }
        }
    }
    errors
}

pub fn iter_required_documents(
    transactions: &[Value],
    required_tag_prefixes: &[String],
) -> Vec<RequiredDocument> {
    let mut documents = Vec::new();

    for txn in transactions {
        let txn_date = txn
            .get("tdate")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let description = txn
            .get("tdescription")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let comment = txn
            .get("tcomment")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let transaction_index = txn.get("tindex").and_then(|v| v.as_i64()).unwrap_or(-1);

        let postings: Vec<&Value> = txn
            .get("tpostings")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().collect())
            .unwrap_or_default();

        let all_postings_slice: Vec<Value> = txn
            .get("tpostings")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let required_postings: Vec<(String, &Value)> = postings
            .iter()
            .filter(|p| posting_requires_document(p, &all_postings_slice, required_tag_prefixes))
            .map(|p| {
                let account = p
                    .get("paccount")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                (account, *p)
            })
            .collect();

        if required_postings.is_empty() {
            continue;
        }

        for (account, posting) in &required_postings {
            let posting_date_str = posting
                .get("pdate")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .unwrap_or(&txn_date);
            let posting_date = match NaiveDate::parse_from_str(posting_date_str, "%Y-%m-%d") {
                Ok(d) => d,
                Err(_) => continue,
            };
            documents.push(RequiredDocument {
                transaction_date: txn_date.clone(),
                description: description.clone(),
                comment: comment.clone(),
                account: account.clone(),
                posting_date,
                amount: posting_amount(posting),
                commodity: posting_commodity(posting),
                transaction_index,
            });
        }
    }

    documents
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn posting(account: &str, tags: &[(&str, &str)]) -> Value {
        let ptags: Vec<Value> = tags.iter().map(|(k, v)| json!([k, v])).collect();
        json!({"paccount": account, "ptags": ptags})
    }

    // --- posting_tags ---

    #[test]
    fn empty_ptags_have_no_defaults() {
        let tags = posting_tags(&posting("income:business:freelance:auteon", &[]));
        assert!(tags.is_empty());
    }

    #[test]
    fn posting_tags_come_from_hledger_json() {
        let p = posting(
            "income:business:reimbursements:health-insurance",
            &[
                ("document_check", "required"),
                ("document_check_match_role", "incoming_only"),
            ],
        );
        let tags = posting_tags(&p);
        assert_eq!(
            tags.get("document_check").map(String::as_str),
            Some("required")
        );
        assert_eq!(
            tags.get("document_check_match_role").map(String::as_str),
            Some("incoming_only")
        );
    }

    #[test]
    fn first_posting_tag_wins_over_later_duplicates() {
        let p = json!({
            "paccount": "income:business:reimbursements:auteon",
            "ptags": [["document_check", "exempt"], ["tax_label_de", "Erstattungen"], ["document_check", "required"]]
        });
        let tags = posting_tags(&p);
        assert_eq!(
            tags.get("document_check").map(String::as_str),
            Some("exempt")
        );
    }

    // --- posting_requires_document ---

    #[test]
    fn requires_document_defaults_to_false_without_tags_or_prefixes() {
        let p = posting("expenses:test", &[("tax_role", "business_expense")]);
        assert!(!posting_requires_document(
            &p,
            std::slice::from_ref(&p),
            &[]
        ));
    }

    #[test]
    fn requires_document_implied_by_configured_tag_prefix() {
        let p = posting("expenses:test", &[("tax_role", "business_expense")]);
        assert!(posting_requires_document(
            &p,
            std::slice::from_ref(&p),
            &["tax_".to_string()]
        ));
    }

    #[test]
    fn counter_account_mode_matches_counter_account() {
        let postings = vec![
            posting(
                "expenses:insurance:health:provider",
                &[("document_check", "exempt")],
            ),
            posting(
                "liabilities:health-insurance",
                &[
                    ("document_check", "counter_account"),
                    (
                        "document_check_counter_account",
                        "expenses:insurance:health:provider",
                    ),
                ],
            ),
        ];
        assert!(!posting_requires_document(&postings[0], &postings, &[]));
        assert!(posting_requires_document(&postings[1], &postings, &[]));
    }

    #[test]
    fn counter_account_mode_matches_any_named_counter_account() {
        let postings = vec![
            posting(
                "expenses:insurance:pension:provider",
                &[("document_check", "exempt")],
            ),
            posting(
                "liabilities:insurance-notices",
                &[
                    ("document_check", "counter_account"),
                    (
                        "document_check_counter_account_a",
                        "expenses:insurance:health:provider",
                    ),
                    (
                        "document_check_counter_account_b",
                        "expenses:insurance:pension:provider",
                    ),
                ],
            ),
        ];
        assert!(posting_requires_document(&postings[1], &postings, &[]));
    }

    #[test]
    fn counter_account_mode_does_not_match_settlement_posting() {
        let postings = vec![
            posting("assets:kontist:geschaeftskonto", &[]),
            posting(
                "liabilities:health-insurance",
                &[
                    ("document_check", "counter_account"),
                    (
                        "document_check_counter_account",
                        "expenses:insurance:health:provider",
                    ),
                ],
            ),
        ];
        assert!(!posting_requires_document(&postings[1], &postings, &[]));
    }

    // --- validate_document_duties ---

    #[test]
    fn validate_duties_requires_counter_account_tag() {
        let transactions = vec![json!({
            "tpostings": [posting(
                "liabilities:test-missing-source",
                &[("tax_role", "ignore"), ("document_check", "counter_account")],
            )]
        })];
        let errors = validate_document_duties(&transactions);
        assert_eq!(errors.len(), 1);
        assert!(errors[0]
            .contains("document_check:counter_account requires document_check_counter_account"));
    }

    #[test]
    fn validate_duties_accepts_tax_tags_without_document_check() {
        let transactions = vec![json!({
            "tpostings": [posting("expenses:test", &[("tax_role", "business_expense")])]
        })];
        assert_eq!(
            validate_document_duties(&transactions),
            Vec::<String>::new()
        );
    }

    // --- iter_required_documents ---

    #[test]
    fn iter_required_documents_includes_counter_account_posting() {
        let transactions = vec![json!({
            "tdate": "2026-01-10",
            "tdescription": "Health Insurance Notice",
            "tcomment": "",
            "tindex": 7,
            "tpostings": [
                {
                    "paccount": "expenses:insurance:health:provider",
                    "pamount": [{"acommodity": "EUR", "aquantity": {"floatingPoint": 220.0}}],
                    "ptags": [["document_check", "exempt"]],
                    "pdate": null
                },
                {
                    "paccount": "liabilities:health-insurance",
                    "pamount": [{"acommodity": "EUR", "aquantity": {"floatingPoint": -220.0}}],
                    "ptags": [
                        ["document_check", "counter_account"],
                        ["document_check_counter_account", "expenses:insurance:health:provider"]
                    ],
                    "pdate": null
                }
            ]
        })];
        let required = iter_required_documents(&transactions, &[]);
        assert_eq!(required.len(), 1);
        assert_eq!(required[0].account, "liabilities:health-insurance");
    }

    #[test]
    fn iter_required_documents_uses_configured_tag_prefixes() {
        let transactions = vec![json!({
            "tdate": "2026-01-10",
            "tdescription": "Expense",
            "tcomment": "",
            "tindex": 8,
            "tpostings": [{
                "paccount": "expenses:business:hosting",
                "pamount": [{"acommodity": "EUR", "aquantity": {"floatingPoint": 19.0}}],
                "ptags": [["tax_role", "business_expense"]],
                "pdate": null
            }]
        })];
        let required = iter_required_documents(&transactions, &["tax_".to_string()]);
        assert_eq!(required.len(), 1);
        assert_eq!(required[0].account, "expenses:business:hosting");
    }
}
