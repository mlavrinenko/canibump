#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Show statuses in color on a terminal, with the note on dead lines",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("cli",),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      one-command: helps[the bumpable and dead lines stand out of a long list at a glance],
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: helps[a dead line names the workaround to rethink, as a bumpable one names the one to drop],
      offline-readers: neutral,
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 10),
)

== Problem

+ `canibump` against a manifest with twenty blockers, one of them bumpable and one dead.
+ 🔴 Every status prints in the same plain text, so the two lines that need you hide among eighteen waiting ones.
+ 🔴 The dead line omits the blocker's `note`, though dead asks you to rethink exactly that workaround. Only bumpable lines show it.
+ 🔴 A stale line prints a raw instant, `(stale, checked 2026-10-10T06:00:00Z)`, and leaves the arithmetic to the reader.
+ Column width uses `str::len`, which counts bytes, so a non-ASCII id misaligns the rest.


== TO-BE

- 🟢 On a terminal, statuses are colored: bumpable green, dead magenta, failed red, waiting dimmed. `--color auto|always|never` (default `auto`) and `NO_COLOR` decide, and piped output stays byte-for-byte plain. `--json` never colors.
- 🟢 A dead line appends its note, like a bumpable one.
- 🟢 A stale line reads `(stale, checked 7h ago)`.
- 🟢 Widths count chars, not bytes.


== Changes

- `tests/cli.rs`: the red e2e tests first. Piped output has no escape codes, `--color always` has them, and a dead line shows its note.
- `src/report.rs` (+ inline tests): styles through `anstyle`, the relative age, the dead note, char widths.
- `src/cli.rs`: the `--color` flag, through `anstream`, which clap already pulls in, so no new crate.
- `README.md`, `docs/manifest.md`, `www/index.html`: refresh sample output that changes.


== Alternatives

- `owo-colors` or `colored`: a new dependency for what `anstyle`/`anstream` already in the tree do, and without clap's TTY and `NO_COLOR` handling.


== Out of scope

- Sorting or grouping lines: manifest order is the user's own order.


== Dogfood

`just canibump` in `~/.nix` shows the llama.cpp line dimmed while it waits and green once it lands.

