#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

core() {
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo test --workspace --all-targets --locked --verbose
  cargo build --workspace --release --locked
}

public_artifacts() {
  while IFS= read -r -d '' file; do
    python -m json.tool "$file" >/dev/null
  done < <(find schemas -type f -name '*.json' -print0)
  python -m json.tool examples/trust-policy.json >/dev/null
  while IFS= read -r line; do
    [ -z "$line" ] || python -c 'import json,sys; json.loads(sys.stdin.read())' <<<"$line"
  done < examples/events.jsonl

  node --check apps/human-translator/app.js
  node --check apps/human-translator/htp-v02.js

  python - <<'PY'
from pathlib import Path
from html.parser import HTMLParser

class RuntimeRefs(HTMLParser):
    def __init__(self):
        super().__init__()
        self.remote = []
    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if tag == 'script':
            value = attrs.get('src', '')
        elif tag == 'link' and attrs.get('rel') == 'stylesheet':
            value = attrs.get('href', '')
        else:
            return
        if value.startswith(('http://', 'https://', '//')):
            self.remote.append(value)

page = Path('apps/human-translator/index.html').read_text(encoding='utf-8')
parser = RuntimeRefs(); parser.feed(page)
if parser.remote:
    raise SystemExit(f'remote runtime dependencies are forbidden: {parser.remote}')
PY

  test -f spec/HTP-v0.2.md
  test -f reference/rust/src/dimension.rs
  ! grep -R --line-number -E 'pub fn derive_closure|struct DtcClosure|struct Topology|CrystallineEngine|NativeWorld' reference/rust
  grep -q 'private_crystalline_closure_included' reference/rust/src/protocol.rs
  grep -q 'HTP 0.2 live' apps/human-translator/index.html
}

example() {
  target/release/htp-live --trust-policy examples/trust-policy.json --profile public --unsigned examples/events.jsonl > /tmp/htp-example.jsonl
  python - <<'PY'
import json
from pathlib import Path
lines = [line for line in Path('/tmp/htp-example.jsonl').read_text().splitlines() if line.strip()]
assert len(lines) == 1, f'expected one completed witness, got {len(lines)}'
record = json.loads(lines[0])
assert record['unsigned'] is True
assert record['profile'] == 'public'
assert record['content']['protocol_version'] == 'ddc-htp/0.2'
assert record['content']['state'] == 'awaiting_authority'
assert record['content']['authority']['required'] == ['system.install']
assert record['content']['authority']['granted'] == []
PY
}

msrv() {
  grep -q '^version = 3$' Cargo.lock
  cargo +1.78.0 check --locked -p htp-reference --lib --bins
}

if [ "$#" -eq 0 ]; then
  core
  public_artifacts
  example
  exit 0
fi

for arg in "$@"; do
  case "$arg" in
    --core) core ;;
    --public) public_artifacts ;;
    --example) example ;;
    --msrv) msrv ;;
    --all) core; public_artifacts; example; msrv ;;
    -h|--help) echo "Usage: $0 [--core] [--public] [--example] [--msrv] [--all]" ;;
    *) echo "Unknown option: $arg" >&2; exit 2 ;;
  esac
done
