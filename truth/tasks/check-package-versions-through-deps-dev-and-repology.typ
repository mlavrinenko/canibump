#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Check package versions through deps.dev and Repology",
  priority: ice(
    impact: 8,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 4,
  kind: "milestone",
  tags: ("packages",),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: helps[crates.io, npm, PyPI, Go and Maven through deps.dev, and every distro repo incl. nixpkgs channels through Repology],
      whole-chain: helps[a version step compares against a literal or an earlier step's release],
      honest-state: neutral,
      offline-readers: neutral,
      typed-not-scripted: helps[two typed HTTP clients cover every ecosystem, no per-registry scripts],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 10),
  links: depends-on(
    "check-github-issue-and-pr-state-from-a-canibump-toml-manifest.typ",
  ),
)

== Problem

+ Wait until nixpkgs unstable packages the llama.cpp release that carries a fix.
+ 🔴 Nothing checks it; the patch stays until someone looks at search.nixos.org.


== TO-BE

- 🟢 `version = "repology:llama.cpp"` with `repo = "nix_unstable"` and `at-least = "0.6.0"` resolves once the repo has it.
- 🟢 `version = "deps.dev:cargo/serde"` does the same for a registry package.
- 🟢 Versions compare by semver when both sides parse, else leniently, so `b7012` and `7012` agree.
- 🟢 Repology sees one request per second at most, with a User-Agent naming canibump.


== Changes

- `src/version.rs`: lenient version compare.
- `src/provider/repology.rs`, `src/provider/depsdev.rs`: the two clients.
- `src/manifest.rs`, `src/eval.rs`: the `version` condition.


== Out of scope

- Per-registry clients. The aggregators cover them.


== Dogfood

`~/.nix/canibump.toml` checks `llama-cpp` in `nix_unstable`.

