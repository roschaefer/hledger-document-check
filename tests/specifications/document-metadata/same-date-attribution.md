# Same-date document attribution

<!--
```scrut {fail_fast: true}
$ source "$TESTDIR/../readme-setup.sh"
```
-->

Distinct amounts resolve individual documents, and enrichment exposes only the
assigned document on each transaction.

```scrut
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank  ; document_check:exempt
> account expenses:domains  ; document_check:required
>
> 2026-03-01 Domain A
>     expenses:domains  4.00 EUR
>     assets:bank
>
> 2026-03-01 Domain B
>     expenses:domains  6.00 EUR
>     assets:bank
> EOF
```

```scrut
$ write_file documents/expenses/domains/2026-03-01-a.pdf <<'EOF'
> A
> EOF
```

```scrut
$ write_file documents/expenses/domains/2026-03-01-b.pdf <<'EOF'
> B
> EOF
```

```scrut
$ write_file documents/expenses/domains/2026-03-01-a.document.yml <<'EOF'
> amount: 4.00
> currency: EUR
> EOF
```

```scrut
$ write_file documents/expenses/domains/2026-03-01-b.document.yml <<'EOF'
> amount: 6.00
> currency: EUR
> EOF
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files --ignore redundant-metadata | sed "s|$PWD|{working_directory}|g"

Document Attribution:
  expenses/domains @ 2026-03-01
    documents/expenses/domains/2026-03-01-a.pdf top-level -> transaction #1: Domain A
    documents/expenses/domains/2026-03-01-b.pdf top-level -> transaction #2: Domain B

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

```scrut
$ hledger-document-check enrich-journal --journal journal.journal --documents documents | sed -n '/^2026/p;/document:/p'
2026-03-01 Domain A
    ; document:2026-03-01-a.pdf
2026-03-01 Domain B
    ; document:2026-03-01-b.pdf
```

Equal amounts without description metadata leave two possible assignments. Counting covers
is not enough to prove attribution.

```scrut
$ sed -i -e 's/4.00/5.00/' -e 's/6.00/5.00/' journal.journal documents/expenses/domains/*.document.yml
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files --ignore redundant-metadata | sed "s|$PWD|{working_directory}|g"

Ambiguous Transaction Groups (1):
  Date-prefixed filenames can only match by account and date.
  These groups have multiple required transactions on the same date in the same account:
  expenses/domains @ 2026-03-01  [2 transactions]
    2026-03-01  5.00 EUR  Domain A
    2026-03-01  5.00 EUR  Domain B

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

Explicit descriptions distinguish the otherwise equal amounts.

```scrut
$ printf 'description: Domain A\n' >> documents/expenses/domains/2026-03-01-a.document.yml
```

```scrut
$ printf 'description: Domain B\n' >> documents/expenses/domains/2026-03-01-b.document.yml
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files --ignore redundant-metadata | sed "s|$PWD|{working_directory}|g"

Document Attribution:
  expenses/domains @ 2026-03-01
    documents/expenses/domains/2026-03-01-a.pdf top-level -> transaction #1: Domain A
    documents/expenses/domains/2026-03-01-b.pdf top-level -> transaction #2: Domain B

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

The descriptions keep identifying the transactions even when document amounts
are wrong. Individual audits detect errors that cancel in the group total.

```scrut
$ sed -i 's/5.00/4.00/' documents/expenses/domains/2026-03-01-a.document.yml
```

```scrut
$ sed -i 's/5.00/6.00/' documents/expenses/domains/2026-03-01-b.document.yml
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files --ignore redundant-metadata | sed "s|$PWD|{working_directory}|g"

Amount Mismatches (2):
  top-level amount 4.00 != transaction #1 (Domain A) 5.00
    accounts/dates: expenses/domains @ 2026-03-01
    documents/expenses/domains/2026-03-01-a.pdf
  top-level amount 6.00 != transaction #2 (Domain B) 5.00
    accounts/dates: expenses/domains @ 2026-03-01
    documents/expenses/domains/2026-03-01-b.pdf

Document Attribution:
  expenses/domains @ 2026-03-01
    documents/expenses/domains/2026-03-01-a.pdf top-level -> transaction #1: Domain A
    documents/expenses/domains/2026-03-01-b.pdf top-level -> transaction #2: Domain B

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
    2 amount mismatches
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

Descriptions can also identify a transaction when its document amount is read
from the PDF rather than declared in metadata. The individual audit still
catches the two offsetting errors.

```scrut
$ write_file documents/expenses/domains/2026-03-01-a.document.yml <<'EOF'
> description: Domain A
> EOF
```

```scrut
$ write_file documents/expenses/domains/2026-03-01-b.document.yml <<'EOF'
> description: Domain B
> EOF
```

```scrut
$ write_pdf documents/expenses/domains/2026-03-01-a.pdf <<'EOF'
> Total EUR 4.00
> EOF
```

```scrut
$ write_pdf documents/expenses/domains/2026-03-01-b.pdf <<'EOF'
> Total EUR 6.00
> EOF
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore duplicate-files --ignore redundant-metadata

Amount Mismatches (2):
  top-level amount 4.00 != transaction #1 (Domain A) 5.00
    accounts/dates: expenses/domains @ 2026-03-01
    documents/expenses/domains/2026-03-01-a.pdf
  top-level amount 6.00 != transaction #2 (Domain B) 5.00
    accounts/dates: expenses/domains @ 2026-03-01
    documents/expenses/domains/2026-03-01-b.pdf

Document Attribution:
  expenses/domains @ 2026-03-01
    documents/expenses/domains/2026-03-01-a.pdf top-level -> transaction #1: Domain A
    documents/expenses/domains/2026-03-01-b.pdf top-level -> transaction #2: Domain B

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
    2 amount mismatches
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
