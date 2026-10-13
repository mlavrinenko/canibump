#!/usr/bin/env bash
# docs/generate-md.sh: render docs/src/readme.typ to README.md, or to the
# path given, with typlite. It reads the captures under www/generated/, so
# www/generate.sh runs first. www/build.sh renders it to a temp file and
# fails when the committed README.md differs.
set -eo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${1:-$REPO/README.md}"
tmp=$(mktemp)
trap 'rm -f "$tmp"' EXIT

if ! err=$(typlite --root "$REPO" --input markdown=1 "$REPO/docs/src/readme.typ" "$tmp" 2>&1); then
    printf '%s\n' "$err" >&2
    exit 1
fi
# typlite keeps an HTML h1, so it becomes a `#` heading here.
{
    echo '<!-- Rendered from docs/src/readme.typ by `just docs-md`. Edit the source. -->'
    echo
    sed -E '1s|^<h1>([^<>]*)</h1>$|# \1|' "$tmp"
} >"$OUT"
