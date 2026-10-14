# See README.md in this directory.

scenario_full_example() {
  local scenario_dir="$1"
  mkdir -p "$scenario_dir"
  cp -R "$DOCUMENT_CHECK_ROOT/example" "$scenario_dir/example"
}
