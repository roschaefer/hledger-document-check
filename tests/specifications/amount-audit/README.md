# Amount Audit

<!--
```scrut {fail_fast: true, output_stream: combined}
$ source "$TESTDIR/../readme-setup.sh"
```
-->

Coverage proves that a document is connected to a required posting group. Amount
audit goes one step further and checks whether linked document totals plausibly
match linked transaction totals.

The amount can come from a parsed PDF total or from `*.document.yml`. Metadata is
preferred when a PDF is hard to parse, when the bank transaction is already
converted into another currency, or when one document covers multiple groups.

Invoices typically show positive amounts, so the audit ignores sign and compares
magnitude only, refunds don't need a negative `amount`.

## Warning Cases

Some linked groups cannot be audited. By default these skip reasons are only
summarized, but `--warn-on amount-audit-skips` prints the details.

Amount audit explains why a linked group could not be compared.

### Every amount-audit skip reason is reported

```scrut
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank                                      ; document_check:exempt
> account expenses:business:unreadable-document            ; document_check:required
> account expenses:business:unclear-document-currency      ; document_check:required
> account expenses:business:missing-transaction-amount     ; document_check:required
> account expenses:business:mixed-transaction-currencies   ; document_check:required
> account expenses:business:currency-mismatch              ; document_check:required
>
> 2026-01-02 Unreadable Document Amount
>     expenses:business:unreadable-document                 10.00 EUR
>     assets:bank                                          -10.00 EUR
>
> 2026-01-03 Unclear Document Currency
>     expenses:business:unclear-document-currency           10.00 EUR
>     assets:bank                                          -10.00 EUR
>
> 2026-01-04 Missing Transaction Amount
>     expenses:business:missing-transaction-amount
>
> 2026-01-05 Mixed Transaction Currencies
>     expenses:business:mixed-transaction-currencies        10.00 EUR
>     assets:bank                                          -10.00 EUR
>     expenses:business:mixed-transaction-currencies        12.00 USD
>     assets:paypal                                        -12.00 USD
>
> 2026-01-06 Currency Mismatch
>     expenses:business:currency-mismatch                   10.00 EUR
>     assets:bank                                          -10.00 EUR
> EOF
```

```scrut
$ write_file documents/expenses/business/unreadable-document/2026-01-02-unreadable.pdf <<'EOF'
> no parseable total in this fake pdf
> EOF
```

```scrut
$ write_file documents/expenses/business/unclear-document-currency/2026-01-03-unclear.pdf <<'EOF'
> unclear currency metadata example
> EOF
```

```scrut
$ write_file documents/expenses/business/missing-transaction-amount/2026-01-04-missing-amount.pdf <<'EOF'
> missing transaction amount metadata example
> EOF
```

```scrut
$ write_file documents/expenses/business/mixed-transaction-currencies/2026-01-05-mixed.pdf <<'EOF'
> mixed transaction currencies metadata example
> EOF
```

```scrut
$ write_file documents/expenses/business/currency-mismatch/2026-01-06-usd-invoice.pdf <<'EOF'
> currency mismatch metadata example
> EOF
```

```scrut
$ write_file documents/expenses/business/unclear-document-currency/2026-01-03-unclear.document.yml <<'EOF'
> covers:
>   - date: 2026-01-03
>     account: expenses:business:unclear-document-currency
>     amount: 10.00
>     currency: EUR
>   - date: 2026-01-03
>     account: expenses:business:unclear-document-currency
>     amount: 10.00
>     currency: USD
> EOF
```

```scrut
$ write_file documents/expenses/business/missing-transaction-amount/2026-01-04-missing-amount.document.yml <<'EOF'
> covers:
>   - date: 2026-01-04
>     account: expenses:business:missing-transaction-amount
>     amount: 10.00
>     currency: EUR
> EOF
```

```scrut
$ write_file documents/expenses/business/mixed-transaction-currencies/2026-01-05-mixed.document.yml <<'EOF'
> covers:
>   - date: 2026-01-05
>     account: expenses:business:mixed-transaction-currencies
>     amount: 22.00
>     currency: EUR
> EOF
```

```scrut
$ write_file documents/expenses/business/currency-mismatch/2026-01-06-usd-invoice.document.yml <<'EOF'
> covers:
>   - date: 2026-01-06
>     account: expenses:business:currency-mismatch
>     amount: 10.00
>     currency: USD
> EOF
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --warn-on amount-audit-skips --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Amount Audit Skips (5):
  document/transaction currency mismatch
    documents/expenses/business/currency-mismatch/2026-01-06-usd-invoice.pdf
      accounts/dates: expenses/business/currency-mismatch @ 2026-01-06
  document amount unreadable
    documents/expenses/business/unreadable-document/2026-01-02-unreadable.pdf
      accounts/dates: expenses/business/unreadable-document @ 2026-01-02
  document currencies unclear
    documents/expenses/business/unclear-document-currency/2026-01-03-unclear.pdf
      accounts/dates: expenses/business/unclear-document-currency @ 2026-01-03
  transaction amount missing
    documents/expenses/business/missing-transaction-amount/2026-01-04-missing-amount.pdf
      accounts/dates: expenses/business/missing-transaction-amount @ 2026-01-04
  transaction currencies mixed
    documents/expenses/business/mixed-transaction-currencies/2026-01-05-mixed.pdf
      accounts/dates: expenses/business/mixed-transaction-currencies @ 2026-01-05

Ambiguous Transaction Groups (1):
  Date-prefixed filenames can only match by account and date.
  These groups have multiple required transactions on the same date in the same account:
  expenses/business/mixed-transaction-currencies @ 2026-01-05  [2 transactions]
    2026-01-05  10.00 EUR  Mixed Transaction Currencies
    2026-01-05  12.00 USD  Mixed Transaction Currencies

Redundant Metadata Fields (10):
  These fields already match what the file's location implies and can be removed.
  documents/expenses/business/currency-mismatch/2026-01-06-usd-invoice.document.yml
    covers[0] date: 2026-01-06
    covers[0] account: expenses:business:currency-mismatch
  documents/expenses/business/missing-transaction-amount/2026-01-04-missing-amount.document.yml
    covers[0] date: 2026-01-04
    covers[0] account: expenses:business:missing-transaction-amount
  documents/expenses/business/mixed-transaction-currencies/2026-01-05-mixed.document.yml
    covers[0] date: 2026-01-05
    covers[0] account: expenses:business:mixed-transaction-currencies
  documents/expenses/business/unclear-document-currency/2026-01-03-unclear.document.yml
    covers[0] date: 2026-01-03
    covers[0] account: expenses:business:unclear-document-currency
    covers[1] date: 2026-01-03
    covers[1] account: expenses:business:unclear-document-currency

Summary:
  Coverage:
    5/5 transaction groups covered
    0 missing-document placeholders
  Open Items:
    0 missing document coverage
    0 unbooked documents
    0 overdue unbooked documents
    0 unmatched documents
    0 unexpected files
    0 duplicate groups
    10 redundant metadata fields
    0 unresolvable cover metadata fields
  Amount Audit:
    0 amount mismatches
    0 linked groups checked
    5 linked groups skipped
    Skip Reasons:
      1 document amount unreadable
      1 document currencies unclear
      1 transaction amount missing
      1 transaction currencies mixed
      1 document/transaction currency mismatch
OK.
```

## Warnings As Failures

Use `--fail-on amount-audit-skips` if unreadable or otherwise skipped amount
checks should break the run.

Skip reasons can be promoted to failures.

### Parsed amount warnings can be made fatal

```scrut
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank                          ; document_check:exempt
> account expenses:business:hosting            ; document_check:required
>
> 2026-01-02 Hosting Provider
>     expenses:business:hosting                 10.00 EUR
>     assets:bank                              -10.00 EUR
> EOF
```

```scrut
$ write_file documents/expenses/business/hosting/2026-01-02-hosting.pdf <<'EOF'
> fake pdf without a readable amount
> EOF
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --fail-on amount-audit-skips --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Amount Audit Skips (1):
  document amount unreadable
    documents/expenses/business/hosting/2026-01-02-hosting.pdf
      accounts/dates: expenses/business/hosting @ 2026-01-02

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

## Override Amount And Currency

For foreign-currency invoices, the ledger may intentionally contain the bank's
converted amount. Put the amount to audit in `*.document.yml`; the comparison is
then made against that metadata amount.

Metadata supplies the auditable amount when the PDF total is not the ledger amount.

### A foreign-currency invoice is audited with the converted bank amount

```scrut
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank                       ; document_check:exempt
> account expenses:business:cloud           ; document_check:required
>
> 2026-01-04 Cloud Provider
>     expenses:business:cloud                92.00 EUR
>     assets:bank                           -92.00 EUR
> EOF
```

```scrut
$ write_file documents/expenses/business/cloud/2026-01-04-cloud.pdf <<'EOF'
> invoice total 100.00 USD, bank booked converted amount
> EOF
```

```scrut
$ write_file documents/expenses/business/cloud/2026-01-04-cloud.document.yml <<'EOF'
> covers:
>   - date: 2026-01-04
>     account: expenses:business:cloud
>     amount: 92.00
>     currency: EUR
> EOF
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Redundant Metadata Fields (2):
  These fields already match what the file's location implies and can be removed.
  documents/expenses/business/cloud/2026-01-04-cloud.document.yml
    covers[0] date: 2026-01-04
    covers[0] account: expenses:business:cloud

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

## Mismatch Is A Failure

If metadata provides a clear amount and currency, a mismatch against the ledger
is a hard failure by default.

A clear metadata amount must match the linked transaction amount.

### Metadata amount differs from the transaction amount

```scrut
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank                       ; document_check:exempt
> account expenses:business:software        ; document_check:required
>
> 2026-01-03 Software Suite
>     expenses:business:software             70.00 EUR
>     assets:bank                           -70.00 EUR
> EOF
```

```scrut
$ write_file documents/expenses/business/software/2026-01-03-suite.pdf <<'EOF'
> software suite invoice
> EOF
```

```scrut
$ write_file documents/expenses/business/software/2026-01-03-suite.document.yml <<'EOF'
> covers:
>   - date: 2026-01-03
>     account: expenses:business:software
>     amount: 100.00
>     currency: EUR
> EOF
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Amount Mismatches (1):
  document total 100.00 != transaction total 70.00
    accounts/dates: expenses/business/software @ 2026-01-03
    documents/expenses/business/software/2026-01-03-suite.pdf

Redundant Metadata Fields (2):
  These fields already match what the file's location implies and can be removed.
  documents/expenses/business/software/2026-01-03-suite.document.yml
    covers[0] date: 2026-01-03
    covers[0] account: expenses:business:software

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
