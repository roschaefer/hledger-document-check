# Contract

`hledger-document-check` decides document requirements from hledger account and posting
tags. The normal setup is to keep reusable defaults in `accounts.journal` and
include that file from the actual journal.

## Required and exempt accounts

- `document_check:required`: every posting to this account needs document coverage.
- `document_check:exempt`: postings to this account never need document coverage.

hledger tags declared on an `account` directive are inherited by its subaccounts, and a
tag on a more specific account overrides the one inherited from its parent. `exempt` can
use this to carve out an exception: tag a parent account `required` and then tag one of
its subaccounts `exempt` to excuse just that subaccount from coverage.

```gherkin
Feature: Required and exempt account tags
  Required and exempt tagged accounts define which postings need documents.

  Scenario: A subaccount can be exempted from a required parent account
    Given a file named "accounts.journal" with content:
      """hledger
      account assets:bank                       ; document_check:exempt
      account expenses:business                 ; document_check:required
      account expenses:business:bank-fees       ; document_check:exempt
      """
    And a file named "journal.journal" with content:
      """hledger
      include accounts.journal

      2026-01-02 Hosting Provider
          expenses:business:hosting        10.00 EUR
          assets:bank                     -10.00 EUR

      2026-01-05 Bank Fee
          expenses:business:bank-fees       2.50 EUR
          assets:bank                      -2.50 EUR
      """
    And a file named "documents/expenses/business/hosting/2026-01-02-hosting.pdf" with content:
      """
      hosting receipt
      """
    When I run "hledger document-check check --journal journal.journal --documents documents --ignore amount-audit-skips --ignore duplicate-files"
    Then the command exits with code 0
    And stdout contains:
      """text
      1/1 transaction groups covered
      ...
      OK.
      """
```

The `expenses:business:hosting` posting inherits `document_check:required` from
`expenses:business` and needs a document. `expenses:business:bank-fees` inherits the
same parent tag but overrides it with its own `document_check:exempt`, so bank fees
never need coverage even though the rest of the business expenses do.

## Counter-account tagged accounts

- `document_check:counter_account`: the posting needs coverage only when another
  posting in the same transaction matches one of the configured counter-account
  patterns.

Counter-account matching is useful when the document belongs to an intermediate
account, not to the payment transaction. For example, a health-insurance notice
can credit a liability account while the actual bank payment later clears that
liability. The notice should be filed once, against the notice transaction, and
the later bank payment should not require a second document.

If one intermediate account can receive notices from several counter accounts,
keep the unnamed `document_check_counter_account` tag for the first pattern or
use named variants for all of them:

```hledger
account liabilities:insurance-notices  ; document_check:counter_account
                                       ; document_check_counter_account_a:expenses:insurance:health:provider
                                       ; document_check_counter_account_b:expenses:insurance:pension:provider
```

The suffix can be any non-empty name. The examples use `_a` and `_b` because they
sort well and stay readable.

```gherkin
Feature: Counter-account tagged accounts
  A counter-account tagged account only needs documents when a transaction touches one
  of its configured counter-account patterns.

  Scenario: A notice needs a document but the payment that later clears it does not
    Given a file named "accounts.journal" with content:
      """hledger
      account assets:bank                              ; document_check:exempt
      account expenses:insurance:health:provider      ; document_check:exempt
      account expenses:insurance:pension:provider     ; document_check:exempt
      account liabilities:insurance-notices           ; document_check:counter_account, document_check_counter_account_a:expenses:insurance:health:provider, document_check_counter_account_b:expenses:insurance:pension:provider
      """
    And a file named "journal.journal" with content:
      """hledger
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
      """
    And files named:
      | path                                                                  | content                  |
      | documents/liabilities/insurance-notices/2026-01-10-health-notice.pdf  | health insurance notice  |
      | documents/liabilities/insurance-notices/2026-02-10-pension-notice.pdf | pension insurance notice |
    When I run "hledger document-check check --journal journal.journal --documents documents --ignore amount-audit-skips --ignore duplicate-files"
    Then the command exits with code 0
    And stdout contains:
      """text
      2/2 transaction groups covered
      ...
      OK.
      """
```

The two notice transactions match `document_check_counter_account_a` and `_b`
respectively, so each needs its own document. The payment transaction posts to
`liabilities:insurance-notices` too, but its other posting is `assets:bank`, which
matches neither counter-account pattern, so the payment is not part of the
transaction groups requiring coverage at all.

```gherkin
Feature: Counter-account configuration validation
  The counter_account tag requires a matching counter-account pattern tag.

  Scenario: Using counter_account without a counter account tag is an error
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                              ; document_check:exempt
      account expenses:insurance:health:provider       ; document_check:exempt
      account liabilities:health-insurance             ; document_check:counter_account

      2026-01-10 Health Insurance Notice
          expenses:insurance:health:provider   220.00 EUR
          liabilities:health-insurance        -220.00 EUR
      """
    And an empty directory named "documents"
    When I run "hledger document-check check --journal journal.journal --documents documents"
    Then the command exits with code 2
    And stdout contains:
      """text
      invalid document_check configuration
      """
    And stdout contains:
      """text
      document_check:counter_account requires document_check_counter_account
      """
```

## Tag prefix document requirements

For imported or domain-specific tag namespaces, a command-line prefix can also
make a posting document-required. This keeps `document_check` explicit by
default, while still allowing another tagging contract to opt in. For example,
a tax tool might tag postings with `tax_role:*`; passing that prefix makes
every tax-relevant transaction subject to document coverage without having to
tag each account with `document_check:required` by hand.

```gherkin
Feature: Tag prefix document requirements
  A configured tag prefix can imply document_check:required.

  Scenario: A prefixed tag can require documents
    Given a file named "journal.journal" with content:
      """hledger
      2026-01-02 Hosting Provider
          expenses:business:hosting        10.00 EUR  ; tax_role:business_expense
          assets:bank                     -10.00 EUR
      """
    And an empty directory named "documents"
    When I run "hledger document-check check --journal journal.journal --documents documents --require-document-for-tag-prefix tax_"
    Then the command exits with code 1
    And stdout contains:
      """text
      Missing Document Coverage (1):
      ...
      2026-01-02  Hosting Provider
      """
```
