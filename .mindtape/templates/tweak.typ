#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: slot(
    "title",
    prompt: "Task title, imperative verb first",
    example: "Fix the typo in the `canibump status` help text",
  ),
  status: done(2026, 10, 4),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

#slot(
  "why",
  kind: "text",
  prompt: "Why the change, one line",
  example: "The help said `state` where the command says `status`",
)
