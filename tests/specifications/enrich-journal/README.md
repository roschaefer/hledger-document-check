# Enrich Journal

<!--
```scrut {fail_fast: true, output_stream: combined}
$ source "$TESTDIR/../readme-setup.sh"
```
-->

The real journal should stay the source of accounting truth. `enrich-journal`
emits a derived journal stream that adds `document:` tags for matched PDF files,
so downstream tools can open the corresponding documents.

This only works for the default account-folder layout and dated matched
documents. Missing-document placeholders are not emitted because they are not
viewable documents. Existing `document:` tags that point at `*.document.yml`
sidecar metadata are filtered from the enriched output for the same reason.

## Fava

For Fava through Beancount, configure the document root in the generated
Beancount file:

```beancount
option "documents" "/absolute/path/to/documents"
```

With that option in place, the default `document:` tag value emitted by
`enrich-journal` is just the matched filename, for example
`document:2026-01-02-hosting.pdf`.

Point Beancount at the document tree and pass the same root as
`--document-tag-root` so the generated document tags resolve below Fava's
configured document root.

A minimal project-local `Justfile` can look like this:

```just
fava *args:
    rm -rf .generated/fava
    mkdir -p .generated/fava

    echo 'option "operating_currency" "EUR"' > .generated/fava/main.beancount
    echo 'option "documents" "'"$(pwd)"'/documents"' >> .generated/fava/main.beancount
    hledger-document-check enrich-journal \
      --journal ./ledger/hledger.journal \
      --documents ./documents \
      --document-tag-root "$(pwd)/documents" \
      | hledger -f - print -O beancount >> .generated/fava/main.beancount
    fava .generated/fava/main.beancount {{args}}
```

A derived journal can expose matched documents to Fava.

### Enriched journal emits document tags for matched documents

```scrut
$ build_scenario scenario_enriched_journal_emits_document_tags_for_matched_documents
```

```scrut
$ hledger-document-check enrich-journal --journal journal.journal --documents documents | sed "s|$PWD|{working_directory}|g"
2026-01-02 Hosting Provider
    ; document:2026-01-02-existing.pdf
    ; document:2026-01-02-hosting.pdf
    expenses:business:hosting       10.00 EUR
    assets:bank                    -10.00 EUR

```

## Absolute Document Paths

Some consumers do not have a separate document-root option like Beancount's
`option "documents"`. For those cases, use `--document-tag-root` to put the path
prefix directly into each `document:` tag.

Pass an absolute document root when the enriched journal should be self-contained
from the consumer's point of view:

```sh
hledger-document-check enrich-journal \
  --journal ./ledger/hledger.journal \
  --documents ./documents \
  --document-tag-root "$(realpath ./documents)"
```

Enriched journal output can include absolute document paths for non-Fava consumers.

### Enriched journal emits document tags with an absolute root

```scrut
$ build_scenario scenario_enriched_journal_emits_document_tags_with_an_absolute_root
```

```scrut
$ hledger-document-check enrich-journal --journal journal.journal --documents documents --document-tag-root /archive/documents | sed "s|$PWD|{working_directory}|g"
2026-01-02 Hosting Provider
    ; document:/archive/documents/expenses/business/hosting/2026-01-02-hosting.pdf
    expenses:business:hosting       10.00 EUR
    assets:bank                    -10.00 EUR

```
