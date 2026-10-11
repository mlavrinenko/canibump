#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Accept days and weeks in the settings ttl",
  kind: "fix",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("manifest", "found"),
  estimate: (src-files: 1),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: neutral,
      offline-readers: neutral,
      typed-not-scripted: helps[a ttl of `1d` reads the way a human writes it],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[docs/manifest.md names the units the ttl takes],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 11),
)

== Problem

+ Write `[settings]` with `ttl = "1d"` (or `"2w"`) and run `canibump status`.
+ 🔴 It exits 1 with ``canibump: .../canibump.toml: settings.ttl: `1d` is not a duration like "6h" or "30m"``. Only hour-based values like `"24h"` parse, because `SignedDuration` refuses units above hours.


== Environment

- `canibump` built from `main` at the commit that filed this, on Linux.


== TO-BE

- 🟢 `ttl` takes weeks, days, hours, minutes and seconds, alone or combined: `"1d"` = 24h, `"2w"` = 336h, `"1d12h"` = 36h. A day counts as 24 hours, which is sound for a freshness window.
- 🟢 Months and years stay refused, as they have no fixed length, and the error names the units it takes.
- 🟢 docs/manifest.md and the manifest schema say the same.


== Changes

- `src/manifest_tests.rs`: the red tests first, `1d`, `2w`, `1d12h`, and a `1mo` error that names the units.
- `src/manifest.rs`: parse the ttl as a `jiff::Span` and convert it with days as 24 hours; reword the error.
- `docs/manifest.md`, `schema/canibump.schema.json` (via `just schema`): name the accepted units.
- `CHANGELOG.md`: refine the settings bullet under `[Unreleased]`.


== Out of scope

- Calendar units. A month has no fixed number of hours.


== Dogfood

Not applicable, `canibump.toml` keeps the default ttl.

