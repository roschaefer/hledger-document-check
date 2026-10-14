# Executable specifications

Each specification chapter has its own directory with one Markdown document
per independently explained behavior. `README.md` is that document when the
feature has only one, so GitHub renders it when the directory is opened.

Every scenario shows its inputs next to the output they produce. Documents
source [`readme-setup.sh`](readme-setup.sh) in a hidden Scrut block, which
provides these helpers:

- `new_scenario` starts a scenario in an empty directory;
- `write_file PATH` writes a journal, sidecar, or other file from a heredoc;
- `write_pdf PATH` writes a one-page PDF showing the heredoc's text.

Scrut gives every document a fresh temporary working directory and checks the
exit code of every block, so a failing setup step fails its scenario.

Read the chapters in this order:

1. [Contract](contract/)
2. [Configuration](configuration/)
3. [Directory and file layout](directory-and-file-layout/)
4. [Special cases with `.document.yml`](document-metadata/)
5. [Amount audit](amount-audit/)
6. [Unbooked invoices](unbooked-invoices/)
7. [Due date](due-date/)
8. [Enrich journal](enrich-journal/)
9. [Full example](../../README.md#run-the-bundled-example)
