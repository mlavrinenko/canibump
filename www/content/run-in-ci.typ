#import "../utils/page.typ": page
#import "../utils/ui.typ": capture-quote, exit-code, terminal
#import "../utils/site.typ": u

#let meta = (
  route: "/run-in-ci/",
  label: "Run it in CI",
  title: "Run it in CI",
  summary: "Let a scheduled job tell you when a workaround can go, through exit codes.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

canibump is built to nag. A run that finds something for you exits non-zero,
so a cron job or a scheduled workflow turns red the day a fix lands.

= Exit codes

The `--help` says it all in one line.

#capture-quote("cli", "^Exit codes")

The codes are bits, so a script can test each one, like `(( code & 2 ))` for
something dead.

= Pick what fails

All five statuses count unless you pick. `[settings] fail-on` picks them for
every run, and `--fail-on` overrides it for one. `waiting` never counts, and
a status that doesn't count still prints its line.

#terminal("ci-fail-on")

Only `dead` counts here, so the run exits #exit-code("ci-fail-on"), 8 plus 2.
`fail-on = []` never fails.

= One blocker in a script

`canibump status <id>` answers for one blocker from the last results, with no
network. It exits 0 for `bumpable`, 1 for `waiting` and 2 for anything else,
errors included.

#terminal("ci-status")

= A weekly workflow

This GitHub workflow checks every Monday and goes red when something wants
you. `GITHUB_TOKEN` lifts the rate limit, and
#link(u("/tokens-and-hosts/"))[Tokens and hosts] covers the other forges.

// docs:hand-typed GitHub runs the workflow, which the docs build can't
```yaml
name: canibump
on:
  schedule:
    - cron: "0 6 * * 1"
  workflow_dispatch:
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - run: |
          curl -sSfLo canibump https://github.com/mlavrinenko/canibump/releases/latest/download/canibump-x86_64-unknown-linux-gnu
          chmod +x canibump
      - run: ./canibump
        env:
          GITHUB_TOKEN: ${{ github.token }}
```
