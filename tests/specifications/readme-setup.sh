# shellcheck shell=bash

set -o pipefail

DOCUMENT_CHECK_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
export DOCUMENT_CHECK_ROOT
SPECIFICATION_WORKING_DIRECTORY=$PWD

# Start every scenario in an empty directory, even when a document has several.
new_scenario() {
  cd "$SPECIFICATION_WORKING_DIRECTORY" || return
  rm -rf scenario && mkdir scenario || return
  cd scenario || return
}

# write_file PATH <<'EOF' ... EOF
write_file() {
  mkdir -p "$(dirname "$1")" && cat >"$1"
}

# write_pdf PATH <<'EOF' ... EOF writes a one-page PDF showing the given text.
write_pdf() {
  local text
  text="$(cat)" || return
  mkdir -p "$(dirname "$1")" && minimal_pdf "$text" >"$1"
}

minimal_pdf() {
  local LC_ALL=C
  local text=$1
  text=${text//\\/\\\\}
  text=${text//(/\\(}
  text=${text//)/\\)}
  local stream=$'BT\n/F1 12 Tf\n72 720 Td\n('"$text"$') Tj\nET'
  local objects=(
    "<< /Type /Catalog /Pages 2 0 R >>"
    "<< /Type /Pages /Kids [3 0 R] /Count 1 >>"
    "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>"
    "<< /Length ${#stream} >>"$'\nstream\n'"$stream"$'\nendstream'
    "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>"
  )
  local body=$'%PDF-1.4\n' offsets=() i
  for i in "${!objects[@]}"; do
    offsets+=("${#body}")
    body+="$((i + 1)) 0 obj"$'\n'"${objects[i]}"$'\nendobj\n'
  done
  local xref_offset=${#body}
  body+="xref"$'\n'"0 $((${#objects[@]} + 1))"$'\n'"0000000000 65535 f "
  for i in "${offsets[@]}"; do
    body+=$'\n'"$(printf '%010d' "$i") 00000 n "
  done
  body+=$'\n'"trailer"$'\n'"<< /Size $((${#objects[@]} + 1)) /Root 1 0 R >>"
  body+=$'\n'"startxref"$'\n'"$xref_offset"$'\n'"%%EOF"$'\n'
  printf '%s' "$body"
}

# Copy the files git tracks, so leftovers such as example/.generated or
# untracked documents cannot change the example's output.
checkout_example() {
  git -C "$DOCUMENT_CHECK_ROOT" ls-files -z -- example |
    tar -C "$DOCUMENT_CHECK_ROOT" --null --files-from - -cf - |
    tar -xf -
}

# fava_linked_documents BEANCOUNT lists the documents a freshly started Fava
# serves for BEANCOUNT. Asking Fava's API for this file's documents proves the
# answer comes from this Fava, not from another process on the same port.
fava_linked_documents() {
  local port log=fava.log fava_pid status=0
  port="$(python3 -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])')" || return
  fava --host 127.0.0.1 --port "$port" "$1" >"$log" 2>&1 &
  fava_pid=$!
  local deadline=$((SECONDS + 30))
  until curl --fail --silent --max-time 2 "http://127.0.0.1:$port/beancount/api/documents" >fava-documents.json; do
    if ! kill -0 "$fava_pid" 2>/dev/null || ((SECONDS > deadline)); then
      cat "$log"
      status=1
      break
    fi
    sleep 0.1
  done
  kill "$fava_pid" 2>/dev/null
  wait "$fava_pid" 2>/dev/null
  ((status == 0)) || return "$status"
  python3 -c 'import json, sys; print(*sorted(d["filename"] for d in json.load(sys.stdin)["data"]), sep="\n")' <fava-documents.json
}
