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

# A blocker waits, upstream merges, and the next run points at the
# workaround: once whole for the site root, then step by step.
capture_copy dir quickstart
capture hero "$dir" canibump
capture_upstream quickstart upstream-merged.json
capture --exit 9 hero "$dir" 'canibump --refresh'
capture_copy dir quickstart
capture quickstart "$dir" canibump
capture_upstream quickstart upstream-merged.json
capture --exit 9 quickstart-landed "$dir" 'canibump --refresh'

# One blocker per forge, each in a state of its own.
capture_copy dir wait-on-a-pr
capture --exit 11 wait-on-a-pr "$dir" canibump

# Commits landing in a tag, a branch, and a tag of a plain git remote, which
# lives here. The fixed clock keeps the hash the manifest names.
capture_remote work https://git.example.org/libbaz.git
capture_git "$work" commit -q --allow-empty -m "libbaz 3.1.0"
capture_git "$work" tag v3.1.0
capture_git "$work" commit -q --allow-empty -m "Fix the crash on an empty locale"
capture_git "$work" commit -q --allow-empty -m "libbaz 3.2.0"
capture_git "$work" tag v3.2.0
capture_git "$work" push -q origin main --tags
capture_copy dir wait-on-a-release
capture --exit 9 wait-on-a-release "$dir" canibump

# Packages on Repology and deps.dev.
capture_copy dir wait-on-a-package
capture --exit 9 wait-on-a-package "$dir" canibump

# A chain merged and released, waiting on nixpkgs until nixpkgs moves.
capture_copy dir chain
capture chain "$dir" canibump
capture_upstream chain upstream-packaged.json
capture --exit 9 chain-landed "$dir" 'canibump --refresh'

# Markers in Nix files, a declared site in package.json, and a blocker that
# nothing places.
capture_copy dir marks
capture --exit 41 marks "$dir" canibump
capture --exit 1 --expect '^waiting +left-pad-pin' marks-status "$dir" 'canibump status left-pad-pin'

# Links and markers no blocker names.
capture_copy dir scan
capture --exit 24 scan "$dir" canibump

# Exit codes for scripts and CI, on the forge fixture.
capture_copy dir wait-on-a-pr
capture --exit 10 ci-fail-on "$dir" 'canibump --fail-on dead'
capture ci-status "$dir" 'canibump status libfoo-crash && echo "go drop the patch"'

# The results a run saved, read back with no network.
capture_copy dir wait-on-a-pr
capture_prep "$dir" 'canibump || true'
capture --exit 11 offline "$dir" 'canibump status'
capture offline-path "$dir" 'canibump status --path'
capture offline-json "$dir" "canibump --json | jq '.entries[0] | {id, status, detail}'"

# The CLI reference: `--help` of canibump and of each command its help lists.
capture_scratch dir
capture cli "$dir" 'canibump --help'
for sub in $("$BIN" --help | awk '/^Commands:/ { on = 1; next } on && NF == 0 { on = 0 } on && $1 != "help" { print $1 }'); do
  capture cli "$dir" "canibump $sub --help"
done

# The source files the internals page may name, so a renamed one fails it.
(cd "$REPO/src" && find . -name '*.rs' ! -name '*_tests.rs' -printf '%P\n' | sort | jq -R . | jq -s .) >"$OUT/src-files.json"

# The schemas the reference pages render.
for name in manifest report state; do "$BIN" schema "$name" >"$OUT/schema-$name.json"; done

# Last, because querying a page compiles it, and a page reads what the steps
# above wrote.
bash "$SCRIPT_DIR/generate-site-pages.sh"

# A capture shows what any reader's canibump prints, so none may name this
# machine's home or checkout.
if grep -rnIF -e "$HOME" -e "$REPO" "$OUT" >&2; then
  echo "generate.sh: a capture above names \$HOME or the checkout" >&2
  exit 1
fi
