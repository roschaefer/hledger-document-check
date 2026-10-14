# Executable specifications

Each specification chapter has its own directory. A chapter contains:

- `setup.bash`, defining reusable `scenario_<behavior>` functions that accept
  the directory in which they should build a scenario;
- one Markdown document per independently explained behavior;
- `README.md` as that document when the feature has only one, so GitHub renders
  it when the directory is opened;
- optional fixtures used by its setup function.

Documents source [`readme-setup.sh`](readme-setup.sh) in a hidden Scrut block,
then call `build_scenario` with the relevant setup function. The helper resets
the scenario directory between behaviors, while Scrut gives every document a
fresh temporary working directory.

Read the chapters in this order:

1. [Contract](contract/)
2. [Configuration](configuration/)
3. [Directory and file layout](directory-and-file-layout/)
4. [Special cases with `.document.yml`](document-metadata/)
5. [Amount audit](amount-audit/)
6. [Unbooked invoices](unbooked-invoices/)
7. [Due date](due-date/)
8. [Enrich journal](enrich-journal/)
9. [Full example](full-example/)
