#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Raise the MSRV to 1.86",
  status: done(2026, 10, 10),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

cargo\_metadata 0.23 (a build dependency of octocrab) declares rust-version 1.86, so the declared 1.85 already fails the CI MSRV job
