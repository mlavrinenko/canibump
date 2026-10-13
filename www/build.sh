#!/usr/bin/env bash
# www/build.sh: build the docs site into www/public/canibump, then index it
# for search, write each page's Markdown copy and llms.txt, and validate every
# link and asset. It fails when README.md differs from its render. Silent on
# success.
#
# `tola build --clean`, because tola does not count a `json()` read as a page
# dependency, so an incremental build keeps a page whose data changed.
# tola prints its progress on success too and has no quiet flag, so
# scripts/quiet.sh replays it only on failure.
set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
QUIET="$SCRIPT_DIR/../scripts/quiet.sh"
cd "$SCRIPT_DIR"

bash generate.sh
# README.md renders from the same captures, so it can't lag behind them.
readme=$(mktemp)
trap 'rm -f "$readme"' EXIT
bash ../docs/generate-md.sh "$readme"
if ! diff -u ../README.md "$readme" >&2; then
    echo "www/build.sh: README.md differs from docs/src/readme.typ; run just docs-md" >&2
    exit 1
fi
"$QUIET" tola build --clean
pagefind --site public/canibump --silent
bash generate-md.sh
"$QUIET" tola validate
