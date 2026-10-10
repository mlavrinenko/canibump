#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Keep docs out of the source dict",
  status: done(2026, 10, 10),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

The source dict says everything but docs yet lists no docs/\*\*, so a tweak that also fixes docs/manifest.md counts two source files and is refused
