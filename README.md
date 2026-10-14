# hledger-document-check

<!--
```scrut {fail_fast: true, output_stream: combined}
$ source "$TESTDIR/tests/specifications/readme-setup.sh" && new_scenario && checkout_example
```
-->

`hledger-document-check` checks hledger transactions against documents on disk.

## Audience

This tool is for anyone who wants to do a document check or voucher audit against
an hledger journal. That can be useful for tax declarations, where relevant
transactions should be backed by filed documents.

The tool supports documents for invoices received and invoices sent to
customers. It can also check whether invoice/document amounts plausibly match the
connected ledger transactions.

## Installation

Required tools:

- [`hledger`](https://hledger.org/) 1.52.1

Optional:

- [`fava`](https://beancount.github.io/fava/) for the example Fava workflow

Download the standalone executable for your platform from the
[latest release](https://github.com/roschaefer/hledger-document-check/releases/tag/latest)
and put it on `PATH`:

```bash
curl -L \
  -o /tmp/hledger-document-check \
  https://github.com/roschaefer/hledger-document-check/releases/download/latest/hledger-document-check-linux-x86_64
install -m 0755 /tmp/hledger-document-check ~/.local/bin/hledger-document-check
```

For macOS on Apple Silicon, use the `hledger-document-check-macos-arm64` asset
instead.

## Usage

### Use it with your own journal and documents

Run the document check against your document folder. If your document folder has
a `hledger-document-check.toml` with a configured journal path, `--documents` is
enough:

```bash
hledger-document-check check --documents path/to/documents
```

Otherwise, pass the journal explicitly:

```bash
hledger-document-check check --journal path/to/hledger.journal --documents path/to/documents
```

To emit a derived journal with `document:` tags, run:

```bash
hledger-document-check enrich-journal --journal path/to/hledger.journal --documents path/to/documents
```

### Run the bundled example

The repository includes a complete example project under `example/`. It
combines the account-tag contract, directory layout, metadata, unbooked
invoices, due dates, amount auditing, and configuration. Run the document check
against it from the repository root:

```scrut
$ hledger-document-check check --documents example/documents --today 2026-05-20

Missing Document Coverage (1):
  2026-04-10  AWS Invoice
    account: Expenses:Business:Hosting:Aws
    amount: 12.34 EUR

Unbooked Documents (2):
  example/documents/Expenses/Business/Hosting/Aws/unbooked/invoice.pdf
    amount: 12.34 EUR
  example/documents/Income/Business/Freelance/Customer/unbooked/invoice.pdf
    amount: unknown

Suggested Moves (1):
  High-confidence unbooked document matches by account, amount, and currency.
  2026-04-10  AWS Invoice
    account: Expenses:Business:Hosting:Aws
    amount: 12.34 EUR
    mv */example/documents/Expenses/Business/Hosting/Aws/unbooked/invoice.pdf */example/documents/Expenses/Business/Hosting/Aws/2026-04-10-invoice.pdf (glob)
    mv */example/documents/Expenses/Business/Hosting/Aws/unbooked/invoice.document.yml */example/documents/Expenses/Business/Hosting/Aws/2026-04-10-invoice.document.yml (glob)
    mv */example/documents/Expenses/Business/Hosting/Aws/unbooked/invoice */example/documents/Expenses/Business/Hosting/Aws/2026-04-10-invoice (glob)

Amount Audit Skips (8):
  document/transaction currency mismatch
    example/documents/Expenses/Business/Currency-mismatch/2026-05-06-usd-invoice.pdf
      accounts/dates: Expenses/Business/Currency-mismatch @ 2026-05-06
  document amount unreadable
    example/documents/Expenses/Business/Unreadable-document/2026-05-02-unreadable.pdf
      accounts/dates: Expenses/Business/Unreadable-document @ 2026-05-02
    example/documents/Expenses/Tax-demo/2026-01-06-tax-demo.pdf
      accounts/dates: Expenses/Tax-demo @ 2026-01-06
    example/documents/Liabilities/Insurance-notices/2026-01-10-health-notice.pdf
      accounts/dates: Liabilities/Insurance-notices @ 2026-01-10
    example/documents/Liabilities/Insurance-notices/2026-02-10-pension-notice.pdf
      accounts/dates: Liabilities/Insurance-notices @ 2026-02-10
  document currencies unclear
    example/documents/Expenses/Business/Unclear-document-currency/2026-05-03-unclear.pdf
      accounts/dates: Expenses/Business/Unclear-document-currency @ 2026-05-03
  transaction amount missing
    example/documents/Expenses/Business/Missing-transaction-amount/2026-05-04-missing-amount.pdf
      accounts/dates: Expenses/Business/Missing-transaction-amount @ 2026-05-04
  transaction currencies mixed
    example/documents/Expenses/Business/Mixed-transaction-currencies/2026-05-05-mixed.pdf
      accounts/dates: Expenses/Business/Mixed-transaction-currencies @ 2026-05-05

Ambiguous Transaction Groups (1):
  Date-prefixed filenames can only match by account and date.
  These groups have multiple required transactions on the same date in the same account:
  Expenses/Business/Mixed-transaction-currencies @ 2026-05-05  [2 transactions]
    2026-05-05  10.00 EUR  Mixed Transaction Currencies
    2026-05-05  12.00 USD  Mixed Transaction Currencies

Summary:
  Coverage:
    19/20 transaction groups covered
    1 missing-document placeholders
  Open Items:
    1 missing document coverage
    2 unbooked documents
    0 overdue unbooked documents
    0 unmatched documents
    0 unexpected files
    0 duplicate groups
    0 redundant metadata fields
    0 unresolvable cover metadata fields
  Amount Audit:
    0 amount mismatches
    8 linked groups checked
    8 linked groups skipped
    Skip Reasons:
      4 document amount unreadable
      1 document currencies unclear
      1 transaction amount missing
      1 transaction currencies mixed
      1 document/transaction currency mismatch
OK.
```

In the `mv` suggestions, `*` stands for the path of your checkout; the
`(glob)` marker tells [Scrut](#specification-by-example) to match it.

To generate a Beancount file for Fava (requires `hledger` and `fava` on
`PATH`), build a sidecar-free copy of the document tree and append the enriched
journal:

```scrut
$ example/rebuild-fava-inputs.sh
```

```scrut
$ hledger-document-check enrich-journal \
>   --journal example/journal.journal \
>   --documents example/documents \
>   --document-tag-root "$(pwd)/example/.generated/fava/documents" \
>   | hledger -f - print -O beancount >> example/.generated/fava/example.beancount
```

Only the documents themselves are copied; their document-check metadata is not
part of the tree served by Fava:

```scrut
$ ls example/.generated/fava/documents/Expenses/Business/Cloud
2026-01-04-cloud.pdf
```

Then open it in Fava:

```bash
fava example/.generated/fava/example.beancount
```

Fava lists every linked document. `fava_linked_documents` is a test helper
from [`readme-setup.sh`](tests/specifications/readme-setup.sh): it starts Fava
on a free port and asks its API for the documents:

```scrut
$ fava_linked_documents example/.generated/fava/example.beancount | sed "s|$PWD/||"
example/.generated/fava/documents/Expenses/Business/Cloud/2026-01-04-cloud.pdf
example/.generated/fava/documents/Expenses/Business/Currency-mismatch/2026-05-06-usd-invoice.pdf
example/.generated/fava/documents/Expenses/Business/Domains/2026-03-01-domain-a.pdf
example/.generated/fava/documents/Expenses/Business/Domains/2026-03-01-domain-b.pdf
example/.generated/fava/documents/Expenses/Business/Hosting/2026-01-02-hosting.pdf
example/.generated/fava/documents/Expenses/Business/Missing-transaction-amount/2026-05-04-missing-amount.pdf
example/.generated/fava/documents/Expenses/Business/Mixed-transaction-currencies/2026-05-05-mixed.pdf
example/.generated/fava/documents/Expenses/Business/Software/2026-04-01-suite.pdf
example/.generated/fava/documents/Expenses/Business/Unclear-document-currency/2026-05-03-unclear.pdf
example/.generated/fava/documents/Expenses/Business/Unreadable-document/2026-05-02-unreadable.pdf
example/.generated/fava/documents/Expenses/Tax-demo/2026-01-06-tax-demo.pdf
example/.generated/fava/documents/Income/Business/Freelance/Customer/2026-03-10-invoice-a.pdf
example/.generated/fava/documents/Income/Business/Freelance/Customer/2026-03-10-invoice-b.pdf
example/.generated/fava/documents/Income/Business/Freelance/Customer/2026-04-20-invoice.pdf
example/.generated/fava/documents/Liabilities/Insurance-notices/2026-01-10-health-notice.pdf
example/.generated/fava/documents/Liabilities/Insurance-notices/2026-02-10-pension-notice.pdf
```

The Fava example uses `enrich-journal` to add `document:` tags, then converts
the journal to Beancount so Fava can browse transactions together with their
linked PDFs.

![Fava Documents view showing linked example PDFs](docs/fava-documents.png)

Because Fava reads Beancount, account names used with this workflow must follow
Beancount naming conventions. The bundled example uses capitalized account
components such as `Expenses:Business:Hosting`.

## Specification By Example

Executable [Scrut](https://github.com/facebookincubator/scrut) specifications
live in [`tests/specifications/`](./tests/specifications/); the `scrut` blocks in
this README are executed too. Run them all with `just test-scrut`.

Specs:

- [Contract](./tests/specifications/contract/)
- [Configuration](./tests/specifications/configuration/)
- [Directory and file layout](./tests/specifications/directory-and-file-layout/)
- [Special cases with `.document.yml`](./tests/specifications/document-metadata/)
- [Amount audit](./tests/specifications/amount-audit/)
- [Unbooked invoices](./tests/specifications/unbooked-invoices/)
- [Due date](./tests/specifications/due-date/)
- [Enrich journal](./tests/specifications/enrich-journal/)

## Development

Enter the reproducible development shell, which includes Rust, `hledger`,
Just, Scrut, and the linting tools:

```bash
nix develop
```

Alternatively, install a [Rust toolchain](https://rustup.rs/),
[`hledger`](https://hledger.org/),
[Just](https://github.com/casey/just), and
[Scrut](https://github.com/facebookincubator/scrut) yourself.

Clone the repository and build:

```bash
git clone git@github.com:roschaefer/hledger-document-check.git
cd hledger-document-check
cargo build --release
# binary is at target/release/hledger-document-check
```

Run the Rust tests:

```bash
just test
```

## Core Concepts

- Account folders mirror ledger accounts, for example
  `Expenses:Business:Hosting` maps to `documents/Expenses/Business/Hosting/`.
- Matched document files start with `YYYY-MM-DD-`.
- Unbooked documents live in an account-local `unbooked/` subfolder.
- Supporting files live in a sibling directory named after the document stem.
- `*.document.yml` sidecars describe non-trivial relationships, due dates, and
  amount metadata.
- `document_check:` account and posting tags decide which transactions require
  document coverage.
- `hledger-document-check enrich-journal` emits a derived journal with `document:` tags for downstream
  tools such as Fava.

## Check Policy Reference

Checks can be configured as `fail`, `warn`, or `ignore` through CLI flags or the
`[checks]` table in `hledger-document-check.toml`.

Built-in check names:

- `invalid-configuration`
- `missing-document-coverage`
- `unbooked-documents`
- `overdue-unbooked-documents`
- `unmatched-documents`
- `unexpected-files`
- `duplicate-files`
- `amount-mismatches`
- `amount-audit-skips`
- `missing-document-placeholders`
- `ambiguous-transaction-groups`
- `redundant-metadata`
- `unresolvable-cover-metadata`

## Configuration

Project defaults can live in `hledger-document-check.toml` inside the document
root. Generate the initial file with:

```scrut
$ hledger-document-check init-config --output documents/hledger-document-check.toml
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

With that file in place, the normal command can be short:

```bash
hledger-document-check check --documents path/to/documents
```
