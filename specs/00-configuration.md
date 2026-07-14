# Configuration

`hledger-document-check` can read `hledger-document-check.toml` from the
document root. Paths inside the config file are resolved relative to the config
file location, so a document-local config can point back to the journal with a
relative path.

The config is optional and intended for stable project defaults: journal path,
tag-prefix requirements, overdue thresholds, and check policy. Command-line flags
remain available for one-off overrides.

```gherkin
Feature: Configuration file creation
  The default config file documents all supported project defaults.

  Scenario: The default config file can be generated
    When I run "hledger-document-check init-config --output hledger-document-check.toml"
    Then the file "hledger-document-check.toml" contains exactly:
      """toml
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
      unresolvable-cover-metadata = "warn"

      [enrich_journal]
      # Prefix written into emitted document: tags.
      # Use an absolute path when the consuming tool has no separate document-root setting.
      # document_tag_root = "/absolute/path/to/documents"
      """

  Scenario: The README command generates the initial config file
    Given an empty directory named "documents"
    When I run "hledger-document-check init-config --output documents/hledger-document-check.toml"
    Then the exit code is 0
    And the file "documents/hledger-document-check.toml" contains exactly:
      """toml
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
      unresolvable-cover-metadata = "warn"

      [enrich_journal]
      # Prefix written into emitted document: tags.
      # Use an absolute path when the consuming tool has no separate document-root setting.
      # document_tag_root = "/absolute/path/to/documents"
      """
```

```gherkin
Feature: Configuration file behavior
  A document-local TOML file can define stable project defaults.

  Scenario: The check command reads defaults from the document root
    Given a file named "journal.journal" with content:
      """hledger
      2026-01-02 Hosting Provider
          expenses:business:hosting        10.00 EUR  ; tax_role:business_expense
          assets:bank                     -10.00 EUR
      """
    And a file named "documents/hledger-document-check.toml" with content:
      """toml
      [ledger]
      journal = "../journal.journal"

      [requirements]
      tag_prefixes = ["tax_"]

      [checks]
      missing-document-coverage = "warn"
      """
    When I run "hledger document-check check --documents documents"
    Then the command exits with code 0
    And stdout contains:
      """text
      Missing Document Coverage (1):
      ...
      2026-01-02  Hosting Provider
      """
```
