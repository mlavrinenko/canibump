#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Bump qahq to 97d0174",
  status: done(2026, 10, 31),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

qahq main moved the shared naersk anchor from mindtape to linecop, so a consumer that drops mindtape keeps the toolchain; the rebuild is unchanged
