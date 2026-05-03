# Executable Specifications

These Markdown files are the source of truth for document-check acceptance tests.
Fenced `gherkin` blocks are extracted into cucumber features at build time by
`build.rs`, then run by `tests/cucumber.rs`.

Read the specs in this order:

1. [Contract](./01-contract.md)
2. [Configuration](./00-configuration.md)
3. [Directory and file layout](./02-directory-and-file-layout.md)
4. [Special cases with `.document.yml`](./03-document-metadata.md)
5. [Amount audit](./04-amount-audit.md)
6. [Unbooked invoices](./05-unbooked-invoices.md)
7. [Due date](./06-due-date.md)
8. [Enrich journal](./07-enrich-journal.md)
9. [Full example](./08-full-example.md)

Use:

```sh
cargo test --test cucumber
```
