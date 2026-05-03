#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
output_path="${1:-$repo_root/docs/fava-documents.png}"

if command -v chromium >/dev/null 2>&1; then
  browser=(chromium)
elif command -v google-chrome >/dev/null 2>&1; then
  browser=(google-chrome)
else
  echo "chromium or google-chrome is required to capture the screenshot" >&2
  exit 1
fi

port="$(
  python3 - <<'PY'
import socket

with socket.socket() as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
)"

cd "$repo_root"
mkdir -p "$(dirname "$output_path")"

example/rebuild-fava-inputs.sh
uv run --no-sync python -m hledger_document_check.cli enrich-journal \
  --journal example/journal.journal \
  --documents example/documents \
  --document-tag-root "$repo_root/example/.generated/fava/documents" \
  | hledger -f - print -O beancount >> example/.generated/fava/example.beancount

fava example/.generated/fava/example.beancount --port "$port" >/tmp/hledger-document-check-fava-screenshot.log 2>&1 &
fava_pid="$!"
trap 'kill "$fava_pid" >/dev/null 2>&1 || true' EXIT

python3 - "$port" <<'PY'
import sys
import time
import urllib.request

port = sys.argv[1]
url = f"http://127.0.0.1:{port}/beancount/documents/"
deadline = time.monotonic() + 10
while time.monotonic() < deadline:
    try:
        with urllib.request.urlopen(url, timeout=0.5) as response:
            if response.status == 200:
                break
    except Exception:
        time.sleep(0.2)
else:
    raise SystemExit(f"Fava did not become ready at {url}")
PY

"${browser[@]}" \
  --headless \
  --no-sandbox \
  --disable-gpu \
  --window-size=840,680 \
  --screenshot="$output_path" \
  "http://127.0.0.1:$port/beancount/documents/" \
  >/dev/null

echo "Wrote $output_path"
