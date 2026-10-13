#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Split CI by toolchain",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
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
      sound-codebase: helps[a docs failure and a code failure show as different jobs, and each job pays only for its own toolchain],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 13),
)

== Problem

+ Push to main and open the `ci.yml` run.
+ 🔴 `check`, `coverage` and `deny` each enter the one dev shell, so each realizes tola, typst, tinymist, cargo-mutants and rust-analyzer it never runs, and each compiles the crate from nothing, since no job caches `target/`.


== TO-BE

- 🟢 `ci.yml` runs `code` (`just check-code` in `nix develop .#rust`) and `docs` (`just check-docs` in `nix develop .#docs`) as separate jobs; `just check` runs both locally.
- 🟢 Every Nix job restores its shell's store and cargo's `target/` from the Actions cache, keyed per shell.


== Changes

- `flake.nix`: `devShells.rust` and `devShells.docs`; `default` keeps every tool.
- `Justfile`: `check-code` and `check-docs`, which `check` runs.
- `.github/actions/nix/action.yml`: Nix install, store cache and cargo cache for one shell.
- `.github/workflows/ci.yml`: a `code` and a `docs` job in place of `check`; every Nix job uses the action.
- `AGENTS.md`, `CONTRIBUTING.md`: the two halves and their shells.


== Rollback

- Revert the commit. No file format or recipe a user runs changes.


== Out of scope

- Reusing CI's release binaries in `release.yml`. Artifacts expire, and a re-run of an old tag must still build.
- `pages.yml` and `release.yml`. `truth/tasks/deploy-the-site-on-release-and-trust-cis-pass-on-the-tag.typ` holds them.
- MindTape in CI. `truth/tasks/put-mindtape-in-the-dev-shell-so-ci-can-check-the-tasks.typ` holds it.


== Dogfood

Not applicable. It changes CI, not canibump's checks.
