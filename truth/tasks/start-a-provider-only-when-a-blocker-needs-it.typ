#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Start a provider only when a blocker needs it",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.85,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("cli", "forges"),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      one-command: helps[a run starts only what its manifest uses, so a Repology-only manifest never spawns `gh`],
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: neutral,
      offline-readers: neutral,
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: helps[`run::check` takes a plain `&Registry` instead of an `FnOnce` and an `Option` dance],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 10),
  links: depends-on("say-on-its-own-line-why-a-provider-could-not-start.typ"),
)

== Problem

+ Write a manifest with one `version = "repology:..."` blocker and run `canibump --refresh` under `strace -f -e execve`.
+ 🔴 It spawns `gh auth token`, and reads glab's and tea's configs, though no blocker needs GitHub, GitLab or Gitea. `defaults::registry()` builds every provider, each with its own HTTP client, on any run that checks anything.
+ 🔴 To spare a fully cached run that cost, `run::check` takes `impl FnOnce() -> Registry` and builds it through `Option::take` inside `get_or_insert_with`: laziness bolted on at the wrong layer.


== TO-BE

- 🟢 Each provider starts on first use, at most once per run, and keeps its start-up error if it fails, as the previous task set up.
- 🟢 A fully cached run starts none, and a manifest without GitHub blockers never runs `gh`.
- 🟢 `run::check` takes `&Registry`; tests pass a registry of fakes as before.


== Changes

- `src/provider/mod.rs` (+ tests): the red test first, a registry slot whose factory counts calls is started once on two asks and never when nothing asks. Then slots become a factory plus a once-cell.
- `src/provider/defaults.rs`: register factories.
- `src/run.rs`: take `&Registry`; drop the lazy-build dance.
- `src/cli.rs`: pass the registry.


== Alternatives

- Keep eager construction: `gh` and config reads still run for unrelated manifests.
- Scan the manifest for kinds up front and build only those: a second walk over conditions that must stay in step with `eval.rs`.


== Out of scope

- Sharing one HTTP client across providers: with lazy start, a run builds only the clients it uses.
- Concurrency and request memoization, which get their own tasks.


== Dogfood

`canibump` in `~/.nix` builds only the GitHub and Repology providers its chain uses.

