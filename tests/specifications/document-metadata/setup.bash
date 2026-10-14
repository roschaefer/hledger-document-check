# shellcheck shell=bash

scenario_a_customer_pays_one_invoice_in_two_rates() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_A_CUSTOMER_PAYS_ONE_INVOICE_IN_TWO_RATES_0'
account assets:bank                                  ; document_check:exempt
account income:business:freelance:customer           ; document_check:required

2026-01-05 Customer Payment Rate 1
    assets:bank                                      60.00 EUR
    income:business:freelance:customer

2026-02-05 Customer Payment Rate 2
    assets:bank                                      40.00 EUR
    income:business:freelance:customer
SCENARIO_A_CUSTOMER_PAYS_ONE_INVOICE_IN_TWO_RATES_0
  mkdir -p "$scenario_dir/documents/income/business/freelance/customer"
  cat >"$scenario_dir/documents/income/business/freelance/customer/customer-project.pdf" <<'SCENARIO_A_CUSTOMER_PAYS_ONE_INVOICE_IN_TWO_RATES_1'
customer project invoice
SCENARIO_A_CUSTOMER_PAYS_ONE_INVOICE_IN_TWO_RATES_1
  mkdir -p "$scenario_dir/documents/income/business/freelance/customer"
  cat >"$scenario_dir/documents/income/business/freelance/customer/customer-project.document.yml" <<'SCENARIO_A_CUSTOMER_PAYS_ONE_INVOICE_IN_TWO_RATES_2'
amount: 100.00
currency: EUR

covers:
  - date: 2026-01-05
    account: income:business:freelance:customer
    amount: 60.00
  - date: 2026-02-05
    account: income:business:freelance:customer
    amount: 40.00
SCENARIO_A_CUSTOMER_PAYS_ONE_INVOICE_IN_TWO_RATES_2
}

scenario_cover_amounts_must_add_up_to_the_document_total_when_total_is_configured() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_COVER_AMOUNTS_MUST_ADD_UP_TO_THE_DOCUMENT_TOTAL_WHEN_TOTAL_IS_CONFIGURED_0'
account assets:bank                                  ; document_check:exempt
account income:business:freelance:customer           ; document_check:required

2026-01-05 Customer Payment Rate 1
    assets:bank                                      60.00 EUR
    income:business:freelance:customer

2026-02-05 Customer Payment Rate 2
    assets:bank                                      30.00 EUR
    income:business:freelance:customer
SCENARIO_COVER_AMOUNTS_MUST_ADD_UP_TO_THE_DOCUMENT_TOTAL_WHEN_TOTAL_IS_CONFIGURED_0
  mkdir -p "$scenario_dir/documents/income/business/freelance/customer"
  cat >"$scenario_dir/documents/income/business/freelance/customer/customer-project.pdf" <<'SCENARIO_COVER_AMOUNTS_MUST_ADD_UP_TO_THE_DOCUMENT_TOTAL_WHEN_TOTAL_IS_CONFIGURED_1'
customer project invoice
SCENARIO_COVER_AMOUNTS_MUST_ADD_UP_TO_THE_DOCUMENT_TOTAL_WHEN_TOTAL_IS_CONFIGURED_1
  mkdir -p "$scenario_dir/documents/income/business/freelance/customer"
  cat >"$scenario_dir/documents/income/business/freelance/customer/customer-project.document.yml" <<'SCENARIO_COVER_AMOUNTS_MUST_ADD_UP_TO_THE_DOCUMENT_TOTAL_WHEN_TOTAL_IS_CONFIGURED_2'
amount: 100.00
currency: EUR

covers:
  - date: 2026-01-05
    account: income:business:freelance:customer
    amount: 60.00
  - date: 2026-02-05
    account: income:business:freelance:customer
    amount: 30.00
SCENARIO_COVER_AMOUNTS_MUST_ADD_UP_TO_THE_DOCUMENT_TOTAL_WHEN_TOTAL_IS_CONFIGURED_2
}

scenario_a_customer_pays_two_invoices_at_once() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_A_CUSTOMER_PAYS_TWO_INVOICES_AT_ONCE_0'
account assets:bank                                  ; document_check:exempt
account income:business:freelance:customer           ; document_check:required

2026-03-01 Customer Batch Payment
    assets:bank                                     100.00 EUR
    income:business:freelance:customer
SCENARIO_A_CUSTOMER_PAYS_TWO_INVOICES_AT_ONCE_0
  mkdir -p "$scenario_dir/documents/income/business/freelance/customer"
  cat >"$scenario_dir/documents/income/business/freelance/customer/2026-03-01-invoice-a.pdf" <<'SCENARIO_A_CUSTOMER_PAYS_TWO_INVOICES_AT_ONCE_1'
invoice a
SCENARIO_A_CUSTOMER_PAYS_TWO_INVOICES_AT_ONCE_1
  mkdir -p "$scenario_dir/documents/income/business/freelance/customer"
  cat >"$scenario_dir/documents/income/business/freelance/customer/2026-03-01-invoice-b.pdf" <<'SCENARIO_A_CUSTOMER_PAYS_TWO_INVOICES_AT_ONCE_2'
invoice b
SCENARIO_A_CUSTOMER_PAYS_TWO_INVOICES_AT_ONCE_2
  mkdir -p "$scenario_dir/documents/income/business/freelance/customer"
  cat >"$scenario_dir/documents/income/business/freelance/customer/2026-03-01-invoice-a.document.yml" <<'SCENARIO_A_CUSTOMER_PAYS_TWO_INVOICES_AT_ONCE_3'
amount: 40.00
currency: EUR

covers:
  - date: 2026-03-01
    account: income:business:freelance:customer
    amount: 40.00
SCENARIO_A_CUSTOMER_PAYS_TWO_INVOICES_AT_ONCE_3
  mkdir -p "$scenario_dir/documents/income/business/freelance/customer"
  cat >"$scenario_dir/documents/income/business/freelance/customer/2026-03-01-invoice-b.document.yml" <<'SCENARIO_A_CUSTOMER_PAYS_TWO_INVOICES_AT_ONCE_4'
amount: 60.00
currency: EUR

covers:
  - date: 2026-03-01
    account: income:business:freelance:customer
    amount: 60.00
SCENARIO_A_CUSTOMER_PAYS_TWO_INVOICES_AT_ONCE_4
}

scenario_cover_amounts_must_add_up_to_the_transaction_amount() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_COVER_AMOUNTS_MUST_ADD_UP_TO_THE_TRANSACTION_AMOUNT_0'
account assets:bank                                  ; document_check:exempt
account income:business:freelance:customer           ; document_check:required

2026-03-01 Customer Batch Payment
    assets:bank                                     100.00 EUR
    income:business:freelance:customer
SCENARIO_COVER_AMOUNTS_MUST_ADD_UP_TO_THE_TRANSACTION_AMOUNT_0
  mkdir -p "$scenario_dir/documents/income/business/freelance/customer"
  cat >"$scenario_dir/documents/income/business/freelance/customer/2026-03-01-invoice-a.pdf" <<'SCENARIO_COVER_AMOUNTS_MUST_ADD_UP_TO_THE_TRANSACTION_AMOUNT_1'
invoice a
SCENARIO_COVER_AMOUNTS_MUST_ADD_UP_TO_THE_TRANSACTION_AMOUNT_1
  mkdir -p "$scenario_dir/documents/income/business/freelance/customer"
  cat >"$scenario_dir/documents/income/business/freelance/customer/2026-03-01-invoice-b.pdf" <<'SCENARIO_COVER_AMOUNTS_MUST_ADD_UP_TO_THE_TRANSACTION_AMOUNT_2'
invoice b
SCENARIO_COVER_AMOUNTS_MUST_ADD_UP_TO_THE_TRANSACTION_AMOUNT_2
  mkdir -p "$scenario_dir/documents/income/business/freelance/customer"
  cat >"$scenario_dir/documents/income/business/freelance/customer/2026-03-01-invoice-a.document.yml" <<'SCENARIO_COVER_AMOUNTS_MUST_ADD_UP_TO_THE_TRANSACTION_AMOUNT_3'
amount: 40.00
currency: EUR

covers:
  - date: 2026-03-01
    account: income:business:freelance:customer
    amount: 40.00
SCENARIO_COVER_AMOUNTS_MUST_ADD_UP_TO_THE_TRANSACTION_AMOUNT_3
  mkdir -p "$scenario_dir/documents/income/business/freelance/customer"
  cat >"$scenario_dir/documents/income/business/freelance/customer/2026-03-01-invoice-b.document.yml" <<'SCENARIO_COVER_AMOUNTS_MUST_ADD_UP_TO_THE_TRANSACTION_AMOUNT_4'
amount: 50.00
currency: EUR

covers:
  - date: 2026-03-01
    account: income:business:freelance:customer
    amount: 50.00
SCENARIO_COVER_AMOUNTS_MUST_ADD_UP_TO_THE_TRANSACTION_AMOUNT_4
}

scenario_two_same_date_invoices_each_cover_one_same_date_transaction() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_TWO_SAME_DATE_INVOICES_EACH_COVER_ONE_SAME_DATE_TRANSACTION_0'
account assets:bank                         ; document_check:exempt
account expenses:business:domains           ; document_check:required

2026-03-01 Domain A
    expenses:business:domains                4.00 EUR
    assets:bank                             -4.00 EUR

2026-03-01 Domain B
    expenses:business:domains                6.00 EUR
    assets:bank                             -6.00 EUR
SCENARIO_TWO_SAME_DATE_INVOICES_EACH_COVER_ONE_SAME_DATE_TRANSACTION_0
  mkdir -p "$scenario_dir/documents/expenses/business/domains"
  cat >"$scenario_dir/documents/expenses/business/domains/2026-03-01-domain-a.pdf" <<'SCENARIO_TWO_SAME_DATE_INVOICES_EACH_COVER_ONE_SAME_DATE_TRANSACTION_1'
domain a
SCENARIO_TWO_SAME_DATE_INVOICES_EACH_COVER_ONE_SAME_DATE_TRANSACTION_1
  mkdir -p "$scenario_dir/documents/expenses/business/domains"
  cat >"$scenario_dir/documents/expenses/business/domains/2026-03-01-domain-b.pdf" <<'SCENARIO_TWO_SAME_DATE_INVOICES_EACH_COVER_ONE_SAME_DATE_TRANSACTION_2'
domain b
SCENARIO_TWO_SAME_DATE_INVOICES_EACH_COVER_ONE_SAME_DATE_TRANSACTION_2
  mkdir -p "$scenario_dir/documents/expenses/business/domains"
  cat >"$scenario_dir/documents/expenses/business/domains/2026-03-01-domain-a.document.yml" <<'SCENARIO_TWO_SAME_DATE_INVOICES_EACH_COVER_ONE_SAME_DATE_TRANSACTION_3'
covers:
  - date: 2026-03-01
    account: expenses:business:domains
    amount: 4.00
    currency: EUR
SCENARIO_TWO_SAME_DATE_INVOICES_EACH_COVER_ONE_SAME_DATE_TRANSACTION_3
  mkdir -p "$scenario_dir/documents/expenses/business/domains"
  cat >"$scenario_dir/documents/expenses/business/domains/2026-03-01-domain-b.document.yml" <<'SCENARIO_TWO_SAME_DATE_INVOICES_EACH_COVER_ONE_SAME_DATE_TRANSACTION_4'
covers:
  - date: 2026-03-01
    account: expenses:business:domains
    amount: 6.00
    currency: EUR
SCENARIO_TWO_SAME_DATE_INVOICES_EACH_COVER_ONE_SAME_DATE_TRANSACTION_4
}

scenario_one_annual_health_insurance_notice_covers_several_subaccounts() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_ONE_ANNUAL_HEALTH_INSURANCE_NOTICE_COVERS_SEVERAL_SUBACCOUNTS_0'
account assets:bank                                      ; document_check:exempt
account expenses:insurance:health:base                   ; document_check:required
account expenses:insurance:health:nursing-care           ; document_check:required

2026-01-01 Annual Health Insurance Base
    expenses:insurance:health:base                       100.00 EUR
    assets:bank                                         -100.00 EUR

2026-01-01 Annual Nursing Care Insurance
    expenses:insurance:health:nursing-care                30.00 EUR
    assets:bank                                          -30.00 EUR
SCENARIO_ONE_ANNUAL_HEALTH_INSURANCE_NOTICE_COVERS_SEVERAL_SUBACCOUNTS_0
  mkdir -p "$scenario_dir/documents/expenses/insurance/health"
  cat >"$scenario_dir/documents/expenses/insurance/health/2026-01-01-annual-notice.pdf" <<'SCENARIO_ONE_ANNUAL_HEALTH_INSURANCE_NOTICE_COVERS_SEVERAL_SUBACCOUNTS_1'
annual health insurance notice
SCENARIO_ONE_ANNUAL_HEALTH_INSURANCE_NOTICE_COVERS_SEVERAL_SUBACCOUNTS_1
  mkdir -p "$scenario_dir/documents/expenses/insurance/health"
  cat >"$scenario_dir/documents/expenses/insurance/health/2026-01-01-annual-notice.document.yml" <<'SCENARIO_ONE_ANNUAL_HEALTH_INSURANCE_NOTICE_COVERS_SEVERAL_SUBACCOUNTS_2'
covers:
  - date: 2026-01-01
    account: expenses:insurance:health:base
    amount: 100.00
    currency: EUR
  - date: 2026-01-01
    account: expenses:insurance:health:nursing-care
    amount: 30.00
    currency: EUR
SCENARIO_ONE_ANNUAL_HEALTH_INSURANCE_NOTICE_COVERS_SEVERAL_SUBACCOUNTS_2
}

scenario_a_covers_entry_repeats_the_filename_date_and_folder_account() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_A_COVERS_ENTRY_REPEATS_THE_FILENAME_DATE_AND_FOLDER_ACCOUNT_0'
account assets:bank                          ; document_check:exempt
account expenses:business:hosting:uberspace  ; document_check:required

2026-01-05 Uberspace Hosting
    expenses:business:hosting:uberspace      4.00 EUR
    assets:bank                             -4.00 EUR
SCENARIO_A_COVERS_ENTRY_REPEATS_THE_FILENAME_DATE_AND_FOLDER_ACCOUNT_0
  mkdir -p "$scenario_dir/documents/expenses/business/hosting/uberspace"
  cat >"$scenario_dir/documents/expenses/business/hosting/uberspace/2026-01-05-invoice.pdf" <<'SCENARIO_A_COVERS_ENTRY_REPEATS_THE_FILENAME_DATE_AND_FOLDER_ACCOUNT_1'
uberspace invoice
SCENARIO_A_COVERS_ENTRY_REPEATS_THE_FILENAME_DATE_AND_FOLDER_ACCOUNT_1
  mkdir -p "$scenario_dir/documents/expenses/business/hosting/uberspace"
  cat >"$scenario_dir/documents/expenses/business/hosting/uberspace/2026-01-05-invoice.document.yml" <<'SCENARIO_A_COVERS_ENTRY_REPEATS_THE_FILENAME_DATE_AND_FOLDER_ACCOUNT_2'
covers:
  - date: 2026-01-05
    account: expenses:business:hosting:uberspace
    amount: 4.00
    currency: EUR
SCENARIO_A_COVERS_ENTRY_REPEATS_THE_FILENAME_DATE_AND_FOLDER_ACCOUNT_2
}

scenario_a_top_level_shorthand_date_account_repeats_the_file_s_own_location() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_A_TOP_LEVEL_SHORTHAND_DATE_ACCOUNT_REPEATS_THE_FILE_S_OWN_LOCATION_0'
account assets:bank                     ; document_check:exempt
account expenses:business:hosting:aws   ; document_check:required

2022-06-03 AWS EMEA
    expenses:business:hosting:aws        2.99 EUR
    assets:bank                         -2.99 EUR
SCENARIO_A_TOP_LEVEL_SHORTHAND_DATE_ACCOUNT_REPEATS_THE_FILE_S_OWN_LOCATION_0
  mkdir -p "$scenario_dir/documents/expenses/business/hosting/aws"
  cat >"$scenario_dir/documents/expenses/business/hosting/aws/2022-06-03-invoice.pdf" <<'SCENARIO_A_TOP_LEVEL_SHORTHAND_DATE_ACCOUNT_REPEATS_THE_FILE_S_OWN_LOCATION_1'
aws invoice
SCENARIO_A_TOP_LEVEL_SHORTHAND_DATE_ACCOUNT_REPEATS_THE_FILE_S_OWN_LOCATION_1
  mkdir -p "$scenario_dir/documents/expenses/business/hosting/aws"
  cat >"$scenario_dir/documents/expenses/business/hosting/aws/2022-06-03-invoice.document.yml" <<'SCENARIO_A_TOP_LEVEL_SHORTHAND_DATE_ACCOUNT_REPEATS_THE_FILE_S_OWN_LOCATION_2'
date: 2022-06-03
account: expenses:business:hosting:aws
amount: 2.99
currency: EUR
SCENARIO_A_TOP_LEVEL_SHORTHAND_DATE_ACCOUNT_REPEATS_THE_FILE_S_OWN_LOCATION_2
}

scenario_an_account_override_that_names_a_different_account_is_not_flagged() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_AN_ACCOUNT_OVERRIDE_THAT_NAMES_A_DIFFERENT_ACCOUNT_IS_NOT_FLAGGED_0'
account assets:bank                                      ; document_check:exempt
account expenses:insurance:health:base                   ; document_check:required
account expenses:insurance:health:nursing-care           ; document_check:required

2026-01-01 Annual Health Insurance Base
    expenses:insurance:health:base                       100.00 EUR
    assets:bank                                         -100.00 EUR

2026-01-01 Annual Nursing Care Insurance
    expenses:insurance:health:nursing-care                30.00 EUR
    assets:bank                                          -30.00 EUR
SCENARIO_AN_ACCOUNT_OVERRIDE_THAT_NAMES_A_DIFFERENT_ACCOUNT_IS_NOT_FLAGGED_0
  mkdir -p "$scenario_dir/documents/expenses/insurance/health"
  cat >"$scenario_dir/documents/expenses/insurance/health/2026-01-01-annual-notice.pdf" <<'SCENARIO_AN_ACCOUNT_OVERRIDE_THAT_NAMES_A_DIFFERENT_ACCOUNT_IS_NOT_FLAGGED_1'
annual health insurance notice
SCENARIO_AN_ACCOUNT_OVERRIDE_THAT_NAMES_A_DIFFERENT_ACCOUNT_IS_NOT_FLAGGED_1
  mkdir -p "$scenario_dir/documents/expenses/insurance/health"
  cat >"$scenario_dir/documents/expenses/insurance/health/2026-01-01-annual-notice.document.yml" <<'SCENARIO_AN_ACCOUNT_OVERRIDE_THAT_NAMES_A_DIFFERENT_ACCOUNT_IS_NOT_FLAGGED_2'
covers:
  - account: expenses:insurance:health:base
    amount: 100.00
    currency: EUR
  - account: expenses:insurance:health:nursing-care
    amount: 30.00
    currency: EUR
SCENARIO_AN_ACCOUNT_OVERRIDE_THAT_NAMES_A_DIFFERENT_ACCOUNT_IS_NOT_FLAGGED_2
}

scenario_a_sidecar_with_only_amount_and_currency_is_not_flagged() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_A_SIDECAR_WITH_ONLY_AMOUNT_AND_CURRENCY_IS_NOT_FLAGGED_0'
account assets:bank                    ; document_check:exempt
account expenses:business:software     ; document_check:required

2026-01-01 Software Suite
    expenses:business:software          92.00 EUR
    assets:bank                        -92.00 EUR
SCENARIO_A_SIDECAR_WITH_ONLY_AMOUNT_AND_CURRENCY_IS_NOT_FLAGGED_0
  mkdir -p "$scenario_dir/documents/expenses/business/software"
  cat >"$scenario_dir/documents/expenses/business/software/2026-01-01-suite.pdf" <<'SCENARIO_A_SIDECAR_WITH_ONLY_AMOUNT_AND_CURRENCY_IS_NOT_FLAGGED_1'
suite invoice
SCENARIO_A_SIDECAR_WITH_ONLY_AMOUNT_AND_CURRENCY_IS_NOT_FLAGGED_1
  mkdir -p "$scenario_dir/documents/expenses/business/software"
  cat >"$scenario_dir/documents/expenses/business/software/2026-01-01-suite.document.yml" <<'SCENARIO_A_SIDECAR_WITH_ONLY_AMOUNT_AND_CURRENCY_IS_NOT_FLAGGED_2'
amount: 92.00
currency: EUR
SCENARIO_A_SIDECAR_WITH_ONLY_AMOUNT_AND_CURRENCY_IS_NOT_FLAGGED_2
}

scenario_a_cover_account_is_missing_a_subaccount_segment_the_folder_implies() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_A_COVER_ACCOUNT_IS_MISSING_A_SUBACCOUNT_SEGMENT_THE_FOLDER_IMPLIES_0'
account assets:bank                                    ; document_check:exempt
account expenses:business:hosting:storage:legacy       ; document_check:required

2026-01-05 Storage Invoice
    expenses:business:hosting:storage:legacy           8.94 EUR
    assets:bank                                        -8.94 EUR
SCENARIO_A_COVER_ACCOUNT_IS_MISSING_A_SUBACCOUNT_SEGMENT_THE_FOLDER_IMPLIES_0
  mkdir -p "$scenario_dir/documents/expenses/business/hosting/storage/legacy"
  cat >"$scenario_dir/documents/expenses/business/hosting/storage/legacy/2026-01-05.pdf" <<'SCENARIO_A_COVER_ACCOUNT_IS_MISSING_A_SUBACCOUNT_SEGMENT_THE_FOLDER_IMPLIES_1'
storage invoice
SCENARIO_A_COVER_ACCOUNT_IS_MISSING_A_SUBACCOUNT_SEGMENT_THE_FOLDER_IMPLIES_1
  mkdir -p "$scenario_dir/documents/expenses/business/hosting/storage/legacy"
  cat >"$scenario_dir/documents/expenses/business/hosting/storage/legacy/2026-01-05.document.yml" <<'SCENARIO_A_COVER_ACCOUNT_IS_MISSING_A_SUBACCOUNT_SEGMENT_THE_FOLDER_IMPLIES_2'
covers:
  - date: 2026-01-05
    account: expenses:business:hosting:storage
    amount: 8.94
    currency: EUR
SCENARIO_A_COVER_ACCOUNT_IS_MISSING_A_SUBACCOUNT_SEGMENT_THE_FOLDER_IMPLIES_2
}

scenario_a_cover_date_doesn_t_match_when_the_document_was_actually_posted() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_A_COVER_DATE_DOESN_T_MATCH_WHEN_THE_DOCUMENT_WAS_ACTUALLY_POSTED_0'
account assets:bank                            ; document_check:exempt
account expenses:business:hosting:mailbox      ; document_check:required

2024-01-05 Mailbox Invoice
    expenses:business:hosting:mailbox          8.94 EUR
    assets:bank                                -8.94 EUR
SCENARIO_A_COVER_DATE_DOESN_T_MATCH_WHEN_THE_DOCUMENT_WAS_ACTUALLY_POSTED_0
  mkdir -p "$scenario_dir/documents/expenses/business/hosting/mailbox"
  cat >"$scenario_dir/documents/expenses/business/hosting/mailbox/2024-01-05-invoice-1234.pdf" <<'SCENARIO_A_COVER_DATE_DOESN_T_MATCH_WHEN_THE_DOCUMENT_WAS_ACTUALLY_POSTED_1'
mailbox invoice
SCENARIO_A_COVER_DATE_DOESN_T_MATCH_WHEN_THE_DOCUMENT_WAS_ACTUALLY_POSTED_1
  mkdir -p "$scenario_dir/documents/expenses/business/hosting/mailbox"
  cat >"$scenario_dir/documents/expenses/business/hosting/mailbox/2024-01-05-invoice-1234.document.yml" <<'SCENARIO_A_COVER_DATE_DOESN_T_MATCH_WHEN_THE_DOCUMENT_WAS_ACTUALLY_POSTED_2'
covers:
  - date: 2024-01-06
    amount: 8.94
    currency: EUR
SCENARIO_A_COVER_DATE_DOESN_T_MATCH_WHEN_THE_DOCUMENT_WAS_ACTUALLY_POSTED_2
}

scenario_a_mistake_in_the_top_level_shorthand_form_is_labeled_top_level_not_covers_n() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_A_MISTAKE_IN_THE_TOP_LEVEL_SHORTHAND_FORM_IS_LABELED_TOP_LEVEL_NOT_COVERS_N_0'
account assets:bank                     ; document_check:exempt
account expenses:business:hosting:aws   ; document_check:required

2022-06-03 AWS EMEA
    expenses:business:hosting:aws        2.99 EUR
    assets:bank                         -2.99 EUR
SCENARIO_A_MISTAKE_IN_THE_TOP_LEVEL_SHORTHAND_FORM_IS_LABELED_TOP_LEVEL_NOT_COVERS_N_0
  mkdir -p "$scenario_dir/documents/expenses/business/hosting/aws"
  cat >"$scenario_dir/documents/expenses/business/hosting/aws/2022-06-03-invoice.pdf" <<'SCENARIO_A_MISTAKE_IN_THE_TOP_LEVEL_SHORTHAND_FORM_IS_LABELED_TOP_LEVEL_NOT_COVERS_N_1'
aws invoice
SCENARIO_A_MISTAKE_IN_THE_TOP_LEVEL_SHORTHAND_FORM_IS_LABELED_TOP_LEVEL_NOT_COVERS_N_1
  mkdir -p "$scenario_dir/documents/expenses/business/hosting/aws"
  cat >"$scenario_dir/documents/expenses/business/hosting/aws/2022-06-03-invoice.document.yml" <<'SCENARIO_A_MISTAKE_IN_THE_TOP_LEVEL_SHORTHAND_FORM_IS_LABELED_TOP_LEVEL_NOT_COVERS_N_2'
date: 2022-06-04
amount: 2.99
currency: EUR
SCENARIO_A_MISTAKE_IN_THE_TOP_LEVEL_SHORTHAND_FORM_IS_LABELED_TOP_LEVEL_NOT_COVERS_N_2
}

scenario_a_non_trivial_wrong_cover_on_a_fully_unmatched_document_is_not_double_reported() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_A_NON_TRIVIAL_WRONG_COVER_ON_A_FULLY_UNMATCHED_DOCUMENT_IS_NOT_DOUBLE_REPORTED_0'
account assets:bank ; document_check:exempt
SCENARIO_A_NON_TRIVIAL_WRONG_COVER_ON_A_FULLY_UNMATCHED_DOCUMENT_IS_NOT_DOUBLE_REPORTED_0
  mkdir -p "$scenario_dir/documents/expenses/business/misc"
  cat >"$scenario_dir/documents/expenses/business/misc/2026-01-01-note.pdf" <<'SCENARIO_A_NON_TRIVIAL_WRONG_COVER_ON_A_FULLY_UNMATCHED_DOCUMENT_IS_NOT_DOUBLE_REPORTED_1'
note
SCENARIO_A_NON_TRIVIAL_WRONG_COVER_ON_A_FULLY_UNMATCHED_DOCUMENT_IS_NOT_DOUBLE_REPORTED_1
  mkdir -p "$scenario_dir/documents/expenses/business/misc"
  cat >"$scenario_dir/documents/expenses/business/misc/2026-01-01-note.document.yml" <<'SCENARIO_A_NON_TRIVIAL_WRONG_COVER_ON_A_FULLY_UNMATCHED_DOCUMENT_IS_NOT_DOUBLE_REPORTED_2'
covers:
  - date: 2026-01-01
    account: expenses:business:nonexistent
    amount: 12.00
    currency: EUR
SCENARIO_A_NON_TRIVIAL_WRONG_COVER_ON_A_FULLY_UNMATCHED_DOCUMENT_IS_NOT_DOUBLE_REPORTED_2
}

scenario_a_legitimate_account_override_that_resolves_to_a_real_transaction_is_not_flagged() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_A_LEGITIMATE_ACCOUNT_OVERRIDE_THAT_RESOLVES_TO_A_REAL_TRANSACTION_IS_NOT_FLAGGED_0'
account assets:bank                                      ; document_check:exempt
account expenses:insurance:health:base                   ; document_check:required
account expenses:insurance:health:nursing-care           ; document_check:required

2026-01-01 Annual Health Insurance Base
    expenses:insurance:health:base                       100.00 EUR
    assets:bank                                         -100.00 EUR

2026-01-01 Annual Nursing Care Insurance
    expenses:insurance:health:nursing-care                30.00 EUR
    assets:bank                                          -30.00 EUR
SCENARIO_A_LEGITIMATE_ACCOUNT_OVERRIDE_THAT_RESOLVES_TO_A_REAL_TRANSACTION_IS_NOT_FLAGGED_0
  mkdir -p "$scenario_dir/documents/expenses/insurance/health"
  cat >"$scenario_dir/documents/expenses/insurance/health/2026-01-01-annual-notice.pdf" <<'SCENARIO_A_LEGITIMATE_ACCOUNT_OVERRIDE_THAT_RESOLVES_TO_A_REAL_TRANSACTION_IS_NOT_FLAGGED_1'
annual health insurance notice
SCENARIO_A_LEGITIMATE_ACCOUNT_OVERRIDE_THAT_RESOLVES_TO_A_REAL_TRANSACTION_IS_NOT_FLAGGED_1
  mkdir -p "$scenario_dir/documents/expenses/insurance/health"
  cat >"$scenario_dir/documents/expenses/insurance/health/2026-01-01-annual-notice.document.yml" <<'SCENARIO_A_LEGITIMATE_ACCOUNT_OVERRIDE_THAT_RESOLVES_TO_A_REAL_TRANSACTION_IS_NOT_FLAGGED_2'
covers:
  - account: expenses:insurance:health:base
    amount: 100.00
    currency: EUR
  - account: expenses:insurance:health:nursing-care
    amount: 30.00
    currency: EUR
SCENARIO_A_LEGITIMATE_ACCOUNT_OVERRIDE_THAT_RESOLVES_TO_A_REAL_TRANSACTION_IS_NOT_FLAGGED_2
}

scenario_a_legitimate_date_override_that_resolves_to_a_real_transaction_is_not_flagged() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_A_LEGITIMATE_DATE_OVERRIDE_THAT_RESOLVES_TO_A_REAL_TRANSACTION_IS_NOT_FLAGGED_0'
account assets:bank                            ; document_check:exempt
account expenses:business:hosting:mailbox      ; document_check:required

2024-01-20 Mailbox Invoice Cleared
    expenses:business:hosting:mailbox          8.94 EUR
    assets:bank                                -8.94 EUR
SCENARIO_A_LEGITIMATE_DATE_OVERRIDE_THAT_RESOLVES_TO_A_REAL_TRANSACTION_IS_NOT_FLAGGED_0
  mkdir -p "$scenario_dir/documents/expenses/business/hosting/mailbox"
  cat >"$scenario_dir/documents/expenses/business/hosting/mailbox/2024-01-05-invoice-1234.pdf" <<'SCENARIO_A_LEGITIMATE_DATE_OVERRIDE_THAT_RESOLVES_TO_A_REAL_TRANSACTION_IS_NOT_FLAGGED_1'
mailbox invoice
SCENARIO_A_LEGITIMATE_DATE_OVERRIDE_THAT_RESOLVES_TO_A_REAL_TRANSACTION_IS_NOT_FLAGGED_1
  mkdir -p "$scenario_dir/documents/expenses/business/hosting/mailbox"
  cat >"$scenario_dir/documents/expenses/business/hosting/mailbox/2024-01-05-invoice-1234.document.yml" <<'SCENARIO_A_LEGITIMATE_DATE_OVERRIDE_THAT_RESOLVES_TO_A_REAL_TRANSACTION_IS_NOT_FLAGGED_2'
covers:
  - date: 2024-01-20
    amount: 8.94
    currency: EUR
SCENARIO_A_LEGITIMATE_DATE_OVERRIDE_THAT_RESOLVES_TO_A_REAL_TRANSACTION_IS_NOT_FLAGGED_2
}

scenario_a_trivial_cover_on_a_genuinely_unmatched_document_is_not_double_reported() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_A_TRIVIAL_COVER_ON_A_GENUINELY_UNMATCHED_DOCUMENT_IS_NOT_DOUBLE_REPORTED_0'
account assets:bank                        ; document_check:exempt
account expenses:business:software         ; document_check:required
SCENARIO_A_TRIVIAL_COVER_ON_A_GENUINELY_UNMATCHED_DOCUMENT_IS_NOT_DOUBLE_REPORTED_0
  mkdir -p "$scenario_dir/documents/expenses/business/software"
  cat >"$scenario_dir/documents/expenses/business/software/2026-01-01-suite.pdf" <<'SCENARIO_A_TRIVIAL_COVER_ON_A_GENUINELY_UNMATCHED_DOCUMENT_IS_NOT_DOUBLE_REPORTED_1'
suite invoice
SCENARIO_A_TRIVIAL_COVER_ON_A_GENUINELY_UNMATCHED_DOCUMENT_IS_NOT_DOUBLE_REPORTED_1
  mkdir -p "$scenario_dir/documents/expenses/business/software"
  cat >"$scenario_dir/documents/expenses/business/software/2026-01-01-suite.document.yml" <<'SCENARIO_A_TRIVIAL_COVER_ON_A_GENUINELY_UNMATCHED_DOCUMENT_IS_NOT_DOUBLE_REPORTED_2'
covers:
  - date: 2026-01-01
    account: expenses:business:software
    amount: 92.00
    currency: EUR
SCENARIO_A_TRIVIAL_COVER_ON_A_GENUINELY_UNMATCHED_DOCUMENT_IS_NOT_DOUBLE_REPORTED_2
}

scenario_one_wrong_cover_among_several_on_an_otherwise_matched_multi_installment_document() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_ONE_WRONG_COVER_AMONG_SEVERAL_ON_AN_OTHERWISE_MATCHED_MULTI_INSTALLMENT_DOCUMENT_0'
account assets:bank                                  ; document_check:exempt
account income:business:freelance:customer           ; document_check:required

2026-01-05 Customer Payment Rate 1
    assets:bank                                      60.00 EUR
    income:business:freelance:customer
SCENARIO_ONE_WRONG_COVER_AMONG_SEVERAL_ON_AN_OTHERWISE_MATCHED_MULTI_INSTALLMENT_DOCUMENT_0
  mkdir -p "$scenario_dir/documents/income/business/freelance/customer"
  cat >"$scenario_dir/documents/income/business/freelance/customer/customer-project.pdf" <<'SCENARIO_ONE_WRONG_COVER_AMONG_SEVERAL_ON_AN_OTHERWISE_MATCHED_MULTI_INSTALLMENT_DOCUMENT_1'
customer project invoice
SCENARIO_ONE_WRONG_COVER_AMONG_SEVERAL_ON_AN_OTHERWISE_MATCHED_MULTI_INSTALLMENT_DOCUMENT_1
  mkdir -p "$scenario_dir/documents/income/business/freelance/customer"
  cat >"$scenario_dir/documents/income/business/freelance/customer/customer-project.document.yml" <<'SCENARIO_ONE_WRONG_COVER_AMONG_SEVERAL_ON_AN_OTHERWISE_MATCHED_MULTI_INSTALLMENT_DOCUMENT_2'
covers:
  - date: 2026-01-05
    amount: 60.00
  - date: 2026-02-05
    amount: 40.00
SCENARIO_ONE_WRONG_COVER_AMONG_SEVERAL_ON_AN_OTHERWISE_MATCHED_MULTI_INSTALLMENT_DOCUMENT_2
}

scenario_a_metadata_sidecar_with_an_unknown_field_is_rejected() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_A_METADATA_SIDECAR_WITH_AN_UNKNOWN_FIELD_IS_REJECTED_0'
account assets:bank ; document_check:exempt
SCENARIO_A_METADATA_SIDECAR_WITH_AN_UNKNOWN_FIELD_IS_REJECTED_0
  mkdir -p "$scenario_dir/documents/expenses/business/software"
  cat >"$scenario_dir/documents/expenses/business/software/2026-01-01-suite.pdf" <<'SCENARIO_A_METADATA_SIDECAR_WITH_AN_UNKNOWN_FIELD_IS_REJECTED_1'
suite invoice
SCENARIO_A_METADATA_SIDECAR_WITH_AN_UNKNOWN_FIELD_IS_REJECTED_1
  mkdir -p "$scenario_dir/documents/expenses/business/software"
  cat >"$scenario_dir/documents/expenses/business/software/2026-01-01-suite.document.yml" <<'SCENARIO_A_METADATA_SIDECAR_WITH_AN_UNKNOWN_FIELD_IS_REJECTED_2'
amount: 100.00
unexpected: true
SCENARIO_A_METADATA_SIDECAR_WITH_AN_UNKNOWN_FIELD_IS_REJECTED_2
}

scenario_metadata_notes_can_contain_arbitrary_text_or_structured_data() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_METADATA_NOTES_CAN_CONTAIN_ARBITRARY_TEXT_OR_STRUCTURED_DATA_0'
account assets:bank                         ; document_check:exempt
account expenses:business:software          ; document_check:required

2026-01-01 Software Suite
    expenses:business:software              92.00 EUR
    assets:bank                            -92.00 EUR
SCENARIO_METADATA_NOTES_CAN_CONTAIN_ARBITRARY_TEXT_OR_STRUCTURED_DATA_0
  mkdir -p "$scenario_dir/documents/expenses/business/software"
  cat >"$scenario_dir/documents/expenses/business/software/2026-01-01-suite.pdf" <<'SCENARIO_METADATA_NOTES_CAN_CONTAIN_ARBITRARY_TEXT_OR_STRUCTURED_DATA_1'
software suite invoice
SCENARIO_METADATA_NOTES_CAN_CONTAIN_ARBITRARY_TEXT_OR_STRUCTURED_DATA_1
  mkdir -p "$scenario_dir/documents/expenses/business/software"
  cat >"$scenario_dir/documents/expenses/business/software/2026-01-02-text-note.document.yml" <<'SCENARIO_METADATA_NOTES_CAN_CONTAIN_ARBITRARY_TEXT_OR_STRUCTURED_DATA_2'
notes: "Keep until the yearly review is done."
SCENARIO_METADATA_NOTES_CAN_CONTAIN_ARBITRARY_TEXT_OR_STRUCTURED_DATA_2
  mkdir -p "$scenario_dir/documents/expenses/business/software"
  cat >"$scenario_dir/documents/expenses/business/software/2026-01-01-suite.document.yml" <<'SCENARIO_METADATA_NOTES_CAN_CONTAIN_ARBITRARY_TEXT_OR_STRUCTURED_DATA_3'
amount: 92.00
currency: EUR
description: Software Suite
notes:
  reason: "Bank converted the foreign-currency charge"
  source_lines:
    - "invoice total: 100.00 USD"
    - "charged amount: 92.00 EUR"
covers:
  - amount: 92.00
    description: Software Suite
SCENARIO_METADATA_NOTES_CAN_CONTAIN_ARBITRARY_TEXT_OR_STRUCTURED_DATA_3
}
