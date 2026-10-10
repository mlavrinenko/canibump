#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Replace just fork and just integrate with worktrunk",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
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
      sound-codebase: helps[the worktree tooling is worktrunk plus a short in-repo config, not a 301-line port that drifts from mindtape's],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 11),
)

== Problem

+ `.just/scripts/fork.sh` and `integrate.sh` (301 lines), plus the `fork` and `integrate` recipes, ported mindtape's delegation here on 2026-10-10.
+ 🔴 mindtape replaced its copy with worktrunk (commit deacb817). Its task `configure-worktrunk-in-the-repo-so-a-fresh-clone-needs-no-host-setup` moves every hook into the repo, because hooks in ~/.nix don't travel with a clone. This port is now a second copy that drifts.


== TO-BE

- 🟢 Same shape as mindtape's result. Finish that task first and copy its files.
- 🟢 `worktrunk` is in the dev shell, `.config/wt.toml` holds the hooks, and `just fork <slug>` / `just land` wrap `wt` with `--no-squash --no-commit --yes`.
- 🟢 The pre-merge gate is `just check`, then the wip check. pre-remove carries `.mmz/cache/*.yaml` and a fresh lcov into the primary checkout.
- 🟢 Nothing names a specific agent tool, and a clone with nix and direnv alone works.


== Changes

- `.just/scripts/fork.sh`, `.just/scripts/integrate.sh`: deleted. mindtape's `wt-tasks-done.sh` and `wt-carry-gate.sh` replace them.
- `.config/wt.toml`, `flake.nix`: as in mindtape, minus `just build-mt`. canibump has no dev `mt` build.
- `Justfile`: `fork` and `land` replace `fork` and `integrate`.
- `outdatty.yaml`: `dev-docs` sources follow the scripts. AGENTS.md says how to delegate.


== Rollback

Revert the commit; the scripts come back unchanged.


== Out of scope

- Doing this before mindtape's task lands: copying a design that hasn't been tested yet means two repos to fix.


== Dogfood

The next delegated task here forks with `just fork` and lands with `just land`.
