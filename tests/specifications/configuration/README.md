# Configuration

<!--
```scrut {fail_fast: true, output_stream: combined}
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

The default config file documents all supported project defaults.

### The default config file can be generated

```scrut
$ build_scenario scenario_the_default_config_file_can_be_generated
```

```scrut
$ hledger-document-check init-config --output hledger-document-check.toml | sed "s|$PWD|{working_directory}|g"
```

```scrut
$ cat hledger-document-check.toml
[ledger]
# journal = "../ledger/hledger.journal"

[documents]
# root = "."

[requirements]
tag_prefixes = []
# tag_prefixes = ["tax_"]

[overdue]
after_days = 14

[checks]
invalid-configuration = "fail"
missing-document-coverage = "fail"
unbooked-documents = "warn"
overdue-unbooked-documents = "fail"
unmatched-documents = "fail"
unexpected-files = "fail"
duplicate-files = "fail"
amount-mismatches = "fail"
amount-audit-skips = "ignore"
missing-document-placeholders = "ignore"
ambiguous-transaction-groups = "warn"
redundant-metadata = "warn"
unresolvable-cover-metadata = "fail"

[enrich_journal]
# Prefix written into emitted document: tags.
# Use an absolute path when the consuming tool has no separate document-root setting.
# document_tag_root = "/absolute/path/to/documents"
```

### The README command generates the initial config file

```scrut
$ build_scenario scenario_the_readme_command_generates_the_initial_config_file
```

```scrut
$ hledger-document-check init-config --output documents/hledger-document-check.toml | sed "s|$PWD|{working_directory}|g"
```

```scrut
$ cat documents/hledger-document-check.toml
[ledger]
# journal = "../ledger/hledger.journal"

[documents]
# root = "."

[requirements]
tag_prefixes = []
# tag_prefixes = ["tax_"]

[overdue]
after_days = 14

[checks]
invalid-configuration = "fail"
missing-document-coverage = "fail"
unbooked-documents = "warn"
overdue-unbooked-documents = "fail"
unmatched-documents = "fail"
unexpected-files = "fail"
duplicate-files = "fail"
amount-mismatches = "fail"
amount-audit-skips = "ignore"
missing-document-placeholders = "ignore"
ambiguous-transaction-groups = "warn"
redundant-metadata = "warn"
unresolvable-cover-metadata = "fail"

[enrich_journal]
# Prefix written into emitted document: tags.
# Use an absolute path when the consuming tool has no separate document-root setting.
# document_tag_root = "/absolute/path/to/documents"
```

A document-local TOML file can define stable project defaults.

### The check command reads defaults from the document root

```scrut
$ build_scenario scenario_the_check_command_reads_defaults_from_the_document_root
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
