#!/usr/bin/env bash
# .just/scripts/integrate.sh — the body of the `integrate` recipe. Runs from the
# repo root, like fork.sh, whose inverse it is.
#
# `just fork` stands a delegated worktree up; this takes it back down, and the
# one thing it does on the way is an assertion nothing else makes. A
# fast-forward writes no commit, so `.githooks/pre-commit`, the only place
# `mmz --is-fresh --tag gate` fires on its own, never runs on the merge.
#
# The assertion is read from the worktree, not from this checkout: mmz's cache
# records are per-tree, so "did the gate pass against the code being merged" can
# only be answered where that code is. It has to be asked before this checkout
# moves and before the worktree is gone, which is why removal is the last line.
#
# It never runs the gate, the same trade the pre-commit hook takes. Asserting a
# prior pass is cheap and is the predicate worth having; running the aggregate
# at merge time is slow at the least convenient moment, and a slow command there
# is how a bypass gets invented. For the same reason there is no waiver flag,
# unlike fork.sh's CANIBUMP_FORK_DIRTY_OK: one would be a supported way to land
# a red branch. The escape hatch is `git merge --ff-only <branch>` by hand,
# which takes a deliberate second command and reads as blessed by nothing.
#
# The branch ref survives the worktree that held it. Removing it is a separate
# judgement, and deleting it after the fast-forward would drop the only name
# left on the work if the integration turned out to be the wrong call.
set -eo pipefail

branch="$1"

# `git worktree list --porcelain` is the only mapping from a branch to the
# directory holding it. A branch checked out nowhere has no tree to ask, which
# is also the shape of "the worktree was already removed" — and a gate record
# that cannot be read is not a gate record that passed.
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
# Resolved up front and tolerating a detached HEAD: read on the last line, after
# the fast-forward landed, a failing `symbolic-ref` would report a successful
# integration as a failed one.
onto=$(git symbolic-ref --short HEAD 2>/dev/null || echo "detached HEAD")
if [ "$wt" = "$here" ]; then
    echo "integrate: this checkout IS the worktree holding '$branch' — nothing here to fast-forward onto it" >&2
    echo "  run it from the checkout holding the branch you are merging INTO" >&2
    exit 1
fi

# `git worktree remove` refuses a dirty tree too, but at the end, after the
# fast-forward has moved this checkout, and a half-done integration is worse than
# a refused one. Untracked files count: removing the tree is where a brand-new
# source file dies.
dirty=$(git -C "$wt" status --porcelain)
if [ -n "$dirty" ]; then
    echo "integrate: $wt has uncommitted changes:" >&2
    sed 's/^/  /' <<<"$dirty" >&2
    echo "  commit them there (which re-runs the gate), or discard them, then integrate again" >&2
    exit 1
fi

# Checked before the gate rather than left to `git merge --ff-only`'s own
# refusal, so a branch that cannot land never spends a gate assertion arguing
# about it. Rebasing is out of scope: a conflict is a person's job.
branch_sha=$(git rev-parse --verify "refs/heads/$branch")
head_sha=$(git rev-parse --verify HEAD)
if ! git merge-base --is-ancestor "$head_sha" "$branch_sha"; then
    echo "integrate: '$branch' is not a fast-forward of $onto" >&2
    echo "  HEAD $head_sha  $branch $branch_sha — rebase the branch onto this checkout first" >&2
    exit 1
fi

# `cd` rather than `direnv exec "$wt"` alone: direnv exec loads that directory's
# environment but keeps the working directory, and mmz resolves its manifest
# from cwd upward. Without the cd it would read this checkout's cache and answer
# about the wrong tree — silently, and green.
#
# The direnv hop is what puts mmz and every gate tool on PATH; fork.sh allows
# the worktree it creates, so it is there to be loaded. A bare command is right
# rather than fatal for a worktree made some other way: a host with the tools
# already on PATH answers the same question.
run_in_worktree() {
    if command -v direnv >/dev/null 2>&1; then
        (cd "$wt" && direnv exec . "$@")
    else
        (cd "$wt" && "$@")
    fi
}

# A branch that names a task in a `Refs:` trailer and leaves it `wip` at its tip
# lost the `done` flip or never finished the task: landing it puts work on this
# checkout whose task still says it is under way, and the repair is a rewrite of
# commits already shared. It is asked before the gate because it is cheap and
# its answer is a one-line fix. The trailer key is the project's `[commits]
# trailer`, and a path value (`truth/tasks/<stem>.typ`) counts as its stem, as
# `mt check --commit-msg` reads it.
key=$(run_in_worktree mt config show --json | jq -r '.commits.trailer // "Refs"')
named=$(git log --format="%(trailers:key=$key,valueonly)" "$head_sha..$branch_sha" \
    | sed -e '/^$/d' -e 's#.*/##' -e 's#\.typ$##' | sort -u)
open=$(run_in_worktree mt ls 'status == "wip"' --format path \
    | sed -e 's#.*/##' -e 's#\.typ$##' | sort -u)
left=$(comm -12 <(printf '%s\n' "$named") <(printf '%s\n' "$open") | sed '/^$/d')
if [ -n "$left" ]; then
    echo "integrate: '$branch' names tasks that are still wip at its tip:" >&2
    sed 's/^/  /' <<<"$left" >&2
    echo "  run \`mt done <stem>\` in $wt and commit it with the task's last work, then integrate again" >&2
    exit 1
fi

# The verdict is `--is-fresh`'s own exit code, and naming each stale or failed
# arm is already mmz's job, so only what mmz cannot know is added: which tree
# was asked, and what to do about the answer. The exit code also separates a
# stale or failed rule (1) from a selection holding no rule at all (7), which
# prints nothing of its own. A branch whose `.mmz/config.yaml` declares no
# gate-tagged rule has to be refused, not passed on the strength of having
# checked nothing.
gate=0
run_in_worktree mmz --is-fresh --tag gate || gate=$?
if [ "$gate" -ne 0 ]; then
    if [ "$gate" -eq 7 ]; then
        echo "integrate: $wt: mmz found no gate-tagged rule — that branch's .mmz/config.yaml declares none" >&2
    else
        echo "integrate: $wt: the arms above are not fresh, so the gate has not passed against '$branch'" >&2
    fi
    echo "  re-run \`just check\` in $wt, then integrate again" >&2
    exit 1
fi

# `-q` because a diffstat of the whole branch is not the answer to "did this
# land". The sha below is.
git merge --ff-only -q "$branch"

# The gate was asserted in the worktree and the fast-forward made this checkout
# the same tree, but mmz's records live in each checkout's own gitignored
# `.mmz/cache/`, so this one still reads stale and the pre-commit hook would
# refuse the next commit until a full `just check` re-ran. A record is portable:
# its `input_digest` is built from repo-relative paths and content hashes, so
# copying the worktree's records over is all it takes, and mmz stays the judge.
# One whose digest does not match this tree (an untracked file here, say) simply
# reads stale. Only `*.yaml` records move, and a record here with a later
# `ran_at` than the worktree's is kept. A failed copy does not un-land the
# merge, so it is named, never fatal.
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

# The same holds for the lcov `just crap` scores against: the fast-forward
# touched every source it changed, so this checkout's lcov predates them and
# crap refuses it until a full `just cover`. The worktree's lcov describes the
# same tree, so it moves too, under three conditions. It must be fresh there,
# asked of the Justfile's own `_lcov-stale-reason` rather than a copied `find`,
# because the copy gets a new mtime and would hide that it predates its sources;
# an unanswered question counts as stale. It must be newer than this checkout's.
# And its `SF:` lines get rewritten, because llvm-cov writes absolute paths and
# crap scores a function whose path names the removed worktree as uncovered. A
# failed copy is named, never fatal.
lcov=target/coverage/lcov.info
stale=$(run_in_worktree just _lcov-stale-reason 2>/dev/null) || stale="unanswered"
if [ -z "$stale" ] && [ -f "$wt/$lcov" ] &&
    { [ ! -f "$here/$lcov" ] || [ "$wt/$lcov" -nt "$here/$lcov" ]; }; then
    mkdir -p "$here/target/coverage" &&
        sed "s#^SF:$wt/#SF:$here/#" "$wt/$lcov" >"$here/$lcov.tmp" &&
        mv "$here/$lcov.tmp" "$here/$lcov" ||
        echo "integrate: could not carry $lcov into $here" >&2
fi

git worktree remove "$wt"
echo "integrate: $wt: gate fresh; $onto fast-forwarded to $(git rev-parse --short HEAD), worktree removed"
