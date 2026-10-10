#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Give the crate crates.io keywords",
  status: done(2026, 10, 10),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

keywords = \[\] keeps canibump out of crates.io keyword search, where someone looking for an upstream or workaround tracker would find it
