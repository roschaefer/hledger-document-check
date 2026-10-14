# Full Example

The repository contains a complete example project under `example/`. It combines
the documented account-tag contract, document directory layout, sidecar metadata,
unbooked invoices, due dates, amount-audit warnings, project configuration, and
Fava input generation.

The document-check behavior of this example is exercised by the
[full-example Scrut transcript](../tests/specifications/full-example/).

The Fava example shows how to use `enrich-journal` as the bridge from hledger to
Fava. The command creates a sidecar-free document tree under
`example/.generated/`, runs `enrich-journal` to add `document:` tags for matched
documents, converts the enriched journal to Beancount with hledger, and starts
Fava with that generated Beancount file. In Fava you can browse transactions
together with their linked PDFs.

Because Fava reads Beancount, account names used with this workflow must follow
Beancount naming conventions. The bundled example uses capitalized account
components such as `Expenses:Business:Hosting`.

```gherkin
Feature: Full repository Fava example
  The checked-out repository includes a runnable Fava integration example.

  Scenario: The example Fava command shows linked documents
    Given the development dependencies are installed
    And the optional Fava dependency is installed
    When I clone the repository
    And I run `example/rebuild-fava-inputs.sh` from the root directory
    And I run a shell command from the root directory:
      """
      hledger-document-check enrich-journal --journal example/journal.journal --documents example/documents --document-tag-root "$(pwd)/example/.generated/fava/documents" | hledger -f - print -O beancount >> example/.generated/fava/example.beancount
      """
    And I start `fava example/.generated/fava/example.beancount` from the root directory
    Then Fava is running at "http://localhost:5000/beancount/documents/"
    And the file "hledger-document-check/example/.generated/fava/example.beancount" contains:
      """text
      option "documents" "{checkout_dir}/example/.generated/fava/documents"
      ...
      ; document:{checkout_dir}/example/.generated/fava/documents/Expenses/Business/Hosting/2026-01-02-hosting.pdf
      """
    And the file "hledger-document-check/example/.generated/fava/documents/Expenses/Business/Cloud/2026-01-04-cloud.pdf" exists
    And the file "hledger-document-check/example/.generated/fava/documents/Expenses/Business/Cloud/2026-01-04-cloud.document.yml" does not exist
```
