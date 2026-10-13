#import "../utils/page.typ": page
#import "../utils/ui.typ": exit-code, file, terminal
#import "../utils/site.typ": REPO, u

#let meta = (
  route: "/quickstart/",
  label: "Quickstart",
  title: "Quickstart",
  summary: "Install canibump, write your first blocker and watch it turn bumpable.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

= Install

Pick one.

- `cargo install canibump`
- `nix profile install github:mlavrinenko/canibump`
- A prebuilt binary from the #link(REPO + "/releases/latest")[releases page]

`canibump completions <shell>` prints tab completion for your shell. The Nix
package installs it for bash, zsh and fish.

= Write a blocker

Say you patched libfoo because it crashes on empty input, and the fix is PR
\#412 upstream. Put a `canibump.toml` next to your code.

#file("quickstart", "canibump.toml")

A blocker has an `id`, a `note` that says what to undo, and one condition.
`state` waits for an issue or PR to close the right way. `[scan]` lists the
files canibump reads to find your workarounds. The `#:schema` line is
optional, and gives you completion and typo squiggles in any editor with
taplo or tombi.

= Mark the workaround

Drop a `canibump:<id>` comment right next to the patch.

#file("quickstart", "pkgs/libfoo.nix")

The marker tells canibump where the workaround lives, so you never write a
file name or a line number into the note.

= Run it

#terminal("quickstart")

The PR is still open, so the blocker is `waiting` and the run exits
#exit-code("quickstart"). Run it again and it reuses this result for a
while, with no request. #link(u("/read-results-offline/"))[Read results
offline] says how long.

= When upstream lands

A week later the PR merges. `--refresh` checks everything right away
instead of trusting the cache.

#terminal("quickstart-landed")

This time it's `bumpable`, with the marked line under it, and the run exits
#exit-code("quickstart-landed"). Drop the patch, then the blocker.

= Next

- #link(u("/run-in-ci/"))[Run it in CI] so you hear about it without asking.
- #link(u("/wait-on-a-release/"))[Wait for a fix to ship] when merged isn't
  enough and you need the release.
- #link(u("/catch-forgotten-workarounds/"))[Catch forgotten workarounds]
  that never got a blocker.
