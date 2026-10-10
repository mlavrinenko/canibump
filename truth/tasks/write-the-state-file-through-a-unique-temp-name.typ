#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Write the state file through a unique temp name",
  status: done(2026, 10, 10),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

Two runs on one manifest, a cron job and a shell, both write \<state>.json.tmp, so interleaved writes can rename a torn file into place that the next run then discards
