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
  status: done(2026, 10, 11),
)

== Problem

+ `cargo test --no-run` builds one executable per file in `tests/` (seven), each linking every dependency again.
+ 🔴 Each binary costs about 10 to 37 MB per profile, and `just cover` builds the set a second time, instrumented.


== TO-BE

- 🟢 `tests/it/main.rs` declares the seven files as modules, plus `common`; `cargo test --no-run` builds one integration binary.
- 🟢 nextest still runs each test in its own process, and every test keeps its name under the `it` binary.


== Changes

- `tests/{cli,color,completions,exit_codes,package,schema,status_id}.rs`: moved to `tests/it/`. A new `tests/it/main.rs` declares them, plus `mod common;`, and cargo autodiscovers it as the `it` target.
- `tests/common/mod.rs`: moved to `tests/it/common/mod.rs`. The four files with their own `mod common;` switch to `use crate::common::...`.
- `tests/fixtures/`: stays put. `tests/schema.rs` reaches it through the manifest dir, so it needs no change.
- CONTRIBUTING.md: `tests/cli.rs` becomes `tests/it/cli.rs`.
- The `tests/**` globs in `.mmz/config.yaml`, the `crap` excludes in the Justfile and `.mindtape/config.toml` still match, so leave them.
- Check: 206 tests before and after (`just test`), and 1 integration executable instead of 7 from `cargo test --no-run`.

== Rollback

Move the files back; nothing else depends on the layout.


== Out of scope

- Inline unit tests, which already share one binary per crate.


== Dogfood

`just test` and `just cover` run the suite as one `it` binary.

