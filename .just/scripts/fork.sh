#!/usr/bin/env bash
# .just/scripts/fork.sh — the body of the `fork` recipe. Runs from the repo
# root, like the recipe that invokes it.
#
# Standing up a worktree for delegated work is `git worktree add`, `direnv
# allow`, a first load of the dev shell, and one assertion: the worktree sits on
# the commit that was meant. Only the assertion is load-bearing, and it is the
# step that gets skipped, because its failure is silent. A worktree branched off
# the wrong base ships a diff that passes its own gate while dropping everything
# the real base carried.
#
# What the assertion can and cannot prove. The base is resolved to a full sha
# once, and `git worktree add` gets that sha rather than the name, so a branch
# that moves in between cannot change the answer; the new worktree's HEAD is
# then read back and compared. It cannot prove the sha is the one the caller
# meant, since the recipe has no access to that intent. So the last output is a
# line to paste into the delegate's prompt: the check here makes the sha
# trustworthy, the check there makes it the right one.
#
# Git's own refusals (an existing branch or directory, a base that is no
# commit) are not re-implemented; git names them better than a copy would.
set -eo pipefail

slug="$1"
base="${2:-HEAD}"

if ! [[ "$slug" =~ ^[a-z0-9][a-z0-9-]*$ ]]; then
    echo "fork: slug '$slug' must be lowercase letters, digits and dashes, starting with a letter or digit" >&2
    exit 1
fi

# Reached through `--git-common-dir` rather than `$(pwd)`, so forking from a
# worktree still lands the new one beside the main checkout instead of nesting a
# `canibump-a-b` beside `canibump-a`. Siblings, never children: a worktree
# inside the repo would put a second `src/` and a second `truth/tasks/` under
# every path-walking tool here — `mt check` would see each task stem twice and
# jscpd would report every file as a duplicate of itself.
main_root=$(dirname "$(git rev-parse --path-format=absolute --git-common-dir)")
fork_root="${CANIBUMP_FORK_ROOT:-$(dirname "$main_root")}"
path="$fork_root/$(basename "$main_root")-$slug"

if ! base_sha=$(git rev-parse --verify --quiet "$base^{commit}"); then
    echo "fork: base '$base' does not resolve to a commit" >&2
    exit 1
fi
head_sha=$(git rev-parse --verify HEAD)

# The one refusal. `git worktree add` copies commits, never the index or the
# working tree, so forking off a dirty HEAD hands the delegate a base missing
# exactly the changes the caller is looking at while typing the command. Only
# when the base is HEAD: naming any other base says the working tree is not the
# subject. Untracked files count — a brand-new source file has no committed
# version to fall back to — and `git status --porcelain` already honours
# `.gitignore`, so `target/` and `.mmz/cache/` do not trip it.
if [ "$base_sha" = "$head_sha" ] && [ -z "$CANIBUMP_FORK_DIRTY_OK" ]; then
    dirty=$(git status --porcelain)
    if [ -n "$dirty" ]; then
        echo "fork: HEAD is dirty — the fork would silently drop these changes:" >&2
        sed 's/^/  /' <<<"$dirty" >&2
        echo "  commit them, or name an explicit base, or set CANIBUMP_FORK_DIRTY_OK=1" >&2
        exit 1
    fi
fi

# Reported, never refused. An explicitly named base is a decision, and forking
# off `main` from a branch that is ahead of it is a legitimate one, but it is
# also the shape of the mistake this recipe guards against, so it is said out
# loud with both hashes rather than left to be noticed at integration.
if git merge-base --is-ancestor "$base_sha" "$head_sha"; then
    behind=$(git rev-list --count "$base_sha..$head_sha")
    if [ "$behind" -gt 0 ]; then
        echo "fork: base is $behind commit(s) behind this checkout's HEAD" >&2
        echo "  base $base_sha  HEAD $head_sha — the fork will not carry those commits" >&2
    fi
else
    echo "fork: base is NOT an ancestor of this checkout's HEAD" >&2
    echo "  base $base_sha  HEAD $head_sha — the fork starts on a divergent line" >&2
fi

git worktree add -b "$slug" "$path" "$base_sha"

# Read back rather than assumed: everything above argues about what should land,
# and this is the only check that knows what did.
wt_head=$(git -C "$path" rev-parse HEAD)
if [ "$wt_head" != "$base_sha" ]; then
    echo "fork: the new worktree is not at the base it was created from" >&2
    echo "  requested $base_sha  worktree HEAD $wt_head" >&2
    echo "  left in place for inspection — remove it with \`git worktree remove $path\`" >&2
    exit 1
fi
wt_branch=$(git -C "$path" symbolic-ref --short HEAD)
if [ "$wt_branch" != "$slug" ]; then
    echo "fork: the new worktree is on '$wt_branch', not '$slug'" >&2
    exit 1
fi

# Without the allow the worktree's `.envrc` loads nothing, and every tool the
# gates run comes from the dev shell, so the failure reads as a missing binary
# rather than a missing allow. Then the shell is loaded, not merely authorised:
# the first load pays the flake evaluation and any fetch it needs, and unpaid it
# would land inside the delegate's first gate run, where a long silent command
# is easily mistaken for a hang.
if command -v direnv >/dev/null 2>&1; then
    direnv allow "$path"
    echo "fork: loading the worktree's dev shell (first evaluation, once per worktree)" >&2
    direnv exec "$path" true
else
    echo "fork: direnv is not on PATH — the worktree's dev shell is NOT authorised" >&2
    echo "  run \`direnv allow\` in $path before running anything from the dev shell" >&2
fi

echo
echo "worktree  $path"
echo "branch    $wt_branch"
echo "base      $base_sha ($base)"
echo
echo "pin the base in the delegate's prompt, and have it assert before any work:"
echo "  cd $path"
echo "  git merge-base --is-ancestor $base_sha HEAD && [ \"\$(git rev-parse HEAD)\" = $base_sha ]"
