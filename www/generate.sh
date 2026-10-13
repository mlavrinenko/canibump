#!/usr/bin/env bash
# www/generate.sh: write www/generated/, everything the pages read that the
# build derives, then the page manifest.
set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT="$SCRIPT_DIR/generated"

rm -rf "$OUT"
mkdir -p "$OUT"

# Last, because querying a page compiles it, and a page reads what the steps
# above wrote.
bash "$SCRIPT_DIR/generate-site-pages.sh"
