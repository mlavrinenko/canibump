#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Print a manifest error once, naming its file and line",
  kind: "fix",
  priority: ice(
    impact: 6,
    confidence: 0.95,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("manifest", "cli", "found"),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      one-command: helps[a broken manifest reads as one line that says which file and line to open],
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: neutral,
      offline-readers: neutral,
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: helps[an e2e test pins the error format, so the duplicate cannot come back],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 10),
)

== Problem

+ `printf '[[blocker]]\nid = "x"\nnote = "n"\nstate = github\n' > canibump.toml && canibump`
+ 🔴 It prints the TOML parse error twice, joined by `: `. `ManifestError::Toml` is `#[error("{0}")]` over a `#[from]` field, so the toml error is both the message and its `source()`, and `main` prints the whole chain with `{error:#}`.
+ 🔴 Neither copy names the file. Under `canibump -m some/canibump.toml` in a cron mail or CI log, `TOML parse error at line 4` points at no file.
+ Add `foo = 1` to a valid blocker and run `canibump`.
+ 🔴 It prints `` blocker `x`: unknown field `foo`, expected `state` ``: no file, no line. The blocker was deserialized from a TOML table that had a span; re-reading it as a bare `Table` threw the span away.


== Environment

- `canibump` built from `main` at the commit that filed this, on Linux.


== TO-BE

- 🟢 Every manifest error prints exactly once.
- 🟢 Every manifest error names the manifest path as given on the command line: a syntax error as `canibump: canibump.toml: TOML parse error at line 4, column 9` plus toml's caret snippet, once.
- 🟢 A blocker error names the line the blocker starts on: `` canibump: canibump.toml:1: blocker `x`: unknown field `foo`, expected `state` ``. The same holds for a missing `id`, a duplicate id and a bad step.
- 🟢 Other error chains, such as a cache I/O error with its OS reason, still print in full.


== Changes

- `tests/cli.rs`: the red e2e tests first. A syntax error prints its message once and names the manifest path. An unknown key in a blocker names `path:line`.
- `src/manifest.rs`: `ManifestError::Toml` stops exposing the toml error twice (`#[error(transparent)]`, or a message field with no `#[source]`). `Manifest::load` puts the path on every error it returns. `RawManifest.blockers` becomes `Vec<toml::Spanned<Table>>`, so each blocker error carries the line its table starts on, computed from the span's byte offset.
- `src/manifest_tests.rs`: `Manifest::parse` reports the right line for an error in the second blocker.
- `README.md` / `docs/manifest.md`: refresh any quoted error text.


== Out of scope

- Per-key spans inside a blocker, such as the exact line of `foo`. The blocker's first line is enough to find it, and per-key spans need `toml_edit`.
- Cache and network errors, which already name their file or item.


== Dogfood

A typo in `~/.nix/canibump.toml` reads as one line naming the file and the blocker's line.

