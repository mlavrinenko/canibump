#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Give the Cachix step a shell",
  status: done(2026, 10, 13),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

A composite action's run step needs shell; the Pages run failed loading the action file without it.
