#!/usr/bin/env bash
# Usage: release-ci.sh VERSION. Waits for ci.yml on HEAD, which `just release`
# has just pushed, and exits 0 only when it is green: the caller tags then.
# Each refusal prints `error:` and the `hint:` lines with the way out.
set -euo pipefail
version=$1
tag="v$version"
refuse() { echo "error: $1" >&2; shift; for h in "$@"; do echo "hint: $h" >&2; done; exit 1; }
head=$(git rev-parse HEAD)
short=$(git rev-parse --short HEAD)
run=""
deadline=$((SECONDS + 300))
while :; do
    run=$(gh run list --commit "$head" --workflow ci.yml --limit 1 \
        --json databaseId --jq '.[0].databaseId // empty') \
        || refuse "can't list ci.yml runs for '$short'" "check gh auth status and that .github/workflows/ci.yml exists"
    [ -n "$run" ] && break
    [ "$SECONDS" -lt "$deadline" ] \
        || refuse "no ci.yml run showed up for '$short' in 5 minutes, so '$tag' was not made" "main is pushed; once ci.yml is green on it, re-run just release $version"
    sleep 5
done
echo "waiting for ci.yml run '$run' on '$short'"
if ! gh run watch "$run" --exit-status --compact --interval 15 >/dev/null; then
    conclusion=$(gh run view "$run" --json conclusion --jq .conclusion 2>/dev/null || true)
    refuse "ci.yml ended '${conclusion:-unknown}' on '$short', so '$tag' was not made" \
        "see why: gh run view $run --log-failed" \
        "fix main and re-run just release $version, or retry a flake with: gh run rerun $run --failed"
fi
