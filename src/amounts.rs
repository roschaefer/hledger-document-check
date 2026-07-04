use fancy_regex::Regex;
use std::path::Path;
use std::sync::OnceLock;

use crate::metadata::metadata_amount_for_document;
use crate::model::{PdfAmount, MONEY_TOLERANCE};

static AMOUNT_TOKEN_RE: OnceLock<Regex> = OnceLock::new();

fn amount_token_re() -> &'static Regex {
    AMOUNT_TOKEN_RE.get_or_init(|| {
        Regex::new(
            r"(?<!\d)(?:USD|EUR|CHF|GBP)?\s*(?:[$€£])?\s*-?\d+(?:[.,]\d{3})*(?:[.,]\d{2})?\s*(?:USD|EUR|CHF|GBP|[$€£])?(?!\d)",
        )
        .unwrap()
    })
}

const AMOUNT_LINE_POSITIVE_HINTS: &[&str] = &[
    "amount due",
    "total due",
    "grand total",
    "invoice total",
    "total amount",
    "amount paid",
    "paid amount",
    "rechnungsbetrag",
    "gesamtbetrag",
    "gesamt",
    "zu zahlen",
    "fälliger betrag",
    "total",
];

const AMOUNT_LINE_VALUE_HINTS: &[&str] = &[
    "amount due",
    "total due",
    "grand total",
    "invoice total",
    "total amount",
    "amount paid",
    "paid amount",
    "rechnungsbetrag",
    "gesamtbetrag",
    "gesamt",
    "zu zahlen",
    "fälliger betrag",
    // "total" is excluded (not in VALUE_HINTS, only in POSITIVE_HINTS)
];

const AMOUNT_LINE_NEGATIVE_HINTS: &[&str] = &[
    "subtotal",
    "sub total",
    "net",
    "ust",
    "mwst",
    "vat",
    "tax",
    "discount",
    "usage",
    "unit price",
    "price per",
    "quantity",
    "qty",
];

const ZERO_AMOUNT_SCORE_PENALTY: i32 = 15;

fn normalize_amount_token(token: &str) -> Option<f64> {
    let cleaned = token
        .to_uppercase()
        .replace("USD", "")
        .replace("EUR", "")
        .replace("CHF", "")
        .replace("GBP", "")
        .replace(['$', '€', '£'], "");
    let cleaned = cleaned.trim().replace(' ', "");

    if cleaned.is_empty() {
        return None;
    }

    let last_comma = cleaned.rfind(',');
    let last_dot = cleaned.rfind('.');

    let normalized = match (last_comma, last_dot) {
        (Some(c), Some(d)) => {
            if c > d {
                // comma is decimal separator: 1.234,56
                cleaned.replace('.', "").replace(',', ".")
            } else {
                // dot is decimal separator: 1,234.56
                cleaned.replace(',', "")
            }
        }
        (Some(_), None) => {
            // only comma: check if it's decimal (exactly 2 digits after)
            let parts: Vec<&str> = cleaned.split(',').collect();
            if parts.len() == 2 && parts[1].len() == 2 {
                cleaned.replace(',', ".")
            } else {
                cleaned.replace(',', "")
            }
        }
        (None, Some(_)) => {
            // only dot: check if it's decimal (exactly 2 digits after)
            let parts: Vec<&str> = cleaned.split('.').collect();
            if parts.len() == 2 && parts[1].len() == 2 {
                cleaned.clone()
            } else {
                cleaned.replace('.', "")
            }
        }
        (None, None) => cleaned.clone(),
    };

    normalized.parse::<f64>().ok().map(|f| f.abs())
}

fn detect_currency(token: &str, line: &str) -> Option<String> {
    let joined = format!("{line} {token}").to_uppercase();
    for currency in ["EUR", "USD", "CHF", "GBP"] {
        if joined.contains(currency) {
            return Some(currency.to_string());
        }
    }
    if joined.contains('€') {
        return Some("EUR".to_string());
    }
    if joined.contains('$') {
        return Some("USD".to_string());
    }
    if joined.contains('£') {
        return Some("GBP".to_string());
    }
    None
}

pub fn parse_pdf_amount(text: &str) -> Option<PdfAmount> {
    let re = amount_token_re();
    let mut best_score: Option<i32> = None;
    let mut best_match: Option<PdfAmount> = None;

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        // Collect amount tokens: (amount, token_str, start_pos)
        let mut amount_tokens: Vec<(f64, String, usize)> = Vec::new();
        for m in re.find_iter(line).flatten() {
            let token = m.as_str();
            if let Some(amount) = normalize_amount_token(token) {
                amount_tokens.push((amount, token.to_string(), m.start()));
            }
        }

        if amount_tokens.is_empty() {
            continue;
        }

        let lowered = line.to_lowercase();
        let mut score: i32 = 0;

        for hint in AMOUNT_LINE_POSITIVE_HINTS {
            if lowered.contains(hint) {
                score += 10;
            }
        }
        for hint in AMOUNT_LINE_NEGATIVE_HINTS {
            if lowered.contains(hint) {
                score -= 6;
            }
        }

        let leading_keywords = [
            "amount",
            "betrag",
            "rechnungsbetrag",
            "grand total",
            "total amount",
            "amount due",
        ];
        if leading_keywords.iter().any(|kw| lowered.starts_with(kw)) {
            score += 10;
        } else if amount_tokens.len() == 1 {
            score += 3;
        }

        if score <= 0 {
            continue;
        }

        // Find candidate: prefer token after the last value hint position
        let value_hint_positions: Vec<usize> = AMOUNT_LINE_VALUE_HINTS
            .iter()
            .filter_map(|hint| lowered.find(hint).map(|pos| pos + hint.len()))
            .collect();

        let (candidate_amount, candidate_token, _) = if !value_hint_positions.is_empty() {
            let hint_end = *value_hint_positions.iter().min().unwrap();
            let hinted: Vec<&(f64, String, usize)> = amount_tokens
                .iter()
                .filter(|(_, _, start)| *start >= hint_end)
                .collect();
            let pool_ref: Vec<&(f64, String, usize)> = if !hinted.is_empty() {
                hinted
            } else {
                amount_tokens.iter().collect()
            };
            pool_ref
                .into_iter()
                .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
                .cloned()
                .unwrap_or_else(|| amount_tokens[0].clone())
        } else {
            amount_tokens
                .iter()
                .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
                .cloned()
                .unwrap_or_else(|| amount_tokens[0].clone())
        };

        if candidate_amount <= MONEY_TOLERANCE {
            score -= ZERO_AMOUNT_SCORE_PENALTY;
        }
        if score <= 0 {
            continue;
        }

        let candidate = PdfAmount {
            amount: candidate_amount,
            currency: detect_currency(&candidate_token, line),
        };

        let is_better = match (best_score, &best_match) {
            (None, _) => true,
            (Some(bs), _) if score > bs => true,
            (Some(bs), Some(bm)) if score == bs && candidate_amount > bm.amount => true,
            _ => false,
        };

        if is_better {
            best_score = Some(score);
            best_match = Some(candidate);
        }
    }

    best_match
}

pub fn extract_pdf_amount(path: &Path) -> Option<PdfAmount> {
    let text = match pdf_extract::extract_text(path) {
        Ok(t) => t,
        Err(_) => return None,
    };
    parse_pdf_amount(&text)
}

pub fn document_amount_for_document(path: &Path) -> Option<PdfAmount> {
    match metadata_amount_for_document(path) {
        Ok(Some(a)) => return Some(a),
        Ok(None) => {}
        Err(_) => {}
    }
    extract_pdf_amount(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> String {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pdf_text")
            .join(name);
        std::fs::read_to_string(path).expect("fixture not found")
    }

    #[test]
    fn parses_english_document_total() {
        let result = parse_pdf_amount(&fixture("english_document_total.txt")).unwrap();
        assert!((result.amount - 11.90).abs() < 0.001);
    }

    #[test]
    fn parses_german_rechnungsbetrag() {
        let result = parse_pdf_amount(&fixture("german_rechnungsbetrag.txt")).unwrap();
        assert!((result.amount - 126.75).abs() < 0.001);
    }

    #[test]
    fn prefers_total_over_subtotal_and_vat() {
        let result = parse_pdf_amount(&fixture("subtotal_and_total.txt")).unwrap();
        assert!((result.amount - 119.00).abs() < 0.001);
    }

    #[test]
    fn prefers_positive_total_over_zero_amount_due() {
        let text = "Invoice FUN-2026-0001\nTotal                                  $19.00\nAmount due                              $0.00";
        let result = parse_pdf_amount(text).unwrap();
        assert!((result.amount - 19.00).abs() < 0.001);
    }

    #[test]
    fn prefers_amount_after_grand_total_label_over_prior_address_number() {
        let text = "Santa Clara, CA 95054 USA                                                                             Grand Total (USD)                         $15.09";
        let result = parse_pdf_amount(text).unwrap();
        assert!((result.amount - 15.09).abs() < 0.001);
    }
}
