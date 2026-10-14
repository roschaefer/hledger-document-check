# Unbooked Invoices

<!--
```scrut {fail_fast: true, output_stream: combined}
$ source "$TESTDIR/../readme-setup.sh"
```
-->

Use an `unbooked/` subfolder inside the account folder for documents that are
real but do not yet have a matching ledger transaction.

Two common cases are:

- you paid an invoice online, but the bank transaction has not cleared yet
- you sent an invoice to a customer, but the customer has not paid yet

Unbooked documents are warnings by default.

Unbooked documents are visible while waiting for the ledger transaction.

### A paid online invoice exists before the bank transaction clears

```scrut
$ build_scenario scenario_a_paid_online_invoice_exists_before_the_bank_transaction_clears
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore amount-audit-skips --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Unbooked Documents (1):
  documents/expenses/business/hosting/aws/unbooked/invoice.pdf
    amount: unknown

Summary:
  Coverage:
    0/0 transaction groups covered
    0 missing-document placeholders
  Open Items:
    0 missing document coverage
    1 unbooked documents
    0 overdue unbooked documents
    0 unmatched documents
    0 unexpected files
    0 duplicate groups
    0 redundant metadata fields
    0 unresolvable cover metadata fields
  Amount Audit:
    0 amount mismatches
    0 linked groups checked
    0 linked groups skipped
    Skip Reasons:
      0 document amount unreadable
      0 document currencies unclear
      0 transaction amount missing
      0 transaction currencies mixed
      0 document/transaction currency mismatch
OK.
```

### A sent invoice exists before the customer pays

```scrut
$ build_scenario scenario_a_sent_invoice_exists_before_the_customer_pays
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --today 2026-05-20 --ignore amount-audit-skips --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Unbooked Documents (1):
  documents/income/business/freelance/customer/unbooked/invoice.pdf
    amount: unknown

Summary:
  Coverage:
    0/0 transaction groups covered
    0 missing-document placeholders
  Open Items:
    0 missing document coverage
    1 unbooked documents
    0 overdue unbooked documents
    0 unmatched documents
    0 unexpected files
    0 duplicate groups
    0 redundant metadata fields
    0 unresolvable cover metadata fields
  Amount Audit:
    0 amount mismatches
    0 linked groups checked
    0 linked groups skipped
    Skip Reasons:
      0 document amount unreadable
      0 document currencies unclear
      0 transaction amount missing
      0 transaction currencies mixed
      0 document/transaction currency mismatch
OK.
```

## Move Suggestions

If a unique required transaction with the same account, amount, and currency
appears later, the check prints an explicit `mv` suggestion. The suggestion
renames the unbooked document to the transaction date. It also moves the sidecar
metadata file and sibling support directory when they exist. The target is one
level up in the account folder. After that rename, a due invoice has a
corresponding transaction and is no longer overdue.

When the PDF amount cannot be read reliably, the unbooked document's
`*.document.yml` sidecar can provide the amount and currency used for matching.

When multiple transactions match an unbooked document no move suggestion is printed.

### An unbooked document has no move suggestion when multiple transactions match

```scrut
$ build_scenario scenario_an_unbooked_document_has_no_move_suggestion_when_multiple_transactions_match
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore amount-audit-skips --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Missing Document Coverage (2):
  2026-01-02  AWS Invoice A
    account: expenses:business:hosting:aws
    amount: 12.34 EUR
  2026-01-02  AWS Invoice B
    account: expenses:business:hosting:aws
    amount: 12.34 EUR

Unbooked Documents (1):
  documents/expenses/business/hosting/aws/unbooked/invoice.pdf
    amount: 12.34 EUR

Ambiguous Transaction Groups (1):
  Date-prefixed filenames can only match by account and date.
  These groups have multiple required transactions on the same date in the same account:
  expenses/business/hosting/aws @ 2026-01-02  [2 transactions]
    2026-01-02  12.34 EUR  AWS Invoice A
    2026-01-02  12.34 EUR  AWS Invoice B

Summary:
  Coverage:
    0/1 transaction groups covered
    0 missing-document placeholders
  Open Items:
    2 missing document coverage
    1 unbooked documents
    0 overdue unbooked documents
    0 unmatched documents
    0 unexpected files
    0 duplicate groups
    0 redundant metadata fields
    0 unresolvable cover metadata fields
  Amount Audit:
    0 amount mismatches
    0 linked groups checked
    0 linked groups skipped
    Skip Reasons:
      0 document amount unreadable
      0 document currencies unclear
      0 transaction amount missing
      0 transaction currencies mixed
      0 document/transaction currency mismatch
[1]
```

The unique-match move behavior is exercised by the
[unique-match move example](#a-matching-transaction-appears-for-an-unbooked-document-with-a-sidecar-amount).

Metadata can supply the amount for an unbooked document whose PDF amount is not readable.

### A matching transaction appears for an unbooked document with a sidecar amount

```scrut
$ build_scenario scenario_a_matching_transaction_appears_for_an_unbooked_document_with_a_sidecar_amount
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore amount-audit-skips --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Missing Document Coverage (1):
  2026-06-10  FlixTrain
    account: expenses:business:transport:train:flixtrain
    amount: 14.99 EUR

Unbooked Documents (1):
  documents/expenses/business/transport/train/flixtrain/unbooked/booking.pdf
    amount: 14.99 EUR

Suggested Moves (1):
  High-confidence unbooked document matches by account, amount, and currency.
  2026-06-10  FlixTrain
    account: expenses:business:transport:train:flixtrain
    amount: 14.99 EUR
    mv {working_directory}/documents/expenses/business/transport/train/flixtrain/unbooked/booking.pdf {working_directory}/documents/expenses/business/transport/train/flixtrain/2026-06-10-booking.pdf
    mv {working_directory}/documents/expenses/business/transport/train/flixtrain/unbooked/booking.document.yml {working_directory}/documents/expenses/business/transport/train/flixtrain/2026-06-10-booking.document.yml
    mv {working_directory}/documents/expenses/business/transport/train/flixtrain/unbooked/booking {working_directory}/documents/expenses/business/transport/train/flixtrain/2026-06-10-booking

Summary:
  Coverage:
    0/1 transaction groups covered
    0 missing-document placeholders
  Open Items:
    1 missing document coverage
    1 unbooked documents
    0 overdue unbooked documents
    0 unmatched documents
    0 unexpected files
    0 duplicate groups
    0 redundant metadata fields
    0 unresolvable cover metadata fields
  Amount Audit:
    0 amount mismatches
    0 linked groups checked
    0 linked groups skipped
    Skip Reasons:
      0 document amount unreadable
      0 document currencies unclear
      0 transaction amount missing
      0 transaction currencies mixed
      0 document/transaction currency mismatch
[1]
```
