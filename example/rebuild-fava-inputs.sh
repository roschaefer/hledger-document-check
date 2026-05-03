#!/usr/bin/env bash
set -euo pipefail

example_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

output_dir="$example_dir/.generated/fava"
source_document_dir="$example_dir/documents"
generated_document_dir="$output_dir/documents"
beancount="$output_dir/example.beancount"

rm -rf "$output_dir"
mkdir -p "$generated_document_dir"

while IFS= read -r -d '' file; do
  target="$generated_document_dir/${file#$source_document_dir/}"
  mkdir -p "$(dirname "$target")"
  cp "$file" "$target"
done < <(
  find "$source_document_dir" \
    -type f \
    ! -name '*.document.yml' \
    ! -name 'hledger-document-check.toml' \
    -print0
)

{
  echo 'option "operating_currency" "EUR"'
  echo '2016-06-14 custom "fava-option" "invert-income-liabilities-equity" "true"'
  echo "option \"documents\" \"$generated_document_dir\""
} > "$beancount"
