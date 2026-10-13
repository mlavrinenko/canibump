#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Publish the docs as a www site built from real runs",
  priority: ice(
    impact: 8,
    confidence: 0.8,
    ease: 3,
  ),
  difficulty: 4,
  kind: "milestone",
  tags: ("docs", "tooling"),
  estimate: (src-files: 5),
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
      docs-match-code: helps[every output on the site is a capture of the binary, every reference is generated from the code],
    ),
  ),
  assignee: "agent",
  status: wip(2026, 10, 13),
)

== Problem

+ Read `README.md`, `docs/manifest.md` and `docs/scan.md`.
+ 🔴 Every `console` block is typed by hand. `bumpable  serde-pin  github:serde-rs/serde#2590 (110af31) is in v1.0.185` matches no run, and nothing fails when the report changes. outdatty only asks a person to look.
+ Run `canibump --help`.
+ 🔴 Its flags and exit codes are retold in the README's Commands and Exit codes sections, in other words.
+ Look for a docs site.
+ 🔴 There is none. No search, no page per topic, no Markdown copy an agent can fetch.


== TO-BE

- 🟢 `www/` builds a tola site, searchable and deployed to GitHub Pages, that walks a reader from the first blocker to CI.
- 🟢 Every terminal block on it is a capture of the binary under test, and every reference is generated from the code.
- 🟢 README.md is rendered from the same captures.
- 🟢 Gates fail on a hand-typed block, on history words and on drift.
- 🟢 Each page reads like a peer explaining the tool, and says each fact once.


== Changes

- `www/`: the site, its captures and fixtures.
- `docs/src/readme.typ`: the README source.
- `scripts/`: the doc gates.
- `docs/manifest.md`, `docs/scan.md`, `docs/architecture.md`: folded into the site.


== Out of scope

- A framework package shared with mindtape. Copy first, extract once both sites settle.
- Checking inline command spans against the CLI parser.


== Dogfood

`just docs-check` builds the site from this checkout's binary, and the Pages workflow publishes it from main.

