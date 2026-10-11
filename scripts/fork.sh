#!/usr/bin/env bash
# Usage: fork.sh SLUG [BASE]. Makes ../canibump-SLUG on a new branch SLUG off
# BASE (default HEAD), allows its dev shell and reads the base back: a fork off
# the wrong base passes its own gate while dropping the real base's work. The
# last lines are for the delegate's prompt, since only the caller knows the base.
set -euo pipefail
slug=${1:?usage: fork.sh SLUG [BASE]}
base=${2:-HEAD}

if ! [[ "$slug" =~ ^[a-z0-9][a-z0-9-]*$ ]]; then
    echo "fork: slug '$slug' must be lowercase letters, digits and dashes, starting with a letter or digit" >&2
    exit 1
fi

# A sibling of the main checkout, even when forking from another worktree. Never
# a child: a nested worktree puts a second src/ and truth/tasks/ under every
# path-walking tool, so `mt check` and jscpd see each file twice.
main_root=$(dirname "$(git rev-parse --path-format=absolute --git-common-dir)")
fork_root="${CANIBUMP_FORK_ROOT:-$(dirname "$main_root")}"
path="$fork_root/$(basename "$main_root")-$slug"

if ! base_sha=$(git rev-parse --verify --quiet "$base^{commit}"); then
    echo "fork: base '$base' does not resolve to a commit" >&2
    exit 1
fi
head_sha=$(git rev-parse --verify HEAD)

# `git worktree add` copies commits, never the working tree, so a dirty HEAD
# would hand the delegate a base missing the changes in front of you. Untracked
# files count; .gitignore is honoured. An explicit other base skips this.
if [ "$base_sha" = "$head_sha" ] && [ -z "${CANIBUMP_FORK_DIRTY_OK:-}" ]; then
    dirty=$(git status --porcelain)
    if [ -n "$dirty" ]; then
        echo "fork: HEAD is dirty, the fork would silently drop these changes:" >&2
        printf '%s\n' "$dirty" | sed 's/^/  /' >&2
        echo "  commit them, or name an explicit base, or set CANIBUMP_FORK_DIRTY_OK=1" >&2
        exit 1
    fi
fi

# Reported, not refused: an older base can be deliberate, but it is also the
# mistake this script guards against.
if git merge-base --is-ancestor "$base_sha" "$head_sha"; then
    behind=$(git rev-list --count "$base_sha..$head_sha")
    if [ "$behind" -gt 0 ]; then
        echo "fork: base is $behind commit(s) behind this checkout's HEAD" >&2
        echo "  base $base_sha  HEAD $head_sha, the fork will not carry those commits" >&2
    fi
else
    echo "fork: base is NOT an ancestor of this checkout's HEAD" >&2
    echo "  base $base_sha  HEAD $head_sha, the fork starts on a divergent line" >&2
fi

git worktree add -b "$slug" "$path" "$base_sha"

wt_head=$(git -C "$path" rev-parse HEAD)
if [ "$wt_head" != "$base_sha" ]; then
    echo "fork: the new worktree is not at the base it was created from" >&2
    echo "  requested $base_sha  worktree HEAD $wt_head" >&2
    echo "  left in place for inspection, remove it with \`git worktree remove $path\`" >&2
    exit 1
fi
wt_branch=$(git -C "$path" symbolic-ref --short HEAD)
if [ "$wt_branch" != "$slug" ]; then
    echo "fork: the new worktree is on '$wt_branch', not '$slug'" >&2
    exit 1
fi

# Load the shell now: the first evaluation is slow and would look like a hang
# inside the delegate's first gate run.
if command -v direnv >/dev/null 2>&1; then
    direnv allow "$path"
    echo "fork: loading the worktree's dev shell (first evaluation, once per worktree)" >&2
    direnv exec "$path" true
else
    echo "fork: direnv is not on PATH, the worktree's dev shell is NOT authorised" >&2
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
