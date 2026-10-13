#import "../utils/page.typ": page
#import "../utils/ui.typ": hero, showcase
#import "../utils/site.typ": NAME, SITE, u

#let meta = (
  route: "/",
  label: "Overview",
  title: NAME,
  summary: SITE.tagline,
  home: true,
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

#hero()

#showcase("hero", "canibump.toml")

= Why

Every repo grows a few patches, pins and overrides that wait on someone
else. A bug fix to merge, a release to ship, a package to reach nixpkgs. You
mean to drop each one once upstream lands, but nobody checks, so they stick
around for years.

Write each one down in `canibump.toml` once, as a blocker. A run asks
upstream where things stand and tells you which ones can go, and where they
sit in your files.

= What a blocker waits on

#table(
  columns: 2,
  table.header[Waits for][Read],
  [An issue fixed, a PR or MR merged], link(u("/wait-on-a-pr/"))[Wait for an issue or PR],
  [A commit in a branch, a tag or a release], link(u("/wait-on-a-release/"))[Wait for a fix to ship],
  [A package at some version in a distro or registry], link(u("/wait-on-a-package/"))[Wait for a package],
  [All of the above, in order], link(u("/chain-steps/"))[Chain the steps],
)

= Statuses

Each blocker gets one line, and the first word says what to do.

#table(
  columns: 2,
  table.header[Status][Means],
  [`waiting`], [Upstream isn't there yet. Nothing to do.],
  [`bumpable`], [It landed. Go drop the workaround.],
  [`dead`], [Closed unmerged, won't fix or a duplicate. Rethink the workaround.],
  [`failed`], [The check broke, say on the network or a rate limit. It never counts as landed.],
  [`unlisted`], [A link or marker in your files that no blocker names.],
  [`unmarked`], [A blocker that nothing in your files places, so its workaround may be gone.],
)

Every status but `waiting` makes the run exit non-zero, so a cron job or CI
can nag you. #link(u("/run-in-ci/"))[Run it in CI] has the codes.
