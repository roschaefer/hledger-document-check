# AGENTS.md

`hledger-document-check` is a Rust CLI that checks hledger transactions against
documents on disk.

- `src/main.rs`: clap CLI entrypoint for `check`, `enrich-journal`, and `init-config`
- `src/check_documents.rs`: main `check` command orchestration and reporting
- `src/enrich_journal.rs`: derived journal output with `document:` tags
- `src/document_tree.rs`: document tree inferred from account folders, dated filenames, unbooked folders, and sidecars
- `src/document_paths.rs`: path parsing/formatting helpers for the document tree
- `src/comparison.rs`: diff between the document tree and journal transactions
- `src/matching.rs`: matching of documents to transactions
- `src/duplicates.rs`: duplicate document detection
- `src/amount_audit.rs` / `src/amounts.rs`: amount extraction from documents and audit against transaction amounts
- `src/journal.rs`: hledger journal parsing and tag handling
- `src/metadata.rs`: `.document.yml` sidecar parsing
- `src/config.rs`: `hledger-document-check.toml` loading/writing
- `src/model.rs`: shared types
- `specs/`: Markdown specifications; fenced `gherkin` blocks are compiled into
  cucumber features at build time by `build.rs` (see `tests/cucumber.rs` for
  step definitions)
- `example/`: runnable example journal, documents, and Fava integration

Keep real journals and documents outside this repository.
