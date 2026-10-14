# Full repository example

The checked-out repository includes a runnable end-to-end example combining
the documented account-tag contract, directory layout, metadata, unbooked
invoices, due dates, amount auditing, and configuration.

## The example document check reports its complete state

The checkout path in suggested commands is normalized so this transcript is
stable across machines and temporary worktrees.

<!--
```scrut {fail_fast: true, output_stream: combined}
$ source "$TESTDIR/../readme-setup.sh"
```
-->

[`setup.bash`](setup.bash) builds an isolated copy of the repository example:

```scrut
$ build_scenario scenario_full_example
```

```scrut
$ set -o pipefail; "$DOCUMENT_CHECK_BIN" check --documents example/documents --today 2026-05-20 | sed "s|$PWD|{checkout_dir}|g"

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
```
