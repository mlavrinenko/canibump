#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Fork agent worktrees with just fork and land them with just integrate",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("tooling",),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: neutral,
      offline-readers: neutral,
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: helps[a delegated branch lands only on a fresh gate, from the base it was forked off],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 10),
)

== Problem

+ Create a worktree for a delegated agent with `git worktree add`, then run its gate there.
+ 🔴 Its `.envrc` is not allowed, so every dev-shell tool reads as missing.
+ 🔴 Nothing checks the worktree sits on the intended base, and landing it is a hand-typed fast-forward that asserts no gate.


== TO-BE

- 🟢 `just fork <slug> [base]` makes `../canibump-<slug>` on a new branch, refuses a dirty HEAD, reads the base back, allows and loads the dev shell, and prints the line to pin in the agent's prompt.
- 🟢 `just integrate <branch>` refuses a dirty worktree, a non-fast-forward, a task left `wip` and a gate that is not fresh, then fast-forwards, carries the gate records and lcov over, and removes the worktree.
- 🟢 Nothing in them names one agent tool; a person runs them the same way.


== Changes

- `.just/scripts/fork.sh`, `.just/scripts/integrate.sh`: ported from ~/projects/home/mindtape/.just/scripts/, minus its dev-mt build; `CANIBUMP_` env names; `src/` for `crates/`.
- `Justfile`: the two recipes.
- `AGENTS.md`: delegate through them.


== Rollback

Revert the commit; worktrees made by it are plain git worktrees, removed with `git worktree remove`.


== Out of scope

- The mindtape originals.
- A direnv whitelist in ~/.nix: it would trust any branch's `.envrc`.


== Dogfood

The next delegated task here forks and lands through these two recipes.

