# shellcheck shell=bash

scenario_a_dated_file_in_the_account_folder_covers_the_required_posting() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_A_DATED_FILE_IN_THE_ACCOUNT_FOLDER_COVERS_THE_REQUIRED_POSTING_0'
account assets:bank                    ; document_check:exempt
account expenses:business:hosting      ; document_check:required

2026-01-02 Hosting Provider
    expenses:business:hosting        10.00 EUR
    assets:bank                     -10.00 EUR
SCENARIO_A_DATED_FILE_IN_THE_ACCOUNT_FOLDER_COVERS_THE_REQUIRED_POSTING_0
  mkdir -p "$scenario_dir/documents/expenses/business/hosting"
  cat >"$scenario_dir/documents/expenses/business/hosting/2026-01-02-hosting.pdf" <<'SCENARIO_A_DATED_FILE_IN_THE_ACCOUNT_FOLDER_COVERS_THE_REQUIRED_POSTING_1'
hosting receipt
SCENARIO_A_DATED_FILE_IN_THE_ACCOUNT_FOLDER_COVERS_THE_REQUIRED_POSTING_1
}

scenario_a_missing_document_placeholder_documents_the_reason_with_notes() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_A_MISSING_DOCUMENT_PLACEHOLDER_DOCUMENTS_THE_REASON_WITH_NOTES_0'
account assets:bank                    ; document_check:exempt
account expenses:business:hosting      ; document_check:required
account expenses:business:banking      ; document_check:required

2026-01-02 Hosting Provider
    expenses:business:hosting        10.00 EUR
    assets:bank                     -10.00 EUR

2026-01-06 Bank Fee
    expenses:business:banking         3.00 EUR
    assets:bank                      -3.00 EUR
SCENARIO_A_MISSING_DOCUMENT_PLACEHOLDER_DOCUMENTS_THE_REASON_WITH_NOTES_0
  mkdir -p "$scenario_dir/documents/expenses/business/hosting"
  cat >"$scenario_dir/documents/expenses/business/hosting/2026-01-02-hosting.pdf" <<'SCENARIO_A_MISSING_DOCUMENT_PLACEHOLDER_DOCUMENTS_THE_REASON_WITH_NOTES_1'
hosting receipt
SCENARIO_A_MISSING_DOCUMENT_PLACEHOLDER_DOCUMENTS_THE_REASON_WITH_NOTES_1
  mkdir -p "$scenario_dir/documents/expenses/business/banking"
  cat >"$scenario_dir/documents/expenses/business/banking/2026-01-06-missing.document.yml" <<'SCENARIO_A_MISSING_DOCUMENT_PLACEHOLDER_DOCUMENTS_THE_REASON_WITH_NOTES_2'
notes: "Bank statement never arrived; requested a reissue from the bank on 2026-01-08."
SCENARIO_A_MISSING_DOCUMENT_PLACEHOLDER_DOCUMENTS_THE_REASON_WITH_NOTES_2
}

scenario_supporting_files_live_below_a_sibling_directory_named_after_the_document() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_SUPPORTING_FILES_LIVE_BELOW_A_SIBLING_DIRECTORY_NAMED_AFTER_THE_DOCUMENT_0'
account assets:bank                    ; document_check:exempt
account expenses:business:hosting      ; document_check:required

2026-01-02 Hosting Provider
    expenses:business:hosting        10.00 EUR
    assets:bank                     -10.00 EUR
SCENARIO_SUPPORTING_FILES_LIVE_BELOW_A_SIBLING_DIRECTORY_NAMED_AFTER_THE_DOCUMENT_0
  mkdir -p "$scenario_dir/documents/expenses/business/hosting"
  cat >"$scenario_dir/documents/expenses/business/hosting/2026-01-02-hosting.pdf" <<'SCENARIO_SUPPORTING_FILES_LIVE_BELOW_A_SIBLING_DIRECTORY_NAMED_AFTER_THE_DOCUMENT_1'
hosting receipt
SCENARIO_SUPPORTING_FILES_LIVE_BELOW_A_SIBLING_DIRECTORY_NAMED_AFTER_THE_DOCUMENT_1
  mkdir -p "$scenario_dir/documents/expenses/business/hosting/2026-01-02-hosting"
  cat >"$scenario_dir/documents/expenses/business/hosting/2026-01-02-hosting/utilization.csv" <<'SCENARIO_SUPPORTING_FILES_LIVE_BELOW_A_SIBLING_DIRECTORY_NAMED_AFTER_THE_DOCUMENT_2'
project,hours
client-a,4
SCENARIO_SUPPORTING_FILES_LIVE_BELOW_A_SIBLING_DIRECTORY_NAMED_AFTER_THE_DOCUMENT_2
}
