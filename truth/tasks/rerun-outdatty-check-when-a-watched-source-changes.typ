#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Rerun outdatty-check when a watched source changes",
  status: done(2026, 10, 13),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

An edit to \`pages.yml\`, \`release.toml\` or \`.config/wt.toml\` left \`outdatty-check\` memoized, so the lock drift reached CI unseen.
