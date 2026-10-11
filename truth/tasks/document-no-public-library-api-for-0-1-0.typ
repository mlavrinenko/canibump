#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Document no public library API for 0.1.0",
  kind: "fix",
  priority: ice(
    impact: 6,
    confidence: 0.85,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("tooling", "found"),
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
      sound-codebase: helps[the crate stops promising semver on types nobody asked to promise],
      docs-match-code: helps[the crate doc says the library is not a stable API],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 11),
)

== Problem

+ Run `cargo doc --no-deps` and list `target/doc/canibump`.
+ 🔴 It holds 14 public modules: `auth`, `cache`, `cli`, `condition`, `eval`, `manifest`, `outcome`, `package`, `provider`, `refs`, `report`, `run`, `schema` and `version`. They expose provider traits, `Registry`, the `clap` types of `cli::Cli`, `glob::Pattern` in `Within::Tags`, `ManifestError::Toml(toml::de::Error)` and `jiff` types, and the public enums are not `#[non_exhaustive]`, so every one is a 0.1.0 promise.
+ Only `tests/it/schema.rs` (`schema::render*`, `Manifest::load`) and `src/main.rs` (`cli::Cli`, `cli::run`, `report::Exit`) use the library.


== Environment

- `canibump` built from `main` at the commit that filed this, on Linux.


== TO-BE

- 🟢 The crate doc says the library exists for the binary and its tests and is not a stable API.
- 🟢 Every module is `#[doc(hidden)]`, and a module nothing outside the crate uses is `pub(crate)` where that costs no edit inside it, so `cargo doc` lists no API.


== Changes

- `src/lib.rs`: the crate doc, `pub(crate)` for the unused modules, `#[doc(hidden)] pub` for `cli`, `manifest`, `report` and `schema`.
- `CHANGELOG.md`: a clause on the library under `[Unreleased]` if a bullet fits.


== Out of scope

- Item visibility inside `eval`, `condition` and `provider/*`, which other work edits in parallel.
- A real library API. Nobody asked for one.


== Dogfood

Not applicable.

