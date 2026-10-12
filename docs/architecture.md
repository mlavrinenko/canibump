# Architecture

canibump reads a manifest, asks each upstream once, and saves what it found.

1. **Load** (`manifest.rs`, `condition.rs`, `refs.rs`, `package.rs`). Parse
   `canibump.toml` into blockers and refuse a bad one with its line number.
2. **Reuse** (`run.rs`, `cache.rs`). A result younger than the TTL comes from
   the state file. The rest gets checked, eight blockers at a time.
3. **Check** (`eval.rs`, `version.rs`). Walk each blocker's chain step by step,
   so a step can hand its tag to the next, and turn it into a status.
4. **Ask** (`provider/`). One typed client per target. Each question is asked
   once per run (`memo.rs`), a client starts only when a blocker needs it
   (`lazy.rs`), and tokens come from the tools you already use (`auth.rs`).
   Repology gets at most one request a second, as its rules ask. A `git:`
   remote is a bare clone of just the refs asked about, under
   `~/.cache/canibump/git/`.
5. **Report** (`report.rs`). One line per blocker or `--json`, and the exit
   code. The state file gets rewritten.
6. **Scan** (`scan.rs`). With a `[scan]`, read the listed files offline and
   print an `unlisted` line for each upstream link no blocker names.

`schema.rs` generates the manifest's JSON Schema from the same types the
parser uses.
