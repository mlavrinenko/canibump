//! Token discovery: reuse what the user already has.

use std::process::{Command, Stdio};

/// The GitHub token: `GITHUB_TOKEN`, then `GH_TOKEN`, then `gh auth token`.
/// `None` means unauthenticated requests.
#[must_use]
pub fn github_token() -> Option<String> {
    github_token_from(|key| std::env::var(key).ok(), gh_auth_token)
}

fn github_token_from(
    env: impl Fn(&str) -> Option<String>,
    fallback: impl FnOnce() -> Option<String>,
) -> Option<String> {
    ["GITHUB_TOKEN", "GH_TOKEN"]
        .into_iter()
        .find_map(|key| env(key).filter(|value| !value.is_empty()))
        .or_else(fallback)
}

/// Asks the `gh` CLI for its stored token, the one place canibump runs
/// another program: gh keeps the token in the system keyring, not a file.
fn gh_auth_token() -> Option<String> {
    let output = Command::new("gh")
        .args(["auth", "token"])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if output.status.success() {
        trimmed(output.stdout)
    } else {
        None
    }
}

fn trimmed(bytes: Vec<u8>) -> Option<String> {
    let text = String::from_utf8(bytes).ok()?;
    let token = text.trim();
    (!token.is_empty()).then(|| token.to_owned())
}

#[cfg(test)]
#[path = "auth_tests.rs"]
mod tests;
