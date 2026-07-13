# Directory and File Layout

The document tree should mirror ledger accounts. The account
`expenses:business:hosting` maps to `documents/expenses/business/hosting/`.
Matched document files start with the posting date, for example
`2026-01-02-hosting.pdf`.

## Account folders and date-prefixed files

This convention makes the default matching rule simple: account folder plus date
prefix covers the required posting group for the same account and posting date.

```gherkin
Feature: Directory and file layout
  Account folders and date-prefixed files provide the default document match.

  Scenario: A dated file in the account folder covers the required posting
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                    ; document_check:exempt
      account expenses:business:hosting      ; document_check:required

      2026-01-02 Hosting Provider
          expenses:business:hosting        10.00 EUR
          assets:bank                     -10.00 EUR
      """
    And a file named "documents/expenses/business/hosting/2026-01-02-hosting.pdf" with content:
      """
      hosting receipt
      """
    When I run "hledger document-check check --journal journal.journal --documents documents --ignore amount-audit-skips --ignore duplicate-files"
    Then the command exits with code 0
    And stdout contains:
      """text
      1/1 transaction groups covered
      ...
      OK.
      """
```

## Missing-document placeholders

If a document is intentionally unavailable, use an empty dated missing-document
placeholder named `YYYY-MM-DD-missing.document.yml` in the same account folder.
The tool derives the account from the folder, the date from the filename, and the
fact that this is a missing-document marker from the `missing` filename part. The
placeholder is counted separately in the summary so it remains visible during
reviews. Do not duplicate this with a `missing` key inside the YAML sidecar; the
filename is the only missing-document marker.

The placeholder does not need to stay empty. Add a `notes` key to record why the
document is missing — the same field documented for regular document metadata in
[03-document-metadata.md](03-document-metadata.md#metadata-schema-validation).

```gherkin
Feature: Missing-document placeholders
  A dated missing-document placeholder covers a required posting when the document itself is unavailable.

  Scenario: A missing-document placeholder documents the reason with notes
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                    ; document_check:exempt
      account expenses:business:hosting      ; document_check:required
      account expenses:business:banking      ; document_check:required

      2026-01-02 Hosting Provider
          expenses:business:hosting        10.00 EUR
          assets:bank                     -10.00 EUR

      2026-01-06 Bank Fee
          expenses:business:banking         3.00 EUR
          assets:bank                      -3.00 EUR
      """
    And files named:
      | path                                                                | content         |
      | documents/expenses/business/hosting/2026-01-02-hosting.pdf          | hosting receipt |
    And a file named "documents/expenses/business/banking/2026-01-06-missing.document.yml" with content:
      """yaml
      notes: "Bank statement never arrived; requested a reissue from the bank on 2026-01-08."
      """
    When I run "hledger document-check check --journal journal.journal --documents documents --ignore amount-audit-skips --ignore duplicate-files"
    Then the command exits with code 0
    And stdout contains:
      """text
      2/2 transaction groups covered
      ...
      1 missing-document placeholders
      ...
      OK.
      """
```

## Supporting files

Supporting files for a document live in a sibling directory named after the PDF
stem. For `2026-01-02-hosting.pdf`, supporting material such as usage exports or
calculation notes goes into `2026-01-02-hosting/`. Files in that support
directory are not treated as separate document inventory.

```gherkin
Feature: Supporting files
  Supporting files are stored below the matched document stem.

  Scenario: Supporting files live below a sibling directory named after the document
    Given a file named "journal.journal" with content:
      """hledger
      account assets:bank                    ; document_check:exempt
      account expenses:business:hosting      ; document_check:required

      2026-01-02 Hosting Provider
          expenses:business:hosting        10.00 EUR
          assets:bank                     -10.00 EUR
      """
    And files named:
      | path                                                                       | content                    |
      | documents/expenses/business/hosting/2026-01-02-hosting.pdf                 | hosting receipt            |
      | documents/expenses/business/hosting/2026-01-02-hosting/utilization.csv     | project,hours\nclient-a,4  |
    When I run "hledger document-check check --journal journal.journal --documents documents --ignore amount-audit-skips --ignore duplicate-files"
    Then the command exits with code 0
    And stdout contains:
      """text
      1/1 transaction groups covered
      ...
      0 unexpected files
      ...
      OK.
      """
```
