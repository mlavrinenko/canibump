#import "../utils/page.typ": page
#import "../utils/schema-ref.typ": reference

#let meta = (
  route: "/manifest/",
  label: "Manifest",
  title: "Manifest reference",
  summary: "Every key `canibump.toml` takes.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

This page is `canibump schema` rendered, so it lists what your binary
accepts. Point your editor at the same schema for completion as you type.

#reference("manifest", "canibump.toml", (
  ("Blocker", "[[blocker]]"),
  ("Step", "[[blocker.all]] and [[blocker.any]]"),
  ("Within", "in"),
  ("Bound", "at-least"),
  ("DeclaredSite", "sites"),
  ("Scan", "[scan]"),
  ("Catcher", "[[scan.catcher]]"),
  ("Settings", "[settings]"),
  ("Status", "status"),
  ("Actionable", "fail-on"),
))
