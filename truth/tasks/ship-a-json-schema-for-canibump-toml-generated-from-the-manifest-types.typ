#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Ship a JSON schema for canibump.toml generated from the manifest types",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("manifest", "tooling", "docs"),
  estimate: (src-files: 2),
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
      docs-match-code: helps[the schema is generated from the manifest types and a test fails once the committed file drifts],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 10),
)

== Problem

+ Open `canibump.toml` in an editor with taplo or tombi.
+ 🔴 There is no schema to point `#:schema` at, so no completion, no hover docs and no squiggle on a misspelled key.


== TO-BE

- 🟢 `canibump schema` prints a JSON Schema generated from the manifest types, doc comments as descriptions.
- 🟢 `schema/canibump.schema.json` is committed, and a test fails when it differs from the generated one.
- 🟢 `just schema` regenerates it; `just install` also writes it to `~/.config/canibump/canibump.schema.json`.
- 🟢 The README shows the `#:schema` line, local file and GitHub raw URL.


== Changes

- `src/schema_tests.rs`: the red tests first, the committed file equals the generated schema, and a fixture copy of `~/.nix/canibump.toml` validates against it while a typo does not.
- `src/schema.rs`: schema-only blocker and step types composed from the condition types, plus `render`.
- `src/condition.rs`, `src/manifest.rs`: derive `JsonSchema` on the raw types; their doc comments become descriptions.
- `src/cli.rs`: the `schema` subcommand, run before the manifest is read.
- `schema/canibump.schema.json`, `Justfile`, `README.md`, `docs/manifest.md`, `outdatty.yaml`: the committed file, `just schema`, the install step, the docs and a drift group.


== Alternatives

- Write the schema by hand: it goes stale the day a key is added.
- Derive from the public `Manifest`: it holds parsed values (`Condition`, `glob::Pattern`), not the TOML shape the editor sees.


== Out of scope

- Editing `~/.nix/canibump.toml`. The `#:schema` line is added there separately.


== Dogfood

`~/.nix/canibump.toml` gets a `#:schema` line and validates against the generated file.

