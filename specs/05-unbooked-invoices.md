# Unbooked Invoices

Use an `unbooked/` subfolder inside the account folder for documents that are
real but do not yet have a matching ledger transaction.

Two common cases are:

- you paid an invoice online, but the bank transaction has not cleared yet
- you sent an invoice to a customer, but the customer has not paid yet

Unbooked documents are warnings by default.

```gherkin
Feature: Unbooked invoices
  Unbooked documents are visible while waiting for the ledger transaction.

  Scenario: A paid online invoice exists before the bank transaction clears
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                         ; document_check:exempt
      account expenses:business:hosting:aws       ; document_check:required
      """
    And a file named "documents/expenses/business/hosting/aws/unbooked/invoice.pdf" with content:
      """
      AWS invoice waiting for bank clearing
      """
    When I run "hledger document-check check --journal journal.journal --documents documents --ignore amount-audit-skips --ignore duplicate-files"
    Then the command exits with code 0
    And stdout contains:
      """text
      Unbooked Documents (1):
      ...
      OK.
      """

  Scenario: A sent invoice exists before the customer pays
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                                  ; document_check:exempt
      account income:business:freelance:customer           ; document_check:required
      """
    And files named:
      | path                                                                       | content                              |
      | documents/income/business/freelance/customer/unbooked/invoice.pdf          | customer invoice waiting for payment |
      | documents/income/business/freelance/customer/unbooked/invoice.document.yml | due_date: 2026-05-31                 |
    When I run "hledger document-check check --journal journal.journal --documents documents --today 2026-05-20 --ignore amount-audit-skips --ignore duplicate-files"
    Then the command exits with code 0
    And stdout contains:
      """text
      Unbooked Documents (1):
      ...
      0 overdue unbooked documents
      """
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

```gherkin
Feature: Unbooked document without a unique match
  When multiple transactions match an unbooked document no move suggestion is printed.

  Scenario: An unbooked document has no move suggestion when multiple transactions match
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                     ; document_check:exempt
      account expenses:business:hosting:aws   ; document_check:required

      2026-01-02 AWS Invoice A
          expenses:business:hosting:aws   12.34 EUR
          assets:bank                    -12.34 EUR

      2026-01-02 AWS Invoice B
          expenses:business:hosting:aws   12.34 EUR
          assets:bank                    -12.34 EUR
      """
    And a PDF file named "documents/expenses/business/hosting/aws/unbooked/invoice.pdf" with text:
      """
      12.34 EUR
      """
    When I run "hledger document-check check --journal journal.journal --documents documents --ignore amount-audit-skips --ignore duplicate-files"
    Then the command exits with code 1
    And stdout contains:
      """text
      Missing Document Coverage (2):
      ...
      Unbooked Documents (1):
      """
    And stdout does not contain:
      """text
      Suggested Moves
      """
```

The unique-match move behavior is exercised by the
[unbooked-move Scrut transcript](../tests/specifications/unbooked-move/).

```gherkin
Feature: Unbooked document move suggestions from metadata amounts
  Metadata can supply the amount for an unbooked document whose PDF amount is not readable.

  Scenario: A matching transaction appears for an unbooked document with a sidecar amount
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                                   ; document_check:exempt
      account expenses:business:transport:train:flixtrain   ; document_check:required

      2026-06-10 FlixTrain
          expenses:business:transport:train:flixtrain   14.99 EUR
          assets:bank                                  -14.99 EUR
      """
    And a PDF file named "documents/expenses/business/transport/train/flixtrain/unbooked/booking.pdf" with text:
      """
      Ticket/Invoice for your reservation
      """
    And a file named "documents/expenses/business/transport/train/flixtrain/unbooked/booking.document.yml" with content:
      """yaml
      covers:
        - amount: 14.99
          currency: EUR
      """
    When I run "hledger document-check check --journal journal.journal --documents documents --ignore amount-audit-skips --ignore duplicate-files"
    Then the command exits with code 1
    And stdout contains:
      """text
      Unbooked Documents (1):
        documents/expenses/business/transport/train/flixtrain/unbooked/booking.pdf
          amount: 14.99 EUR

      Suggested Moves (1):
        High-confidence unbooked document matches by account, amount, and currency.
        2026-06-10  FlixTrain
          account: expenses:business:transport:train:flixtrain
          amount: 14.99 EUR
          mv {work_dir}/documents/expenses/business/transport/train/flixtrain/unbooked/booking.pdf {work_dir}/documents/expenses/business/transport/train/flixtrain/2026-06-10-booking.pdf
          mv {work_dir}/documents/expenses/business/transport/train/flixtrain/unbooked/booking.document.yml {work_dir}/documents/expenses/business/transport/train/flixtrain/2026-06-10-booking.document.yml
      """
```
