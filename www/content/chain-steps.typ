#import "../utils/page.typ": page
#import "../utils/ui.typ": file, terminal

#let meta = (
  route: "/chain-steps/",
  label: "A chain",
  title: "Chain the steps",
  summary: "Wait for a fix to merge, ship and reach your distro, with one blocker.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

Real waits come in chains. Your patch can go once the PR merges, the fix
ships in a tag, and nixpkgs picks up that tag. Three blockers would turn
`bumpable` one by one, and none of them alone means the patch can go. One
chain does.

#file("chain", "canibump.toml")

`[[blocker.all]]` holds when every step holds, checked in order. It stops at
the first step that doesn't and names it.

#terminal("chain")

The `released` step found the tag, and `at-least = { step = "released" }`
hands it to the next step. nixpkgs moves, and the whole chain holds.

#terminal("chain-landed")

= Rules

- `[[blocker.any]]` holds when one step does, for a fix that may ship two
  ways. Steps nest, so an `any` can sit inside an `all`.
- A step's `id` names it in the output, and lets a later step read the tag
  it found.
- A `step` reference points at an earlier step of the same `all`, or of an
  enclosing one, since those surely held. An `any` may hold through another
  step, so canibump refuses a reference into it. Put both in one `all`.
- A dead or failed step never lets the chain read `bumpable`. In an `any`, a
  failed step beats a waiting one, and the whole `any` is dead only when every
  step is.
