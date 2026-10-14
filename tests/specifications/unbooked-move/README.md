
# Unbooked document move suggestions

An unbooked document with a unique account, amount, and currency match produces
commands for moving the document, its metadata, and its supporting directory.
The scenario starts from committed fixtures copied into an isolated temporary
directory.

## A unique match produces a complete move suggestion

<!--
```scrut {fail_fast: true, output_stream: combined}
$ source "$TESTDIR/../readme-setup.sh"
```
-->

[`setup.bash`](setup.bash) builds the journal and document tree:

```scrut
$ build_scenario scenario_unbooked_move
```

```scrut
$ set -o pipefail; "$DOCUMENT_CHECK_BIN" check --journal journal.journal --documents documents --today 2026-01-15 --ignore amount-audit-skips --ignore duplicate-files | sed "s|$PWD|{work_dir}|g"; \
> status=${PIPESTATUS[0]}; echo "exit code: $status"; test "$status" -eq 1

Missing Document Coverage (1):
  2026-01-02  AWS Invoice
    account: expenses:business:hosting:aws
    amount: 12.34 EUR

Unbooked Documents (1):
  documents/expenses/business/hosting/aws/unbooked/invoice.pdf
    amount: 12.34 EUR

Suggested Moves (1):
  High-confidence unbooked document matches by account, amount, and currency.
  2026-01-02  AWS Invoice
    account: expenses:business:hosting:aws
    amount: 12.34 EUR
    mv {work_dir}/documents/expenses/business/hosting/aws/unbooked/invoice.pdf {work_dir}/documents/expenses/business/hosting/aws/2026-01-02-invoice.pdf
    mv {work_dir}/documents/expenses/business/hosting/aws/unbooked/invoice.document.yml {work_dir}/documents/expenses/business/hosting/aws/2026-01-02-invoice.document.yml
    mv {work_dir}/documents/expenses/business/hosting/aws/unbooked/invoice {work_dir}/documents/expenses/business/hosting/aws/2026-01-02-invoice

Summary:
  Coverage:
    0/1 transaction groups covered
    0 missing-document placeholders
  Open Items:
    1 missing document coverage
    1 unbooked documents
    0 overdue unbooked documents
    0 unmatched documents
    0 unexpected files
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
exit code: 1
```
