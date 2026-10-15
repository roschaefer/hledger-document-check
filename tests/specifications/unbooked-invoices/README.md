# Unbooked Invoices

<!--
```scrut {fail_fast: true}
$ source "$TESTDIR/../readme-setup.sh"
```
-->

Use an `unbooked/` subfolder inside the account folder for documents that are
real but do not yet have a matching ledger transaction.

Two common cases are:

- you paid an invoice online, but the bank transaction has not cleared yet
- you sent an invoice to a customer, but the customer has not paid yet

Unbooked documents are warnings by default.

Unbooked documents are visible while waiting for the ledger transaction.

### A paid online invoice exists before the bank transaction clears

```scrut
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank                         ; document_check:exempt
> account expenses:business:hosting:aws       ; document_check:required
> EOF
```

```scrut
$ write_file documents/expenses/business/hosting/aws/unbooked/invoice.pdf <<'EOF'
> AWS invoice waiting for bank clearing
> EOF
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore amount-audit-skips --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Unbooked Documents (1):
  documents/expenses/business/hosting/aws/unbooked/invoice.pdf
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

### A sent invoice exists before the customer pays

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
> customer invoice waiting for payment
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

## Move Suggestions

If a unique required transaction with the same account, amount, and currency
appears later, the check prints an explicit `mv` suggestion. The suggestion
renames the unbooked document to the transaction date. It also moves the sidecar
metadata file and sibling support directory when they exist. The target is one
level up in the account folder. After that rename, a due invoice has a
corresponding transaction and is no longer overdue.

When the PDF amount cannot be read reliably, the unbooked document's
`*.document.yml` sidecar can provide the amount and currency used for matching.

When multiple transactions match an unbooked document no move suggestion is printed.

### An unbooked document has no move suggestion when multiple transactions match

```scrut
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank                     ; document_check:exempt
> account expenses:business:hosting:aws   ; document_check:required
>
> 2026-01-02 AWS Invoice A
>     expenses:business:hosting:aws   12.34 EUR
>     assets:bank                    -12.34 EUR
>
> 2026-01-02 AWS Invoice B
>     expenses:business:hosting:aws   12.34 EUR
>     assets:bank                    -12.34 EUR
> EOF
```

```scrut
$ write_pdf documents/expenses/business/hosting/aws/unbooked/invoice.pdf <<'EOF'
> 12.34 EUR
> EOF
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore amount-audit-skips --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Missing Document Coverage (2):
  2026-01-02  AWS Invoice A
    account: expenses:business:hosting:aws
    amount: 12.34 EUR
  2026-01-02  AWS Invoice B
    account: expenses:business:hosting:aws
    amount: 12.34 EUR

Unbooked Documents (1):
  documents/expenses/business/hosting/aws/unbooked/invoice.pdf
    amount: 12.34 EUR

Ambiguous Transaction Groups (1):
  Date-prefixed filenames can only match by account and date.
  These groups have multiple required transactions on the same date in the same account:
  expenses/business/hosting/aws @ 2026-01-02  [2 transactions]
    2026-01-02  12.34 EUR  AWS Invoice A
    2026-01-02  12.34 EUR  AWS Invoice B

Summary:
  Coverage:
    0/1 transaction groups covered
    0 missing-document placeholders
  Open Items:
    2 missing document coverage
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
[1]
```

A unique unbooked document match produces an explicit rename suggestion.

### A matching transaction appears after an unbooked document was filed

```scrut
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank                         ; document_check:exempt
> account expenses:business:hosting:aws       ; document_check:required
>
> 2026-01-02 AWS Invoice
>     expenses:business:hosting:aws   12.34 EUR
>     assets:bank                    -12.34 EUR
> EOF
```

The amount is read from the PDF text:

```scrut
$ write_pdf documents/expenses/business/hosting/aws/unbooked/invoice.pdf <<'EOF'
> Amount Due 12.34 EUR
> EOF
```

```scrut
$ write_file documents/expenses/business/hosting/aws/unbooked/invoice.document.yml <<'EOF'
> due_date: 2026-01-31
> EOF
```

```scrut
$ write_file documents/expenses/business/hosting/aws/unbooked/invoice/usage.csv <<'EOF'
> line,item
> EOF
```

On 2026-01-15 the invoice is not due yet, so it is unbooked but not overdue:

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --today 2026-01-15 --ignore amount-audit-skips --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

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
    mv {working_directory}/documents/expenses/business/hosting/aws/unbooked/invoice.pdf {working_directory}/documents/expenses/business/hosting/aws/2026-01-02-invoice.pdf
    mv {working_directory}/documents/expenses/business/hosting/aws/unbooked/invoice.document.yml {working_directory}/documents/expenses/business/hosting/aws/2026-01-02-invoice.document.yml
    mv {working_directory}/documents/expenses/business/hosting/aws/unbooked/invoice {working_directory}/documents/expenses/business/hosting/aws/2026-01-02-invoice

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
[1]
```

After the suggested moves, the invoice has a corresponding transaction, so it
is not overdue even after its due date:

```scrut
$ mv documents/expenses/business/hosting/aws/unbooked/invoice.pdf documents/expenses/business/hosting/aws/2026-01-02-invoice.pdf
```

```scrut
$ mv documents/expenses/business/hosting/aws/unbooked/invoice.document.yml documents/expenses/business/hosting/aws/2026-01-02-invoice.document.yml
```

```scrut
$ mv documents/expenses/business/hosting/aws/unbooked/invoice documents/expenses/business/hosting/aws/2026-01-02-invoice
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --today 2026-02-15 --ignore amount-audit-skips --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

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


Metadata can supply the amount for an unbooked document whose PDF amount is not readable.

### A matching transaction appears for an unbooked document with a sidecar amount

```scrut
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank                                   ; document_check:exempt
> account expenses:business:transport:train:flixtrain   ; document_check:required
>
> 2026-06-10 FlixTrain
>     expenses:business:transport:train:flixtrain   14.99 EUR
>     assets:bank                                  -14.99 EUR
> EOF
```

```scrut
$ write_pdf documents/expenses/business/transport/train/flixtrain/unbooked/booking.pdf <<'EOF'
> Ticket/Invoice for your reservation
> EOF
```

```scrut
$ write_file documents/expenses/business/transport/train/flixtrain/unbooked/booking.document.yml <<'EOF'
> covers:
>   - amount: 14.99
>     currency: EUR
> EOF
```

```scrut
$ write_file documents/expenses/business/transport/train/flixtrain/unbooked/booking/usage.csv <<'EOF'
> date,distance_km
> 2026-06-10,450
> EOF
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore amount-audit-skips --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Missing Document Coverage (1):
  2026-06-10  FlixTrain
    account: expenses:business:transport:train:flixtrain
    amount: 14.99 EUR

Unbooked Documents (1):
  documents/expenses/business/transport/train/flixtrain/unbooked/booking.pdf
    amount: 14.99 EUR

Suggested Moves (1):
  High-confidence unbooked document matches by account, amount, and currency.
  2026-06-10  FlixTrain
    account: expenses:business:transport:train:flixtrain
    amount: 14.99 EUR
    mv {working_directory}/documents/expenses/business/transport/train/flixtrain/unbooked/booking.pdf {working_directory}/documents/expenses/business/transport/train/flixtrain/2026-06-10-booking.pdf
    mv {working_directory}/documents/expenses/business/transport/train/flixtrain/unbooked/booking.document.yml {working_directory}/documents/expenses/business/transport/train/flixtrain/2026-06-10-booking.document.yml
    mv {working_directory}/documents/expenses/business/transport/train/flixtrain/unbooked/booking {working_directory}/documents/expenses/business/transport/train/flixtrain/2026-06-10-booking

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
[1]
```
