# Full Example

The repository contains a complete example project under `example/`. It combines
the documented account-tag contract, document directory layout, sidecar metadata,
unbooked invoices, due dates, amount-audit warnings, project configuration, and
Fava input generation.

```gherkin
Feature: Full repository example
  The checked-out repository includes a runnable end-to-end example.

  Scenario: The example document check command works
    Given the development dependencies are installed
    When I clone the repository
    And I run `hledger-document-check check --documents example/documents --today 2026-05-20` from the root directory
    Then I see this output:
      """text

      Missing Document Coverage (1):
        2026-04-10  AWS Invoice
          account: Expenses:Business:Hosting:Aws
          amount: 12.34 EUR

      Unbooked Documents (2):
        example/documents/Expenses/Business/Hosting/Aws/unbooked/invoice.pdf
          amount: 12.34 EUR
        example/documents/Income/Business/Freelance/Customer/unbooked/invoice.pdf
          amount: unknown

      Suggested Moves (1):
        High-confidence unbooked document matches by account, amount, and currency.
        2026-04-10  AWS Invoice
          account: Expenses:Business:Hosting:Aws
          amount: 12.34 EUR
          mv {checkout_dir}/example/documents/Expenses/Business/Hosting/Aws/unbooked/invoice.pdf {checkout_dir}/example/documents/Expenses/Business/Hosting/Aws/2026-04-10-invoice.pdf
          mv {checkout_dir}/example/documents/Expenses/Business/Hosting/Aws/unbooked/invoice.document.yml {checkout_dir}/example/documents/Expenses/Business/Hosting/Aws/2026-04-10-invoice.document.yml
          mv {checkout_dir}/example/documents/Expenses/Business/Hosting/Aws/unbooked/invoice {checkout_dir}/example/documents/Expenses/Business/Hosting/Aws/2026-04-10-invoice

      Amount Audit Skips (8):
        document/transaction currency mismatch
          example/documents/Expenses/Business/Currency-mismatch/2026-05-06-usd-invoice.pdf
            accounts/dates: Expenses/Business/Currency-mismatch @ 2026-05-06
        document amount unreadable
          example/documents/Expenses/Business/Unreadable-document/2026-05-02-unreadable.pdf
            accounts/dates: Expenses/Business/Unreadable-document @ 2026-05-02
          example/documents/Expenses/Tax-demo/2026-01-06-tax-demo.pdf
            accounts/dates: Expenses/Tax-demo @ 2026-01-06
          example/documents/Liabilities/Insurance-notices/2026-01-10-health-notice.pdf
            accounts/dates: Liabilities/Insurance-notices @ 2026-01-10
          example/documents/Liabilities/Insurance-notices/2026-02-10-pension-notice.pdf
            accounts/dates: Liabilities/Insurance-notices @ 2026-02-10
        document currencies unclear
          example/documents/Expenses/Business/Unclear-document-currency/2026-05-03-unclear.pdf
            accounts/dates: Expenses/Business/Unclear-document-currency @ 2026-05-03
        transaction amount missing
          example/documents/Expenses/Business/Missing-transaction-amount/2026-05-04-missing-amount.pdf
            accounts/dates: Expenses/Business/Missing-transaction-amount @ 2026-05-04
        transaction currencies mixed
          example/documents/Expenses/Business/Mixed-transaction-currencies/2026-05-05-mixed.pdf
            accounts/dates: Expenses/Business/Mixed-transaction-currencies @ 2026-05-05

      Ambiguous Transaction Groups (1):
        Date-prefixed filenames can only match by account and date.
        These groups have multiple required transactions on the same date in the same account:
        Expenses/Business/Mixed-transaction-currencies @ 2026-05-05  [2 transactions]
          2026-05-05  10.00 EUR  Mixed Transaction Currencies
          2026-05-05  12.00 USD  Mixed Transaction Currencies

      Summary:
        Coverage:
          19/20 transaction groups covered
          1 missing-document placeholders
        Open Items:
          1 missing document coverage
          2 unbooked documents
          0 overdue unbooked documents
          0 unmatched documents
          0 unexpected files
          0 duplicate groups
          0 redundant metadata fields
          0 unresolvable cover metadata fields
        Amount Audit:
          0 amount mismatches
          8 linked groups checked
          8 linked groups skipped
          Skip Reasons:
            4 document amount unreadable
            1 document currencies unclear
            1 transaction amount missing
            1 transaction currencies mixed
            1 document/transaction currency mismatch
      OK.
      """
    And the exit code is 0
```

The Fava example shows how to use `enrich-journal` as the bridge from hledger to
Fava. The command creates a sidecar-free document tree under
`example/.generated/`, runs `enrich-journal` to add `document:` tags for matched
documents, converts the enriched journal to Beancount with hledger, and starts
Fava with that generated Beancount file. In Fava you can browse transactions
together with their linked PDFs.

Because Fava reads Beancount, account names used with this workflow must follow
Beancount naming conventions. The bundled example uses capitalized account
components such as `Expenses:Business:Hosting`.

```gherkin
Feature: Full repository Fava example
  The checked-out repository includes a runnable Fava integration example.

  Scenario: The example Fava command shows linked documents
    Given the development dependencies are installed
    And the optional Fava dependency is installed
    When I clone the repository
    And I run `example/rebuild-fava-inputs.sh` from the root directory
    And I run a shell command from the root directory:
      """
      hledger-document-check enrich-journal --journal example/journal.journal --documents example/documents --document-tag-root "$(pwd)/example/.generated/fava/documents" | hledger -f - print -O beancount >> example/.generated/fava/example.beancount
      """
    And I start `fava example/.generated/fava/example.beancount` from the root directory
    Then Fava is running at "http://localhost:5000/beancount/documents/"
    And the file "hledger-document-check/example/.generated/fava/example.beancount" contains:
      """text
      option "documents" "{checkout_dir}/example/.generated/fava/documents"
      ...
      ; document:{checkout_dir}/example/.generated/fava/documents/Expenses/Business/Hosting/2026-01-02-hosting.pdf
      """
    And the file "hledger-document-check/example/.generated/fava/documents/Expenses/Business/Cloud/2026-01-04-cloud.pdf" exists
    And the file "hledger-document-check/example/.generated/fava/documents/Expenses/Business/Cloud/2026-01-04-cloud.document.yml" does not exist
```
