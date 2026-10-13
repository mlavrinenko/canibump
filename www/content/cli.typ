#import "../utils/page.typ": page
#import "../utils/ui.typ": help-sections

#let meta = (
  route: "/cli/",
  label: "CLI",
  title: "Command line",
  summary: "Every command and flag, straight from `--help`.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

A bare `canibump` runs `check`. Each command reads `canibump.toml` from where
you run it, and `-m` points it at another one.

#help-sections("cli")
