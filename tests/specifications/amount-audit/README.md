# Amount Audit

<!--
```scrut {fail_fast: true, output_stream: combined}
$ source "$TESTDIR/../readme-setup.sh"
```
-->

Coverage proves that a document is connected to a required posting group. Amount
audit goes one step further and checks whether linked document totals plausibly
match linked transaction totals.

The amount can come from a parsed PDF total or from `*.document.yml`. Metadata is
preferred when a PDF is hard to parse, when the bank transaction is already
converted into another currency, or when one document covers multiple groups.

Invoices typically show positive amounts, so the audit ignores sign and compares
magnitude only, refunds don't need a negative `amount`.

## Warning Cases

Some linked groups cannot be audited. By default these skip reasons are only
summarized, but `--warn-on amount-audit-skips` prints the details.

Amount audit explains why a linked group could not be compared.

### Every amount-audit skip reason is reported

```scrut
$ build_scenario scenario_every_amount_audit_skip_reason_is_reported
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --warn-on amount-audit-skips --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Amount Audit Skips (5):
  document/transaction currency mismatch
    documents/expenses/business/currency-mismatch/2026-01-06-usd-invoice.pdf
      accounts/dates: expenses/business/currency-mismatch @ 2026-01-06
  document amount unreadable
    documents/expenses/business/unreadable-document/2026-01-02-unreadable.pdf
      accounts/dates: expenses/business/unreadable-document @ 2026-01-02
  document currencies unclear
    documents/expenses/business/unclear-document-currency/2026-01-03-unclear.pdf
      accounts/dates: expenses/business/unclear-document-currency @ 2026-01-03
  transaction amount missing
    documents/expenses/business/missing-transaction-amount/2026-01-04-missing-amount.pdf
      accounts/dates: expenses/business/missing-transaction-amount @ 2026-01-04
  transaction currencies mixed
    documents/expenses/business/mixed-transaction-currencies/2026-01-05-mixed.pdf
      accounts/dates: expenses/business/mixed-transaction-currencies @ 2026-01-05

Ambiguous Transaction Groups (1):
  Date-prefixed filenames can only match by account and date.
  These groups have multiple required transactions on the same date in the same account:
  expenses/business/mixed-transaction-currencies @ 2026-01-05  [2 transactions]
    2026-01-05  10.00 EUR  Mixed Transaction Currencies
    2026-01-05  12.00 USD  Mixed Transaction Currencies

Redundant Metadata Fields (10):
  These fields already match what the file's location implies and can be removed.
  documents/expenses/business/currency-mismatch/2026-01-06-usd-invoice.document.yml
    covers[0] date: 2026-01-06
    covers[0] account: expenses:business:currency-mismatch
  documents/expenses/business/missing-transaction-amount/2026-01-04-missing-amount.document.yml
    covers[0] date: 2026-01-04
    covers[0] account: expenses:business:missing-transaction-amount
  documents/expenses/business/mixed-transaction-currencies/2026-01-05-mixed.document.yml
    covers[0] date: 2026-01-05
    covers[0] account: expenses:business:mixed-transaction-currencies
  documents/expenses/business/unclear-document-currency/2026-01-03-unclear.document.yml
    covers[0] date: 2026-01-03
    covers[0] account: expenses:business:unclear-document-currency
    covers[1] date: 2026-01-03
    covers[1] account: expenses:business:unclear-document-currency

Summary:
  Coverage:
    5/5 transaction groups covered
    0 missing-document placeholders
  Open Items:
    0 missing document coverage
    0 unbooked documents
    0 overdue unbooked documents
    0 unmatched documents
    0 unexpected files
    0 duplicate groups
    10 redundant metadata fields
    0 unresolvable cover metadata fields
  Amount Audit:
    0 amount mismatches
    0 linked groups checked
    5 linked groups skipped
    Skip Reasons:
      1 document amount unreadable
      1 document currencies unclear
      1 transaction amount missing
      1 transaction currencies mixed
      1 document/transaction currency mismatch
OK.
```

## Warnings As Failures

Use `--fail-on amount-audit-skips` if unreadable or otherwise skipped amount
checks should break the run.

Skip reasons can be promoted to failures.

### Parsed amount warnings can be made fatal

```scrut
$ build_scenario scenario_parsed_amount_warnings_can_be_made_fatal
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --fail-on amount-audit-skips --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Amount Audit Skips (1):
  document amount unreadable
    documents/expenses/business/hosting/2026-01-02-hosting.pdf
      accounts/dates: expenses/business/hosting @ 2026-01-02

Summary:
  Coverage:
    1/1 transaction groups covered
    0 missing-document placeholders
  Open Items:
    0 missing document coverage
    0 unbooked documents
    0 overdue unbooked documents
    0 unmatched documents
    0 unexpected files
    0 duplicate groups
    0 redundant metadata fields
    0 unresolvable cover metadata fields
  Amount Audit:
    0 amount mismatches
    0 linked groups checked
    1 linked groups skipped
    Skip Reasons:
      1 document amount unreadable
      0 document currencies unclear
      0 transaction amount missing
      0 transaction currencies mixed
      0 document/transaction currency mismatch
[1]
```

## Override Amount And Currency

For foreign-currency invoices, the ledger may intentionally contain the bank's
converted amount. Put the amount to audit in `*.document.yml`; the comparison is
then made against that metadata amount.

Metadata supplies the auditable amount when the PDF total is not the ledger amount.

### A foreign-currency invoice is audited with the converted bank amount

```scrut
$ build_scenario scenario_a_foreign_currency_invoice_is_audited_with_the_converted_bank_amount
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Redundant Metadata Fields (2):
  These fields already match what the file's location implies and can be removed.
  documents/expenses/business/cloud/2026-01-04-cloud.document.yml
    covers[0] date: 2026-01-04
    covers[0] account: expenses:business:cloud

Summary:
  Coverage:
    1/1 transaction groups covered
    0 missing-document placeholders
  Open Items:
    0 missing document coverage
    0 unbooked documents
    0 overdue unbooked documents
    0 unmatched documents
    0 unexpected files
    0 duplicate groups
    2 redundant metadata fields
    0 unresolvable cover metadata fields
  Amount Audit:
    0 amount mismatches
    1 linked groups checked
    0 linked groups skipped
    Skip Reasons:
      0 document amount unreadable
      0 document currencies unclear
      0 transaction amount missing
      0 transaction currencies mixed
      0 document/transaction currency mismatch
OK.
```

## Mismatch Is A Failure

If metadata provides a clear amount and currency, a mismatch against the ledger
is a hard failure by default.

A clear metadata amount must match the linked transaction amount.

### Metadata amount differs from the transaction amount

```scrut
$ build_scenario scenario_metadata_amount_differs_from_the_transaction_amount
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Amount Mismatches (1):
  document total 100.00 != transaction total 70.00
    accounts/dates: expenses/business/software @ 2026-01-03
    documents/expenses/business/software/2026-01-03-suite.pdf

Redundant Metadata Fields (2):
  These fields already match what the file's location implies and can be removed.
  documents/expenses/business/software/2026-01-03-suite.document.yml
    covers[0] date: 2026-01-03
    covers[0] account: expenses:business:software

Summary:
  Coverage:
    1/1 transaction groups covered
    0 missing-document placeholders
  Open Items:
    0 missing document coverage
    0 unbooked documents
    0 overdue unbooked documents
    0 unmatched documents
    0 unexpected files
    0 duplicate groups
    2 redundant metadata fields
    0 unresolvable cover metadata fields
  Amount Audit:
    1 amount mismatches
    1 linked groups checked
    0 linked groups skipped
    Skip Reasons:
      0 document amount unreadable
      0 document currencies unclear
      0 transaction amount missing
      0 transaction currencies mixed
      0 document/transaction currency mismatch
[1]
```
