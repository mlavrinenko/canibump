#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Keep the publish dry run from freezing the dev build after a release",
  kind: "fix",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("tooling",),
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
      sound-codebase: helps[`just test` after a release tests the code in `src/`, not the packaged copy],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 13),
)

== Problem

+ Run `cargo package --allow-dirty`, as `just release`'s hook does through `cargo publish --dry-run`.
+ Edit a doc comment in `src/manifest.rs`, then run `cargo build -v`.
+ 🔴 It prints `Fresh canibump v0.2.0`, and the binary keeps the old text. The verify build rewrote `target/debug/.fingerprint/canibump-*/dep-lib-canibump` to list `package/canibump-0.2.0/src/*.rs`, so cargo checks that copy, never `src/`.


== Environment

- canibump 0.2.0, right after its release, cargo 1.98.0 on Linux, with and without `RUSTC_WRAPPER`.


== TO-BE

- 🟢 The hook's publish dry run builds in its own target dir, so a release leaves the dev build's fingerprints alone.
- 🟢 The first `cargo build` after a release rebuilds an edited `src/`.


== Changes

- `tests/it/release_hook.rs`: the red test, the publish dry run gets a `--target-dir` of its own.
- `scripts/release-hook.sh`: pass `--target-dir target/package-verify`.


== Out of scope

- Cargo itself. Its verify build should not share the root crate's fingerprint, and the issue is drafted for the maintainer to file.


== Dogfood

The next `just release` leaves `just test` testing this checkout's `src/`.

