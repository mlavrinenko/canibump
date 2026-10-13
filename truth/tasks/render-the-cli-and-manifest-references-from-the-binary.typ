#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Render the CLI and manifest references from the binary",
  kind: "docs",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("docs", "cli", "manifest"),
  estimate: (src-files: 4),
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
      docs-match-code: helps[a new flag, manifest key or report field reaches the docs with no hand edit],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 13),
  links: parent("publish-the-docs-as-a-www-site-built-from-real-runs.typ")
    + depends-on(
      "capture-canibump-runs-for-the-docs-against-a-mock-upstream.typ",
    ),
)

== Problem

+ Compare `canibump --help` with the README's Commands and Exit codes sections.
+ 🔴 The README retells each flag and code in other words, and nothing fails when one changes.
+ Compare `canibump schema` with `docs/manifest.md`.
+ 🔴 The manifest keys are described twice, in the doc comments and in prose.


== TO-BE

- 🟢 The CLI page shows the captured `--help` of `canibump` and of each subcommand its help lists.
- 🟢 The manifest page renders every table and key of `canibump schema`, with its type and doc comment.
- 🟢 The JSON page renders `canibump schema report` and `canibump schema state` the same way.


== Source of truth

- `src/cli.rs`: the commands and flags.
- `canibump schema`, `schema report`, `schema state`.


== Changes

- `www/generate.sh`: capture each `--help` and each schema.
- `www/utils/schema-ref.typ`: render a JSON Schema as tables.
- `www/content/cli.typ`, `www/content/manifest.typ`, `www/content/json.typ`: the reference pages.


== Out of scope

- A hidden JSON dump of the clap tree. `--help` already says everything a reader needs.


== Dogfood

The site's reference pages render this checkout's CLI and schemas.

