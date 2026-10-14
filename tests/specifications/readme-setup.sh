# shellcheck shell=bash

set -o pipefail

DOCUMENT_CHECK_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
export DOCUMENT_CHECK_ROOT
SPECIFICATION_WORKING_DIRECTORY=$PWD

build_scenario() {
  cd "$SPECIFICATION_WORKING_DIRECTORY" || return
  source "$TESTDIR/setup.bash"
  local scenario_dir="$SPECIFICATION_WORKING_DIRECTORY/scenario"
  rm -rf "$scenario_dir"
  "$1" "$scenario_dir" >/dev/null 2>&1 || return
  cd "$scenario_dir" || return
}
