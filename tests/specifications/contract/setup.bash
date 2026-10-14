# shellcheck shell=bash

scenario_a_subaccount_can_be_exempted_from_a_required_parent_account() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/accounts.journal" <<'SCENARIO_A_SUBACCOUNT_CAN_BE_EXEMPTED_FROM_A_REQUIRED_PARENT_ACCOUNT_0'
account assets:bank                       ; document_check:exempt
account expenses:business                 ; document_check:required
account expenses:business:bank-fees       ; document_check:exempt
SCENARIO_A_SUBACCOUNT_CAN_BE_EXEMPTED_FROM_A_REQUIRED_PARENT_ACCOUNT_0
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_A_SUBACCOUNT_CAN_BE_EXEMPTED_FROM_A_REQUIRED_PARENT_ACCOUNT_1'
include accounts.journal

2026-01-02 Hosting Provider
    expenses:business:hosting        10.00 EUR
    assets:bank                     -10.00 EUR

2026-01-05 Bank Fee
    expenses:business:bank-fees       2.50 EUR
    assets:bank                      -2.50 EUR
SCENARIO_A_SUBACCOUNT_CAN_BE_EXEMPTED_FROM_A_REQUIRED_PARENT_ACCOUNT_1
  mkdir -p "$scenario_dir/documents/expenses/business/hosting"
  cat >"$scenario_dir/documents/expenses/business/hosting/2026-01-02-hosting.pdf" <<'SCENARIO_A_SUBACCOUNT_CAN_BE_EXEMPTED_FROM_A_REQUIRED_PARENT_ACCOUNT_2'
hosting receipt
SCENARIO_A_SUBACCOUNT_CAN_BE_EXEMPTED_FROM_A_REQUIRED_PARENT_ACCOUNT_2
}

scenario_a_notice_needs_a_document_but_the_payment_that_later_clears_it_does_not() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/accounts.journal" <<'SCENARIO_A_NOTICE_NEEDS_A_DOCUMENT_BUT_THE_PAYMENT_THAT_LATER_CLEARS_IT_DOES_NOT_0'
account assets:bank                              ; document_check:exempt
account expenses:insurance:health:provider      ; document_check:exempt
account expenses:insurance:pension:provider     ; document_check:exempt
account liabilities:insurance-notices           ; document_check:counter_account, document_check_counter_account_a:expenses:insurance:health:provider, document_check_counter_account_b:expenses:insurance:pension:provider
SCENARIO_A_NOTICE_NEEDS_A_DOCUMENT_BUT_THE_PAYMENT_THAT_LATER_CLEARS_IT_DOES_NOT_0
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_A_NOTICE_NEEDS_A_DOCUMENT_BUT_THE_PAYMENT_THAT_LATER_CLEARS_IT_DOES_NOT_1'
include accounts.journal

2026-01-10 Health Insurance Notice
    expenses:insurance:health:provider   220.00 EUR
    liabilities:insurance-notices       -220.00 EUR

2026-01-15 Health Insurance Payment
    liabilities:insurance-notices        220.00 EUR
    assets:bank                         -220.00 EUR

2026-02-10 Pension Insurance Notice
    expenses:insurance:pension:provider  80.00 EUR
    liabilities:insurance-notices       -80.00 EUR
SCENARIO_A_NOTICE_NEEDS_A_DOCUMENT_BUT_THE_PAYMENT_THAT_LATER_CLEARS_IT_DOES_NOT_1
  mkdir -p "$scenario_dir/documents/liabilities/insurance-notices"
  cat >"$scenario_dir/documents/liabilities/insurance-notices/2026-01-10-health-notice.pdf" <<'SCENARIO_A_NOTICE_NEEDS_A_DOCUMENT_BUT_THE_PAYMENT_THAT_LATER_CLEARS_IT_DOES_NOT_2'
health insurance notice
SCENARIO_A_NOTICE_NEEDS_A_DOCUMENT_BUT_THE_PAYMENT_THAT_LATER_CLEARS_IT_DOES_NOT_2
  mkdir -p "$scenario_dir/documents/liabilities/insurance-notices"
  cat >"$scenario_dir/documents/liabilities/insurance-notices/2026-02-10-pension-notice.pdf" <<'SCENARIO_A_NOTICE_NEEDS_A_DOCUMENT_BUT_THE_PAYMENT_THAT_LATER_CLEARS_IT_DOES_NOT_3'
pension insurance notice
SCENARIO_A_NOTICE_NEEDS_A_DOCUMENT_BUT_THE_PAYMENT_THAT_LATER_CLEARS_IT_DOES_NOT_3
}

scenario_using_counter_account_without_a_counter_account_tag_is_an_error() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_USING_COUNTER_ACCOUNT_WITHOUT_A_COUNTER_ACCOUNT_TAG_IS_AN_ERROR_0'
account assets:bank                              ; document_check:exempt
account expenses:insurance:health:provider       ; document_check:exempt
account liabilities:health-insurance             ; document_check:counter_account

2026-01-10 Health Insurance Notice
    expenses:insurance:health:provider   220.00 EUR
    liabilities:health-insurance        -220.00 EUR
SCENARIO_USING_COUNTER_ACCOUNT_WITHOUT_A_COUNTER_ACCOUNT_TAG_IS_AN_ERROR_0
  mkdir -p "$scenario_dir/documents"
}

scenario_a_prefixed_tag_can_require_documents() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_A_PREFIXED_TAG_CAN_REQUIRE_DOCUMENTS_0'
2026-01-02 Hosting Provider
    expenses:business:hosting        10.00 EUR  ; tax_role:business_expense
    assets:bank                     -10.00 EUR
SCENARIO_A_PREFIXED_TAG_CAN_REQUIRE_DOCUMENTS_0
  mkdir -p "$scenario_dir/documents"
}
