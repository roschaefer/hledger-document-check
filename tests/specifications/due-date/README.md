# Due Date

<!--
```scrut {fail_fast: true, output_stream: combined}
$ source "$TESTDIR/../readme-setup.sh"
```
-->

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

Unbooked documents become failures once they are overdue.

### A sent invoice is not yet due

```scrut
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank                                  ; document_check:exempt
> account income:business:freelance:customer           ; document_check:required
> EOF
```

```scrut
$ write_file documents/income/business/freelance/customer/unbooked/invoice.pdf <<'EOF'
> customer invoice
> EOF
```

```scrut
$ write_file documents/income/business/freelance/customer/unbooked/invoice.document.yml <<'EOF'
> due_date: 2026-05-31
> EOF
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --today 2026-05-20 --ignore amount-audit-skips --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Unbooked Documents (1):
  documents/income/business/freelance/customer/unbooked/invoice.pdf
    amount: unknown

Summary:
  Coverage:
    0/0 transaction groups covered
    0 missing-document placeholders
  Open Items:
    0 missing document coverage
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
OK.
```

### A due sent invoice without a transaction fails

```scrut
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank                                  ; document_check:exempt
> account income:business:freelance:customer           ; document_check:required
> EOF
```

```scrut
$ write_file documents/income/business/freelance/customer/unbooked/invoice.pdf <<'EOF'
> overdue customer invoice
> EOF
```

```scrut
$ write_file documents/income/business/freelance/customer/unbooked/invoice.document.yml <<'EOF'
> due_date: 2026-04-15
> EOF
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --today 2026-05-01 --overdue-after-days 14 --ignore amount-audit-skips --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Unbooked Documents (1):
  documents/income/business/freelance/customer/unbooked/invoice.pdf
    amount: unknown

Overdue Unbooked Documents (1):
  Unbooked documents whose due_date is more than 14 days ago.
  documents/income/business/freelance/customer/unbooked/invoice.pdf
    due_date: 2026-04-15
    days overdue: 16

Summary:
  Coverage:
    0/0 transaction groups covered
    0 missing-document placeholders
  Open Items:
    0 missing document coverage
    1 unbooked documents
    1 overdue unbooked documents
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
[1]
```

### A due sent invoice with a matching transaction is OK

```scrut
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank                                  ; document_check:exempt
> account income:business:freelance:customer           ; document_check:required
>
> 2026-04-20 Customer Payment
>     assets:bank                                     100.00 EUR
>     income:business:freelance:customer
> EOF
```

```scrut
$ write_file documents/income/business/freelance/customer/2026-04-20-invoice.pdf <<'EOF'
> customer invoice already paid
> EOF
```

```scrut
$ write_file documents/income/business/freelance/customer/2026-04-20-invoice.document.yml <<'EOF'
> due_date: 2026-04-15
> covers:
>   - date: 2026-04-20
>     account: income:business:freelance:customer
>     amount: 100.00
>     currency: EUR
> EOF
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --today 2026-05-01 --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Redundant Metadata Fields (2):
  These fields already match what the file's location implies and can be removed.
  documents/income/business/freelance/customer/2026-04-20-invoice.document.yml
    covers[0] date: 2026-04-20
    covers[0] account: income:business:freelance:customer

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
