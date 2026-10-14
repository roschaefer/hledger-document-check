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
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank                                  ; document_check:exempt
> account income:business:freelance:customer           ; document_check:required
>
> 2026-01-05 Customer Payment Rate 1
>     assets:bank                                      60.00 EUR
>     income:business:freelance:customer
>
> 2026-02-05 Customer Payment Rate 2
>     assets:bank                                      40.00 EUR
>     income:business:freelance:customer
> EOF
```

```scrut
$ write_file documents/income/business/freelance/customer/customer-project.pdf <<'EOF'
> customer project invoice
> EOF
```

```scrut
$ write_file documents/income/business/freelance/customer/customer-project.document.yml <<'EOF'
> amount: 100.00
> currency: EUR
>
> covers:
>   - date: 2026-01-05
>     account: income:business:freelance:customer
>     amount: 60.00
>   - date: 2026-02-05
>     account: income:business:freelance:customer
>     amount: 40.00
> EOF
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
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank                                  ; document_check:exempt
> account income:business:freelance:customer           ; document_check:required
>
> 2026-01-05 Customer Payment Rate 1
>     assets:bank                                      60.00 EUR
>     income:business:freelance:customer
>
> 2026-02-05 Customer Payment Rate 2
>     assets:bank                                      30.00 EUR
>     income:business:freelance:customer
> EOF
```

```scrut
$ write_file documents/income/business/freelance/customer/customer-project.pdf <<'EOF'
> customer project invoice
> EOF
```

```scrut
$ write_file documents/income/business/freelance/customer/customer-project.document.yml <<'EOF'
> amount: 100.00
> currency: EUR
>
> covers:
>   - date: 2026-01-05
>     account: income:business:freelance:customer
>     amount: 60.00
>   - date: 2026-02-05
>     account: income:business:freelance:customer
>     amount: 30.00
> EOF
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
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank                                  ; document_check:exempt
> account income:business:freelance:customer           ; document_check:required
>
> 2026-03-01 Customer Batch Payment
>     assets:bank                                     100.00 EUR
>     income:business:freelance:customer
> EOF
```

```scrut
$ write_file documents/income/business/freelance/customer/2026-03-01-invoice-a.pdf <<'EOF'
> invoice a
> EOF
```

```scrut
$ write_file documents/income/business/freelance/customer/2026-03-01-invoice-b.pdf <<'EOF'
> invoice b
> EOF
```

```scrut
$ write_file documents/income/business/freelance/customer/2026-03-01-invoice-a.document.yml <<'EOF'
> amount: 40.00
> currency: EUR
>
> covers:
>   - date: 2026-03-01
>     account: income:business:freelance:customer
>     amount: 40.00
> EOF
```

```scrut
$ write_file documents/income/business/freelance/customer/2026-03-01-invoice-b.document.yml <<'EOF'
> amount: 60.00
> currency: EUR
>
> covers:
>   - date: 2026-03-01
>     account: income:business:freelance:customer
>     amount: 60.00
> EOF
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
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank                                  ; document_check:exempt
> account income:business:freelance:customer           ; document_check:required
>
> 2026-03-01 Customer Batch Payment
>     assets:bank                                     100.00 EUR
>     income:business:freelance:customer
> EOF
```

```scrut
$ write_file documents/income/business/freelance/customer/2026-03-01-invoice-a.pdf <<'EOF'
> invoice a
> EOF
```

```scrut
$ write_file documents/income/business/freelance/customer/2026-03-01-invoice-b.pdf <<'EOF'
> invoice b
> EOF
```

```scrut
$ write_file documents/income/business/freelance/customer/2026-03-01-invoice-a.document.yml <<'EOF'
> amount: 40.00
> currency: EUR
>
> covers:
>   - date: 2026-03-01
>     account: income:business:freelance:customer
>     amount: 40.00
> EOF
```

```scrut
$ write_file documents/income/business/freelance/customer/2026-03-01-invoice-b.document.yml <<'EOF'
> amount: 50.00
> currency: EUR
>
> covers:
>   - date: 2026-03-01
>     account: income:business:freelance:customer
>     amount: 50.00
> EOF
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
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank                         ; document_check:exempt
> account expenses:business:domains           ; document_check:required
>
> 2026-03-01 Domain A
>     expenses:business:domains                4.00 EUR
>     assets:bank                             -4.00 EUR
>
> 2026-03-01 Domain B
>     expenses:business:domains                6.00 EUR
>     assets:bank                             -6.00 EUR
> EOF
```

```scrut
$ write_file documents/expenses/business/domains/2026-03-01-domain-a.pdf <<'EOF'
> domain a
> EOF
```

```scrut
$ write_file documents/expenses/business/domains/2026-03-01-domain-b.pdf <<'EOF'
> domain b
> EOF
```

```scrut
$ write_file documents/expenses/business/domains/2026-03-01-domain-a.document.yml <<'EOF'
> covers:
>   - date: 2026-03-01
>     account: expenses:business:domains
>     amount: 4.00
>     currency: EUR
> EOF
```

```scrut
$ write_file documents/expenses/business/domains/2026-03-01-domain-b.document.yml <<'EOF'
> covers:
>   - date: 2026-03-01
>     account: expenses:business:domains
>     amount: 6.00
>     currency: EUR
> EOF
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
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank                                      ; document_check:exempt
> account expenses:insurance:health:base                   ; document_check:required
> account expenses:insurance:health:nursing-care           ; document_check:required
>
> 2026-01-01 Annual Health Insurance Base
>     expenses:insurance:health:base                       100.00 EUR
>     assets:bank                                         -100.00 EUR
>
> 2026-01-01 Annual Nursing Care Insurance
>     expenses:insurance:health:nursing-care                30.00 EUR
>     assets:bank                                          -30.00 EUR
> EOF
```

```scrut
$ write_file documents/expenses/insurance/health/2026-01-01-annual-notice.pdf <<'EOF'
> annual health insurance notice
> EOF
```

```scrut
$ write_file documents/expenses/insurance/health/2026-01-01-annual-notice.document.yml <<'EOF'
> covers:
>   - date: 2026-01-01
>     account: expenses:insurance:health:base
>     amount: 100.00
>     currency: EUR
>   - date: 2026-01-01
>     account: expenses:insurance:health:nursing-care
>     amount: 30.00
>     currency: EUR
> EOF
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
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank                          ; document_check:exempt
> account expenses:business:hosting:uberspace  ; document_check:required
>
> 2026-01-05 Uberspace Hosting
>     expenses:business:hosting:uberspace      4.00 EUR
>     assets:bank                             -4.00 EUR
> EOF
```

```scrut
$ write_file documents/expenses/business/hosting/uberspace/2026-01-05-invoice.pdf <<'EOF'
> uberspace invoice
> EOF
```

```scrut
$ write_file documents/expenses/business/hosting/uberspace/2026-01-05-invoice.document.yml <<'EOF'
> covers:
>   - date: 2026-01-05
>     account: expenses:business:hosting:uberspace
>     amount: 4.00
>     currency: EUR
> EOF
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
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank                     ; document_check:exempt
> account expenses:business:hosting:aws   ; document_check:required
>
> 2022-06-03 AWS EMEA
>     expenses:business:hosting:aws        2.99 EUR
>     assets:bank                         -2.99 EUR
> EOF
```

```scrut
$ write_file documents/expenses/business/hosting/aws/2022-06-03-invoice.pdf <<'EOF'
> aws invoice
> EOF
```

```scrut
$ write_file documents/expenses/business/hosting/aws/2022-06-03-invoice.document.yml <<'EOF'
> date: 2022-06-03
> account: expenses:business:hosting:aws
> amount: 2.99
> currency: EUR
> EOF
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
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank                                      ; document_check:exempt
> account expenses:insurance:health:base                   ; document_check:required
> account expenses:insurance:health:nursing-care           ; document_check:required
>
> 2026-01-01 Annual Health Insurance Base
>     expenses:insurance:health:base                       100.00 EUR
>     assets:bank                                         -100.00 EUR
>
> 2026-01-01 Annual Nursing Care Insurance
>     expenses:insurance:health:nursing-care                30.00 EUR
>     assets:bank                                          -30.00 EUR
> EOF
```

```scrut
$ write_file documents/expenses/insurance/health/2026-01-01-annual-notice.pdf <<'EOF'
> annual health insurance notice
> EOF
```

```scrut
$ write_file documents/expenses/insurance/health/2026-01-01-annual-notice.document.yml <<'EOF'
> covers:
>   - account: expenses:insurance:health:base
>     amount: 100.00
>     currency: EUR
>   - account: expenses:insurance:health:nursing-care
>     amount: 30.00
>     currency: EUR
> EOF
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
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank                    ; document_check:exempt
> account expenses:business:software     ; document_check:required
>
> 2026-01-01 Software Suite
>     expenses:business:software          92.00 EUR
>     assets:bank                        -92.00 EUR
> EOF
```

```scrut
$ write_file documents/expenses/business/software/2026-01-01-suite.pdf <<'EOF'
> suite invoice
> EOF
```

```scrut
$ write_file documents/expenses/business/software/2026-01-01-suite.document.yml <<'EOF'
> amount: 92.00
> currency: EUR
> EOF
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
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank                                    ; document_check:exempt
> account expenses:business:hosting:storage:legacy       ; document_check:required
>
> 2026-01-05 Storage Invoice
>     expenses:business:hosting:storage:legacy           8.94 EUR
>     assets:bank                                        -8.94 EUR
> EOF
```

```scrut
$ write_file documents/expenses/business/hosting/storage/legacy/2026-01-05.pdf <<'EOF'
> storage invoice
> EOF
```

```scrut
$ write_file documents/expenses/business/hosting/storage/legacy/2026-01-05.document.yml <<'EOF'
> covers:
>   - date: 2026-01-05
>     account: expenses:business:hosting:storage
>     amount: 8.94
>     currency: EUR
> EOF
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
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank                            ; document_check:exempt
> account expenses:business:hosting:mailbox      ; document_check:required
>
> 2024-01-05 Mailbox Invoice
>     expenses:business:hosting:mailbox          8.94 EUR
>     assets:bank                                -8.94 EUR
> EOF
```

```scrut
$ write_file documents/expenses/business/hosting/mailbox/2024-01-05-invoice-1234.pdf <<'EOF'
> mailbox invoice
> EOF
```

```scrut
$ write_file documents/expenses/business/hosting/mailbox/2024-01-05-invoice-1234.document.yml <<'EOF'
> covers:
>   - date: 2024-01-06
>     amount: 8.94
>     currency: EUR
> EOF
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
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank                     ; document_check:exempt
> account expenses:business:hosting:aws   ; document_check:required
>
> 2022-06-03 AWS EMEA
>     expenses:business:hosting:aws        2.99 EUR
>     assets:bank                         -2.99 EUR
> EOF
```

```scrut
$ write_file documents/expenses/business/hosting/aws/2022-06-03-invoice.pdf <<'EOF'
> aws invoice
> EOF
```

```scrut
$ write_file documents/expenses/business/hosting/aws/2022-06-03-invoice.document.yml <<'EOF'
> date: 2022-06-04
> amount: 2.99
> currency: EUR
> EOF
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
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank ; document_check:exempt
> EOF
```

```scrut
$ write_file documents/expenses/business/misc/2026-01-01-note.pdf <<'EOF'
> note
> EOF
```

```scrut
$ write_file documents/expenses/business/misc/2026-01-01-note.document.yml <<'EOF'
> covers:
>   - date: 2026-01-01
>     account: expenses:business:nonexistent
>     amount: 12.00
>     currency: EUR
> EOF
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
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank                                      ; document_check:exempt
> account expenses:insurance:health:base                   ; document_check:required
> account expenses:insurance:health:nursing-care           ; document_check:required
>
> 2026-01-01 Annual Health Insurance Base
>     expenses:insurance:health:base                       100.00 EUR
>     assets:bank                                         -100.00 EUR
>
> 2026-01-01 Annual Nursing Care Insurance
>     expenses:insurance:health:nursing-care                30.00 EUR
>     assets:bank                                          -30.00 EUR
> EOF
```

```scrut
$ write_file documents/expenses/insurance/health/2026-01-01-annual-notice.pdf <<'EOF'
> annual health insurance notice
> EOF
```

```scrut
$ write_file documents/expenses/insurance/health/2026-01-01-annual-notice.document.yml <<'EOF'
> covers:
>   - account: expenses:insurance:health:base
>     amount: 100.00
>     currency: EUR
>   - account: expenses:insurance:health:nursing-care
>     amount: 30.00
>     currency: EUR
> EOF
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
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank                            ; document_check:exempt
> account expenses:business:hosting:mailbox      ; document_check:required
>
> 2024-01-20 Mailbox Invoice Cleared
>     expenses:business:hosting:mailbox          8.94 EUR
>     assets:bank                                -8.94 EUR
> EOF
```

```scrut
$ write_file documents/expenses/business/hosting/mailbox/2024-01-05-invoice-1234.pdf <<'EOF'
> mailbox invoice
> EOF
```

```scrut
$ write_file documents/expenses/business/hosting/mailbox/2024-01-05-invoice-1234.document.yml <<'EOF'
> covers:
>   - date: 2024-01-20
>     amount: 8.94
>     currency: EUR
> EOF
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
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank                        ; document_check:exempt
> account expenses:business:software         ; document_check:required
> EOF
```

```scrut
$ write_file documents/expenses/business/software/2026-01-01-suite.pdf <<'EOF'
> suite invoice
> EOF
```

```scrut
$ write_file documents/expenses/business/software/2026-01-01-suite.document.yml <<'EOF'
> covers:
>   - date: 2026-01-01
>     account: expenses:business:software
>     amount: 92.00
>     currency: EUR
> EOF
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
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank                                  ; document_check:exempt
> account income:business:freelance:customer           ; document_check:required
>
> 2026-01-05 Customer Payment Rate 1
>     assets:bank                                      60.00 EUR
>     income:business:freelance:customer
> EOF
```

```scrut
$ write_file documents/income/business/freelance/customer/customer-project.pdf <<'EOF'
> customer project invoice
> EOF
```

```scrut
$ write_file documents/income/business/freelance/customer/customer-project.document.yml <<'EOF'
> covers:
>   - date: 2026-01-05
>     amount: 60.00
>   - date: 2026-02-05
>     amount: 40.00
> EOF
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
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank ; document_check:exempt
> EOF
```

```scrut
$ write_file documents/expenses/business/software/2026-01-01-suite.pdf <<'EOF'
> suite invoice
> EOF
```

```scrut
$ write_file documents/expenses/business/software/2026-01-01-suite.document.yml <<'EOF'
> amount: 100.00
> unexpected: true
> EOF
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
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank                         ; document_check:exempt
> account expenses:business:software          ; document_check:required
>
> 2026-01-01 Software Suite
>     expenses:business:software              92.00 EUR
>     assets:bank                            -92.00 EUR
> EOF
```

```scrut
$ write_file documents/expenses/business/software/2026-01-01-suite.pdf <<'EOF'
> software suite invoice
> EOF
```

```scrut
$ write_file documents/expenses/business/software/2026-01-01-text-note.pdf <<'EOF'
> text-only notes document
> EOF
```

```scrut
$ write_file documents/expenses/business/software/2026-01-01-text-note.document.yml <<'EOF'
> notes: "Keep until the yearly review is done."
> EOF
```

```scrut
$ write_file documents/expenses/business/software/2026-01-01-suite.document.yml <<'EOF'
> amount: 92.00
> currency: EUR
> description: Software Suite
> notes:
>   reason: "Bank converted the foreign-currency charge"
>   source_lines:
>     - "invoice total: 100.00 USD"
>     - "charged amount: 92.00 EUR"
> covers:
>   - amount: 92.00
>     description: Software Suite
> EOF
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

A sidecar is metadata about a document, so it cannot stand on its own.

### A metadata file without a matching document is reported

```scrut
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank                         ; document_check:exempt
> account expenses:business:software          ; document_check:required
>
> 2026-01-01 Software Suite
>     expenses:business:software              92.00 EUR
>     assets:bank                            -92.00 EUR
> EOF
```

```scrut
$ write_file documents/expenses/business/software/2026-01-01-suite.pdf <<'EOF'
> software suite invoice
> EOF
```

The sidecar's date does not match the PDF, so it describes no document:

```scrut
$ write_file documents/expenses/business/software/2026-01-02-text-note.document.yml <<'EOF'
> notes: "Keep until the yearly review is done."
> EOF
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Unexpected Files (1):
  documents/expenses/business/software/2026-01-02-text-note.document.yml  [metadata file has no matching document PDF; use YYYY-MM-DD-missing.document.yml for missing documents]

Summary:
  Coverage:
    1/1 transaction groups covered
    0 missing-document placeholders
  Open Items:
    0 missing document coverage
    0 unbooked documents
    0 overdue unbooked documents
    0 unmatched documents
    1 unexpected files
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
