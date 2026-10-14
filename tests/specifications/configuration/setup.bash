# shellcheck shell=bash

scenario_the_default_config_file_can_be_generated() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
}

scenario_the_readme_command_generates_the_initial_config_file() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/documents"
}

scenario_the_check_command_reads_defaults_from_the_document_root() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_THE_CHECK_COMMAND_READS_DEFAULTS_FROM_THE_DOCUMENT_ROOT_0'
2026-01-02 Hosting Provider
    expenses:business:hosting        10.00 EUR  ; tax_role:business_expense
    assets:bank                     -10.00 EUR
SCENARIO_THE_CHECK_COMMAND_READS_DEFAULTS_FROM_THE_DOCUMENT_ROOT_0
  mkdir -p "$scenario_dir/documents"
  cat >"$scenario_dir/documents/hledger-document-check.toml" <<'SCENARIO_THE_CHECK_COMMAND_READS_DEFAULTS_FROM_THE_DOCUMENT_ROOT_1'
[ledger]
journal = "../journal.journal"

[requirements]
tag_prefixes = ["tax_"]

[checks]
missing-document-coverage = "warn"
SCENARIO_THE_CHECK_COMMAND_READS_DEFAULTS_FROM_THE_DOCUMENT_ROOT_1
}
