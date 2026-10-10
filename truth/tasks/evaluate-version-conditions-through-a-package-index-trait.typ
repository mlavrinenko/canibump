#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Evaluate version conditions through a package index trait",
  kind: "feat",
  priority: ice(
    impact: 8,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("manifest", "packages"),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: helps[one condition covers a registry package and a distro package],
      whole-chain: neutral,
      honest-state: neutral,
      offline-readers: neutral,
      typed-not-scripted: helps[a `PackageIndex` trait any aggregator implements],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 10),
  links: parent("check-package-versions-through-deps-dev-and-repology.typ"),
)

== Problem

+ Wait until nixpkgs unstable packages llama.cpp 0.6.0 or later.
+ 🔴 The manifest has no way to say "package P in repo R is at least V".


== TO-BE

- 🟢 `version = "repology:llama.cpp"`, `repo = "nix_unstable"`, optional `srcname`, and `at-least = "0.6.0"` parse; so does `version = "deps.dev:cargo/serde"`.
- 🟢 It is bumpable once the highest version found is at least the bound, and says which version it saw.
- 🟢 A package missing from the repo waits; an unknown project fails.


== Changes

- `src/eval_tests.rs`: the red tests first, against a fake index.
- `src/refs.rs`: `PackageRef`.
- `src/manifest.rs`: the `version` condition.
- `src/provider/mod.rs`: `PackageIndex`.
- `src/eval.rs`: the comparison.


== Alternatives

- One condition per index: two keys that mean the same thing.


== Out of scope

- The HTTP clients. The next task adds them.


== Dogfood

Not applicable until an index answers; the next task wires them.

