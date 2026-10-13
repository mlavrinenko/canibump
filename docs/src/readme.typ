// docs/src/readme.typ: the source of README.md, which docs/generate-md.sh
// renders with typlite. The example is the site root's capture, and the
// docs index is the site's nav, so neither can drift.
#import "/www/utils/site.typ": NAME, PAGES, REPO, SITE, SITE_URL
#import "/www/utils/ui.typ": file, terminal

// A link to a page of the site. The route must exist, or the render fails.
#let www(route, body) = {
  assert(route in PAGES, message: "no docs page at " + route)
  link(SITE_URL + route, body)
}

#let badge(alt, img, href) = html.elem("a", attrs: (href: href), html.elem(
  "img",
  attrs: (alt: alt, src: img),
))

#html.elem("h1")[#NAME]

#badge("CI", REPO + "/actions/workflows/ci.yml/badge.svg", REPO + "/actions/workflows/ci.yml")
#badge("crates.io", "https://img.shields.io/crates/v/canibump.svg", "https://crates.io/crates/canibump")
#badge("License: MIT", "https://img.shields.io/crates/l/canibump.svg", "LICENSE-MIT")

#SITE.tagline

#file("hero", "canibump.toml")

#terminal("hero")

You write each patch, pin or override down once, with the issue, release or
package it waits on. canibump asks upstream and says which ones can go, and
where they sit in your files. #www("/quickstart/")[The quickstart] walks you
through it.

= Install

- `cargo install canibump`
- `nix profile install github:mlavrinenko/canibump`
- A prebuilt binary from the #link(REPO + "/releases/latest")[releases page]

= Docs

Everything lives at #link(SITE_URL + "/")[#SITE_URL.split("://").last()].

#let groups = json("/www/generated/site-pages.json").groups
#for group in groups.filter(g => g.title != "Start") [
  - *#group.title.* #group.routes.map(r => www(r, PAGES.at(r).label)).join[, ]
]

= License

#link("LICENSE-MIT")[MIT].
