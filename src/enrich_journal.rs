use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::comparison::load_document_journal_diff;
use crate::journal::{load_transactions, validate_document_duties};
use crate::model::METADATA_SUFFIX;

pub struct EnrichArgs {
    pub journal: Option<PathBuf>,
    pub documents: PathBuf,
    pub document_tag_root: Option<String>,
    pub require_document_for_tag_prefixes: Vec<String>,
}

fn load_print_blocks(journal_path: Option<&Path>) -> Result<Vec<String>> {
    let mut cmd = Command::new("hledger");
    if let Some(path) = journal_path {
        cmd.arg("-f").arg(path);
    }
    cmd.arg("print");
    let output = cmd.output().context("running hledger print")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("hledger print failed: {stderr}");
    }
    let text = String::from_utf8(output.stdout).context("hledger print output is not UTF-8")?;
    let blocks: Vec<String> = text
        .trim()
        .split("\n\n")
        .filter(|b| !b.trim().is_empty())
        .map(|b| b.to_string())
        .collect();
    Ok(blocks)
}

fn is_metadata_document_tag_line(line: &str) -> bool {
    let stripped = line.trim();
    if !stripped.starts_with("; document:") {
        return false;
    }
    let tag_value = stripped[";\u{20}document:".len()..].trim();
    tag_value.ends_with(METADATA_SUFFIX)
}

fn inject_document_tags(block: &str, document_tags: &[String]) -> String {
    let lines: Vec<&str> = block
        .lines()
        .filter(|l| !is_metadata_document_tag_line(l))
        .collect();

    if document_tags.is_empty() {
        return lines.join("\n");
    }

    // Find first posting line: starts with "    " but not "    ;"
    let insert_at = lines[1..]
        .iter()
        .enumerate()
        .find(|(_, line)| line.starts_with("    ") && !line.starts_with("    ;"))
        .map(|(i, _)| i + 1)
        .unwrap_or(lines.len());

    let doc_lines: Vec<String> = document_tags
        .iter()
        .map(|tag| format!("    ; document:{tag}"))
        .collect();

    let mut result = lines[..insert_at].to_vec();
    for line in &doc_lines {
        result.push(line.as_str());
    }
    result.extend_from_slice(&lines[insert_at..]);
    result.join("\n")
}

fn document_tag_value(
    path: &Path,
    document_root: &Path,
    document_tag_root: Option<&str>,
) -> String {
    match document_tag_root {
        None => path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string(),
        Some(root) => {
            let rel = path.strip_prefix(document_root).unwrap_or(path);
            Path::new(root).join(rel).to_string_lossy().into_owned()
        }
    }
}

pub fn run_enrich(args: EnrichArgs) -> i32 {
    let transactions = match load_transactions(args.journal.as_deref()) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("ERROR: {e}");
            return 2;
        }
    };

    let duty_errors = validate_document_duties(&transactions);
    if !duty_errors.is_empty() {
        for error in &duty_errors {
            println!("{error}");
        }
        return 2;
    }

    let diff = load_document_journal_diff(
        transactions.clone(),
        &args.documents,
        &args.require_document_for_tag_prefixes,
    );
    let docs_by_index = diff.documents_by_transaction_index();

    let blocks = match load_print_blocks(args.journal.as_deref()) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("ERROR: {e}");
            return 2;
        }
    };

    if blocks.len() != transactions.len() {
        eprintln!("internal error: hledger print output did not align with hledger json output");
        return 2;
    }

    for (txn, block) in transactions.iter().zip(blocks.iter()) {
        let txn_index = txn.get("tindex").and_then(|v| v.as_i64()).unwrap_or(-1);
        let document_tags: Vec<String> = docs_by_index
            .get(&txn_index)
            .map(|paths| {
                paths
                    .iter()
                    .map(|p| {
                        document_tag_value(p, &args.documents, args.document_tag_root.as_deref())
                    })
                    .collect()
            })
            .unwrap_or_default();

        println!("{}", inject_document_tags(block, &document_tags));
        println!();
    }

    0
}
