#import "../utils/page.typ": page
#import "../utils/schema-ref.typ": reference

#let meta = (
  route: "/state-file/",
  label: "State file",
  title: "State file",
  summary: "Every field of the file each run saves its results to.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

This page is `canibump schema state` rendered.

#reference(
  "state",
  "state file",
  (("Entry", "entries"), ("Status", "status")),
  object: "object",
)
