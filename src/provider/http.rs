//! The HTTP client the JSON APIs share, and its error mapping.

use std::time::Duration;

use reqwest::{Client, RequestBuilder, StatusCode, Url};
use serde::de::DeserializeOwned;

use super::CheckError;

/// `canibump/<version> (+<repo>)`, which Repology's API rules ask for.
pub const USER_AGENT: &str = concat!(
    "canibump/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/mlavrinenko/canibump)"
);

/// A client with canibump's User-Agent and timeouts.
pub fn client() -> Result<Client, CheckError> {
    Client::builder()
        .user_agent(USER_AGENT)
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

/// Sends a request and decodes its JSON, naming `what` in any error.
pub async fn send_json<T: DeserializeOwned>(
    request: RequestBuilder,
    what: &str,
) -> Result<T, CheckError> {
    let network = |error: &dyn std::fmt::Display| CheckError::Network(format!("{what}: {error}"));
    let response = request.send().await.map_err(|error| network(&error))?;
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
