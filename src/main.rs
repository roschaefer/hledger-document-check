mod amount_audit;
mod amounts;
mod check_documents;
mod comparison;
mod config;
mod document_paths;
mod document_tree;
mod duplicates;
mod enrich_journal;
mod journal;
mod matching;
mod metadata;
mod model;

use chrono::NaiveDate;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(
    name = "hledger-document-check",
    about = "Check document files against an hledger journal.",
    version = VERSION,
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Write a default hledger-document-check.toml.
    InitConfig {
        /// Path to write. Defaults to hledger-document-check.toml.
        #[arg(long, default_value = "hledger-document-check.toml")]
        output: PathBuf,
    },
    /// Check document coverage against transactions.
    Check {
        /// Path to hledger-document-check.toml.
        #[arg(long)]
        config: Option<PathBuf>,
        /// Path to the hledger journal.
        #[arg(short = 'f', long)]
        journal: Option<PathBuf>,
        /// Path to the document root. Defaults to the current working directory.
        #[arg(long)]
        documents: Option<PathBuf>,
        /// Repeatable tag-key prefix that implies document_check:required.
        #[arg(long = "require-document-for-tag-prefix")]
        require_document_for_tag_prefixes: Vec<String>,
        /// Repeatable check name to treat as a failure.
        #[arg(long)]
        fail_on: Vec<String>,
        /// Repeatable check name to treat as a warning.
        #[arg(long)]
        warn_on: Vec<String>,
        /// Repeatable check name to summarize without detailed reporting.
        #[arg(long = "ignore")]
        ignore_checks: Vec<String>,
        /// Override today's date for deterministic checks (YYYY-MM-DD).
        #[arg(long)]
        today: Option<String>,
        /// Fail unbooked documents when today is more than this many days after due_date.
        #[arg(long)]
        overdue_after_days: Option<u32>,
    },
    /// Emit a derived hledger journal with document: tags for matched transactions.
    EnrichJournal {
        /// Path to hledger-document-check.toml.
        #[arg(long)]
        config: Option<PathBuf>,
        /// Path to the hledger journal.
        #[arg(short = 'f', long)]
        journal: Option<PathBuf>,
        /// Path to the document root. Defaults to the current working directory.
        #[arg(long)]
        documents: Option<PathBuf>,
        /// Optional path prefix for emitted document: tag values.
        #[arg(long)]
        document_tag_root: Option<String>,
        /// Repeatable tag-key prefix that implies document_check:required.
        #[arg(long = "require-document-for-tag-prefix")]
        require_document_for_tag_prefixes: Vec<String>,
        /// Repeatable check name to treat as a failure.
        #[arg(long)]
        fail_on: Vec<String>,
        /// Repeatable check name to treat as a warning.
        #[arg(long)]
        warn_on: Vec<String>,
        /// Repeatable check name to summarize without detailed reporting.
        #[arg(long = "ignore")]
        ignore_checks: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    let exit_code = run(cli);
    std::process::exit(exit_code);
}

fn run(cli: Cli) -> i32 {
    match cli.command {
        Commands::InitConfig { output } => {
            if let Err(e) = config::write_default_config(&output) {
                eprintln!("ERROR: {e}");
                return 1;
            }
            0
        }

        Commands::Check {
            config: config_path,
            journal,
            documents,
            require_document_for_tag_prefixes,
            fail_on,
            warn_on,
            ignore_checks,
            today,
            overdue_after_days,
        } => {
            let documents_str = documents
                .as_ref()
                .and_then(|p| p.to_str())
                .map(|s| s.to_string());
            let config_str = config_path
                .as_ref()
                .and_then(|p| p.to_str())
                .map(|s| s.to_string());
            let config_file =
                config::discover_config_path(documents_str.as_deref(), config_str.as_deref());
            let cfg = match config::load_config(config_file.as_deref()) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("ERROR: {e}");
                    return 2;
                }
            };

            let effective_documents = documents
                .map(|p| p.canonicalize().unwrap_or(p))
                .or_else(|| cfg.documents.as_ref().map(PathBuf::from))
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

            let effective_journal = journal.or_else(|| cfg.journal.as_ref().map(PathBuf::from));

            let effective_prefixes = if require_document_for_tag_prefixes.is_empty() {
                cfg.tag_prefixes.clone()
            } else {
                require_document_for_tag_prefixes
            };

            let effective_fail_on = if fail_on.is_empty() {
                cfg.checks
                    .iter()
                    .filter(|(_, v)| v.as_str() == "fail")
                    .map(|(k, _)| k.clone())
                    .collect()
            } else {
                fail_on
            };
            let effective_warn_on = if warn_on.is_empty() {
                cfg.checks
                    .iter()
                    .filter(|(_, v)| v.as_str() == "warn")
                    .map(|(k, _)| k.clone())
                    .collect()
            } else {
                warn_on
            };
            let effective_ignore = if ignore_checks.is_empty() {
                cfg.checks
                    .iter()
                    .filter(|(_, v)| v.as_str() == "ignore")
                    .map(|(k, _)| k.clone())
                    .collect()
            } else {
                ignore_checks
            };

            let parsed_today = today.as_deref().and_then(|s| {
                NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .map_err(|_| {
                        eprintln!("ERROR: invalid --today date: {s}");
                    })
                    .ok()
            });
            if today.is_some() && parsed_today.is_none() {
                return 2;
            }

            let effective_overdue = overdue_after_days.or(cfg.overdue_after_days).unwrap_or(14);

            check_documents::run_check(check_documents::CheckArgs {
                journal: effective_journal,
                documents: effective_documents,
                require_document_for_tag_prefixes: effective_prefixes,
                fail_on: effective_fail_on,
                warn_on: effective_warn_on,
                ignore_checks: effective_ignore,
                today: parsed_today,
                overdue_after_days: effective_overdue,
            })
        }

        Commands::EnrichJournal {
            config: config_path,
            journal,
            documents,
            document_tag_root,
            require_document_for_tag_prefixes,
            fail_on: _,
            warn_on: _,
            ignore_checks: _,
        } => {
            let documents_str = documents
                .as_ref()
                .and_then(|p| p.to_str())
                .map(|s| s.to_string());
            let config_str = config_path
                .as_ref()
                .and_then(|p| p.to_str())
                .map(|s| s.to_string());
            let config_file =
                config::discover_config_path(documents_str.as_deref(), config_str.as_deref());
            let cfg = match config::load_config(config_file.as_deref()) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("ERROR: {e}");
                    return 2;
                }
            };

            let effective_documents = documents
                .map(|p| p.canonicalize().unwrap_or(p))
                .or_else(|| cfg.documents.as_ref().map(PathBuf::from))
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

            let effective_journal = journal.or_else(|| cfg.journal.as_ref().map(PathBuf::from));

            let effective_tag_root = document_tag_root.or(cfg.document_tag_root);

            let effective_prefixes = if require_document_for_tag_prefixes.is_empty() {
                cfg.tag_prefixes
            } else {
                require_document_for_tag_prefixes
            };

            enrich_journal::run_enrich(enrich_journal::EnrichArgs {
                journal: effective_journal,
                documents: effective_documents,
                document_tag_root: effective_tag_root,
                require_document_for_tag_prefixes: effective_prefixes,
            })
        }
    }
}
