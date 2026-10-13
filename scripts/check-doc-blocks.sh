#!/usr/bin/env bash
# scripts/check-doc-blocks.sh: fail on a code block typed into a doc source.
# Runs from the repo root.
#
# A typed block shows text nothing runs, so it drifts from what canibump
# prints and accepts, and the reader can't tell. A block on the site comes
# from a capture or a fixture file instead (www/README.md says how). What
# counts as typed is defined in scripts/doc-blocks-lib.sh.
#
# A block no capture can hold carries `// docs:hand-typed <reason>` on the
# line above it. A marker without a reason, or with no block under it, fails.
set -eo pipefail

# shellcheck source=scripts/doc-blocks-lib.sh
source scripts/doc-blocks-lib.sh

# The control: a scan that finds nothing looks like a scanner that reads
# nothing, as after a typstyle bump changes the `--ast` format.
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
printf '%s\n' '= Bad' '```sh' 'canibump' '```' '#raw("a\nb", block: true)' \
    '#let f(x) = raw(x, block: true)' '#f("typed")' '// docs:hand-typed' >"$tmp/bad.typ"
want=$(printf '%s\n' 2 5 7 8)
got=$(doc_blocks_scan "$tmp/bad.typ" | cut -f2)
if [ "$got" != "$want" ]; then
    echo "check-doc-blocks: the control found lines $(echo $got) in a file with blocks on $(echo $want)" >&2
    exit 1
fi

doc_sources_load
fail=0
while IFS=$'\t' read -r f line kind what; do
    if [ "$kind" = marker ]; then
        echo "check-doc-blocks: $f:$line: docs:hand-typed $what" >&2
    else
        echo "check-doc-blocks: $f:$line: $what is typed by hand; show a capture or a fixture file, or mark it // docs:hand-typed <reason>" >&2
    fi
    fail=1
done < <(doc_blocks_scan "${DOC_SOURCES[@]}")
exit $fail
