# Executable specifications

Each feature has its own directory. A feature directory contains:

- `setup.bash`, defining a reusable `scenario_<feature>` function that accepts
  the directory in which it should build the scenario;
- one Markdown document per independently explained behavior;
- `README.md` as that document when the feature has only one, so GitHub renders
  it when the directory is opened;
- optional fixtures used by its setup function.

Documents source [`readme-setup.sh`](readme-setup.sh) in a hidden Scrut block,
then call `build_scenario` with their feature's setup function. Scrut gives each
document a fresh temporary working directory, while every test case in that
document shares the scenario built there.
