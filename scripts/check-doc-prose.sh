#!/usr/bin/env bash
# scripts/check-doc-prose.sh: fail on prose a reader trips over, in the doc
# sources' rendered text. Runs from the repo root.
#
# - History words (`now`, `anymore`, `formerly`, `used to`, `no longer`). A
#   reader needs what the tool does, and git log holds how it got there.
# - An em dash, or a colon inside a sentence. Split the sentence instead.
# - An all-caps word outside `ALLOWED`, which reads as shouting.
#
# Raw spans, blocks, code and double-quoted text are skipped, since they are
# code or a quotation. scripts/doc-blocks-lib.sh yields the rendered text.
set -eo pipefail

# shellcheck source=scripts/doc-blocks-lib.sh
source scripts/doc-blocks-lib.sh

ALLOWED="API ASCII CI CLI CSS GPU HTML HTTP HTTPS ID JSON MIT MR OS PR README RFC
SHA SSH TOML TTL UI URL UTC UTF XDG YAML"

scan() {
    awk -F'\t' -v allowed="$ALLOWED" '
        BEGIN { n = split(allowed, a, /[ \n]+/); for (i = 1; i <= n; i++) ok[a[i]] = 1 }
        $3 != "prose" { next }
        {
            t = $4
            gsub(/"[^"]*"/, " ", t)
            low = tolower(t)
            if (match(low, /(^|[^a-z])(now|anymore|formerly|used to|no longer)([^a-z]|$)/))
                hit("tells history", substr(t, RSTART, RLENGTH))
            if (index(t, "—")) hit("has an em dash", "—")
            if (match(t, /[^ :]:( |$)/)) hit("has a colon in a sentence", substr(t, RSTART, RLENGTH))
            n = split(t, words, /[^A-Za-z0-9]+/)
            for (i = 1; i <= n; i++) {
                w = words[i]
                sub(/s$/, "", w)
                if (w ~ /^[A-Z][A-Z0-9]+$/ && !ok[w]) hit("shouts", w)
            }
        }
        function hit(why, what) { printf "check-doc-prose: %s:%s: %s: %s\n", $1, $2, why, what; bad = 1 }
        END { exit bad }
    '
}

# The control: a scan that passes this would pass anything.
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
printf '%s\n' '= Bad' 'It now prints this.' 'One — two.' 'Two modes: fast and slow.' 'This is VERY bad.' \
    'Fine `now` and "now" and a PR or two PRs on PyPI.' >"$tmp/bad.typ"
if [ "$(doc_extract "$tmp/bad.typ" | scan | cut -d: -f3 | tr '\n' ' ')" != "2 3 4 5 " ]; then
    echo "check-doc-prose: the control did not flag exactly lines 2 to 5" >&2
    exit 1
fi

doc_sources_load
doc_extract "${DOC_SOURCES[@]}" | awk -F'\t' '$1 ~ /^(www\/content|docs\/src)\//' | scan
