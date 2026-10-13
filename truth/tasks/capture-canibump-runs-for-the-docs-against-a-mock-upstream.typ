#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Capture canibump runs for the docs against a mock upstream",
  kind: "dev",
  priority: ice(
    impact: 8,
    confidence: 0.8,
    ease: 4,
  ),
  difficulty: 3,
  tags: ("docs", "tooling"),
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
      docs-match-code: helps[a page cannot show a command that did not run, or output it did not print],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 13),
  links: parent("publish-the-docs-as-a-www-site-built-from-real-runs.typ")
    + depends-on("scaffold-the-www-docs-site-from-mindtapes-framework.typ"),
)

== Problem

+ Read the `console` blocks in `README.md`.
+ 🔴 Each is typed by hand, and a run needs the network, so nothing can check one against the binary.


== TO-BE

- 🟢 `www/generate.sh` builds the binary and runs it against each fixture under `www/examples/` in a pseudo-terminal, writing a record per capture.
- 🟢 A mock upstream answers every provider through `CANIBUMP_*_API` from the fixture's `upstream.json`, and `git:` remotes are local repos, so no capture touches the network.
- 🟢 The build fails on an unexpected exit code, a request the mock does not know, or output naming the build machine.
- 🟢 `terminal(slug)` shows a record, and `file(slug, path)` a fixture file the capture read.


== Changes

- `www/capture.sh`: the capture library, ported from mindtape.
- `www/mock-upstream.py`: the mock, stdlib only.
- `www/generate.sh`: runs the captures.
- `www/utils/ui.typ`: `terminal`, `file`, `exit-code`.
- `www/examples/quickstart/`: the first fixture.
- `flake.nix`: python3.


== Rollback

Revert the commit. Pages that read records fail to build, so revert those with it.


== Out of scope

- The pages. Their own task.


== Dogfood

The site's quickstart shows a capture of this checkout's binary.

