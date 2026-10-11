//! Token discovery: reuse what the user already has.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use serde::Deserialize;

/// The token for the GitHub API at `api`. On `api.github.com` it is
/// github.com's: `GITHUB_TOKEN`, `GH_TOKEN`, then `gh auth token`. On any
/// other host it is that host's, the way gh picks one: `GH_ENTERPRISE_TOKEN`,
/// `GITHUB_ENTERPRISE_TOKEN`, then `gh auth token --hostname <host>`.
/// `None` means unauthenticated requests.
#[must_use]
pub fn github_token(api: &str) -> Option<String> {
    github_token_from(api, |key| std::env::var(key).ok(), gh_auth_token)
}

fn github_token_from(
    api: &str,
    env: impl Fn(&str) -> Option<String>,
    fallback: impl FnOnce(&str) -> Option<String>,
) -> Option<String> {
    let host = github_host(api)?;
    let keys = if host == "github.com" {
        ["GITHUB_TOKEN", "GH_TOKEN"]
    } else {
        ["GH_ENTERPRISE_TOKEN", "GITHUB_ENTERPRISE_TOKEN"]
    };
    keys.into_iter()
        .find_map(|key| env(key).filter(|value| !value.is_empty()))
        .or_else(|| fallback(&host))
}

/// The host gh files `api`'s token under: `github.com` for
/// `api.github.com`, else the API's own host and port.
fn github_host(api: &str) -> Option<String> {
    let url = reqwest::Url::parse(api).ok()?;
    let host = url.host_str()?;
    if host == "api.github.com" {
        return Some("github.com".to_owned());
    }
    Some(
        url.port()
            .map_or_else(|| host.to_owned(), |port| format!("{host}:{port}")),
    )
}

/// Asks the `gh` CLI for `host`'s stored token, the one place canibump runs
/// another program: gh keeps the token in the system keyring, not a file.
fn gh_auth_token(host: &str) -> Option<String> {
    let output = Command::new("gh")
        .args(["auth", "token", "--hostname", host])
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

/// Redmine API keys by host: `REDMINE_API_KEY` for `REDMINE_URL`'s host.
#[must_use]
pub fn redmine_keys() -> HashMap<String, String> {
    redmine_keys_from(|key| std::env::var(key).ok())
}

fn redmine_keys_from(env: impl Fn(&str) -> Option<String>) -> HashMap<String, String> {
    let set = |key: &str| env(key).filter(|value| !value.is_empty());
    match (set("REDMINE_URL"), set("REDMINE_API_KEY")) {
        (Some(url), Some(key)) => HashMap::from([(bare_host(&url), key)]),
        _ => HashMap::new(),
    }
}

/// How to sign in to a Jira host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JiraAuth {
    /// A personal access token (Jira Data Center).
    Bearer(String),
    /// An email and API token (Jira Cloud).
    Basic(String, String),
}

/// Jira credentials by host: `JIRA_TOKEN`, or `JIRA_USER` with
/// `JIRA_API_TOKEN`, for `JIRA_URL`'s host.
#[must_use]
pub fn jira_auth() -> HashMap<String, JiraAuth> {
    jira_auth_from(|key| std::env::var(key).ok())
}

fn jira_auth_from(env: impl Fn(&str) -> Option<String>) -> HashMap<String, JiraAuth> {
    let set = |key: &str| env(key).filter(|value| !value.is_empty());
    let Some(host) = set("JIRA_URL").map(|url| bare_host(&url)) else {
        return HashMap::new();
    };
    let auth = set("JIRA_TOKEN")
        .map(JiraAuth::Bearer)
        .or_else(|| Some(JiraAuth::Basic(set("JIRA_USER")?, set("JIRA_API_TOKEN")?)));
    auth.map(|auth| HashMap::from([(host, auth)]))
        .unwrap_or_default()
}

#[cfg(test)]
#[path = "auth_tests.rs"]
mod tests;
