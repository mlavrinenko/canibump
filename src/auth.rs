//! Token discovery: reuse what the user already has.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use serde::Deserialize;

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

/// GitLab tokens by host: glab's config, then `GITLAB_TOKEN` for
/// `GITLAB_HOST` (default gitlab.com). A token never goes to another host.
#[must_use]
pub fn gitlab_tokens() -> HashMap<String, String> {
    let config = glab_config_path().and_then(|path| std::fs::read_to_string(path).ok());
    gitlab_tokens_from(|key| std::env::var(key).ok(), config.as_deref())
}

fn env_path(key: &str) -> Option<PathBuf> {
    std::env::var_os(key)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

/// `$XDG_CONFIG_HOME`, else `~/.config`.
fn config_dir() -> Option<PathBuf> {
    env_path("XDG_CONFIG_HOME").or_else(|| env_path("HOME").map(|home| home.join(".config")))
}

fn glab_config_path() -> Option<PathBuf> {
    let dir =
        env_path("GLAB_CONFIG_DIR").or_else(|| config_dir().map(|dir| dir.join("glab-cli")))?;
    Some(dir.join("config.yml"))
}

#[derive(Deserialize)]
struct GlabConfig {
    #[serde(default)]
    hosts: HashMap<String, GlabHost>,
}

#[derive(Deserialize)]
struct GlabHost {
    token: Option<String>,
}

/// `https://host/` or `host` to `host`.
fn bare_host(text: &str) -> String {
    let text = text.split_once("://").map_or(text, |(_, rest)| rest);
    text.trim_end_matches('/').to_owned()
}

fn gitlab_tokens_from(
    env: impl Fn(&str) -> Option<String>,
    glab_config: Option<&str>,
) -> HashMap<String, String> {
    let mut tokens: HashMap<String, String> = glab_config
        .and_then(|text| serde_yaml_ng::from_str::<GlabConfig>(text).ok())
        .map(|config| {
            config
                .hosts
                .into_iter()
                .filter_map(|(host, entry)| {
                    Some((host, entry.token.filter(|token| !token.is_empty())?))
                })
                .collect()
        })
        .unwrap_or_default();
    if let Some(token) = env("GITLAB_TOKEN").filter(|token| !token.is_empty()) {
        let host = env("GITLAB_HOST").map_or_else(
            || crate::refs::GITLAB_COM.to_owned(),
            |host| bare_host(&host),
        );
        tokens.insert(host, token);
    }
    tokens
}

/// Gitea tokens by host: tea's config logins, then `GITEA_TOKEN` for
/// `GITEA_HOST`. Without `GITEA_HOST`, `GITEA_TOKEN` goes nowhere.
#[must_use]
pub fn gitea_tokens() -> HashMap<String, String> {
    let config = config_dir()
        .map(|dir| dir.join("tea").join("config.yml"))
        .and_then(|path| std::fs::read_to_string(path).ok());
    gitea_tokens_from(|key| std::env::var(key).ok(), config.as_deref())
}

#[derive(Deserialize)]
struct TeaConfig {
    #[serde(default)]
    logins: Vec<TeaLogin>,
}

#[derive(Deserialize)]
struct TeaLogin {
    url: String,
    #[serde(default)]
    token: String,
}

fn gitea_tokens_from(
    env: impl Fn(&str) -> Option<String>,
    tea_config: Option<&str>,
) -> HashMap<String, String> {
    let mut tokens: HashMap<String, String> = tea_config
        .and_then(|text| serde_yaml_ng::from_str::<TeaConfig>(text).ok())
        .map(|config| {
            config
                .logins
                .into_iter()
                .filter(|login| !login.token.is_empty())
                .map(|login| (bare_host(&login.url), login.token))
                .collect()
        })
        .unwrap_or_default();
    let token = env("GITEA_TOKEN").filter(|token| !token.is_empty());
    if let (Some(token), Some(host)) = (token, env("GITEA_HOST")) {
        tokens.insert(bare_host(&host), token);
    }
    tokens
}

#[cfg(test)]
#[path = "auth_tests.rs"]
mod tests;
