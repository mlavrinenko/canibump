#import "../utils/page.typ": page
#import "../utils/ui.typ": file, terminal
#import "../utils/site.typ": u

#let meta = (
  route: "/catch-forgotten-workarounds/",
  label: "Catch forgotten ones",
  title: "Catch forgotten workarounds",
  summary: "Flag every upstream link in your files that no blocker names.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

A pin nobody wrote a blocker for never shows up anywhere, so it lives
forever. Give canibump a `[scan]` and it reads your files for issue, PR and
MR links, then flags every one no blocker names.

#file("scan", "canibump.toml")

#file("scan", "hosts/gpu.nix")

#terminal("scan")

The libfoo link belongs to a blocker, and the line marked `canibump-ignore`
is just background reading. The rest want a blocker, or a delete.

= What gets read

- `paths` and `exclude` are globs relative to the manifest. `*` stays inside
  a directory and `**` crosses them.
- A scan never enters `.git` or a symlinked directory, so a Nix `result` link
  can't drag the store in.
- An `exclude` ending in `/**` skips its directory unread.
- The manifest itself is never read, and neither is a file that isn't UTF-8.
- A line holding `canibump-ignore` is skipped.

The scan is local and cheap, so `canibump status` runs it too. The same scan
finds the #link(u("/mark-workarounds/"))[markers] that place each workaround.

= Catchers

A catcher is a regex run over each line, plus a `ref` template its named
groups fill. The built-ins catch GitHub issues and PRs, GitLab issues and
MRs, and Codeberg issues and PRs. A caught ref matches a blocker when it reads
the same as the blocker's ref in the output, case aside.

`[[scan.catcher]]` adds your own, like the Jira one above, which caught the
Justfile's link.

#file("scan", "Justfile")

Reuse a built-in's id to replace it, or set `enabled = false` to turn one
off. The #link(u("/manifest/#scan"))[manifest reference] lists the
built-in ids.
