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

## Captures

Every terminal block on the site is a run of the binary this checkout
builds. [generate.sh](generate.sh) builds it and runs it against a fixture in
`examples/<name>/`, and [capture.sh](capture.sh) records each command, what it
printed and its exit code in `generated/<slug>.json`. A page shows a record
with `terminal("<slug>")`, a file of its fixture with
`file("<slug>", "canibump.toml")`, one printed line with `capture-line` (or
`capture-quote` for a prose line), and the last exit code with `exit-code`.
`src("eval.rs")` names a source file and fails the build once it's gone.

- Upstream is [mock-upstream.py](mock-upstream.py). It answers every provider
  from the fixture's `upstream.json`, a map from request path to JSON body.
  `capture_upstream <fixture> <file>` swaps the answers mid-session, so a page
  can show a PR merging.
- A `git:` remote is a local repo. `capture_remote` makes one and reroutes its
  URL with `insteadOf`, and `capture_git` commits with a fixed clock, so the
  hashes a manifest names stay put.
- The build fails when a capture exits other than expected, asks the mock for
  a route nobody wrote, or prints the capture area, your home or the checkout.
  Exit 1 and 2 are errors, so expecting one needs `--expect <regex>`.
- Each `capture_copy` starts the fixture with no state file, and
  `capture_prep` runs a setup step the page doesn't show.

A capture is a fixture plus a few lines in generate.sh.

```bash
capture_copy dir my-fixture
capture --exit 9 my-slug "$dir" 'canibump --refresh'
```

## References

The reference pages hold no prose about flags or keys. `content/cli.typ`
shows the captured `--help` of `canibump` and of every command its help
lists. `content/manifest.typ`, `report.typ` and `state-file.typ` render
`canibump schema` with [utils/schema-ref.typ](utils/schema-ref.typ), so each
key reads its doc comment.

- A new schema type needs a section in its page, or the build fails naming it.
- A doc comment's first paragraph is what readers see. A rustdoc link like
  ``[`VERSION`]`` fails the build, because it means nothing in an editor's
  hover or on the site.

## Gates

`just check` runs three docs arms, and mmz skips each while its inputs stay
unchanged.

- `docs-build` is the full build. It fails on a capture that exits wrong or
  asks the mock for an unknown route, a dead link, a missing asset, a summary
  over one sentence, a schema type no reference renders, and a rustdoc link in
  a doc comment.
- `check-doc-blocks` fails on a code block typed into a page or a helper. A
  block no capture can hold gets `// docs:hand-typed <reason>` above it.
- `check-doc-prose` fails on history words, an em dash, a colon inside a
  sentence and all-caps shouting in a page's text.

Both scanners read Typst's parse tree through
[scripts/doc-blocks-lib.sh](../scripts/doc-blocks-lib.sh), ported from
mindtape, and first run a control on known-bad input, so a scanner that reads
nothing can't pass.

## Voice

Write like a dev showing the tool to a friend. Say "you", use contractions,
keep sentences short. Open with the reader's problem, show the manifest and
the run, then the rules. Cut every sentence the page works without.

- One idea per sentence. No em dashes and no colons inside a sentence, so
  split it instead.
- Today's behaviour only, since git log holds the history.
- State a fact on one page and link to it from the others.
- Narrate a run sparingly, because the terminal block already shows it. The
  `site-examples` group in outdatty.yaml flags the pages when a fixture
  changes.

## Layout

- `content/*.typ` holds the pages, one per route.
- `utils/site.typ` holds the name, the URL and the sidebar (`NAV`).
- `utils/ui.typ` holds the components pages call.
- `templates/` and `assets/` hold the chrome and the styles.
- `examples/` holds the fixtures the captures run in.
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
