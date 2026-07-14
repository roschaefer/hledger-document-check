# Special Cases With `.document.yml`

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

```gherkin
Feature: One document can cover many transactions
  A document metadata file can connect one invoice to several required postings.

  Scenario: A customer pays one invoice in two rates
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                                  ; document_check:exempt
      account income:business:freelance:customer           ; document_check:required

      2026-01-05 Customer Payment Rate 1
          assets:bank                                      60.00 EUR
          income:business:freelance:customer

      2026-02-05 Customer Payment Rate 2
          assets:bank                                      40.00 EUR
          income:business:freelance:customer
      """
    And a file named "documents/income/business/freelance/customer/customer-project.pdf" with content:
      """
      customer project invoice
      """
    And a file named "documents/income/business/freelance/customer/customer-project.document.yml" with content:
      """yaml
      amount: 100.00
      currency: EUR

      covers:
        - date: 2026-01-05
          account: income:business:freelance:customer
          amount: 60.00
        - date: 2026-02-05
          account: income:business:freelance:customer
          amount: 40.00
      """
    When I run "hledger document-check check --journal journal.journal --documents documents --ignore duplicate-files"
    Then the command exits with code 0
    And stdout contains:
      """text
      2/2 transaction groups covered
      ...
      0 amount mismatches
      """

  Scenario: Cover amounts must add up to the document total when total is configured
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                                  ; document_check:exempt
      account income:business:freelance:customer           ; document_check:required

      2026-01-05 Customer Payment Rate 1
          assets:bank                                      60.00 EUR
          income:business:freelance:customer

      2026-02-05 Customer Payment Rate 2
          assets:bank                                      30.00 EUR
          income:business:freelance:customer
      """
    And a file named "documents/income/business/freelance/customer/customer-project.pdf" with content:
      """
      customer project invoice
      """
    And a file named "documents/income/business/freelance/customer/customer-project.document.yml" with content:
      """yaml
      amount: 100.00
      currency: EUR

      covers:
        - date: 2026-01-05
          account: income:business:freelance:customer
          amount: 60.00
        - date: 2026-02-05
          account: income:business:freelance:customer
          amount: 30.00
      """
    When I run "hledger document-check check --journal journal.journal --documents documents --ignore duplicate-files"
    Then the command exits with code 1
    And stdout contains:
      """text
      Amount Mismatches (1):
      ...
      allocated document amounts 90.00 != declared document total 100.00
      """
```

## Multiple Invoices Covering One Transaction

A customer may pay two invoices in one bank transaction. Store both invoice PDFs
and let each metadata file point to the same ledger posting group with its share
of the total transaction amount. If each document is paid completely in the
batch, set both the document amount and the cover amount to the invoice total. If
a document is only partly paid, set top-level `amount` to the document total and
`covers[].amount` to the paid part.

```gherkin
Feature: Many documents can cover one transaction
  Several invoice files can connect to the same required posting group.

  Scenario: A customer pays two invoices at once
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                                  ; document_check:exempt
      account income:business:freelance:customer           ; document_check:required

      2026-03-01 Customer Batch Payment
          assets:bank                                     100.00 EUR
          income:business:freelance:customer
      """
    And files named:
      | path                                                                       | content   |
      | documents/income/business/freelance/customer/2026-03-01-invoice-a.pdf      | invoice a |
      | documents/income/business/freelance/customer/2026-03-01-invoice-b.pdf      | invoice b |
    And a file named "documents/income/business/freelance/customer/2026-03-01-invoice-a.document.yml" with content:
      """yaml
      amount: 40.00
      currency: EUR

      covers:
        - date: 2026-03-01
          account: income:business:freelance:customer
          amount: 40.00
      """
    And a file named "documents/income/business/freelance/customer/2026-03-01-invoice-b.document.yml" with content:
      """yaml
      amount: 60.00
      currency: EUR

      covers:
        - date: 2026-03-01
          account: income:business:freelance:customer
          amount: 60.00
      """
    When I run "hledger document-check check --journal journal.journal --documents documents --ignore duplicate-files"
    Then the command exits with code 0
    And stdout contains:
      """text
      1/1 transaction groups covered
      ...
      0 amount mismatches
      """

  Scenario: Cover amounts must add up to the transaction amount
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                                  ; document_check:exempt
      account income:business:freelance:customer           ; document_check:required

      2026-03-01 Customer Batch Payment
          assets:bank                                     100.00 EUR
          income:business:freelance:customer
      """
    And files named:
      | path                                                                       | content   |
      | documents/income/business/freelance/customer/2026-03-01-invoice-a.pdf      | invoice a |
      | documents/income/business/freelance/customer/2026-03-01-invoice-b.pdf      | invoice b |
    And a file named "documents/income/business/freelance/customer/2026-03-01-invoice-a.document.yml" with content:
      """yaml
      amount: 40.00
      currency: EUR

      covers:
        - date: 2026-03-01
          account: income:business:freelance:customer
          amount: 40.00
      """
    And a file named "documents/income/business/freelance/customer/2026-03-01-invoice-b.document.yml" with content:
      """yaml
      amount: 50.00
      currency: EUR

      covers:
        - date: 2026-03-01
          account: income:business:freelance:customer
          amount: 50.00
      """
    When I run "hledger document-check check --journal journal.journal --documents documents --ignore duplicate-files"
    Then the command exits with code 1
    And stdout contains:
      """text
      Amount Mismatches (1):
      ...
      document total 90.00 != transaction total 100.00
      """
```

## Multiple Invoices On The Same Date

Date-prefixed filenames only know account and date. If two required transactions
share the same account and date, metadata removes the ambiguity by listing the
amount that belongs to each file.

```gherkin
Feature: Metadata disambiguates same-date documents
  Metadata can resolve several invoices for the same account and date.

  Scenario: Two same-date invoices each cover one same-date transaction
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                         ; document_check:exempt
      account expenses:business:domains           ; document_check:required

      2026-03-01 Domain A
          expenses:business:domains                4.00 EUR
          assets:bank                             -4.00 EUR

      2026-03-01 Domain B
          expenses:business:domains                6.00 EUR
          assets:bank                             -6.00 EUR
      """
    And files named:
      | path                                                             | content  |
      | documents/expenses/business/domains/2026-03-01-domain-a.pdf      | domain a |
      | documents/expenses/business/domains/2026-03-01-domain-b.pdf      | domain b |
    And a file named "documents/expenses/business/domains/2026-03-01-domain-a.document.yml" with content:
      """yaml
      covers:
        - date: 2026-03-01
          account: expenses:business:domains
          amount: 4.00
          currency: EUR
      """
    And a file named "documents/expenses/business/domains/2026-03-01-domain-b.document.yml" with content:
      """yaml
      covers:
        - date: 2026-03-01
          account: expenses:business:domains
          amount: 6.00
          currency: EUR
      """
    When I run "hledger document-check check --journal journal.journal --documents documents --ignore duplicate-files"
    Then the command exits with code 0
    And stdout contains:
      """text
      1/1 transaction groups covered
      ...
      0 amount mismatches
      """
```

## Override Account For Many Invoices

Some documents are filed under a parent folder but cover transactions in several
subaccounts. Health-insurance annual notices are a typical case: one notice can
describe health, nursing-care, and surcharge postings. Each cover entry can
override the account explicitly.

```gherkin
Feature: Metadata can override account matching
  A document filed in one folder can cover postings in several subaccounts.

  Scenario: One annual health-insurance notice covers several subaccounts
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                                      ; document_check:exempt
      account expenses:insurance:health:base                   ; document_check:required
      account expenses:insurance:health:nursing-care           ; document_check:required

      2026-01-01 Annual Health Insurance Base
          expenses:insurance:health:base                       100.00 EUR
          assets:bank                                         -100.00 EUR

      2026-01-01 Annual Nursing Care Insurance
          expenses:insurance:health:nursing-care                30.00 EUR
          assets:bank                                          -30.00 EUR
      """
    And a file named "documents/expenses/insurance/health/2026-01-01-annual-notice.pdf" with content:
      """
      annual health insurance notice
      """
    And a file named "documents/expenses/insurance/health/2026-01-01-annual-notice.document.yml" with content:
      """yaml
      covers:
        - date: 2026-01-01
          account: expenses:insurance:health:base
          amount: 100.00
          currency: EUR
        - date: 2026-01-01
          account: expenses:insurance:health:nursing-care
          amount: 30.00
          currency: EUR
      """
    When I run "hledger document-check check --journal journal.journal --documents documents --ignore duplicate-files"
    Then the command exits with code 0
    And stdout contains:
      """text
      2/2 transaction groups covered
      ...
      0 amount mismatches
      """
```

## Redundant Metadata

A `covers[].date`/`covers[].account` (or the top-level shorthand `date`/`account`)
that exactly matches what the file's own location already implies — the
filename date and the account folder — is redundant: removing it changes
nothing about how the document matches. The `redundant-metadata` check finds
these fields so they can be deleted. It only reports a `warn` by default, so it
does not fail the build on its own.

```gherkin
Feature: Redundant metadata fields are flagged
  Explicit date/account values that duplicate the file's own location are
  reported so they can be removed.

  Scenario: A covers entry repeats the filename date and folder account
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                          ; document_check:exempt
      account expenses:business:hosting:uberspace  ; document_check:required

      2026-01-05 Uberspace Hosting
          expenses:business:hosting:uberspace      4.00 EUR
          assets:bank                             -4.00 EUR
      """
    And a file named "documents/expenses/business/hosting/uberspace/2026-01-05-invoice.pdf" with content:
      """
      uberspace invoice
      """
    And a file named "documents/expenses/business/hosting/uberspace/2026-01-05-invoice.document.yml" with content:
      """yaml
      covers:
        - date: 2026-01-05
          account: expenses:business:hosting:uberspace
          amount: 4.00
          currency: EUR
      """
    When I run "hledger document-check check --journal journal.journal --documents documents --ignore duplicate-files"
    Then the command exits with code 0
    And stdout contains:
      """text
      Redundant Metadata Fields (2):
      ...
        documents/expenses/business/hosting/uberspace/2026-01-05-invoice.document.yml
          covers[0] date: 2026-01-05
          covers[0] account: expenses:business:hosting:uberspace
      """

  Scenario: A top-level shorthand date/account repeats the file's own location
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                     ; document_check:exempt
      account expenses:business:hosting:aws   ; document_check:required

      2022-06-03 AWS EMEA
          expenses:business:hosting:aws        2.99 EUR
          assets:bank                         -2.99 EUR
      """
    And a file named "documents/expenses/business/hosting/aws/2022-06-03-invoice.pdf" with content:
      """
      aws invoice
      """
    And a file named "documents/expenses/business/hosting/aws/2022-06-03-invoice.document.yml" with content:
      """yaml
      date: 2022-06-03
      account: expenses:business:hosting:aws
      amount: 2.99
      currency: EUR
      """
    When I run "hledger document-check check --journal journal.journal --documents documents --ignore duplicate-files"
    Then the command exits with code 0
    And stdout contains:
      """text
      Redundant Metadata Fields (2):
      ...
        documents/expenses/business/hosting/aws/2022-06-03-invoice.document.yml
          top-level date: 2022-06-03
          top-level account: expenses:business:hosting:aws
      """

  Scenario: An account override that names a different account is not flagged
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                                      ; document_check:exempt
      account expenses:insurance:health:base                   ; document_check:required
      account expenses:insurance:health:nursing-care           ; document_check:required

      2026-01-01 Annual Health Insurance Base
          expenses:insurance:health:base                       100.00 EUR
          assets:bank                                         -100.00 EUR

      2026-01-01 Annual Nursing Care Insurance
          expenses:insurance:health:nursing-care                30.00 EUR
          assets:bank                                          -30.00 EUR
      """
    And a file named "documents/expenses/insurance/health/2026-01-01-annual-notice.pdf" with content:
      """
      annual health insurance notice
      """
    And a file named "documents/expenses/insurance/health/2026-01-01-annual-notice.document.yml" with content:
      """yaml
      covers:
        - account: expenses:insurance:health:base
          amount: 100.00
          currency: EUR
        - account: expenses:insurance:health:nursing-care
          amount: 30.00
          currency: EUR
      """
    When I run "hledger document-check check --journal journal.journal --documents documents --ignore duplicate-files"
    Then the command exits with code 0
    And stdout does not contain:
      """text
      Redundant Metadata Fields
      """

  Scenario: A sidecar with only amount and currency is not flagged
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                    ; document_check:exempt
      account expenses:business:software     ; document_check:required

      2026-01-01 Software Suite
          expenses:business:software          92.00 EUR
          assets:bank                        -92.00 EUR
      """
    And a file named "documents/expenses/business/software/2026-01-01-suite.pdf" with content:
      """
      suite invoice
      """
    And a file named "documents/expenses/business/software/2026-01-01-suite.document.yml" with content:
      """yaml
      amount: 92.00
      currency: EUR
      """
    When I run "hledger document-check check --journal journal.journal --documents documents --ignore duplicate-files"
    Then the command exits with code 0
    And stdout does not contain:
      """text
      Redundant Metadata Fields
      """
```

## Unresolvable Cover Metadata

A `covers[]` entry — its account, its date, or both — that doesn't match any
required transaction is silently dropped: matching falls back to (or is
carried by) the file's own directory/filename identity instead, so the
document can still end up matched correctly and nothing else surfaces the
mistake. The `unresolvable-cover-metadata` check flags these covers
directly, and suggests the file's own location as the likely intended value
when that location does have a matching transaction. A cover that asserts
nothing beyond the file's own location is never flagged this way even when it
resolves to nothing — that's just an unmatched document, already reported by
`unmatched-documents`. This check only reports a `warn` by default, so it
does not fail the build on its own.

```gherkin
Feature: Unresolvable cover metadata is flagged
  A cover whose account/date pair matches no required transaction is
  reported, with a suggestion when the file's own location would have
  matched instead.

  Scenario: A cover account is missing a subaccount segment the folder implies
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                                    ; document_check:exempt
      account expenses:business:hosting:storage:legacy       ; document_check:required

      2026-01-05 Storage Invoice
          expenses:business:hosting:storage:legacy           8.94 EUR
          assets:bank                                        -8.94 EUR
      """
    And a file named "documents/expenses/business/hosting/storage/legacy/2026-01-05.pdf" with content:
      """
      storage invoice
      """
    And a file named "documents/expenses/business/hosting/storage/legacy/2026-01-05.document.yml" with content:
      """yaml
      covers:
        - date: 2026-01-05
          account: expenses:business:hosting:storage
          amount: 8.94
          currency: EUR
      """
    When I run "hledger document-check check --journal journal.journal --documents documents --ignore duplicate-files"
    Then the command exits with code 0
    And stdout contains:
      """text
      Unresolvable Cover Metadata (1):
      ...
        documents/expenses/business/hosting/storage/legacy/2026-01-05.document.yml
          covers[0] declares expenses/business/hosting/storage @ 2026-01-05 — no such transaction; the file's location implies expenses/business/hosting/storage/legacy @ 2026-01-05 instead
      """

  Scenario: A cover date doesn't match when the document was actually posted
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                            ; document_check:exempt
      account expenses:business:hosting:mailbox      ; document_check:required

      2024-01-05 Mailbox Invoice
          expenses:business:hosting:mailbox          8.94 EUR
          assets:bank                                -8.94 EUR
      """
    And a file named "documents/expenses/business/hosting/mailbox/2024-01-05-invoice-1234.pdf" with content:
      """
      mailbox invoice
      """
    And a file named "documents/expenses/business/hosting/mailbox/2024-01-05-invoice-1234.document.yml" with content:
      """yaml
      covers:
        - date: 2024-01-06
          amount: 8.94
          currency: EUR
      """
    When I run "hledger document-check check --journal journal.journal --documents documents --ignore duplicate-files"
    Then the command exits with code 0
    And stdout contains:
      """text
      Unresolvable Cover Metadata (1):
      ...
        documents/expenses/business/hosting/mailbox/2024-01-05-invoice-1234.document.yml
          covers[0] declares expenses/business/hosting/mailbox @ 2024-01-06 — no such transaction; the file's location implies expenses/business/hosting/mailbox @ 2024-01-05 instead
      """

  Scenario: A mistake in the top-level shorthand form is labeled top-level, not covers[N]
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                     ; document_check:exempt
      account expenses:business:hosting:aws   ; document_check:required

      2022-06-03 AWS EMEA
          expenses:business:hosting:aws        2.99 EUR
          assets:bank                         -2.99 EUR
      """
    And a file named "documents/expenses/business/hosting/aws/2022-06-03-invoice.pdf" with content:
      """
      aws invoice
      """
    And a file named "documents/expenses/business/hosting/aws/2022-06-03-invoice.document.yml" with content:
      """yaml
      date: 2022-06-04
      amount: 2.99
      currency: EUR
      """
    When I run "hledger document-check check --journal journal.journal --documents documents --ignore duplicate-files"
    Then the command exits with code 0
    And stdout contains:
      """text
      Unresolvable Cover Metadata (1):
      ...
        documents/expenses/business/hosting/aws/2022-06-03-invoice.document.yml
          top-level declares expenses/business/hosting/aws @ 2022-06-04 — no such transaction; the file's location implies expenses/business/hosting/aws @ 2022-06-03 instead
      """

  Scenario: A cover matches nothing, with no location-based suggestion either
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank ; document_check:exempt
      """
    And a file named "documents/expenses/business/misc/2026-01-01-note.pdf" with content:
      """
      note
      """
    And a file named "documents/expenses/business/misc/2026-01-01-note.document.yml" with content:
      """yaml
      covers:
        - date: 2026-01-01
          account: expenses:business:nonexistent
          amount: 12.00
          currency: EUR
      """
    When I run "hledger document-check check --journal journal.journal --documents documents --ignore duplicate-files --ignore unmatched-documents"
    Then the command exits with code 0
    And stdout contains:
      """text
      Unresolvable Cover Metadata (1):
      ...
        documents/expenses/business/misc/2026-01-01-note.document.yml
          covers[0] declares expenses/business/nonexistent @ 2026-01-01 — no matching required transaction found
      """

  Scenario: A legitimate account override that resolves to a real transaction is not flagged
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                                      ; document_check:exempt
      account expenses:insurance:health:base                   ; document_check:required
      account expenses:insurance:health:nursing-care           ; document_check:required

      2026-01-01 Annual Health Insurance Base
          expenses:insurance:health:base                       100.00 EUR
          assets:bank                                         -100.00 EUR

      2026-01-01 Annual Nursing Care Insurance
          expenses:insurance:health:nursing-care                30.00 EUR
          assets:bank                                          -30.00 EUR
      """
    And a file named "documents/expenses/insurance/health/2026-01-01-annual-notice.pdf" with content:
      """
      annual health insurance notice
      """
    And a file named "documents/expenses/insurance/health/2026-01-01-annual-notice.document.yml" with content:
      """yaml
      covers:
        - account: expenses:insurance:health:base
          amount: 100.00
          currency: EUR
        - account: expenses:insurance:health:nursing-care
          amount: 30.00
          currency: EUR
      """
    When I run "hledger document-check check --journal journal.journal --documents documents --ignore duplicate-files"
    Then the command exits with code 0
    And stdout does not contain:
      """text
      Unresolvable Cover Metadata
      """

  Scenario: A legitimate date override that resolves to a real transaction is not flagged
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                            ; document_check:exempt
      account expenses:business:hosting:mailbox      ; document_check:required

      2024-01-20 Mailbox Invoice Cleared
          expenses:business:hosting:mailbox          8.94 EUR
          assets:bank                                -8.94 EUR
      """
    And a file named "documents/expenses/business/hosting/mailbox/2024-01-05-invoice-1234.pdf" with content:
      """
      mailbox invoice
      """
    And a file named "documents/expenses/business/hosting/mailbox/2024-01-05-invoice-1234.document.yml" with content:
      """yaml
      covers:
        - date: 2024-01-20
          amount: 8.94
          currency: EUR
      """
    When I run "hledger document-check check --journal journal.journal --documents documents --ignore duplicate-files"
    Then the command exits with code 0
    And stdout does not contain:
      """text
      Unresolvable Cover Metadata
      """

  Scenario: A trivial cover on a genuinely unmatched document is not double-reported
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                        ; document_check:exempt
      account expenses:business:software         ; document_check:required
      """
    And a file named "documents/expenses/business/software/2026-01-01-suite.pdf" with content:
      """
      suite invoice
      """
    And a file named "documents/expenses/business/software/2026-01-01-suite.document.yml" with content:
      """yaml
      covers:
        - date: 2026-01-01
          account: expenses:business:software
          amount: 92.00
          currency: EUR
      """
    When I run "hledger document-check check --journal journal.journal --documents documents --ignore duplicate-files"
    Then the command exits with code 1
    And stdout contains:
      """text
      Unmatched Documents (1):
      """
    And stdout does not contain:
      """text
      Unresolvable Cover Metadata
      """

  Scenario: One wrong cover among several on an otherwise-matched multi-installment document
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                                  ; document_check:exempt
      account income:business:freelance:customer           ; document_check:required

      2026-01-05 Customer Payment Rate 1
          assets:bank                                      60.00 EUR
          income:business:freelance:customer
      """
    And a file named "documents/income/business/freelance/customer/customer-project.pdf" with content:
      """
      customer project invoice
      """
    And a file named "documents/income/business/freelance/customer/customer-project.document.yml" with content:
      """yaml
      covers:
        - date: 2026-01-05
          amount: 60.00
        - date: 2026-02-05
          amount: 40.00
      """
    When I run "hledger document-check check --journal journal.journal --documents documents --ignore duplicate-files"
    Then the command exits with code 0
    And stdout contains:
      """text
      Unresolvable Cover Metadata (1):
      ...
        documents/income/business/freelance/customer/customer-project.document.yml
          covers[1] declares income/business/freelance/customer @ 2026-02-05 — no matching required transaction found
      """
```

## Metadata Schema Validation

Sidecar metadata is validated before it is used for matching. Unknown fields,
wrong collection shapes, malformed YAML, invalid dates, and invalid amounts are
reported as invalid metadata. Put arbitrary human or structured annotations
under `notes`; the checker stores them but does not use them for matching.

```gherkin
Feature: Metadata sidecars are schema-validated
  Invalid document metadata is rejected instead of being silently ignored.

  Scenario: A metadata sidecar with an unknown field is rejected
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank ; document_check:exempt
      """
    And a file named "documents/expenses/business/software/2026-01-01-suite.pdf" with content:
      """
      suite invoice
      """
    And a file named "documents/expenses/business/software/2026-01-01-suite.document.yml" with content:
      """yaml
      amount: 100.00
      unexpected: true
      """
    When I run "hledger document-check check --journal journal.journal --documents documents --ignore duplicate-files"
    Then the command exits with code 1
    And stdout contains:
      """text
      Unexpected Files (1):
        documents/expenses/business/software/2026-01-01-suite.document.yml  [invalid metadata:
      """

  Scenario: Metadata notes can contain arbitrary text or structured data
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                         ; document_check:exempt
      account expenses:business:software          ; document_check:required

      2026-01-01 Software Suite
          expenses:business:software              92.00 EUR
          assets:bank                            -92.00 EUR
      """
    And files named:
      | path                                                                         | content                                         |
      | documents/expenses/business/software/2026-01-01-suite.pdf                    | software suite invoice                          |
      | documents/expenses/business/software/2026-01-02-text-note.document.yml       | notes: "Keep until the yearly review is done."  |
    And a file named "documents/expenses/business/software/2026-01-01-suite.document.yml" with content:
      """yaml
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
      """
    When I run "hledger document-check check --journal journal.journal --documents documents --ignore duplicate-files"
    Then the command exits with code 1
    And stdout contains:
      """text
      1/1 transaction groups covered
      """
    And stdout contains:
      """text
      documents/expenses/business/software/2026-01-02-text-note.document.yml  [metadata file has no matching document PDF;
      """
```
