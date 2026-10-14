# Special Cases With `.document.yml`

<!--
```scrut {fail_fast: true, output_stream: combined}
$ source "$TESTDIR/../readme-setup.sh"
```
-->

The filename convention intentionally stays small. Use a sibling
`*.document.yml` file whenever account/date matching is ambiguous or too simple.

The `covers` list names the ledger posting groups covered by a document. A cover
entry can specify:

- `date`: posting date in `YYYY-MM-DD` format
- `account`: ledger account, written with `:` separators
- `amount`: document amount allocated to that posting group
- `currency`: currency for that amount
- `description`: optional human description for the matched transaction

The top-level `amount` and `currency` describe the document itself. A
`covers[].amount` value is the document amount allocated to the matched ledger
posting group. The actual transaction amount always comes from hledger.

```yaml
amount: 100.00
currency: EUR

covers:
  - date: 2026-01-05
    account: income:business:freelance:customer
    amount: 60.00
    currency: EUR
    description: Customer payment rate 1
  - date: 2026-02-05
    account: income:business:freelance:customer
    amount: 40.00
    currency: EUR
    description: Customer payment rate 2
```

When top-level `amount` is present, all cover amounts for that document must add
up to it, otherwise the amount audit fails. When no top-level amount is present,
the tool trusts the listed cover amounts and only compares the connected
component against the ledger.

For a metadata file that covers one transaction, the cover fields can also be
written directly at the top level:

```yaml
date: 2022-06-03
account: Expenses:Business:Hosting:AWS
amount: 2.99
currency: EUR
description: AWS EMEA
```

## Single Invoice Covering Many Transactions

A customer may pay one invoice in rates. One invoice document can cover several
bank transactions by listing every covered posting group.

A document metadata file can connect one invoice to several required postings.

### A customer pays one invoice in two rates

```scrut
$ build_scenario scenario_a_customer_pays_one_invoice_in_two_rates
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Redundant Metadata Fields (2):
  These fields already match what the file's location implies and can be removed.
  documents/income/business/freelance/customer/customer-project.document.yml
    covers[0] account: income:business:freelance:customer
    covers[1] account: income:business:freelance:customer

Summary:
  Coverage:
    2/2 transaction groups covered
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

### Cover amounts must add up to the document total when total is configured

```scrut
$ build_scenario scenario_cover_amounts_must_add_up_to_the_document_total_when_total_is_configured
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Amount Mismatches (1):
  allocated document amounts 90.00 != declared document total 100.00
    accounts/dates: income/business/freelance/customer @ 2026-01-05, 2026-02-05
    documents/income/business/freelance/customer/customer-project.pdf

Redundant Metadata Fields (2):
  These fields already match what the file's location implies and can be removed.
  documents/income/business/freelance/customer/customer-project.document.yml
    covers[0] account: income:business:freelance:customer
    covers[1] account: income:business:freelance:customer

Summary:
  Coverage:
    2/2 transaction groups covered
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

## Multiple Invoices Covering One Transaction

A customer may pay two invoices in one bank transaction. Store both invoice PDFs
and let each metadata file point to the same ledger posting group with its share
of the total transaction amount. If each document is paid completely in the
batch, set both the document amount and the cover amount to the invoice total. If
a document is only partly paid, its cover amounts do not yet add up to the
top-level document amount, so the amount audit reports the outstanding
difference until covers for the remaining payments are added.

Several invoice files can connect to the same required posting group.

### A customer pays two invoices at once

```scrut
$ build_scenario scenario_a_customer_pays_two_invoices_at_once
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Redundant Metadata Fields (4):
  These fields already match what the file's location implies and can be removed.
  documents/income/business/freelance/customer/2026-03-01-invoice-a.document.yml
    covers[0] date: 2026-03-01
    covers[0] account: income:business:freelance:customer
  documents/income/business/freelance/customer/2026-03-01-invoice-b.document.yml
    covers[0] date: 2026-03-01
    covers[0] account: income:business:freelance:customer

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
    4 redundant metadata fields
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

### Cover amounts must add up to the transaction amount

```scrut
$ build_scenario scenario_cover_amounts_must_add_up_to_the_transaction_amount
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Amount Mismatches (1):
  document total 90.00 != transaction total 100.00
    accounts/dates: income/business/freelance/customer @ 2026-03-01
    documents/income/business/freelance/customer/2026-03-01-invoice-a.pdf
    documents/income/business/freelance/customer/2026-03-01-invoice-b.pdf

Redundant Metadata Fields (4):
  These fields already match what the file's location implies and can be removed.
  documents/income/business/freelance/customer/2026-03-01-invoice-a.document.yml
    covers[0] date: 2026-03-01
    covers[0] account: income:business:freelance:customer
  documents/income/business/freelance/customer/2026-03-01-invoice-b.document.yml
    covers[0] date: 2026-03-01
    covers[0] account: income:business:freelance:customer

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
    4 redundant metadata fields
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

## Multiple Invoices On The Same Date

Date-prefixed filenames only know account and date. If two required transactions
share the same account and date, metadata can list the amount contributed by
each file. The checker audits the files and transactions as one connected group;
it does not attribute an individual file to an individual transaction.

Metadata can resolve several invoices for the same account and date.

### Two same-date invoices cover one same-date transaction group

```scrut
$ build_scenario scenario_two_same_date_invoices_cover_one_same_date_transaction_group
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Redundant Metadata Fields (4):
  These fields already match what the file's location implies and can be removed.
  documents/expenses/business/domains/2026-03-01-domain-a.document.yml
    covers[0] date: 2026-03-01
    covers[0] account: expenses:business:domains
  documents/expenses/business/domains/2026-03-01-domain-b.document.yml
    covers[0] date: 2026-03-01
    covers[0] account: expenses:business:domains

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
    4 redundant metadata fields
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

## Override Account For Many Invoices

Some documents are filed under a parent folder but cover transactions in several
subaccounts. Health-insurance annual notices are a typical case: one notice can
describe health, nursing-care, and surcharge postings. Each cover entry can
override the account explicitly.

A document filed in one folder can cover postings in several subaccounts.

### One annual health-insurance notice covers several subaccounts

```scrut
$ build_scenario scenario_one_annual_health_insurance_notice_covers_several_subaccounts
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Redundant Metadata Fields (2):
  These fields already match what the file's location implies and can be removed.
  documents/expenses/insurance/health/2026-01-01-annual-notice.document.yml
    covers[0] date: 2026-01-01
    covers[1] date: 2026-01-01

Summary:
  Coverage:
    2/2 transaction groups covered
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

## Redundant Metadata

A `covers[].date`/`covers[].account` (or the top-level shorthand `date`/`account`)
that exactly matches what the file's own location already implies — the
filename date and the account folder — is redundant: removing it changes
nothing about how the document matches. The `redundant-metadata` check finds
these fields so they can be deleted. It only reports a `warn` by default, so it
does not fail the build on its own.

Explicit date/account values that duplicate the file's own location are
reported so they can be removed.

### A covers entry repeats the filename date and folder account

```scrut
$ build_scenario scenario_a_covers_entry_repeats_the_filename_date_and_folder_account
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Redundant Metadata Fields (2):
  These fields already match what the file's location implies and can be removed.
  documents/expenses/business/hosting/uberspace/2026-01-05-invoice.document.yml
    covers[0] date: 2026-01-05
    covers[0] account: expenses:business:hosting:uberspace

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

### A top-level shorthand date/account repeats the file's own location

```scrut
$ build_scenario scenario_a_top_level_shorthand_date_account_repeats_the_file_s_own_location
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Redundant Metadata Fields (2):
  These fields already match what the file's location implies and can be removed.
  documents/expenses/business/hosting/aws/2022-06-03-invoice.document.yml
    top-level date: 2022-06-03
    top-level account: expenses:business:hosting:aws

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

### An account override that names a different account is not flagged

```scrut
$ build_scenario scenario_an_account_override_that_names_a_different_account_is_not_flagged
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Summary:
  Coverage:
    2/2 transaction groups covered
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

### A sidecar with only amount and currency is not flagged

```scrut
$ build_scenario scenario_a_sidecar_with_only_amount_and_currency_is_not_flagged
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

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

## Unresolvable Cover Metadata

A `covers[]` entry — its account, its date, or both — that doesn't match any
required transaction is silently dropped: matching falls back to (or is
carried by) the file's own directory/filename identity instead, so the
document can still end up matched correctly and nothing else surfaces the
mistake. The `unresolvable-cover-metadata` check flags these covers
directly, and suggests the file's own location as the likely intended value
when that location does have a matching transaction. There's no legitimate
reason for a cover to resolve to nothing — every real use (redundant
restatement, an account/date override, a multi-installment document) points
at a transaction that actually exists — so this check fails the build by
default, the same as `unmatched-documents`.

Two kinds of cover are never flagged this way even when they resolve to
nothing: one that asserts nothing beyond the file's own location (that's
`redundant-metadata`'s job to clean up, not a mistake), and any cover on a
document that has no resolved groups at all — that document is entirely
unmatched, already reported once by `unmatched-documents` instead of once
per bad cover.

A cover whose account/date pair matches no required transaction is
reported, with a suggestion when the file's own location would have
matched instead.

### A cover account is missing a subaccount segment the folder implies

```scrut
$ build_scenario scenario_a_cover_account_is_missing_a_subaccount_segment_the_folder_implies
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Redundant Metadata Fields (1):
  These fields already match what the file's location implies and can be removed.
  documents/expenses/business/hosting/storage/legacy/2026-01-05.document.yml
    covers[0] date: 2026-01-05

Unresolvable Cover Metadata (1):
  A cover doesn't correspond to any required transaction; the metadata is probably wrong.
  documents/expenses/business/hosting/storage/legacy/2026-01-05.document.yml
    covers[0] declares expenses/business/hosting/storage @ 2026-01-05 — no such transaction; the file's location implies expenses/business/hosting/storage/legacy @ 2026-01-05 instead

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
    1 redundant metadata fields
    1 unresolvable cover metadata fields
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
[1]
```

### A cover date doesn't match when the document was actually posted

```scrut
$ build_scenario scenario_a_cover_date_doesn_t_match_when_the_document_was_actually_posted
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Unresolvable Cover Metadata (1):
  A cover doesn't correspond to any required transaction; the metadata is probably wrong.
  documents/expenses/business/hosting/mailbox/2024-01-05-invoice-1234.document.yml
    covers[0] declares expenses/business/hosting/mailbox @ 2024-01-06 — no such transaction; the file's location implies expenses/business/hosting/mailbox @ 2024-01-05 instead

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
    1 unresolvable cover metadata fields
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
[1]
```

### A mistake in the top-level shorthand form is labeled top-level, not covers[N]

```scrut
$ build_scenario scenario_a_mistake_in_the_top_level_shorthand_form_is_labeled_top_level_not_covers_n
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Unresolvable Cover Metadata (1):
  A cover doesn't correspond to any required transaction; the metadata is probably wrong.
  documents/expenses/business/hosting/aws/2022-06-03-invoice.document.yml
    top-level declares expenses/business/hosting/aws @ 2022-06-04 — no such transaction; the file's location implies expenses/business/hosting/aws @ 2022-06-03 instead

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
    1 unresolvable cover metadata fields
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
[1]
```

### A non-trivial wrong cover on a fully-unmatched document is not double-reported

```scrut
$ build_scenario scenario_a_non_trivial_wrong_cover_on_a_fully_unmatched_document_is_not_double_reported
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Unmatched Documents (1):
  documents/expenses/business/misc/2026-01-01-note.pdf
    amount: 12.00 EUR

Redundant Metadata Fields (1):
  These fields already match what the file's location implies and can be removed.
  documents/expenses/business/misc/2026-01-01-note.document.yml
    covers[0] date: 2026-01-01

Summary:
  Coverage:
    0/0 transaction groups covered
    0 missing-document placeholders
  Open Items:
    0 missing document coverage
    0 unbooked documents
    0 overdue unbooked documents
    1 unmatched documents
    0 unexpected files
    0 duplicate groups
    1 redundant metadata fields
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

### A legitimate account override that resolves to a real transaction is not flagged

```scrut
$ build_scenario scenario_a_legitimate_account_override_that_resolves_to_a_real_transaction_is_not_flagged
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Summary:
  Coverage:
    2/2 transaction groups covered
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

### A legitimate date override that resolves to a real transaction is not flagged

```scrut
$ build_scenario scenario_a_legitimate_date_override_that_resolves_to_a_real_transaction_is_not_flagged
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

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

### A trivial cover on a genuinely unmatched document is not double-reported

```scrut
$ build_scenario scenario_a_trivial_cover_on_a_genuinely_unmatched_document_is_not_double_reported
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Unmatched Documents (1):
  documents/expenses/business/software/2026-01-01-suite.pdf
    amount: 92.00 EUR

Redundant Metadata Fields (2):
  These fields already match what the file's location implies and can be removed.
  documents/expenses/business/software/2026-01-01-suite.document.yml
    covers[0] date: 2026-01-01
    covers[0] account: expenses:business:software

Summary:
  Coverage:
    0/0 transaction groups covered
    0 missing-document placeholders
  Open Items:
    0 missing document coverage
    0 unbooked documents
    0 overdue unbooked documents
    1 unmatched documents
    0 unexpected files
    0 duplicate groups
    2 redundant metadata fields
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

### One wrong cover among several on an otherwise-matched multi-installment document

```scrut
$ build_scenario scenario_one_wrong_cover_among_several_on_an_otherwise_matched_multi_installment_document
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Unresolvable Cover Metadata (1):
  A cover doesn't correspond to any required transaction; the metadata is probably wrong.
  documents/income/business/freelance/customer/customer-project.document.yml
    covers[1] declares income/business/freelance/customer @ 2026-02-05 — no matching required transaction found

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
    1 unresolvable cover metadata fields
  Amount Audit:
    0 amount mismatches
    0 linked groups checked
    1 linked groups skipped
    Skip Reasons:
      0 document amount unreadable
      1 document currencies unclear
      0 transaction amount missing
      0 transaction currencies mixed
      0 document/transaction currency mismatch
[1]
```

## Metadata Schema Validation

Sidecar metadata is validated before it is used for matching. Unknown fields,
wrong collection shapes, malformed YAML, invalid dates, and invalid amounts are
reported as invalid metadata. Put arbitrary human or structured annotations
under `notes`; the checker stores them but does not use them for matching.

Invalid document metadata is rejected instead of being silently ignored.

### A metadata sidecar with an unknown field is rejected

```scrut
$ build_scenario scenario_a_metadata_sidecar_with_an_unknown_field_is_rejected
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Unmatched Documents (1):
  documents/expenses/business/software/2026-01-01-suite.pdf
    amount: unknown

Unexpected Files (1):
  documents/expenses/business/software/2026-01-01-suite.document.yml  [invalid metadata: metadata root: additional properties are not allowed ('unexpected' was unexpected)]

Summary:
  Coverage:
    0/0 transaction groups covered
    0 missing-document placeholders
  Open Items:
    0 missing document coverage
    0 unbooked documents
    0 overdue unbooked documents
    1 unmatched documents
    1 unexpected files
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

### Metadata notes can contain arbitrary text or structured data

```scrut
$ build_scenario scenario_metadata_notes_can_contain_arbitrary_text_or_structured_data
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

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
