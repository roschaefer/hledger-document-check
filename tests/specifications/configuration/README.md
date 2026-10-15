# Configuration

<!--
```scrut {fail_fast: true}
$ source "$TESTDIR/../readme-setup.sh"
```
-->

`hledger-document-check` can read `hledger-document-check.toml` from the
document root. Paths inside the config file are resolved relative to the config
file location, so a document-local config can point back to the journal with a
relative path.

The config is optional and intended for stable project defaults: journal path,
tag-prefix requirements, overdue thresholds, and check policy. Command-line flags
remain available for one-off overrides.

The [README](../../../README.md#configuration) shows how `init-config`
generates the default config file, which documents all supported project
defaults.

A document-local TOML file can define stable project defaults.

### The check command reads defaults from the document root

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
$ write_file documents/hledger-document-check.toml <<'EOF'
> [ledger]
> journal = "../journal.journal"
>
> [requirements]
> tag_prefixes = ["tax_"]
>
> [checks]
> missing-document-coverage = "warn"
> EOF
```

```scrut
$ hledger-document-check check --documents documents | sed "s|$PWD|{working_directory}|g"

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
OK.
```
