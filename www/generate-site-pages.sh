#!/usr/bin/env bash
# www/generate-site-pages.sh: collect every page's
# `#metadata((route, label, title, summary, home)) <page-meta>` block into
# www/generated/site-pages.json, which www/utils/site.typ reads as `PAGES`.
# Querying a page compiles it, so www/generate.sh runs this after the captures
# the pages read.
set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(cd "$SCRIPT_DIR/.." && pwd)"
OUT="$SCRIPT_DIR/generated"
mkdir -p "$OUT" "$REPO/target"

# Install a file only once it parses, so a failed run keeps the last good one.
emit() {
  local name="$1" dest="$OUT/.$1.tmp.$$"
  cat >"$dest"
  if ! jq -e . "$dest" >/dev/null 2>&1; then
    rm -f "$dest"
    echo "generate-site-pages.sh: refusing to install malformed $name" >&2
    return 1
  fi
  mv -f "$dest" "$OUT/$name"
}

# The first pass. Compiling a page reads `PAGES`, so the file must exist,
# even empty, before the first query.
if [ ! -f "$OUT/site-pages.json" ]; then
  jq -n '{pages: {}}' | emit site-pages.json
fi

workdir="$(mktemp -d -p "$REPO/target")"
trap 'rm -rf "$workdir"' EXIT

# One query per page. `--one` fails a page with no block or two, and typst's
# error names the file. The HTML target is needed because the layout calls
# `html.elem`.
pages_json="{}"
routes_json="[]"
summary_problems=""
shopt -s globstar
for page in "$REPO"/www/content/**/*.typ; do
  stem="${page#"$REPO"/www/content/}"
  stem="${stem%.typ}"
  if ! meta="$(typst query --root "$REPO/www" "$page" '<page-meta>' \
      --field value --one --target html --features html 2>"$workdir/err")"; then
    echo "generate-site-pages.sh: $page has no <page-meta> block:" >&2
    cat "$workdir/err" >&2
    exit 1
  fi
  route="$(jq -r '.route' <<<"$meta")"
  pages_json="$(jq --argjson meta "$meta" --arg route "$route" \
    '. + {($route): ($meta | del(.route))}' <<<"$pages_json")"
  routes_json="$(jq --arg stem "$stem" --arg route "$route" \
    '. + [{stem: $stem, route: $route}]' <<<"$routes_json")"
  # The summary is the page's lede and its <meta name="description">, so it
  # stays one short sentence. Backticks count for nothing, and a `.` inside a
  # code span ends no sentence.
  problem="$(jq -r '(.summary // "") as $s | [
      ($s | gsub("`"; "") | length | select(. > 120) | "\(.) characters"),
      ($s | gsub("`[^`]*`"; "code") | select(test("[.!?]\\s+\\S"))
        | "more than one sentence")
    ] | join(", ")' <<<"$meta")"
  if [ -n "$problem" ]; then
    summary_problems+="  $stem: $problem"$'\n'
  fi
done

if [ -n "$summary_problems" ]; then
  echo "generate-site-pages.sh: a page summary must be one sentence of at most 120 characters:" >&2
  printf '%s' "$summary_problems" >&2
  exit 1
fi

# Two pages declaring one route fail, naming both.
dupes="$(jq -c 'group_by(.route) | map(select(length > 1))' <<<"$routes_json")"
if [ "$(jq 'length' <<<"$dupes")" -gt 0 ]; then
  echo "generate-site-pages.sh: two pages declare the same route:" >&2
  jq -r '.[] | "  " + .[0].route + ": " + (map(.stem) | join(", "))' <<<"$dupes" >&2
  exit 1
fi

# The site URL, prefix, repo and nav, read off site.typ through a driver
# file. The pages come out in nav order, and `groups` feeds llms.txt.
printf '#import "/www/utils/site.typ": NAV, PREFIX, REPO, SITE_URL\n#metadata((url: SITE_URL, prefix: PREFIX, repo: REPO, nav: NAV.map(g => g.items).flatten(), groups: NAV.map(g => (title: g.title, routes: g.items)))) <site-meta>\n' \
  >"$workdir/site-meta-driver.typ"
site_meta="$(typst query --root "$REPO" "$workdir/site-meta-driver.typ" '<site-meta>' \
  --field value --one)"

jq -n --argjson site "$site_meta" --argjson pages "$pages_json" '
  ($site.nav | map(select(. as $r | $pages | has($r)))) as $ordered
  | (($pages | keys) - $ordered) as $rest
  | {
      url: $site.url,
      prefix: $site.prefix,
      repo: $site.repo,
      groups: $site.groups,
      pages: (reduce ($ordered + $rest)[] as $r ({}; . + {($r): $pages[$r]})),
    }
' | emit site-pages.json
