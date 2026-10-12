#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Declare a workaround's site in the manifest for files without comments",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("manifest",),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: helps[any format, JSON, lockfiles, generated files, takes a site],
      whole-chain: neutral,
      honest-state: helps[a pin in a JSON file counts as a site, so it never reads unmarked],
      offline-readers: neutral,
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 13),
  links: depends-on(
    "report-a-blocker-whose-workaround-has-no-site-as-unmarked.typ",
  ),
)

== Problem

+ Pin a dependency in `package.json` as a workaround, add its blocker, configure `[scan]`.
+ 🔴 JSON takes no comment, so no `canibump:<id>` marker can sit there, and the blocker reads `unmarked`.


== TO-BE

- 🟢 A blocker takes `sites = [{ file = "package.json", pattern = '"left-pad": "=1\.3\.0"' }]`: a path relative to the manifest and a regex run per line.
- 🟢 Each line it matches is a site, printed and listed like a marker's.
- 🟢 A declared site that matches nothing prints `unmarked`, so a removed pin still shows up.
- 🟢 A missing file is a site that matches nothing: `unmarked  a  package.json is missing`. A bad regex fails the manifest, naming the blocker.
- 🟢 Declared sites are read with or without `[scan]`. A blocker that declares sites is never unmarked for lacking a marker.


== Changes

- `src/sites_tests.rs`: the red tests first: a declared site parses, a bad regex names its blocker.
- `src/scan_tests.rs`: a matching line is a site, no match leaves the blocker unmarked.
- `src/sites.rs`: parse the `sites` key and read the declared files; `src/manifest.rs` and `src/schema.rs` take it.
- `src/scan.rs`, `src/report.rs`: fold declared sites into the findings; an unmarked line carries its reason.
- `docs/scan.md`, `CHANGELOG.md`, `schema/*`.


== Alternatives

- A JSON pointer: robust to reformatting but JSON only, and needs a span-aware parser to give a line.
- Scan JSON string values for markers: a marker in data pollutes it.


== Out of scope

- Multi-line patterns: one line per site keeps reports exact.


== Dogfood

Not applicable: this repo's workarounds all sit in files that take comments.

