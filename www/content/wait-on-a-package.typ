#import "../utils/page.typ": page
#import "../utils/ui.typ": file, terminal
#import "../utils/site.typ": u

#let meta = (
  route: "/wait-on-a-package/",
  label: "A package",
  title: "Wait for a package",
  summary: "Hold an overlay until your distro or registry ships the version you need.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

The fix is released, but nixpkgs or crates.io doesn't have it yet. `version`
waits for a package to reach `at-least`.

#file("wait-on-a-package", "canibump.toml")

#terminal("wait-on-a-package")

= Where versions come from

- *Repology* covers distro repos, nixpkgs channels included. `repo` picks
  one, like `nix_unstable` or `nix_stable_25_11`. Add `srcname` when the repo
  ships several packages under one project.
- *deps.dev* covers the language registries, crates.io, npm and PyPI among
  them. The #link(u("/manifest/#blocker"))[manifest reference] lists every
  one.

A pre-release sits below its release, so `1.1.0-rc1` still waits for
`1.1.0`. The #link(u("/wait-on-a-release/#version-order"))[version order]
is the same one tags use.

Repology asks for at most one request a second, and canibump keeps to it, so
a manifest with many Repology blockers takes a moment.
