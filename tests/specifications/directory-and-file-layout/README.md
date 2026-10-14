# Directory and File Layout

<!--
```scrut {fail_fast: true, output_stream: combined}
$ source "$TESTDIR/../readme-setup.sh"
```
-->

The document tree should mirror ledger accounts. The account
`expenses:business:hosting` maps to `documents/expenses/business/hosting/`.
Matched document files start with the posting date, for example
`2026-01-02-hosting.pdf`.

## Account folders and date-prefixed files

This convention makes the default matching rule simple: account folder plus date
prefix covers the required posting group for the same account and posting date.

Account folders and date-prefixed files provide the default document match.

### A dated file in the account folder covers the required posting

```scrut
$ build_scenario scenario_a_dated_file_in_the_account_folder_covers_the_required_posting
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

A dated missing-document placeholder covers a required posting when the document itself is unavailable.

### A missing-document placeholder documents the reason with notes

```scrut
$ build_scenario scenario_a_missing_document_placeholder_documents_the_reason_with_notes
```

```scrut
$ hledger-document-check check --journal journal.journal --documents documents --ignore amount-audit-skips --ignore duplicate-files | sed "s|$PWD|{working_directory}|g"

Summary:
  Coverage:
    2/2 transaction groups covered
    1 missing-document placeholders
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

## Supporting files

Supporting files for a document live in a sibling directory named after the PDF
stem. For `2026-01-02-hosting.pdf`, supporting material such as usage exports or
calculation notes goes into `2026-01-02-hosting/`. Files in that support
directory are not treated as separate document inventory.

Supporting files are stored below the matched document stem.

### Supporting files live below a sibling directory named after the document

```scrut
$ build_scenario scenario_supporting_files_live_below_a_sibling_directory_named_after_the_document
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
