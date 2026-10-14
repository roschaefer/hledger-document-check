# See README.md in this directory.

scenario_unbooked_move() {
  local scenario_dir="$1"
  cp -R "$TESTDIR/fixture" "$scenario_dir"
  cp \
    "$DOCUMENT_CHECK_ROOT/example/documents/Expenses/Business/Hosting/Aws/unbooked/invoice.pdf" \
    "$scenario_dir/documents/expenses/business/hosting/aws/unbooked/invoice.pdf"
}
