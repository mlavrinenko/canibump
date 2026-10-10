#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Link the integration tests into one binary",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.8,
    ease: 8,
  ),
  difficulty: 1,
  tags: ("tests",),
  estimate: (src-files: 1),
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
      sound-codebase: helps[one link of every dependency for the e2e suite instead of seven],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 11),
)

== Problem

+ `cargo test --no-run` builds one executable per file in `tests/` (seven), each linking every dependency again.
+ 🔴 Each binary costs about 10 to 37 MB per profile, and `just cover` builds the set a second time, instrumented.


== TO-BE

- 🟢 `tests/it/main.rs` declares the seven files as modules, plus `common`; `cargo test --no-run` builds one integration binary.
- 🟢 nextest still runs each test in its own process, and every test keeps its name under the `it` binary.


== Changes

- `tests/*.rs`: moved under `tests/it/`, `mod common;` lines become `use` of the shared module.
- `Cargo.toml`: only if autodiscovery needs a `[[test]]` entry.
- `mmz`, `outdatty`, `linecop` and coverage ignore paths that name `tests/*.rs`: updated.


== Rollback

Move the files back; nothing else depends on the layout.


== Out of scope

- Inline unit tests, which already share one binary per crate.


== Dogfood

`just test` and `just cover` run the suite as one `it` binary.

