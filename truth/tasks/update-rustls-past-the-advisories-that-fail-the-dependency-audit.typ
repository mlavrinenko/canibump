#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Update rustls past the advisories that fail the dependency audit",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  estimate: (src-files: 0),
  tags: ("found",),
  x-goals: (
    product: (
      one-command: neutral,
      many-targets: neutral,
      whole-chain: neutral,
      honest-state: neutral,
      offline-readers: neutral,
      typed-not-scripted: neutral,
    ),
    process: (
      sound-codebase: helps[CI's Dependency Audit goes green on main again],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 30),
)

== Problem

+ Pushing the commit that puts `mt` in the dev shell, whose CI ran the audit for the first time since Oct 13
+ 🔴 #raw("Cargo.lock"): `Cargo.lock` pins `rustls` 0.23.45, which six advisories published since the last green run now flag, so `cargo deny` fails CI's Dependency Audit


CI run 40950717180, job Dependency Audit:
```
error[vulnerability]: `Debug` impls for client session values expose TLS secrets
error[vulnerability]: Certificate verification may accept distrusted intermediates
```
RUSTSEC-2026-0346, -0347, -0348, -0349, -0382 and -0386.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("Cargo.lock").

== TO-BE

- 🟢 `just deny` passes, with `rustls` at a release that fixes every listed advisory.


== Changes

- `Cargo.lock`: `cargo update -p rustls`, plus whatever crate a fix needs beside it.
- `CHANGELOG.md`: a Security entry naming the advisories.


== Rollback

- Revert the lockfile.


== Out of scope

- Ignoring an advisory in `deny.toml`, which hides it rather than fixing it.


== Dogfood

CI's Dependency Audit on main.

