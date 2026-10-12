//! The integration suite, linked into one binary: a single link of every
//! dependency instead of one per file. nextest still runs each test alone.

mod cli;
mod color;
mod common;
mod completions;
mod exit_codes;
mod package;
mod reached;
mod release_hook;
mod schema;
mod sites_on;
mod status_id;
mod wt_carry_lcov;
