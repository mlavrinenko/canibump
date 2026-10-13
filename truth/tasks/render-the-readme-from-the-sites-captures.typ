#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Render the README from the site's captures",
  kind: "docs",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("docs", "tooling"),
  estimate: (src-files: 3),
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
      docs-match-code: helps[the README's example is a capture, and a gate fails when the file drifts from its source],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 13),
  links: parent("publish-the-docs-as-a-www-site-built-from-real-runs.typ")
    + depends-on("write-the-docs-pages-from-captured-runs.typ"),
)

== Problem

+ Change a status word in `src/report.rs`, then run `just check`.
+ 🔴 The README keeps printing the old word. outdatty flags it, and someone has to retype it.


== TO-BE

- 🟢 `docs/src/readme.typ` renders README.md through typlite, from the captures the site shows.
- 🟢 The README says what canibump does, shows one manifest and its run, and links the site for the rest.
- 🟢 `docs-md-check` fails when README.md differs from a fresh render.


== Source of truth

- The captures under `www/generated/`.


== Changes

- `docs/src/readme.typ`, `docs/generate-md.sh`: the source and its render.
- `README.md`: rendered.
- `scripts/docs-md-check.sh`: the drift gate.
- `Justfile`, `.mmz/config.yaml`: `docs-md`, `docs-md-check`.


== Out of scope

- CONTRIBUTING.md and AGENTS.md. Agents read them whole, and they show no runs.


== Dogfood

The repo's own README is rendered this way.

