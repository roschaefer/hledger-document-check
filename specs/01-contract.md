# Contract

`hledger-document-check` decides document requirements from hledger account and posting
tags. The normal setup is to keep reusable defaults in `accounts.journal` and
include that file from the actual journal.

The `document_check` tag has three public values:

- `document_check:required`: every posting to this account needs document coverage.
- `document_check:exempt`: postings to this account never need document coverage.
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
Feature: Document check account contract
  Required, exempt, and counter-account tagged accounts define the document requirements.

  Scenario: Account tags decide which postings require documents
    Given a file named "accounts.journal" with content:
      """hledger
      account assets:bank                              ; document_check:exempt
      account expenses:business:hosting               ; document_check:required
      account expenses:insurance:health:provider      ; document_check:exempt
      account expenses:insurance:pension:provider     ; document_check:exempt
      account liabilities:insurance-notices           ; document_check:counter_account, document_check_counter_account_a:expenses:insurance:health:provider, document_check_counter_account_b:expenses:insurance:pension:provider
      """
    And a file named "journal.journal" with content:
      """hledger
      include accounts.journal

      2026-01-02 Hosting Provider
          expenses:business:hosting        10.00 EUR
          assets:bank                     -10.00 EUR

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
      | documents/expenses/business/hosting/2026-01-02-hosting.pdf            | hosting receipt          |
      | documents/liabilities/insurance-notices/2026-01-10-health-notice.pdf  | health insurance notice  |
      | documents/liabilities/insurance-notices/2026-02-10-pension-notice.pdf | pension insurance notice |
    When I run "hledger document-check check --journal journal.journal --documents documents --ignore amount-audit-skips --ignore duplicate-files"
    Then the command exits with code 0
    And stdout contains:
      """text
      3/3 transaction groups covered
      ...
      OK.
      """
```

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

For imported or domain-specific tag namespaces, a command-line prefix can also
make a posting document-required. This keeps `document_check` explicit by
default, while still allowing another tagging contract to opt in.

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
