#import "../utils/page.typ": page
#import "../utils/ui.typ": file, terminal
#import "../utils/site.typ": u

#let meta = (
  route: "/mark-workarounds/",
  label: "Mark them",
  title: "Mark where each workaround sits",
  summary: "Put a `canibump:<id>` comment next to each workaround, and a bumpable line tells you which files to open.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

A note like "see line 12 of gpu.nix" rots the first time someone edits above
it. A marker moves with the code. Write `canibump:<id>` in a comment next to
the workaround, and the why and the exit plan stay in the manifest, once.

#file("marks", "canibump.toml")

#file("marks", "hosts/gpu.nix")

#terminal("marks")

`libfoo-crash` sits in two files, so both show up. `libbar-arm64` has no
marker anywhere, so it gets an `unmarked` line.

= Markers

- canibump finds markers in the files `[scan]` lists.
  #link(u("/catch-forgotten-workarounds/"))[Catch forgotten workarounds]
  covers what it reads.
- One blocker can sit in many places, so mark each one.
- An id takes ASCII letters, digits, `_` and `-`. So `canibump: foo` with a
  space, `canibump::path` and `xcanibump:foo` mark nothing.
- A marker that names no blocker prints as `unlisted`.

= Which lines show their sites

A check prints the sites under `bumpable` and `dead` lines, the ones you act
on. `canibump status` prints them under every line, and so does
`canibump status <id>` for its one blocker.

#terminal("marks-status")

`[settings] sites-on` picks the statuses for every run, and `--sites-on`
overrides it for one.

= Files that take no comment

JSON has no comments, so a pin in `package.json` can't carry a marker. Give
the blocker `sites` instead, a file relative to the manifest and a regex run
over each line. That's `left-pad-pin` above.

#file("marks", "package.json")

Every matching line is a site. Declared sites are read with or without
`[scan]`, and a blocker that declares any needs no marker. Once the pin goes,
so does the match, and the blocker turns `unmarked`. A bad regex fails the
manifest and names the blocker.

= Unmarked blockers

A blocker nothing places is one whose workaround may be gone already, or
lives in a file `paths` misses. Either way you see an `unmarked` line, and the
run exits non-zero. Mark the workaround, widen `paths`, or drop the blocker.
To let it pass, leave `unmarked` out of
#link(u("/run-in-ci/#pick-what-fails"))[`fail-on`].
