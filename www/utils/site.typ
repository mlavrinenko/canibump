// Site-wide constants: the name, the site URL and its path prefix, and the
// nav. The name and the URL are read from tola.toml. A page's label, title and
// summary live on the page, and www/generate-site-pages.sh collects them into
// `PAGES`, so this file never restates one.

#import "mode.typ": MARKDOWN

#let REPO = "https://github.com/mlavrinenko/canibump"

// Typst resolves a relative path against this file, so the read works under
// both roots the file is compiled with: `--root www` for the pages and
// `--root .` for the site-meta query in www/generate-site-pages.sh.
#let TOLA = toml("../tola.toml")
#let SITE_URL = TOLA.site.info.url.trim("/", at: end)
#let NAME = TOLA.site.info.title

// GitHub Pages serves the site under the URL's path ("/canibump"), so u()
// prefixes page links with it. Assets stay source-relative (/assets/...),
// because tola prefixes those itself.
#let _url-path(url) = {
  let parts = url.split("://").last().split("/")
  if parts.len() < 2 { "" } else { "/" + parts.slice(1).join("/") }
}
#let PREFIX = _url-path(SITE_URL)

// The Markdown copy of a route: `/scan/#markers` is `/scan/index.md#markers`.
#let md-path(path) = {
  let (base, ..frag) = path.split("#")
  let file = if base.ends-with("/") { base + "index.md" } else { base }
  if frag == () { file } else { file + "#" + frag.join("#") }
}

// A page link. The Markdown copy (mode.typ) links the other copies by full
// URL, because an agent often reads one away from the site.
#let u(path) = if MARKDOWN { SITE_URL + md-path(path) } else { PREFIX + path }

// The button beside a page's title that copies the page as Markdown. The
// script in www/templates/layout.typ does the copy, and CSS shows `data-tip`.
#let md-copy-button(route) = html.elem("button", attrs: (
  class: "md-copy",
  type: "button",
  "aria-label": "Copy as Markdown",
  "data-tip": "Copy as Markdown",
  "data-md": PREFIX + md-path(route),
))

#let SITE = (
  title: NAME,
  tagline: TOLA.site.info.description,
)

// The sidebar: each group's title and its pages in reading order. A group is
// named for the reader's job, so a reader scans one short group.
#let NAV = (
  (title: "Start", items: ("/",)),
  (
    title: "Reference",
    items: ("/cli/", "/manifest/", "/report/", "/state-file/"),
  ),
)

// Child pages the sidebar nests under an entry, by the entry's route.
#let NAV_CHILDREN = (:)

// The page manifest, route -> {label, title, summary, home?}. It is empty
// during www/generate-site-pages.sh's first pass, which compiles the pages to
// fill it.
#let PAGES = json("../generated/site-pages.json").pages

// One page by route: none during that first pass, else its entry. An unknown
// route panics, because a route typed in NAV or in prose must exist.
#let page-meta(route) = {
  if PAGES.len() == 0 { return none }
  if type(route) == str and route in PAGES { return PAGES.at(route) }
  panic(
    "unknown docs-site route: "
      + repr(route)
      + ", known routes: "
      + PAGES.keys().join(", "),
  )
}

// A summary is one sentence of Typst markup. `render-summary` makes it the
// page's lede, or with `plain: true` the text of <meta name="description">.
#let _plain-text(it) = {
  if type(it) == str { return it }
  if it.has("text") { return it.text }
  if it.has("children") { return it.children.map(_plain-text).join("") }
  if it.has("body") { return _plain-text(it.body) }
  if it.func() == smartquote { return if it.double { "\"" } else { "'" } }
  " "
}
#let render-summary(text, plain: false) = {
  if text == none { return none }
  let body = eval(text, mode: "markup")
  if plain { _plain-text(body) } else { body }
}
