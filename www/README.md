# The docs site

The site at <https://mlavrinenko.github.io/canibump/> builds from this
directory with [tola](https://github.com/tola-rs/tola-ssg), a static site
generator that compiles Typst pages to HTML. Everything comes from the dev
shell.

```bash
just docs-serve   # live reload on http://127.0.0.1:5278/canibump/
just docs-build   # the full build into www/public, silent on success
```

`just docs-build` runs [build.sh](build.sh). It regenerates `generated/`,
builds with `tola build --clean`, indexes the result with
[Pagefind](https://pagefind.app), writes a Markdown copy of every page plus
`llms.txt` ([generate-md.sh](generate-md.sh)), and runs `tola validate`, which
fails on a dead link or a missing asset. `.github/workflows/pages.yml` runs the
same recipe on every push to main and deploys `public/canibump`.

## Layout

- `content/*.typ` holds the pages, one per route.
- `utils/site.typ` holds the name, the URL and the sidebar (`NAV`).
- `utils/ui.typ` holds the components pages call.
- `templates/` and `assets/` hold the chrome and the styles.
- `generated/` is build output the pages read, never committed.

## A page

A page declares its route, sidebar label, title and one-sentence summary once,
at the top, and both the build and the sidebar read them from there.

```typst
#import "../utils/page.typ": page

#let meta = (
  route: "/scan/",
  label: "Scan",
  title: "Catch forgotten workarounds",
  summary: "Flag every upstream link no blocker names.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)
```

Add the route to `NAV` in `utils/site.typ`, or the page builds but no sidebar
links it. [generate-site-pages.sh](generate-site-pages.sh) fails on two pages
with one route, and on a summary longer than one sentence or 120 characters.

Typst's HTML export makes paragraphs only on a page with block elements like
headings, so a page of bare text renders as one line in its Markdown copy.
