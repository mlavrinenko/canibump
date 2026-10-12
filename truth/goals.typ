// What canibump is for. Every open task gives each goal a verdict in its
// `x-goals: (product: ...)` field, checked by `check-goals(...)` from
// truth/lib.typ. How this repo builds canibump is a separate set, in
// process-goals.typ.

#let goals = (
  one-command: (
    title: "One command says what is bumpable",
    acid-test: [Running `canibump` in a repo lists every blocker's state in one
      line each. It exits 0 when nothing counted showed up, else 8 plus 1
      for bumpable, 2 for dead, 4 for failed and 16 for an unlisted link, and `--fail-on` or
      `[settings] fail-on` picks what counts.],
  ),
  many-targets: (
    title: "Every blocker fits",
    acid-test: [Issues, PRs/MRs, commits reaching a branch or tag, releases and
      package versions across GitHub, GitLab, Gitea/Forgejo, git remotes,
      deps.dev and Repology declare in the same manifest.],
  ),
  whole-chain: (
    title: "Waits on the whole chain",
    acid-test: [An entry resolves only when every step holds, e.g. merged AND
      released AND packaged in the channel.],
  ),
  honest-state: (
    title: "Wontfix is not landed",
    acid-test: [A PR closed unmerged, an issue closed as wontfix, or a failed
      network check never reports as resolved.],
  ),
  offline-readers: (
    title: "Others read the state offline",
    acid-test: [Another tool reads every entry's last state and check time
      from the cache file without network access.],
  ),
  typed-not-scripted: (
    title: "One typed binary",
    acid-test: [No check shells out to another language's tool; adding a
      target is a typed provider.],
  ),
)
