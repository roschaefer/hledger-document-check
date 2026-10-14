# shellcheck shell=bash

scenario_a_paid_online_invoice_exists_before_the_bank_transaction_clears() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_A_PAID_ONLINE_INVOICE_EXISTS_BEFORE_THE_BANK_TRANSACTION_CLEARS_0'
account assets:bank                         ; document_check:exempt
account expenses:business:hosting:aws       ; document_check:required
SCENARIO_A_PAID_ONLINE_INVOICE_EXISTS_BEFORE_THE_BANK_TRANSACTION_CLEARS_0
  mkdir -p "$scenario_dir/documents/expenses/business/hosting/aws/unbooked"
  cat >"$scenario_dir/documents/expenses/business/hosting/aws/unbooked/invoice.pdf" <<'SCENARIO_A_PAID_ONLINE_INVOICE_EXISTS_BEFORE_THE_BANK_TRANSACTION_CLEARS_1'
AWS invoice waiting for bank clearing
SCENARIO_A_PAID_ONLINE_INVOICE_EXISTS_BEFORE_THE_BANK_TRANSACTION_CLEARS_1
}

scenario_a_sent_invoice_exists_before_the_customer_pays() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_A_SENT_INVOICE_EXISTS_BEFORE_THE_CUSTOMER_PAYS_0'
account assets:bank                                  ; document_check:exempt
account income:business:freelance:customer           ; document_check:required
SCENARIO_A_SENT_INVOICE_EXISTS_BEFORE_THE_CUSTOMER_PAYS_0
  mkdir -p "$scenario_dir/documents/income/business/freelance/customer/unbooked"
  cat >"$scenario_dir/documents/income/business/freelance/customer/unbooked/invoice.pdf" <<'SCENARIO_A_SENT_INVOICE_EXISTS_BEFORE_THE_CUSTOMER_PAYS_1'
customer invoice waiting for payment
SCENARIO_A_SENT_INVOICE_EXISTS_BEFORE_THE_CUSTOMER_PAYS_1
  mkdir -p "$scenario_dir/documents/income/business/freelance/customer/unbooked"
  cat >"$scenario_dir/documents/income/business/freelance/customer/unbooked/invoice.document.yml" <<'SCENARIO_A_SENT_INVOICE_EXISTS_BEFORE_THE_CUSTOMER_PAYS_2'
due_date: 2026-05-31
SCENARIO_A_SENT_INVOICE_EXISTS_BEFORE_THE_CUSTOMER_PAYS_2
}

scenario_an_unbooked_document_has_no_move_suggestion_when_multiple_transactions_match() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_AN_UNBOOKED_DOCUMENT_HAS_NO_MOVE_SUGGESTION_WHEN_MULTIPLE_TRANSACTIONS_MATCH_0'
account assets:bank                     ; document_check:exempt
account expenses:business:hosting:aws   ; document_check:required

2026-01-02 AWS Invoice A
    expenses:business:hosting:aws   12.34 EUR
    assets:bank                    -12.34 EUR

2026-01-02 AWS Invoice B
    expenses:business:hosting:aws   12.34 EUR
    assets:bank                    -12.34 EUR
SCENARIO_AN_UNBOOKED_DOCUMENT_HAS_NO_MOVE_SUGGESTION_WHEN_MULTIPLE_TRANSACTIONS_MATCH_0
  mkdir -p "$scenario_dir/documents/expenses/business/hosting/aws/unbooked"
  base64 --decode >"$scenario_dir/documents/expenses/business/hosting/aws/unbooked/invoice.pdf" <<'SCENARIO_AN_UNBOOKED_DOCUMENT_HAS_NO_MOVE_SUGGESTION_WHEN_MULTIPLE_TRANSACTIONS_MATCH_1'
JVBERi0xLjQKMSAwIG9iago8PCAvVHlwZSAvQ2F0YWxvZyAvUGFnZXMgMiAwIFIgPj4KZW5kb2JqCjIgMCBvYmoKPDwgL1R5cGUgL1BhZ2VzIC9LaWRzIFszIDAgUl0gL0NvdW50IDEgPj4KZW5kb2JqCjMgMCBvYmoKPDwgL1R5cGUgL1BhZ2UgL1BhcmVudCAyIDAgUiAvTWVkaWFCb3ggWzAgMCA2MTIgNzkyXSAvQ29udGVudHMgNCAwIFIgL1Jlc291cmNlcyA8PCAvRm9udCA8PCAvRjEgNSAwIFIgPj4gPj4gPj4KZW5kb2JqCjQgMCBvYmoKPDwgL0xlbmd0aCA0MCA+PgpzdHJlYW0KQlQKL0YxIDEyIFRmCjcyIDcyMCBUZAooMTIuMzQgRVVSKSBUagpFVAplbmRzdHJlYW0KZW5kb2JqCjUgMCBvYmoKPDwgL1R5cGUgL0ZvbnQgL1N1YnR5cGUgL1R5cGUxIC9CYXNlRm9udCAvSGVsdmV0aWNhID4+CmVuZG9iagp4cmVmCjAgNgowMDAwMDAwMDAwIDY1NTM1IGYgCjAwMDAwMDAwMDkgMDAwMDAgbiAKMDAwMDAwMDA1OCAwMDAwMCBuIAowMDAwMDAwMTE1IDAwMDAwIG4gCjAwMDAwMDAyNDEgMDAwMDAgbiAKMDAwMDAwMDMzMSAwMDAwMCBuIAp0cmFpbGVyCjw8IC9TaXplIDYgL1Jvb3QgMSAwIFIgPj4Kc3RhcnR4cmVmCjQwMQolJUVPRgo=
SCENARIO_AN_UNBOOKED_DOCUMENT_HAS_NO_MOVE_SUGGESTION_WHEN_MULTIPLE_TRANSACTIONS_MATCH_1
}

scenario_a_matching_transaction_appears_for_an_unbooked_document_with_a_sidecar_amount() {
  local scenario_dir=$1
  mkdir -p "$scenario_dir"
  mkdir -p "$scenario_dir/."
  cat >"$scenario_dir/journal.journal" <<'SCENARIO_A_MATCHING_TRANSACTION_APPEARS_FOR_AN_UNBOOKED_DOCUMENT_WITH_A_SIDECAR_AMOUNT_0'
account assets:bank                                   ; document_check:exempt
account expenses:business:transport:train:flixtrain   ; document_check:required

2026-06-10 FlixTrain
    expenses:business:transport:train:flixtrain   14.99 EUR
    assets:bank                                  -14.99 EUR
SCENARIO_A_MATCHING_TRANSACTION_APPEARS_FOR_AN_UNBOOKED_DOCUMENT_WITH_A_SIDECAR_AMOUNT_0
  mkdir -p "$scenario_dir/documents/expenses/business/transport/train/flixtrain/unbooked"
  base64 --decode >"$scenario_dir/documents/expenses/business/transport/train/flixtrain/unbooked/booking.pdf" <<'SCENARIO_A_MATCHING_TRANSACTION_APPEARS_FOR_AN_UNBOOKED_DOCUMENT_WITH_A_SIDECAR_AMOUNT_1'
JVBERi0xLjQKMSAwIG9iago8PCAvVHlwZSAvQ2F0YWxvZyAvUGFnZXMgMiAwIFIgPj4KZW5kb2JqCjIgMCBvYmoKPDwgL1R5cGUgL1BhZ2VzIC9LaWRzIFszIDAgUl0gL0NvdW50IDEgPj4KZW5kb2JqCjMgMCBvYmoKPDwgL1R5cGUgL1BhZ2UgL1BhcmVudCAyIDAgUiAvTWVkaWFCb3ggWzAgMCA2MTIgNzkyXSAvQ29udGVudHMgNCAwIFIgL1Jlc291cmNlcyA8PCAvRm9udCA8PCAvRjEgNSAwIFIgPj4gPj4gPj4KZW5kb2JqCjQgMCBvYmoKPDwgL0xlbmd0aCA2NiA+PgpzdHJlYW0KQlQKL0YxIDEyIFRmCjcyIDcyMCBUZAooVGlja2V0L0ludm9pY2UgZm9yIHlvdXIgcmVzZXJ2YXRpb24pIFRqCkVUCmVuZHN0cmVhbQplbmRvYmoKNSAwIG9iago8PCAvVHlwZSAvRm9udCAvU3VidHlwZSAvVHlwZTEgL0Jhc2VGb250IC9IZWx2ZXRpY2EgPj4KZW5kb2JqCnhyZWYKMCA2CjAwMDAwMDAwMDAgNjU1MzUgZiAKMDAwMDAwMDAwOSAwMDAwMCBuIAowMDAwMDAwMDU4IDAwMDAwIG4gCjAwMDAwMDAxMTUgMDAwMDAgbiAKMDAwMDAwMDI0MSAwMDAwMCBuIAowMDAwMDAwMzU3IDAwMDAwIG4gCnRyYWlsZXIKPDwgL1NpemUgNiAvUm9vdCAxIDAgUiA+PgpzdGFydHhyZWYKNDI3CiUlRU9GCg==
SCENARIO_A_MATCHING_TRANSACTION_APPEARS_FOR_AN_UNBOOKED_DOCUMENT_WITH_A_SIDECAR_AMOUNT_1
  mkdir -p "$scenario_dir/documents/expenses/business/transport/train/flixtrain/unbooked"
  cat >"$scenario_dir/documents/expenses/business/transport/train/flixtrain/unbooked/booking.document.yml" <<'SCENARIO_A_MATCHING_TRANSACTION_APPEARS_FOR_AN_UNBOOKED_DOCUMENT_WITH_A_SIDECAR_AMOUNT_2'
covers:
  - amount: 14.99
    currency: EUR
SCENARIO_A_MATCHING_TRANSACTION_APPEARS_FOR_AN_UNBOOKED_DOCUMENT_WITH_A_SIDECAR_AMOUNT_2
  mkdir -p "$scenario_dir/documents/expenses/business/transport/train/flixtrain/unbooked/booking"
  cat >"$scenario_dir/documents/expenses/business/transport/train/flixtrain/unbooked/booking/usage.csv" <<'SCENARIO_A_MATCHING_TRANSACTION_APPEARS_FOR_AN_UNBOOKED_DOCUMENT_WITH_A_SIDECAR_AMOUNT_3'
date,distance_km
2026-06-10,450
SCENARIO_A_MATCHING_TRANSACTION_APPEARS_FOR_AN_UNBOOKED_DOCUMENT_WITH_A_SIDECAR_AMOUNT_3
}
