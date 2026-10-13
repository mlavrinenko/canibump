#import "../utils/page.typ": page
#import "../utils/ui.typ": exit-code, terminal
#import "../utils/site.typ": u

#let meta = (
  route: "/read-results-offline/",
  label: "Read results offline",
  title: "Read results offline",
  summary: "Every run saves what it found, so you and your tools can read it without asking upstream again.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

= The cache

A result stays fresh for `[settings] ttl`, 6 hours unless you say otherwise,
and a run within that window reuses it instead of asking upstream. A failed
check is never fresh, so the next run retries it. `--refresh` checks
everything right away.

= Last results

`canibump status` prints what the last run found, with every line's sites,
and asks nobody. It exits the way a check does, here
#exit-code("offline").

#terminal("offline")

It skips blockers you dropped from the manifest since, and warns about one
whose condition changed.

= The state file

#terminal("offline-path")

The file lives under `$XDG_CACHE_HOME`, or `~/.cache` without it, and
`%LOCALAPPDATA%` on Windows. Its name is the manifest's absolute path, so
every checkout gets its own. `--cache` or `[settings] cache` moves it,
relative to the manifest. A state file canibump can't parse gets dropped with
a warning and rewritten.

The #link(u("/state-file/"))[state file reference] lists every field.

= JSON

`--json` prints the run as one JSON document instead of lines.

#terminal("offline-json")

The #link(u("/report/"))[JSON report reference] lists every field.
`canibump schema report` and `canibump schema state` print the JSON Schemas
that match your binary.
