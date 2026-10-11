//! The HTTP client the JSON APIs share, and its error mapping.

use std::time::Duration;

use reqwest::{Client, RequestBuilder, StatusCode, Url, redirect};
use serde::de::DeserializeOwned;

use super::CheckError;

/// `canibump/<version> (+<repo>)`, which Repology's API rules ask for.
pub const USER_AGENT: &str = concat!(
    "canibump/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/mlavrinenko/canibump)"
);

/// How many redirects one request follows, reqwest's default.
const MAX_REDIRECTS: usize = 10;

/// A redirect off the origin a request started on, refused.
#[derive(Debug, thiserror::Error)]
#[error("refused a redirect to another origin: {0}")]
struct CrossOrigin(Url);

/// Follows a redirect only within the origin (scheme, host, port) the
/// request started on. reqwest strips `Authorization` across origins but
/// not `PRIVATE-TOKEN` or `X-Redmine-API-Key`, and a policy sees URLs, not
/// headers, so every request refuses the hop: no credential leaves its host.
fn same_origin(attempt: redirect::Attempt<'_>) -> redirect::Action {
    let start = attempt.previous().first().map(Url::origin);
    if start.is_some_and(|start| start != attempt.url().origin()) {
        let next = attempt.url().clone();
        return attempt.error(CrossOrigin(next));
    }
    if attempt.previous().len() > MAX_REDIRECTS {
        return attempt.error("too many redirects");
    }
    attempt.follow()
}

/// A client with canibump's User-Agent, timeouts and same-origin redirects.
pub fn client() -> Result<Client, CheckError> {
    Client::builder()
        .user_agent(USER_AGENT)
        .redirect(redirect::Policy::custom(same_origin))
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| CheckError::Unsupported(format!("HTTP client: {error}")))
}

/// Parses an API root, so a bad override fails at start-up.
pub fn base(text: &str) -> Result<Url, CheckError> {
    Url::parse(text).map_err(|error| CheckError::Unsupported(format!("API URL `{text}`: {error}")))
}

/// `base` when given (a mock, a mirror), else `https://<host>`.
pub fn host_url(base_url: Option<&Url>, host: &str) -> Result<Url, CheckError> {
    match base_url {
        Some(url) => Ok(url.clone()),
        None => base(&format!("https://{host}")),
    }
}

/// GETs `url` and decodes its JSON, naming `what` in any error.
pub async fn get_json<T: DeserializeOwned>(
    client: &Client,
    url: Url,
    what: &str,
) -> Result<T, CheckError> {
    send_json(client.get(url), what).await
}

/// A refused redirect says why, which reqwest's own text leaves out.
fn why(error: &reqwest::Error) -> String {
    match std::error::Error::source(error) {
        Some(reason) if error.is_redirect() => reason.to_string(),
        _ => error.to_string(),
    }
}

/// Sends a request and decodes its JSON, naming `what` in any error.
pub async fn send_json<T: DeserializeOwned>(
    request: RequestBuilder,
    what: &str,
) -> Result<T, CheckError> {
    let network = |error: &dyn std::fmt::Display| CheckError::Network(format!("{what}: {error}"));
    let response = request
        .send()
        .await
        .map_err(|error| network(&why(&error)))?;
    let status = response.status();
    let what = what.to_owned();
    match status {
        StatusCode::NOT_FOUND => return Err(CheckError::NotFound(what)),
        StatusCode::TOO_MANY_REQUESTS => return Err(CheckError::RateLimited(what)),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => return Err(CheckError::Auth(what)),
        status if !status.is_success() => {
            return Err(CheckError::Network(format!(
                "{what}: HTTP {}",
                status.as_u16()
            )));
        }
        _ => {}
    }
    response.json().await.map_err(|error| network(&error))
}

#[cfg(test)]
#[path = "http_tests.rs"]
mod tests;
