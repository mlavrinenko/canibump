#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Make the prose slots of the templates raw and unescape the filed tasks",
  kind: "fix",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("tooling", "found"),
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
      sound-codebase: helps[no filed task carries escaped markup],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 15),
)

== Problem

+ Run `mt add -t tweak --describe` or `mt add -t finding --describe` and read the slot table. The slots `why`, `context` and `observation` are `kind: "text"`, and their examples show code spans.
+ 🔴 `mt add` escapes a `text` slot, so a code span typed there is filed as a backslash before each backtick and each `<`, `*`, `#` or `@` inside it. 3 task files already carry that damage, among them `truth/tasks/keep-docs-out-of-the-source-dict.typ`, `truth/tasks/rerun-outdatty-check-when-a-watched-source-changes.typ` and `truth/tasks/write-the-state-file-through-a-unique-temp-name.typ`.

== Environment

- `mt` 0.3.0 on Linux, in the `canibump` checkout at its `main` branch.

== TO-BE

- 🟢 The prose slots of the tweak and finding templates are `kind: "raw"`, so a code span typed into them files as a code span.
- 🟢 The listed task files hold their code spans and plain characters again, and `mt check` is still green.

== Changes

- `.mindtape/templates/tweak.typ`: `why` becomes `kind: "raw"`.
- `.mindtape/templates/finding.typ`: `context` and `observation` become `kind: "raw"`.
- The task files with escaped markup: drop the backslash before each backtick, `<`, `*`, `#` and `@`, and find them with `rg -l` over the task root.
- `mindtape` commits 98ce1e44 and 64dfa4f9 show the same fix in its own repo.

== Out of scope

- Slots that hold a number or a short label, such as `estimate`. Escaping leaves those unchanged.

== Dogfood

Not applicable. The change repairs this repo's own templates and task files.
