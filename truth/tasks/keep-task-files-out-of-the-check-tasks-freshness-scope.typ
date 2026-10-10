#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Keep task files out of the check-tasks freshness scope",
  status: done(2026, 10, 10),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

Closing a leaf and its milestone in one mt done staled the gate between the two flips
