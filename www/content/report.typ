#import "../utils/page.typ": page
#import "../utils/schema-ref.typ": reference

#let meta = (
  route: "/report/",
  label: "JSON report",
  title: "JSON report",
  summary: "Every field `canibump --json` prints.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

This page is `canibump schema report` rendered.

#reference(
  "report",
  "--json",
  (
    ("Row", "entries"),
    ("Site", "sites"),
    ("Unlisted", "unlisted"),
    ("Unmarked", "unmarked"),
    ("Status", "status"),
  ),
  object: "object",
)
