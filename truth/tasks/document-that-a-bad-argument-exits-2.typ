#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Document that a bad argument exits 2",
  status: done(2026, 10, 10),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

clap exits 2 on a usage error, but help, README and the site said bad arguments exit 1.
