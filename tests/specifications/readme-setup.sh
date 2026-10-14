# shellcheck shell=bash

DOCUMENT_CHECK_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
export DOCUMENT_CHECK_ROOT

build_scenario() {
  source "$TESTDIR/setup.bash"
  local scenario_dir="$PWD/scenario"
  "$1" "$scenario_dir" >/dev/null 2>&1 || return
  cd "$scenario_dir" || return
}
