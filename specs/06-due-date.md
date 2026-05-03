# Due Date

`due_date` belongs in the sidecar metadata for unbooked invoices that should be
tracked as unpaid:

```yaml
due_date: 2026-04-15
```

The tool does not infer when an invoice was sent and does not calculate payment
terms from invoice text. Maintaining the due date is the user's responsibility,
and the presence of `due_date` is enough to opt the unbooked document into the
overdue check.

By default, an invoice is overdue when today's date is more than 14 days after
`due_date`. The examples use `--today` only to keep the executable
specifications deterministic; normal runs use the system date.

```gherkin
Feature: Due dates for unbooked documents
  Unbooked documents become failures once they are overdue.

  Scenario: A sent invoice is not yet due
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                                  ; document_check:exempt
      account income:business:freelance:customer           ; document_check:required
      """
    And files named:
      | path                                                                       | content              |
      | documents/income/business/freelance/customer/unbooked/invoice.pdf          | customer invoice     |
      | documents/income/business/freelance/customer/unbooked/invoice.document.yml | due_date: 2026-05-31 |
    When I run "hledger document-check check --journal journal.journal --documents documents --today 2026-05-20 --ignore amount-audit-skips --ignore duplicate-files"
    Then the command exits with code 0
    And stdout contains:
      """text
      Unbooked Documents (1):
      ...
      0 overdue unbooked documents
      """

  Scenario: A due sent invoice without a transaction fails
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                                  ; document_check:exempt
      account income:business:freelance:customer           ; document_check:required
      """
    And files named:
      | path                                                                       | content                  |
      | documents/income/business/freelance/customer/unbooked/invoice.pdf          | overdue customer invoice |
      | documents/income/business/freelance/customer/unbooked/invoice.document.yml | due_date: 2026-04-15     |
    When I run "hledger document-check check --journal journal.journal --documents documents --today 2026-05-01 --overdue-after-days 14 --ignore amount-audit-skips --ignore duplicate-files"
    Then the command exits with code 1
    And stdout contains:
      """text
      Overdue Unbooked Documents (1):
      ...
      due_date: 2026-04-15
      """

  Scenario: A due sent invoice with a matching transaction is OK
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                                  ; document_check:exempt
      account income:business:freelance:customer           ; document_check:required

      2026-04-20 Customer Payment
          assets:bank                                     100.00 EUR
          income:business:freelance:customer
      """
    And a file named "documents/income/business/freelance/customer/2026-04-20-invoice.pdf" with content:
      """
      customer invoice already paid
      """
    And a file named "documents/income/business/freelance/customer/2026-04-20-invoice.document.yml" with content:
      """yaml
      due_date: 2026-04-15
      covers:
        - date: 2026-04-20
          account: income:business:freelance:customer
          amount: 100.00
          currency: EUR
      """
    When I run "hledger document-check check --journal journal.journal --documents documents --today 2026-05-01 --ignore duplicate-files"
    Then the command exits with code 0
    And stdout contains:
      """text
      0 overdue unbooked documents
      ...
      OK.
      """
```
