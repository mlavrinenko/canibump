#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Parse canibump.toml into typed blockers",
  kind: "feat",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("manifest",),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: helps[one reference grammar, `github:owner/repo#N`, that later forms extend],
      whole-chain: neutral,
      honest-state: neutral,
      offline-readers: neutral,
      typed-not-scripted: helps[each condition parses into its own type, and an unknown key is an error naming the blocker],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 10),
  links: parent(
    "check-github-issue-and-pr-state-from-a-canibump-toml-manifest.typ",
  ),
)

== Problem

+ Write a `canibump.toml` with a `[[blocker]]` whose condition is `state = "github:serde-rs/serde#2590"`.
+ 🔴 Nothing reads it: the crate still only greets.


== TO-BE

- 🟢 `Manifest::parse` turns the file into blockers with an id, a note and a typed condition.
- 🟢 A missing condition, two conditions, an unknown key, a duplicate id or a malformed reference fails naming the blocker.


== Changes

- `src/manifest.rs`: the red tests first, then `Manifest`, `Blocker`, `Condition` and the dispatch on the condition key.
- `src/refs.rs`: `ItemRef` and its parser.
- `src/lib.rs`: drop the template's greeter, export the modules.


== Alternatives

- A serde untagged enum: its error says only "did not match any variant".


== Out of scope

- Conditions other than `state`. Their milestones add them.


== Dogfood

`~/.nix/canibump.toml` parses with it once milestone 6 writes it.

