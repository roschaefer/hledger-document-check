# shellcheck shell=bash

scenario_a_sent_invoice_is_not_yet_due() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_A_SENT_INVOICE_IS_NOT_YET_DUE_0'
account assets:bank                                  ; document_check:exempt
account income:business:freelance:customer           ; document_check:required
SCENARIO_A_SENT_INVOICE_IS_NOT_YET_DUE_0
  mkdir -p "$scenario_dir/documents/income/business/freelance/customer/unbooked"
  cat >"$scenario_dir/documents/income/business/freelance/customer/unbooked/invoice.pdf" <<'SCENARIO_A_SENT_INVOICE_IS_NOT_YET_DUE_1'
customer invoice
SCENARIO_A_SENT_INVOICE_IS_NOT_YET_DUE_1
  mkdir -p "$scenario_dir/documents/income/business/freelance/customer/unbooked"
  cat >"$scenario_dir/documents/income/business/freelance/customer/unbooked/invoice.document.yml" <<'SCENARIO_A_SENT_INVOICE_IS_NOT_YET_DUE_2'
due_date: 2026-05-31
SCENARIO_A_SENT_INVOICE_IS_NOT_YET_DUE_2
}

scenario_a_due_sent_invoice_without_a_transaction_fails() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_A_DUE_SENT_INVOICE_WITHOUT_A_TRANSACTION_FAILS_0'
account assets:bank                                  ; document_check:exempt
account income:business:freelance:customer           ; document_check:required
SCENARIO_A_DUE_SENT_INVOICE_WITHOUT_A_TRANSACTION_FAILS_0
  mkdir -p "$scenario_dir/documents/income/business/freelance/customer/unbooked"
  cat >"$scenario_dir/documents/income/business/freelance/customer/unbooked/invoice.pdf" <<'SCENARIO_A_DUE_SENT_INVOICE_WITHOUT_A_TRANSACTION_FAILS_1'
overdue customer invoice
SCENARIO_A_DUE_SENT_INVOICE_WITHOUT_A_TRANSACTION_FAILS_1
  mkdir -p "$scenario_dir/documents/income/business/freelance/customer/unbooked"
  cat >"$scenario_dir/documents/income/business/freelance/customer/unbooked/invoice.document.yml" <<'SCENARIO_A_DUE_SENT_INVOICE_WITHOUT_A_TRANSACTION_FAILS_2'
due_date: 2026-04-15
SCENARIO_A_DUE_SENT_INVOICE_WITHOUT_A_TRANSACTION_FAILS_2
}

scenario_a_due_sent_invoice_with_a_matching_transaction_is_ok() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_A_DUE_SENT_INVOICE_WITH_A_MATCHING_TRANSACTION_IS_OK_0'
account assets:bank                                  ; document_check:exempt
account income:business:freelance:customer           ; document_check:required

2026-04-20 Customer Payment
    assets:bank                                     100.00 EUR
    income:business:freelance:customer
SCENARIO_A_DUE_SENT_INVOICE_WITH_A_MATCHING_TRANSACTION_IS_OK_0
  mkdir -p "$scenario_dir/documents/income/business/freelance/customer"
  cat >"$scenario_dir/documents/income/business/freelance/customer/2026-04-20-invoice.pdf" <<'SCENARIO_A_DUE_SENT_INVOICE_WITH_A_MATCHING_TRANSACTION_IS_OK_1'
customer invoice already paid
SCENARIO_A_DUE_SENT_INVOICE_WITH_A_MATCHING_TRANSACTION_IS_OK_1
  mkdir -p "$scenario_dir/documents/income/business/freelance/customer"
  cat >"$scenario_dir/documents/income/business/freelance/customer/2026-04-20-invoice.document.yml" <<'SCENARIO_A_DUE_SENT_INVOICE_WITH_A_MATCHING_TRANSACTION_IS_OK_2'
due_date: 2026-04-15
covers:
  - date: 2026-04-20
    account: income:business:freelance:customer
    amount: 100.00
    currency: EUR
SCENARIO_A_DUE_SENT_INVOICE_WITH_A_MATCHING_TRANSACTION_IS_OK_2
}
