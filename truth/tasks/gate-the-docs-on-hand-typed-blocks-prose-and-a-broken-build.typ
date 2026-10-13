#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Gate the docs on hand-typed blocks, prose and a broken build",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
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
      docs-match-code: helps[a block typed into a page, or a page that stops building, fails just check],
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

+ Type a made-up `console` block into a page under `www/content/`, then run `just check`.
+ 🔴 It passes. Only review would catch the block.


== TO-BE

- 🟢 `just check` runs `docs-check`, which builds the site, validates its links and fails on any capture error.
- 🟢 `check-doc-blocks` fails on a code block written into a page, unless a `// docs:hand-typed <reason>` comment sits above it.
- 🟢 `check-doc-prose` fails on history words, an all-caps word and an em dash in a page's prose.
- 🟢 mmz skips each while its inputs are unchanged.


== Changes

- `scripts/doc-blocks-lib.sh`: the typstyle-AST reader, ported from mindtape.
- `scripts/check-doc-blocks.sh`, `scripts/check-doc-prose.sh`: the two gates.
- `Justfile`: `docs-check`, `check-doc-blocks`, `check-doc-prose` in `check`.
- `.mmz/config.yaml`: their rules and a `www` scope.
- `flake.nix`: typstyle.


== Rollback

Drop the arms from `check` and revert the commit.


== Out of scope

- Checking inline command spans against the CLI parser.


== Dogfood

`just check` gates this repo's own site.

