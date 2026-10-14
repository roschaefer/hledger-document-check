# shellcheck shell=bash

scenario_enriched_journal_emits_document_tags_for_matched_documents() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_ENRICHED_JOURNAL_EMITS_DOCUMENT_TAGS_FOR_MATCHED_DOCUMENTS_0'
account assets:bank                         ; document_check:exempt
account expenses:business:hosting           ; document_check:required

2026-01-02 Hosting Provider
    ; document:2026-01-02-hosting.document.yml
    ; document:2026-01-02-existing.pdf
    expenses:business:hosting                10.00 EUR
    assets:bank                             -10.00 EUR
SCENARIO_ENRICHED_JOURNAL_EMITS_DOCUMENT_TAGS_FOR_MATCHED_DOCUMENTS_0
  mkdir -p "$scenario_dir/documents/expenses/business/hosting"
  cat >"$scenario_dir/documents/expenses/business/hosting/2026-01-02-hosting.pdf" <<'SCENARIO_ENRICHED_JOURNAL_EMITS_DOCUMENT_TAGS_FOR_MATCHED_DOCUMENTS_1'
hosting invoice
SCENARIO_ENRICHED_JOURNAL_EMITS_DOCUMENT_TAGS_FOR_MATCHED_DOCUMENTS_1
}

scenario_enriched_journal_emits_document_tags_with_an_absolute_root() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_ENRICHED_JOURNAL_EMITS_DOCUMENT_TAGS_WITH_AN_ABSOLUTE_ROOT_0'
account assets:bank                         ; document_check:exempt
account expenses:business:hosting           ; document_check:required

2026-01-02 Hosting Provider
    expenses:business:hosting                10.00 EUR
    assets:bank                             -10.00 EUR
SCENARIO_ENRICHED_JOURNAL_EMITS_DOCUMENT_TAGS_WITH_AN_ABSOLUTE_ROOT_0
  mkdir -p "$scenario_dir/documents/expenses/business/hosting"
  cat >"$scenario_dir/documents/expenses/business/hosting/2026-01-02-hosting.pdf" <<'SCENARIO_ENRICHED_JOURNAL_EMITS_DOCUMENT_TAGS_WITH_AN_ABSOLUTE_ROOT_1'
hosting invoice
SCENARIO_ENRICHED_JOURNAL_EMITS_DOCUMENT_TAGS_WITH_AN_ABSOLUTE_ROOT_1
}
