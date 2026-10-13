#import "../utils/page.typ": page
#import "../utils/ui.typ": file, terminal
#import "../utils/site.typ": u

#let meta = (
  route: "/wait-on-a-release/",
  label: "A fix to ship",
  title: "Wait for a fix to ship",
  summary: "Hold a pin until the commit that fixes it lands in a branch, a tag or a release.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

Merged isn't shipped. When you pin a dependency, you wait for the release
that carries the fix. `reached` takes a commit, or a PR and its merge commit,
and `in` says where it has to show up.

#file("wait-on-a-release", "canibump.toml")

#terminal("wait-on-a-release")

= Where it lands

`in` takes exactly one of three.

- `branch` holds once the branch contains the commit.
- `tag` holds once that one tag contains it.
- `tags` takes a glob like `v*` and finds the first matching tag, in version
  order, that contains it. `since` skips the tags below a version.

`reached` reads history on GitHub and on any git remote. A target elsewhere
fails the manifest. An issue has no merge commit, so it reads `failed` here.
Wait on it with #link(u("/wait-on-a-pr/"))[`state`] instead.

= Release lines

Tags fall into release lines by their first two numbers, so `v1.4.2` and
`v1.4.0-rc1` share `1.4`, and all `b`-builds share one. A fix shipped only on
an old line still counts.

canibump asks each line's newest tag, then binary-searches for the first line
whose newest tag has the commit. A line whose newest tag lacks it counts as
not having it, since a line forked before the fix looks the same. A tag cut
from another branch costs extra requests, at most one per tag, never a wrong
answer.

= Version order

One order covers semver, `v`-tags, dates and builds like `b7012`.

#table(
  columns: 2,
  table.header[Versions][Why],
  [`v1.0.0` = `1.0.0` = `1.0.0+build`], [A letter prefix and `+build` drop.],
  [`1.0` = `1.0.0`], [A missing trailing part counts as zero.],
  [`b999` < `b1000`], [Numbers compare as numbers.],
  [`1.0.0-rc9` < `1.0.0-rc10` < `1.0.0`], [Letters after the numbers mean a pre-release.],
)

= Any git remote

`git:<url>@<sha>` works with any remote git can fetch. canibump keeps a bare
clone of just the refs it asks about under `~/.cache/canibump/git/`. A short
sha that matches two commits fails and asks for more digits.
