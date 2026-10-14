# shellcheck shell=bash

scenario_every_amount_audit_skip_reason_is_reported() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_EVERY_AMOUNT_AUDIT_SKIP_REASON_IS_REPORTED_0'
account assets:bank                                      ; document_check:exempt
account expenses:business:unreadable-document            ; document_check:required
account expenses:business:unclear-document-currency      ; document_check:required
account expenses:business:missing-transaction-amount     ; document_check:required
account expenses:business:mixed-transaction-currencies   ; document_check:required
account expenses:business:currency-mismatch              ; document_check:required

2026-01-02 Unreadable Document Amount
    expenses:business:unreadable-document                 10.00 EUR
    assets:bank                                          -10.00 EUR

2026-01-03 Unclear Document Currency
    expenses:business:unclear-document-currency           10.00 EUR
    assets:bank                                          -10.00 EUR

2026-01-04 Missing Transaction Amount
    expenses:business:missing-transaction-amount

2026-01-05 Mixed Transaction Currencies
    expenses:business:mixed-transaction-currencies        10.00 EUR
    assets:bank                                          -10.00 EUR
    expenses:business:mixed-transaction-currencies        12.00 USD
    assets:paypal                                        -12.00 USD

2026-01-06 Currency Mismatch
    expenses:business:currency-mismatch                   10.00 EUR
    assets:bank                                          -10.00 EUR
SCENARIO_EVERY_AMOUNT_AUDIT_SKIP_REASON_IS_REPORTED_0
  mkdir -p "$scenario_dir/documents/expenses/business/unreadable-document"
  cat >"$scenario_dir/documents/expenses/business/unreadable-document/2026-01-02-unreadable.pdf" <<'SCENARIO_EVERY_AMOUNT_AUDIT_SKIP_REASON_IS_REPORTED_1'
no parseable total in this fake pdf
SCENARIO_EVERY_AMOUNT_AUDIT_SKIP_REASON_IS_REPORTED_1
  mkdir -p "$scenario_dir/documents/expenses/business/unclear-document-currency"
  cat >"$scenario_dir/documents/expenses/business/unclear-document-currency/2026-01-03-unclear.pdf" <<'SCENARIO_EVERY_AMOUNT_AUDIT_SKIP_REASON_IS_REPORTED_2'
unclear currency metadata example
SCENARIO_EVERY_AMOUNT_AUDIT_SKIP_REASON_IS_REPORTED_2
  mkdir -p "$scenario_dir/documents/expenses/business/missing-transaction-amount"
  cat >"$scenario_dir/documents/expenses/business/missing-transaction-amount/2026-01-04-missing-amount.pdf" <<'SCENARIO_EVERY_AMOUNT_AUDIT_SKIP_REASON_IS_REPORTED_3'
missing transaction amount metadata example
SCENARIO_EVERY_AMOUNT_AUDIT_SKIP_REASON_IS_REPORTED_3
  mkdir -p "$scenario_dir/documents/expenses/business/mixed-transaction-currencies"
  cat >"$scenario_dir/documents/expenses/business/mixed-transaction-currencies/2026-01-05-mixed.pdf" <<'SCENARIO_EVERY_AMOUNT_AUDIT_SKIP_REASON_IS_REPORTED_4'
mixed transaction currencies metadata example
SCENARIO_EVERY_AMOUNT_AUDIT_SKIP_REASON_IS_REPORTED_4
  mkdir -p "$scenario_dir/documents/expenses/business/currency-mismatch"
  cat >"$scenario_dir/documents/expenses/business/currency-mismatch/2026-01-06-usd-invoice.pdf" <<'SCENARIO_EVERY_AMOUNT_AUDIT_SKIP_REASON_IS_REPORTED_5'
currency mismatch metadata example
SCENARIO_EVERY_AMOUNT_AUDIT_SKIP_REASON_IS_REPORTED_5
  mkdir -p "$scenario_dir/documents/expenses/business/unclear-document-currency"
  cat >"$scenario_dir/documents/expenses/business/unclear-document-currency/2026-01-03-unclear.document.yml" <<'SCENARIO_EVERY_AMOUNT_AUDIT_SKIP_REASON_IS_REPORTED_6'
covers:
  - date: 2026-01-03
    account: expenses:business:unclear-document-currency
    amount: 10.00
    currency: EUR
  - date: 2026-01-03
    account: expenses:business:unclear-document-currency
    amount: 10.00
    currency: USD
SCENARIO_EVERY_AMOUNT_AUDIT_SKIP_REASON_IS_REPORTED_6
  mkdir -p "$scenario_dir/documents/expenses/business/missing-transaction-amount"
  cat >"$scenario_dir/documents/expenses/business/missing-transaction-amount/2026-01-04-missing-amount.document.yml" <<'SCENARIO_EVERY_AMOUNT_AUDIT_SKIP_REASON_IS_REPORTED_7'
covers:
  - date: 2026-01-04
    account: expenses:business:missing-transaction-amount
    amount: 10.00
    currency: EUR
SCENARIO_EVERY_AMOUNT_AUDIT_SKIP_REASON_IS_REPORTED_7
  mkdir -p "$scenario_dir/documents/expenses/business/mixed-transaction-currencies"
  cat >"$scenario_dir/documents/expenses/business/mixed-transaction-currencies/2026-01-05-mixed.document.yml" <<'SCENARIO_EVERY_AMOUNT_AUDIT_SKIP_REASON_IS_REPORTED_8'
covers:
  - date: 2026-01-05
    account: expenses:business:mixed-transaction-currencies
    amount: 22.00
    currency: EUR
SCENARIO_EVERY_AMOUNT_AUDIT_SKIP_REASON_IS_REPORTED_8
  mkdir -p "$scenario_dir/documents/expenses/business/currency-mismatch"
  cat >"$scenario_dir/documents/expenses/business/currency-mismatch/2026-01-06-usd-invoice.document.yml" <<'SCENARIO_EVERY_AMOUNT_AUDIT_SKIP_REASON_IS_REPORTED_9'
covers:
  - date: 2026-01-06
    account: expenses:business:currency-mismatch
    amount: 10.00
    currency: USD
SCENARIO_EVERY_AMOUNT_AUDIT_SKIP_REASON_IS_REPORTED_9
}

scenario_parsed_amount_warnings_can_be_made_fatal() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_PARSED_AMOUNT_WARNINGS_CAN_BE_MADE_FATAL_0'
account assets:bank                          ; document_check:exempt
account expenses:business:hosting            ; document_check:required

2026-01-02 Hosting Provider
    expenses:business:hosting                 10.00 EUR
    assets:bank                              -10.00 EUR
SCENARIO_PARSED_AMOUNT_WARNINGS_CAN_BE_MADE_FATAL_0
  mkdir -p "$scenario_dir/documents/expenses/business/hosting"
  cat >"$scenario_dir/documents/expenses/business/hosting/2026-01-02-hosting.pdf" <<'SCENARIO_PARSED_AMOUNT_WARNINGS_CAN_BE_MADE_FATAL_1'
fake pdf without a readable amount
SCENARIO_PARSED_AMOUNT_WARNINGS_CAN_BE_MADE_FATAL_1
}

scenario_a_foreign_currency_invoice_is_audited_with_the_converted_bank_amount() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_A_FOREIGN_CURRENCY_INVOICE_IS_AUDITED_WITH_THE_CONVERTED_BANK_AMOUNT_0'
account assets:bank                       ; document_check:exempt
account expenses:business:cloud           ; document_check:required

2026-01-04 Cloud Provider
    expenses:business:cloud                92.00 EUR
    assets:bank                           -92.00 EUR
SCENARIO_A_FOREIGN_CURRENCY_INVOICE_IS_AUDITED_WITH_THE_CONVERTED_BANK_AMOUNT_0
  mkdir -p "$scenario_dir/documents/expenses/business/cloud"
  cat >"$scenario_dir/documents/expenses/business/cloud/2026-01-04-cloud.pdf" <<'SCENARIO_A_FOREIGN_CURRENCY_INVOICE_IS_AUDITED_WITH_THE_CONVERTED_BANK_AMOUNT_1'
invoice total 100.00 USD, bank booked converted amount
SCENARIO_A_FOREIGN_CURRENCY_INVOICE_IS_AUDITED_WITH_THE_CONVERTED_BANK_AMOUNT_1
  mkdir -p "$scenario_dir/documents/expenses/business/cloud"
  cat >"$scenario_dir/documents/expenses/business/cloud/2026-01-04-cloud.document.yml" <<'SCENARIO_A_FOREIGN_CURRENCY_INVOICE_IS_AUDITED_WITH_THE_CONVERTED_BANK_AMOUNT_2'
covers:
  - date: 2026-01-04
    account: expenses:business:cloud
    amount: 92.00
    currency: EUR
SCENARIO_A_FOREIGN_CURRENCY_INVOICE_IS_AUDITED_WITH_THE_CONVERTED_BANK_AMOUNT_2
}

scenario_metadata_amount_differs_from_the_transaction_amount() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_METADATA_AMOUNT_DIFFERS_FROM_THE_TRANSACTION_AMOUNT_0'
account assets:bank                       ; document_check:exempt
account expenses:business:software        ; document_check:required

2026-01-03 Software Suite
    expenses:business:software             70.00 EUR
    assets:bank                           -70.00 EUR
SCENARIO_METADATA_AMOUNT_DIFFERS_FROM_THE_TRANSACTION_AMOUNT_0
  mkdir -p "$scenario_dir/documents/expenses/business/software"
  cat >"$scenario_dir/documents/expenses/business/software/2026-01-03-suite.pdf" <<'SCENARIO_METADATA_AMOUNT_DIFFERS_FROM_THE_TRANSACTION_AMOUNT_1'
software suite invoice
SCENARIO_METADATA_AMOUNT_DIFFERS_FROM_THE_TRANSACTION_AMOUNT_1
  mkdir -p "$scenario_dir/documents/expenses/business/software"
  cat >"$scenario_dir/documents/expenses/business/software/2026-01-03-suite.document.yml" <<'SCENARIO_METADATA_AMOUNT_DIFFERS_FROM_THE_TRANSACTION_AMOUNT_2'
covers:
  - date: 2026-01-03
    account: expenses:business:software
    amount: 100.00
    currency: EUR
SCENARIO_METADATA_AMOUNT_DIFFERS_FROM_THE_TRANSACTION_AMOUNT_2
}
