#!/usr/bin/env bash
# www/generate.sh: write www/generated/, everything the pages read that the
# build derives. It builds canibump and records it running against the
# fixtures under www/examples/ (www/capture.sh says how), then writes the page
# manifest. The build regenerates all of it, so no page can drift from the
# binary.
set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(cd "$SCRIPT_DIR/.." && pwd)"
OUT="$SCRIPT_DIR/generated"
BIN="$REPO/target/debug/canibump"

# The debug build, the binary the test suite keeps warm.
(cd "$REPO" && cargo build -q)

rm -rf "$OUT"
mkdir -p "$OUT"

# shellcheck source=www/capture.sh
. "$SCRIPT_DIR/capture.sh"
capture_init

# The quickstart: a blocker waits, upstream merges, and the next run points
# at the workaround.
capture_copy dir quickstart
capture quickstart "$dir" canibump
capture_upstream quickstart upstream-merged.json
capture --exit 9 quickstart "$dir" 'canibump --refresh'

# Last, because querying a page compiles it, and a page reads what the steps
# above wrote.
bash "$SCRIPT_DIR/generate-site-pages.sh"

# A capture shows what any reader's canibump prints, so none may name this
# machine's home or checkout.
if grep -rnIF -e "$HOME" -e "$REPO" "$OUT" >&2; then
  echo "generate.sh: a capture above names \$HOME or the checkout" >&2
  exit 1
fi
