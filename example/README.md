# hledger-document-check Example

Run this example from the repository root:

```bash
just example-check-documents
```

To generate a Beancount file and open it in Fava, with `fava` available on
`PATH`:

```bash
just example-fava
```

This command writes a sidecar-free document copy and a Beancount file under
`example/.generated/fava/` on every run. Fava uses Beancount account naming
rules, so the example accounts and document folders use capitalized account
components such as `Expenses:Business:Hosting`.

The example includes account tags, document folders, sidecar metadata,
unbooked documents, due dates, amount-audit warnings, missing-document
placeholders, and Fava input generation.
