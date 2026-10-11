#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Rewrite the README in the maintainer's voice",
  kind: "docs",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("docs",),
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
      sound-codebase: neutral,
      docs-match-code: helps[a README short enough to keep in step with the code],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 11),
)

== Problem

+ Read `README.md` and `docs/manifest.md`.
+ 🔴 170 and 194 lines. The example notes name the maintainer's own `apps/llama-cpp-16gb.nix`, the manifest doc says `It guards a patch in the author's NixOS flake`, a paragraph explains the terminal colors, and the exit codes and request counts are told twice.
+ Read `CHANGELOG.md`.
+ 🔴 131 lines of dev log for a crate that never shipped: `/pulls/N` call counts, a `Changed` and a `Fixed` section against no release.


== TO-BE

- 🟢 The README says what canibump does and why in the maintainer's words, then the manifest and output examples, the statuses, the conditions, the commands, the exit codes, `--json` and editor support.
- 🟢 `docs/manifest.md` is the full reference, `docs/architecture.md` says how a run works in five steps.
- 🟢 The changelog reads as the first release's notes, for the person running the tool.
- 🟢 No doc or fixture names the maintainer's files.


== Source of truth

- `canibump --help` and `src/report.rs`: the commands and the exit codes.
- `src/run.rs`, `src/provider/`: what a run does.


== Changes

- `README.md`, `docs/manifest.md`, `CHANGELOG.md`: rewritten.
- `docs/architecture.md`: new.
- `tests/fixtures/nix-canibump.toml`: the note names `pkgs/llama-cpp.nix`.
- `outdatty.yaml`: `docs/architecture.md` follows `src/**`.


== Out of scope

- The Justfile, CONTRIBUTING.md and www/. A parallel task cleans those.


== Dogfood

Not applicable: docs only.

