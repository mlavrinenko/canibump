#import "../utils/page.typ": page
#import "../utils/ui.typ": src

#let meta = (
  route: "/how-it-works/",
  label: "How a run works",
  title: "How a run works",
  summary: "What happens between reading the manifest and printing the lines, file by file.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

canibump reads a manifest, asks each upstream once, and saves what it found.
Here's the trip, with the files that do each part, for when you want to
change one.

+ *Load* (#src("manifest.rs"), #src("condition.rs"), #src("refs.rs"),
  #src("package.rs")). Parse `canibump.toml` into blockers and refuse a bad
  one with its line number.
+ *Reuse* (#src("run.rs"), #src("cache.rs")). A result younger than the TTL
  comes from the state file. The rest get checked, eight blockers at a time.
+ *Check* (#src("eval.rs"), #src("version.rs")). Walk each blocker's chain
  step by step, so a step can hand its tag to the next, and turn it into a
  status.
+ *Ask* (`provider/`). One typed client per target. Each question is asked
  once per run (#src("provider/memo.rs")), a client starts only when a blocker
  needs it (#src("provider/lazy.rs")), and tokens come from the tools you
  already use (#src("auth.rs")). Repology gets at most one request a second.
  A `git:` remote is a bare clone of just the refs asked about.
+ *Scan* (#src("scan.rs"), #src("sites.rs")). With a `[scan]`, read the
  listed files for upstream links and `canibump:<id>` markers, offline. A
  marker some blocker has is one of its sites, a link or marker no blocker
  names is unlisted, and a blocker with no site is unmarked. #src("sites.rs")
  reads the sites a blocker declares in files that take no comment.
+ *Report* (#src("report.rs")). One line per blocker or `--json`, the sites
  under the lines that want them, a line per scan find, and the exit code.
  The state file gets rewritten.

#src("schema.rs") generates the JSON Schemas from the same types the parser
and the report use, which is why the reference pages can't drift.
