//! canibump: can I bump yet? Checks the upstream blockers a repo's
//! workarounds wait on and says which became resolved.
//!
//! This library exists for the `canibump` binary and its tests. It is not a
//! stable API: its modules are hidden from the docs, and any of it may change
//! in any release.

mod auth;
mod cache;
#[doc(hidden)]
pub mod cli;
mod condition;
mod eval;
#[doc(hidden)]
pub mod manifest;
mod outcome;
mod package;
#[doc(hidden)]
pub mod provider;
#[doc(hidden)]
pub mod refs;
#[doc(hidden)]
pub mod report;
mod run;
mod scan;
#[doc(hidden)]
pub mod schema;
mod version;
