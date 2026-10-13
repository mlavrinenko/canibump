#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Scaffold the www docs site from mindtape's framework",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("docs", "tooling"),
  estimate: (src-files: 18),
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
      docs-match-code: helps[a page's route, title and summary are declared once, on the page],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 13),
  links: parent("publish-the-docs-as-a-www-site-built-from-real-runs.typ"),
)

== Problem

+ Look for a docs site in the repo.
+ 🔴 There is none, so long docs live in `docs/*.md` with no search, no navigation and no Markdown copy for agents.


== TO-BE

- 🟢 `just docs-serve` serves a tola site with a sidebar, search, a theme toggle and a copy-as-Markdown button.
- 🟢 `just docs-build` writes the HTML, a Pagefind index, a Markdown copy of each page and `llms.txt`.
- 🟢 A page declares its route, title and summary once, and the build fails on a page missing from the nav or a dead link.
- 🟢 `.github/workflows/pages.yml` deploys the site on push to main.


== Changes

- `www/tola.toml`, `www/templates/`, `www/utils/{site,page,mode,tola,ui}.typ`, `www/assets/`: ported from `~/projects/home/mindtape/www/`, mindtape's pieces cut.
- `www/generate-site-pages.sh`, `www/generate-md.sh`: the page manifest, the Markdown copies and `llms.txt`.
- `www/content/index.typ`: a placeholder root.
- `Justfile`: `docs-build`, `docs-serve`.
- `flake.nix`, `flake.lock`: tola, pagefind, typst, tinymist, util-linux.
- `.gitignore`: the build output.
- `.github/workflows/pages.yml`: build and deploy.


== Rollback

Revert the commit. Nothing outside `www/`, the Justfile, the flake and the workflow reads it.


== Out of scope

- Captures and pages. Their own tasks.
- Gating the build in `just check`. The gate task wires it once captures exist.


== Dogfood

`just docs-build` builds the site this repo publishes.

