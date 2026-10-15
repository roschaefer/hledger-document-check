# Contract

<!--
```scrut {fail_fast: true}
$ source "$TESTDIR/../readme-setup.sh"
```
-->

`hledger-document-check` decides document requirements from hledger account and posting
tags. The normal setup is to keep reusable defaults in `accounts.journal` and
include that file from the actual journal.

## Required and exempt accounts

- `document_check:required`: every posting to this account needs document coverage.
- `document_check:exempt`: postings to this account never need document coverage.

hledger tags declared on an `account` directive are inherited by its subaccounts, and a
tag on a more specific account overrides the one inherited from its parent. `exempt` can
use this to carve out an exception: tag a parent account `required` and then tag one of
its subaccounts `exempt` to excuse just that subaccount from coverage.

Required and exempt tagged accounts define which postings need documents.

### A subaccount can be exempted from a required parent account

```scrut
$ new_scenario
```

```scrut
$ write_file accounts.journal <<'EOF'
> account assets:bank                       ; document_check:exempt
> account expenses:business                 ; document_check:required
> account expenses:business:bank-fees       ; document_check:exempt
> EOF
```

```scrut
$ write_file journal.journal <<'EOF'
> include accounts.journal
>
> 2026-01-02 Hosting Provider
>     expenses:business:hosting        10.00 EUR
>     assets:bank                     -10.00 EUR
>
> 2026-01-05 Bank Fee
>     expenses:business:bank-fees       2.50 EUR
>     assets:bank                      -2.50 EUR
> EOF
```

```scrut
$ write_file documents/expenses/business/hosting/2026-01-02-hosting.pdf <<'EOF'
> hosting receipt
> EOF
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore amount-audit-skips --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

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
OK.
```

The `expenses:business:hosting` posting inherits `document_check:required` from
`expenses:business` and needs a document. `expenses:business:bank-fees` inherits the
same parent tag but overrides it with its own `document_check:exempt`, so bank fees
never need coverage even though the rest of the business expenses do.

## Counter-account tagged accounts

- `document_check:counter_account`: the posting needs coverage only when another
  posting in the same transaction matches one of the configured counter-account
  patterns.

Counter-account matching is useful when the document belongs to an intermediate
account, not to the payment transaction. For example, a health-insurance notice
can credit a liability account while the actual bank payment later clears that
liability. The notice should be filed once, against the notice transaction, and
the later bank payment should not require a second document.

If one intermediate account can receive notices from several counter accounts,
keep the unnamed `document_check_counter_account` tag for the first pattern or
use named variants for all of them:

```hledger
account liabilities:insurance-notices  ; document_check:counter_account
                                       ; document_check_counter_account_a:expenses:insurance:health:provider
                                       ; document_check_counter_account_b:expenses:insurance:pension:provider
```

The suffix can be any non-empty name. The examples use `_a` and `_b` because they
sort well and stay readable.

A counter-account tagged account only needs documents when a transaction touches one
of its configured counter-account patterns.

### A notice needs a document but the payment that later clears it does not

```scrut
$ new_scenario
```

```scrut
$ write_file accounts.journal <<'EOF'
> account assets:bank                              ; document_check:exempt
> account expenses:insurance:health:provider      ; document_check:exempt
> account expenses:insurance:pension:provider     ; document_check:exempt
> account liabilities:insurance-notices           ; document_check:counter_account, document_check_counter_account_a:expenses:insurance:health:provider, document_check_counter_account_b:expenses:insurance:pension:provider
> EOF
```

```scrut
$ write_file journal.journal <<'EOF'
> include accounts.journal
>
> 2026-01-10 Health Insurance Notice
>     expenses:insurance:health:provider   220.00 EUR
>     liabilities:insurance-notices       -220.00 EUR
>
> 2026-01-15 Health Insurance Payment
>     liabilities:insurance-notices        220.00 EUR
>     assets:bank                         -220.00 EUR
>
> 2026-02-10 Pension Insurance Notice
>     expenses:insurance:pension:provider  80.00 EUR
>     liabilities:insurance-notices       -80.00 EUR
> EOF
```

```scrut
$ write_file documents/liabilities/insurance-notices/2026-01-10-health-notice.pdf <<'EOF'
> health insurance notice
> EOF
```

```scrut
$ write_file documents/liabilities/insurance-notices/2026-02-10-pension-notice.pdf <<'EOF'
> pension insurance notice
> EOF
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore amount-audit-skips --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Summary:
  Coverage:
    2/2 transaction groups covered
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
    2 linked groups skipped
    Skip Reasons:
      2 document amount unreadable
      0 document currencies unclear
      0 transaction amount missing
      0 transaction currencies mixed
      0 document/transaction currency mismatch
OK.
```

The two notice transactions match `document_check_counter_account_a` and `_b`
respectively, so each needs its own document. The payment transaction posts to
`liabilities:insurance-notices` too, but its other posting is `assets:bank`, which
matches neither counter-account pattern, so the payment is not part of the
transaction groups requiring coverage at all.

The counter_account tag requires a matching counter-account pattern tag.

### Using counter_account without a counter account tag is an error

```scrut
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank                              ; document_check:exempt
> account expenses:insurance:health:provider       ; document_check:exempt
> account liabilities:health-insurance             ; document_check:counter_account
>
> 2026-01-10 Health Insurance Notice
>     expenses:insurance:health:provider   220.00 EUR
>     liabilities:health-insurance        -220.00 EUR
> EOF
```

```scrut
$ mkdir -p documents
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents | sed "s|$PWD|{working_directory}|g"
ERROR: invalid document_check configuration:

  liabilities:health-insurance: document_check:counter_account requires document_check_counter_account

Ensure document_check and any required document_check_counter_account tags are configured correctly.
[2]
```

## Tag prefix document requirements

For imported or domain-specific tag namespaces, a command-line prefix can also
make a posting document-required. This keeps `document_check` explicit by
default, while still allowing another tagging contract to opt in. For example,
a tax tool might tag postings with `tax_role:*`; passing that prefix makes
every tax-relevant transaction subject to document coverage without having to
tag each account with `document_check:required` by hand.

A configured tag prefix can imply document_check:required.

### A prefixed tag can require documents

```scrut
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> 2026-01-02 Hosting Provider
>     expenses:business:hosting        10.00 EUR  ; tax_role:business_expense
>     assets:bank                     -10.00 EUR
> EOF
```

```scrut
$ mkdir -p documents
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --require-document-for-tag-prefix tax_ | sed "s|$PWD|{working_directory}|g"

Missing Document Coverage (1):
  2026-01-02  Hosting Provider
    account: expenses:business:hosting
    amount: 10.00 EUR

Summary:
  Coverage:
    0/1 transaction groups covered
    0 missing-document placeholders
  Open Items:
    1 missing document coverage
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
    0 linked groups skipped
    Skip Reasons:
      0 document amount unreadable
      0 document currencies unclear
      0 transaction amount missing
      0 transaction currencies mixed
      0 document/transaction currency mismatch
[1]
```
