//! canibump: can I bump yet? Checks the upstream blockers a repo's
//! workarounds wait on and says which became resolved.

pub mod auth;
pub mod cache;
pub mod cli;
pub mod condition;
pub mod eval;
pub mod manifest;
pub mod outcome;
pub mod provider;
pub mod refs;
pub mod report;
pub mod run;
pub mod version;
