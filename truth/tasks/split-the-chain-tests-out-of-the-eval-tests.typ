#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Split the chain tests out of the eval tests",
  status: done(2026, 10, 10),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

src/eval\_tests.rs sat at exactly the 500-line cap and ejectest cannot split an ejected file, so the next eval test would fail just check
