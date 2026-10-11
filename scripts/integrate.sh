#!/usr/bin/env bash
# Usage: integrate.sh BRANCH. Fast-forwards this checkout onto a fork.sh branch
# whose worktree gate is fresh, carries the gate records and lcov over, removes
# the worktree. A fast-forward fires no pre-commit hook, so this is the only
# gate the merge meets. It runs no gate and has no waiver (`git merge --ff-only`).
set -euo pipefail
branch=${1:?usage: integrate.sh BRANCH}

# The only mapping from a branch to the directory holding it. No worktree means
# no gate record to read.
wt=$(git worktree list --porcelain | awk -v ref="refs/heads/$branch" '
    /^worktree / { path = substr($0, 10) }
    /^branch /   { if (substr($0, 8) == ref) { print path; exit } }
')
if [ -z "$wt" ]; then
    echo "integrate: no worktree has '$branch' checked out" >&2
    echo "  a branch with no worktree carries no gate record to read; \`just fork $branch\` stands one up" >&2
    exit 1
fi

here=$(git rev-parse --show-toplevel)
# Resolved now, and tolerant of a detached HEAD: a failure on the last line would
# report a landed integration as failed.
onto=$(git symbolic-ref --short HEAD 2>/dev/null || echo "detached HEAD")
if [ "$wt" = "$here" ]; then
    echo "integrate: this checkout IS the worktree holding '$branch', nothing here to fast-forward onto it" >&2
    echo "  run it from the checkout holding the branch you are merging INTO" >&2
    exit 1
fi

# `git worktree remove` refuses a dirty tree too, but only at the end, after the
# fast-forward. Untracked files count: a brand-new source file dies with the tree.
dirty=$(git -C "$wt" status --porcelain)
if [ -n "$dirty" ]; then
    echo "integrate: $wt has uncommitted changes:" >&2
    printf '%s\n' "$dirty" | sed 's/^/  /' >&2
    echo "  commit them there (which re-runs the gate), or discard them, then integrate again" >&2
    exit 1
fi

# Before the gate, so a branch that cannot land never spends a gate assertion.
# Rebasing is a person's job.
branch_sha=$(git rev-parse --verify "refs/heads/$branch")
head_sha=$(git rev-parse --verify HEAD)
if ! git merge-base --is-ancestor "$head_sha" "$branch_sha"; then
    echo "integrate: '$branch' is not a fast-forward of $onto" >&2
    echo "  HEAD $head_sha  $branch $branch_sha, rebase the branch onto this checkout first" >&2
    exit 1
fi

# `cd`, not just `direnv exec`: mmz resolves its manifest from the working
# directory, so without the cd it would answer about this checkout, green.
run_in_worktree() {
    if command -v direnv >/dev/null 2>&1; then
        (cd "$wt" && direnv exec . "$@")
    else
        (cd "$wt" && "$@")
    fi
}

# A task named in a `Refs:` trailer and still `wip` at the tip lost its `done`
# flip, and the fix would rewrite shared commits. A path value counts as its
# stem, as `mt check --commit-msg` reads it.
key=$(run_in_worktree mt config show --json | jq -r '.commits.trailer // "Refs"')
named=$(git log --format="%(trailers:key=$key,valueonly)" "$head_sha..$branch_sha" \
    | sed -e '/^$/d' -e 's#.*/##' -e 's#\.typ$##' | sort -u)
open=$(run_in_worktree mt ls 'status == "wip"' --format path \
    | sed -e 's#.*/##' -e 's#\.typ$##' | sort -u)
left=$(comm -12 <(printf '%s\n' "$named") <(printf '%s\n' "$open") | sed '/^$/d')
if [ -n "$left" ]; then
    echo "integrate: '$branch' names tasks that are still wip at its tip:" >&2
    printf '%s\n' "$left" | sed 's/^/  /' >&2
    echo "  run \`mt done <stem>\` in $wt and commit it with the task's last work, then integrate again" >&2
    exit 1
fi

# Exit 7 is a selection with no rule at all, which must be refused rather than
# passed on the strength of having checked nothing.
gate=0
run_in_worktree mmz --is-fresh --tag gate || gate=$?
if [ "$gate" -ne 0 ]; then
    if [ "$gate" -eq 7 ]; then
        echo "integrate: $wt: mmz found no gate-tagged rule, that branch's .mmz/config.yaml declares none" >&2
    else
        echo "integrate: $wt: the arms above are not fresh, so the gate has not passed against '$branch'" >&2
    fi
    echo "  re-run \`just check\` in $wt, then integrate again" >&2
    exit 1
fi

git merge --ff-only -q "$branch"

# mmz records live in each checkout's gitignored `.mmz/cache/`, so this one
# reads stale until a full `just check`. A record is portable (repo-relative
# paths, content hashes): copy the worktree's, keep a later one here. One whose
# digest does not match this tree simply reads stale. A failed copy is named,
# never fatal.
if [ -d "$wt/.mmz/cache" ]; then
    mkdir -p "$here/.mmz/cache"
    ran_at() { sed -n 's/^ran_at: *//p' "$1" 2>/dev/null | head -n 1; }
    for record in "$wt"/.mmz/cache/*.yaml; do
        [ -e "$record" ] || continue
        dest="$here/.mmz/cache/$(basename "$record")"
        if [ -e "$dest" ] && [ "$(ran_at "$dest" || echo 0)" -gt "$(ran_at "$record" || echo 0)" ] 2>/dev/null; then
            continue
        fi
        cp "$record" "$dest" || echo "integrate: could not carry $(basename "$record") into $here/.mmz/cache" >&2
    done
fi

# The same for the lcov `just crap` scores: the fast-forward made this
# checkout's one stale. The worktree's moves if crap.sh finds it fresh there (an
# unanswered question counts as stale) and it is newer than ours. Its `SF:`
# lines are rewritten, since llvm-cov writes absolute paths.
lcov=target/coverage/lcov.info
stale=$(run_in_worktree scripts/crap.sh --stale-reason 2>/dev/null) || stale="unanswered"
if [ -z "$stale" ] && [ -f "$wt/$lcov" ] &&
    { [ ! -f "$here/$lcov" ] || [ "$wt/$lcov" -nt "$here/$lcov" ]; }; then
    mkdir -p "$here/target/coverage" &&
        sed "s#^SF:$wt/#SF:$here/#" "$wt/$lcov" >"$here/$lcov.tmp" &&
        mv "$here/$lcov.tmp" "$here/$lcov" ||
        echo "integrate: could not carry $lcov into $here" >&2
fi

git worktree remove "$wt"
echo "integrate: $wt: gate fresh; $onto fast-forwarded to $(git rev-parse --short HEAD), worktree removed"
