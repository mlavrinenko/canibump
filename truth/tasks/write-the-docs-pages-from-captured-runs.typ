#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Write the docs pages from captured runs",
  kind: "docs",
  priority: ice(
    impact: 8,
    confidence: 0.8,
    ease: 4,
  ),
  difficulty: 2,
  tags: ("docs",),
  estimate: (src-files: 30),
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
      docs-match-code: helps[`docs/*.md` fold into pages whose every example ran],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 13),
  links: parent("publish-the-docs-as-a-www-site-built-from-real-runs.typ")
    + depends-on(
      "render-the-cli-and-manifest-references-from-the-binary.typ",
      "gate-the-docs-on-hand-typed-blocks-prose-and-a-broken-build.typ",
    ),
)

== Problem

+ Read `docs/manifest.md`, `docs/scan.md` and `docs/architecture.md`.
+ 🔴 One long page per area, hand-typed examples, and a reader who wants to drop a patch once a PR merges has to read the whole manifest grammar first.


== TO-BE

- 🟢 Pages answer a reader's job: wait on a PR, a release, a package or a chain, mark workarounds, catch forgotten ones, run in CI, read results offline.
- 🟢 Each shows a fixture manifest and the run it produced, and states each fact once.
- 🟢 The voice is a peer explaining the tool. No filler, no history, no em dashes.
- 🟢 `docs/*.md` are gone, their facts on the site.


== Source of truth

- `src/eval.rs`, `src/scan.rs`, `src/report.rs`, `src/provider/`: the behaviour each page states.


== Changes

- `www/content/*.typ`: the pages.
- `www/examples/*/`: a fixture per capture.
- `www/generate.sh`: their captures.
- `www/utils/site.typ`: the nav.
- `docs/manifest.md`, `docs/scan.md`, `docs/architecture.md`: deleted.
- `outdatty.yaml`, `outdatty.lock`: the pages follow `src/**`.


== Out of scope

- The README. Its own task.


== Dogfood

The site documents this checkout's binary.

