#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Publish to crates.io through Trusted Publishing",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (src-files: 1),
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
      sound-codebase: helps[a release stops depending on a stored token that expires and fails the publish],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 30),
)

== Problem

+ Open `.github/workflows/release.yml` and read the `publish` job.
+ It publishes with `CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}`, a repo secret stored by hand.
+ 🔴 It breaks when that token expires. On 2026-10-28 mindtape's release failed this way: its stored token, set 2026-03-07, had expired, and `error: failed to publish mindtape v0.3.0 to registry at https://crates.io ... (status 403 Forbidden): authentication failed` stopped the run at the irreversible step.

== TO-BE

- 🟢 The `publish` job holds `permissions: {contents: read, id-token: write}`, and a step `uses: rust-lang/crates-io-auth-action@c6f97d42243bad5fab37ca0427f495c86d5b1a18 # v1.0.5` with `id: auth` runs before the publish.
- 🟢 The publish step reads `CARGO_REGISTRY_TOKEN: ${{ steps.auth.outputs.token }}`, a token that is revoked when the job ends.
- 🟢 A crate crates.io does not have yet cannot be created through Trusted Publishing. Such a crate is routed to a short-lived `publish-new` token in `CARGO_NEW_CRATE_TOKEN`, as mindtape d2f7e9f2 does.
- 🟢 Owner step, per published crate (`canibump`): on `https://crates.io/crates/canibump/settings`, open Trusted Publishing, Add, GitHub, and enter owner `mlavrinenko`, repo `canibump`, workflow `release.yml`, environment `crates-io`.
- 🟢 After the first green publish, `gh secret delete CARGO_REGISTRY_TOKEN -R mlavrinenko/canibump` removes the stored token.

== Changes

- `.github/workflows/release.yml`: the `publish` job gets the permissions, the auth step and the token output above, as nix-why-build 9743ef71 and mindtape b958a586 and d2f7e9f2 did.

== Rollback

- Revert the commit and the job reads the stored `CARGO_REGISTRY_TOKEN` again. Keep that secret until the first green Trusted Publishing run, so a revert never meets a missing token.

== Out of scope

- The release workflow's other jobs, and a cargo or crates.io version bump.
- Deleting the stored secret before the first green publish.

== Dogfood

The next tagged release runs the new publish job, so that run is the proof.
