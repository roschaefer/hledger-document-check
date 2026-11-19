# Partial Payments

<!--
```scrut {fail_fast: true}
$ source "$TESTDIR/../readme-setup.sh"
```
-->

A document is sometimes paid in several installments: a property tax notice in
quarterly payments, a laptop bought on a payment plan, or a customer invoice
paid in parts. The top-level `amount` stays the full document total, and each
cover records one payment booked so far.

A payment that is still to come is declared explicitly with `outstanding`, so
that the covers plus `outstanding` add up to the document total. The check lists
such a document under `partially-paid-documents`, which warns by default. Any
shortfall that is not declared is an amount mismatch, so a mistyped or forgotten
cover still fails.

The scenarios below follow one invoice from its first to its last payment. Each
step changes one thing.

```scrut
$ new_scenario
```

### The first installment does not cover the invoice

The customer pays 60.00 EUR of a 100.00 EUR invoice.

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank             ; document_check:exempt
> account income:customer         ; document_check:required
>
> 2026-01-05 Customer Payment Rate 1
>     assets:bank                 60.00 EUR
>     income:customer
> EOF
```

```scrut
$ write_file documents/income/customer/invoice.pdf <<'EOF'
> customer invoice
> EOF
```

```scrut
$ write_file documents/income/customer/invoice.document.yml <<'EOF'
> amount: 100.00
> currency: EUR
> covers:
>   - date: 2026-01-05
>     amount: 60.00
> EOF
```

Without a declaration, the gap of 40.00 EUR may as well be a typo or a
forgotten cover, so the check fails.

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files

Amount Mismatches (1):
  allocated document amounts 60.00 != declared document total 100.00
    accounts/dates: income/customer @ 2026-01-05
    documents/income/customer/invoice.pdf

Summary:
  Coverage:
    1/1 transaction groups covered
    0 missing-document placeholders
  Open Items:
    0 missing document coverage
    0 unbooked documents
    0 overdue unbooked documents
    0 partially paid documents
    0 unmatched documents
    0 unexpected files
    0 duplicate groups
    0 redundant metadata fields
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

### Declaring the rest as outstanding

The only change is the new `outstanding` line.

```scrut
$ write_file documents/income/customer/invoice.document.yml <<'EOF'
> amount: 100.00
> currency: EUR
> outstanding: 40.00
> covers:
>   - date: 2026-01-05
>     amount: 60.00
> EOF
```

The invoice is now partially paid, which is a warning.

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files

Partially Paid Documents (1):
  Covers record the payments so far; the declared rest is outstanding.
  documents/income/customer/invoice.pdf
    paid: 60.00 EUR of 100.00 EUR
    outstanding: 40.00 EUR
    accounts/dates: income/customer @ 2026-01-05

Summary:
  Coverage:
    1/1 transaction groups covered
    0 missing-document placeholders
  Open Items:
    0 missing document coverage
    0 unbooked documents
    0 overdue unbooked documents
    1 partially paid documents
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

### The second installment makes `outstanding` stale

The customer pays the remaining 40.00 EUR, and the payment is booked with a
second cover.

```scrut
$ cat >>journal.journal <<'EOF'
>
> 2026-02-05 Customer Payment Rate 2
>     assets:bank                 40.00 EUR
>     income:customer
> EOF
```

```scrut
$ cat >>documents/income/customer/invoice.document.yml <<'EOF'
>   - date: 2026-02-05
>     amount: 40.00
> EOF
```

The covers plus the old `outstanding` now exceed the document total. The
mismatch is the reminder to update the declaration.

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files

Amount Mismatches (1):
  allocated document amounts 100.00 != declared document total 100.00 minus outstanding 40.00 = 60.00
    accounts/dates: income/customer @ 2026-01-05, 2026-02-05
    documents/income/customer/invoice.pdf

Summary:
  Coverage:
    2/2 transaction groups covered
    0 missing-document placeholders
  Open Items:
    0 missing document coverage
    0 unbooked documents
    0 overdue unbooked documents
    0 partially paid documents
    0 unmatched documents
    0 unexpected files
    0 duplicate groups
    0 redundant metadata fields
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

### Removing `outstanding` once the invoice is fully paid

The only change is the removed `outstanding` line.

```scrut
$ write_file documents/income/customer/invoice.document.yml <<'EOF'
> amount: 100.00
> currency: EUR
> covers:
>   - date: 2026-01-05
>     amount: 60.00
>   - date: 2026-02-05
>     amount: 40.00
> EOF
```

The covers add up to the document total, and the invoice is no longer listed.

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files

Summary:
  Coverage:
    2/2 transaction groups covered
    0 missing-document placeholders
  Open Items:
    0 missing document coverage
    0 unbooked documents
    0 overdue unbooked documents
    0 partially paid documents
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

### Paying more than the document total

The customer accidentally pays another 10.00 EUR, and it is booked with a third
cover.

```scrut
$ cat >>journal.journal <<'EOF'
>
> 2026-03-05 Customer Payment Rate 3
>     assets:bank                 10.00 EUR
>     income:customer
> EOF
```

```scrut
$ cat >>documents/income/customer/invoice.document.yml <<'EOF'
>   - date: 2026-03-05
>     amount: 10.00
> EOF
```

Covers exceeding the document total are an amount mismatch.

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files

Amount Mismatches (1):
  allocated document amounts 110.00 != declared document total 100.00
    accounts/dates: income/customer @ 2026-01-05, 2026-02-05, 2026-03-05
    documents/income/customer/invoice.pdf

Summary:
  Coverage:
    3/3 transaction groups covered
    0 missing-document placeholders
  Open Items:
    0 missing document coverage
    0 unbooked documents
    0 overdue unbooked documents
    0 partially paid documents
    0 unmatched documents
    0 unexpected files
    0 duplicate groups
    0 redundant metadata fields
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
