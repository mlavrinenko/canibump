#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Track this repo's own upstream workarounds in canibump.toml",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.6,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("dogfood", "tooling"),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: helps[the repo exercises a second real manifest besides `~/.nix`, across more providers],
      whole-chain: neutral,
      honest-state: neutral,
      offline-readers: neutral,
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: helps[a test loads the repo's own manifest, so a grammar change that breaks real use fails `just check`],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 10),
)

== Problem

+ AGENTS.md: the tool should use itself where it applies. `ls canibump.toml` in this repo.
+ 🔴 There is none, yet the repo carries workarounds that wait on upstream. `flake.nix` points cargo-llvm-cov at a standalone LLVM because nixpkgs' rustc ships no `llvm-tools-preview`. The Justfile and `.config/nextest.toml` move `--status-level` into config because `cargo llvm-cov nextest` accepts the flag and ignores it.
+ Each such comment explains a workaround, but nothing says when it can go.


== TO-BE

- 🟢 A `canibump.toml` at the repo root, with `#:schema ./schema/canibump.schema.json`, lists every workaround whose upstream issue or PR exists. Each note names the file and line to change.
- 🟢 Every reference is a real upstream item, verified with `gh` or by opening it. A workaround with no upstream ticket is named in the final report, never invented.
- 🟢 `just canibump` runs it (network-bound, outside `just check`), and a test asserts the file loads through `Manifest::load`.


== Changes

- `canibump.toml`: the blockers.
- `Justfile`: the `canibump` recipe, which runs `cargo run --quiet --` against the repo's own manifest.
- `tests/schema.rs` or `tests/cli.rs`: the repo manifest loads and validates against the schema.
- `CONTRIBUTING.md`, `AGENTS.md`: one line each on keeping the manifest current; `outdatty.yaml` if a group should couple it.


== Rollback

- Delete `canibump.toml` and the recipe; nothing else reads them.


== Out of scope

- Fixing the workarounds themselves.


== Dogfood

This is the dogfood: `just canibump` in this repo reports when each workaround can go.

