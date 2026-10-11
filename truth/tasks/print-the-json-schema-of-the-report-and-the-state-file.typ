#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Print the JSON Schema of the report and the state file",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("cli", "cache"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: neutral,
      offline-readers: helps[a tool reading the state file checks it against a schema the binary prints],
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the README links schemas generated from the types, and a test fails when they lag],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 11),
)

== Problem

+ Look for a description of `canibump --json` or of the state file.
+ 🔴 docs/manifest.md shows one hand-written state file line; the shape of `--json` is nowhere, though both are what other tools read.


== TO-BE

- 🟢 `canibump schema report` and `canibump schema state` print JSON Schemas of `--json` and the state file; bare `canibump schema` still prints the manifest's.
- 🟢 Both are committed under `schema/`, `just schema` rewrites all three, and a test fails while one lags.
- 🟢 A real `--json` run and its state file validate against them.


== Changes

- `tests/schema.rs`: the red tests: committed equals generated, the subcommand prints them, a real run validates.
- `src/cli.rs`: `schema [manifest|report|state]`.
- `src/schema.rs`: render the two new schemas.
- `src/report.rs`, `src/cache.rs`, `src/outcome.rs`: derive `JsonSchema`, timestamps as strings.
- `schema/report.schema.json`, `schema/state.schema.json`: generated.
- `README.md`, `docs/manifest.md`, `CHANGELOG.md`: link them.


== Alternatives

- Committed files written by a test, no CLI. A tool reading the state file then can't ask its own binary for the matching schema.
- A second subcommand per schema. Three words for one idea; `schema <what>` reads better.


== Out of scope

- Versioning the state file format. `version` already exists; the schema documents it.


== Dogfood

`canibump schema state` validates this repo's own state file after `just canibump`.

