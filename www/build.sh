#!/usr/bin/env bash
# www/build.sh: build the docs site into www/public/canibump, then index it
# for search, write each page's Markdown copy and llms.txt, and validate every
# link and asset. Silent on success.
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
"$QUIET" tola build --clean
pagefind --site public/canibump --silent
bash generate-md.sh
"$QUIET" tola validate
