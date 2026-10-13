#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Take the qahq tools from Cachix in CI",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("tooling", "cib"),
  estimate: (src-files: 2),
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
      sound-codebase: helps[a docs run stops spending minutes compiling tools it never calls],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 13),
)

== Problem

+ Push to main and open the Pages run's log.
+ 🔴 Most of the 19-minute build step is Nix compiling the qahq tools from source: `mmz-0.12.0` alone takes 476s, plus `jscpd`, `outdatty` and repeated `config` derivations.
+ `gc-max-store-size-linux: 5G` purges those paths before the store cache is saved, so the next runner builds them again, and neither workflow reads `qahq.cachix.org`, which answers 200 for `mmz-0.12.0` and whose key the host already trusts.


== TO-BE

- 🟢 `.#docs` holds the site toolchain, the Rust bits the captures need and `mmz`; the code gates' tools stay in `.#rust`.
- 🟢 Runners substitute from `qahq.cachix.org` and `mlavrinenko.cachix.org` instead of compiling what those caches carry.


== Changes

- `flake.nix`: `docsTools` becomes mmz plus the site tools and the Rust bits the captures need; `devShells.docs` uses it alone, and the default shell keeps the union.
- `.github/actions/nix/action.yml`: append the two substituters and their keys to `nix.conf`, read-only.
- `outdatty.lock`: refreshed, since `flake.nix` is a dev-docs source.


== Rollback

- Revert the commit. CI compiles the qahq tools from source again, as it did.


== Out of scope

- Pushing to the caches from CI. A maintainer's machine fills them.
- `tola`, `typst` and `tinymist`, which come from the store cache or nixpkgs.
- Raising the 5G cap. The smaller closure is the fix.


== Dogfood

Not applicable. It changes the dev shell and CI, not canibump's checks.

