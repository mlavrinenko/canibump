#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Split the long recipes out of the Justfile",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (src-files: 13),
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
      sound-codebase: helps[a recipe reads in one screen, and a script can be shellchecked],
      docs-match-code: helps[`just --list` says what each recipe does],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 11),
)

== Problem

+ Run `just --list`.
+ 🔴 It lists 33 recipes with no groups, many described by the last line of a long comment: `check  # Per-rule scopes and the `gate` tag live in .mmz/config.yaml.`
+ Open `Justfile`.
+ 🔴 479 lines: `release` alone is 87 lines of bash, and most recipes carry a paragraph of history above them.
+ Run `just cover`, `just crap`, `just machete`, `just check-dry` and `just deny` on a green tree.
+ 🔴 Each prints a header, a table or a clone report, and `just deny` about 150 lines of warnings.


== TO-BE

- 🟢 `just` lists the daily recipes, `just --list` lists all of them in groups, each with a one-line `[doc()]`.
- 🟢 No recipe body runs past about five lines; longer ones live in `scripts/`, which shellcheck passes.
- 🟢 A green `just test` prints its summary line; a green `cover`, `crap`, `machete`, `check-dry` or `deny` prints nothing.


== Changes

- `Justfile`: `[doc()]` and `[group()]` on every recipe, comments cut to the core, bodies over five lines moved out, `_lcov-stale-reason` folded into `crap`.
- `scripts/cover.sh`, `scripts/crap.sh`, `scripts/eject.sh`, `scripts/release.sh`: the moved bodies. `crap.sh --stale-reason` is the staleness test `integrate.sh` asks.
- `scripts/fork.sh`, `scripts/integrate.sh`: moved from `.just/scripts/` with their comments cut to the core; `.just/` is gone.
- `scripts/quiet.sh`: run a command and print its output only when it fails.
- `.mmz/config.yaml`: `scripts/**` joins the inputs every gate shares.
- `outdatty.yaml`: `scripts/**` is tracked and couples to the dev docs.
- `canibump.toml`: the doctest note cites the line the Justfile now has.
- `truth/tasks/`: the two open tasks that name `.just/scripts` or a Justfile line name `scripts/` instead.
- `CONTRIBUTING.md`, `AGENTS.md`: start with `just`, point at the groups and `scripts/`.


== Rollback

- Revert the commit. The recipe names and arguments stay the same.


== Out of scope

- A shellcheck gate over `scripts/`. Its own task once the scripts settle.
- The `fork` branch name and what `fork.sh` and `integrate.sh` check. The worktrunk task replaces them.


== Dogfood

Every gate run goes through the new recipes.
