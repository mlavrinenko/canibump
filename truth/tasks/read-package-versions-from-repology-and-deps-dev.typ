#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Read package versions from Repology and deps.dev",
  kind: "feat",
  priority: ice(
    impact: 8,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("packages",),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: helps[every distro repo incl. nixpkgs channels, and crates.io, npm, PyPI, Go, Maven, NuGet and RubyGems through two clients],
      whole-chain: neutral,
      honest-state: neutral,
      offline-readers: neutral,
      typed-not-scripted: helps[two typed reqwest clients, no per-registry scripts],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 10),
  links: parent("check-package-versions-through-deps-dev-and-repology.typ")
    + depends-on(
      "evaluate-version-conditions-through-a-package-index-trait.typ",
    ),
)

== Problem

+ Declare `version = "repology:llama.cpp"` with `repo = "nix_unstable"`.
+ 🔴 No index answers, so the check fails.


== TO-BE

- 🟢 Repology is asked at most once per second per run, with a User-Agent naming canibump and its repo, and once per project.
- 🟢 deps.dev's `/v3/systems/<system>/packages/<name>` lists every published version.


== Changes

- `src/provider/repology_tests.rs`, `src/provider/depsdev_tests.rs`: the red tests first, against httpmock.
- `src/provider/repology.rs`, `src/provider/depsdev.rs`: the clients.
- `src/cli.rs`: register them.


== Alternatives

- Per-registry clients: seven clients where two aggregators cover them.


== Out of scope

- Repology's bulk endpoint. One project per blocker is the shape canibump has.


== Dogfood

`canibump` reads the real `llama-cpp` version in `nix_unstable`.

