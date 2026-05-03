# Amount Audit

Coverage proves that a document is connected to a required posting group. Amount
audit goes one step further and checks whether linked document totals plausibly
match linked transaction totals.

The amount can come from a parsed PDF total or from `*.document.yml`. Metadata is
preferred when a PDF is hard to parse, when the bank transaction is already
converted into another currency, or when one document covers multiple groups.

## Warning Cases

Some linked groups cannot be audited. By default these skip reasons are only
summarized, but `--warn-on amount-audit-skips` prints the details.

```gherkin
Feature: Amount audit skip reasons
  Amount audit explains why a linked group could not be compared.

  Scenario: Every amount-audit skip reason is reported
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                                      ; document_check:exempt
      account expenses:business:unreadable-document            ; document_check:required
      account expenses:business:unclear-document-currency      ; document_check:required
      account expenses:business:missing-transaction-amount     ; document_check:required
      account expenses:business:mixed-transaction-currencies   ; document_check:required
      account expenses:business:currency-mismatch              ; document_check:required

      2026-01-02 Unreadable Document Amount
          expenses:business:unreadable-document                 10.00 EUR
          assets:bank                                          -10.00 EUR

      2026-01-03 Unclear Document Currency
          expenses:business:unclear-document-currency           10.00 EUR
          assets:bank                                          -10.00 EUR

      2026-01-04 Missing Transaction Amount
          expenses:business:missing-transaction-amount

      2026-01-05 Mixed Transaction Currencies
          expenses:business:mixed-transaction-currencies        10.00 EUR
          assets:bank                                          -10.00 EUR
          expenses:business:mixed-transaction-currencies        12.00 USD
          assets:paypal                                        -12.00 USD

      2026-01-06 Currency Mismatch
          expenses:business:currency-mismatch                   10.00 EUR
          assets:bank                                          -10.00 EUR
      """
    And files named:
      | path                                                                                     | content                                        |
      | documents/expenses/business/unreadable-document/2026-01-02-unreadable.pdf                | no parseable total in this fake pdf            |
      | documents/expenses/business/unclear-document-currency/2026-01-03-unclear.pdf             | unclear currency metadata example              |
      | documents/expenses/business/missing-transaction-amount/2026-01-04-missing-amount.pdf     | missing transaction amount metadata example    |
      | documents/expenses/business/mixed-transaction-currencies/2026-01-05-mixed.pdf            | mixed transaction currencies metadata example  |
      | documents/expenses/business/currency-mismatch/2026-01-06-usd-invoice.pdf                 | currency mismatch metadata example             |
    And a file named "documents/expenses/business/unclear-document-currency/2026-01-03-unclear.document.yml" with content:
      """yaml
      covers:
        - date: 2026-01-03
          account: expenses:business:unclear-document-currency
          amount: 10.00
          currency: EUR
        - date: 2026-01-03
          account: expenses:business:unclear-document-currency
          amount: 10.00
          currency: USD
      """
    And a file named "documents/expenses/business/missing-transaction-amount/2026-01-04-missing-amount.document.yml" with content:
      """yaml
      covers:
        - date: 2026-01-04
          account: expenses:business:missing-transaction-amount
          amount: 10.00
          currency: EUR
      """
    And a file named "documents/expenses/business/mixed-transaction-currencies/2026-01-05-mixed.document.yml" with content:
      """yaml
      covers:
        - date: 2026-01-05
          account: expenses:business:mixed-transaction-currencies
          amount: 22.00
          currency: EUR
      """
    And a file named "documents/expenses/business/currency-mismatch/2026-01-06-usd-invoice.document.yml" with content:
      """yaml
      covers:
        - date: 2026-01-06
          account: expenses:business:currency-mismatch
          amount: 10.00
          currency: USD
      """
    When I run "hledger document-check check --journal journal.journal --documents documents --warn-on amount-audit-skips --ignore duplicate-files"
    Then the command exits with code 0
    And stdout contains:
      """text
      Amount Audit Skips (5):
      ...
      1 document amount unreadable
      ...
      1 document currencies unclear
      ...
      1 transaction amount missing
      ...
      1 transaction currencies mixed
      ...
      1 document/transaction currency mismatch
      """
```

## Warnings As Failures

Use `--fail-on amount-audit-skips` if unreadable or otherwise skipped amount
checks should break the run.

```gherkin
Feature: Amount audit warnings can fail
  Skip reasons can be promoted to failures.

  Scenario: Parsed amount warnings can be made fatal
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                          ; document_check:exempt
      account expenses:business:hosting            ; document_check:required

      2026-01-02 Hosting Provider
          expenses:business:hosting                 10.00 EUR
          assets:bank                              -10.00 EUR
      """
    And a file named "documents/expenses/business/hosting/2026-01-02-hosting.pdf" with content:
      """
      fake pdf without a readable amount
      """
    When I run "hledger document-check check --journal journal.journal --documents documents --fail-on amount-audit-skips --ignore duplicate-files"
    Then the command exits with code 1
    And stdout contains:
      """text
      Amount Audit Skips (1):
      ...
      1 linked groups skipped
      """
```

## Override Amount And Currency

For foreign-currency invoices, the ledger may intentionally contain the bank's
converted amount. Put the amount to audit in `*.document.yml`; the comparison is
then made against that metadata amount.

```gherkin
Feature: Metadata can override document amount and currency
  Metadata supplies the auditable amount when the PDF total is not the ledger amount.

  Scenario: A foreign-currency invoice is audited with the converted bank amount
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                       ; document_check:exempt
      account expenses:business:cloud           ; document_check:required

      2026-01-04 Cloud Provider
          expenses:business:cloud                92.00 EUR
          assets:bank                           -92.00 EUR
      """
    And a file named "documents/expenses/business/cloud/2026-01-04-cloud.pdf" with content:
      """
      invoice total 100.00 USD, bank booked converted amount
      """
    And a file named "documents/expenses/business/cloud/2026-01-04-cloud.document.yml" with content:
      """yaml
      covers:
        - date: 2026-01-04
          account: expenses:business:cloud
          amount: 92.00
          currency: EUR
      """
    When I run "hledger document-check check --journal journal.journal --documents documents --ignore duplicate-files"
    Then the command exits with code 0
    And stdout contains:
      """text
      0 amount mismatches
      ...
      OK.
      """
```

## Mismatch Is A Failure

If metadata provides a clear amount and currency, a mismatch against the ledger
is a hard failure by default.

```gherkin
Feature: Amount mismatches fail by default
  A clear metadata amount must match the linked transaction amount.

  Scenario: Metadata amount differs from the transaction amount
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                       ; document_check:exempt
      account expenses:business:software        ; document_check:required

      2026-01-03 Software Suite
          expenses:business:software             70.00 EUR
          assets:bank                           -70.00 EUR
      """
    And a file named "documents/expenses/business/software/2026-01-03-suite.pdf" with content:
      """
      software suite invoice
      """
    And a file named "documents/expenses/business/software/2026-01-03-suite.document.yml" with content:
      """yaml
      covers:
        - date: 2026-01-03
          account: expenses:business:software
          amount: 100.00
          currency: EUR
      """
    When I run "hledger document-check check --journal journal.journal --documents documents --ignore duplicate-files"
    Then the command exits with code 1
    And stdout contains:
      """text
      Amount Mismatches (1):
      ...
      document total 100.00 != transaction total 70.00
      """
```
