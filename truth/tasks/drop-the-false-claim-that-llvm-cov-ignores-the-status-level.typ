#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Drop the false claim that llvm-cov ignores the status level",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 1.0,
    ease: 8,
  ),
  difficulty: 1,
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
      sound-codebase: neutral,
      docs-match-code: helps[no comment sends a reader to file an upstream bug that does not exist],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 10),
)

== Problem

+ Here, run `cargo llvm-cov nextest --workspace --no-report --status-level pass`; `.config/nextest.toml` says `fail`.
+ It prints 178 `PASS` lines: the flag wins over the config (cargo-llvm-cov 0.9.0, nextest 0.9.143, and a fresh one-crate repro agrees).
+ 🔴 `.config/nextest.toml`, the `cover` comment in `Justfile`, `CONTRIBUTING.md`, `.mmz/config.yaml` and `outdatty.yaml` say llvm-cov accepts the flag and ignores it.


== TO-BE

- 🟢 The status level is stated once, in `.config/nextest.toml`, where both runners read it; `just test` drops its `--status-level fail`.
- 🟢 No file claims llvm-cov ignores the flag; each comment that argued it says only why the level lives in one place.


== Changes

- `.config/nextest.toml`: the comment.
- `Justfile`: `test` without the flag; the `test` and `cover` comments.
- `.mmz/config.yaml`, `outdatty.yaml`: their comments on the level.
- `CONTRIBUTING.md`, `AGENTS.md`: where they name the flag.


== Rollback

Revert the commit; output is the same either way, since the config already sets `fail`.


== Out of scope

- cratemplate, where the claim came from: fixed in that repo.


== Dogfood

`just test` and `just cover` here print only the summary on a green run, both from the one config line.

