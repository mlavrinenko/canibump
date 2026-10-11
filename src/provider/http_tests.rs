use std::collections::HashMap;

use httpmock::Method::GET;
use httpmock::{Mock, MockServer};
use serde_json::{Value, json};

use super::super::gitea::Gitea;
use super::super::gitlab::GitLab;
use super::super::redmine::Redmine;
use super::super::{CheckError, ItemState, Tracker};
use super::{Url, credential};
use crate::refs::ItemRef;

const GITLAB_ISSUE: &str = "/api/v4/projects/group%2Fproj/issues/1";
const REDMINE_ISSUE: &str = "/issues/1.json";
const GITEA_ISSUE: &str = "/api/v1/repos/o/r/issues/1";

/// A credential for `server` and for the item's own `host`, so a test
/// holds whichever one the provider looks up.
fn keys(server: &MockServer, host: &str) -> HashMap<String, String> {
    HashMap::from([
        (server.address().to_string(), "s3cret".to_owned()),
        (host.to_owned(), "s3cret".to_owned()),
    ])
}

/// Answers `path` on `server` with `body`, carrying `header` or not.
async fn answer<'a>(server: &'a MockServer, path: &str, header: &str, body: &Value) -> Mock<'a> {
    server
        .mock_async(|when, then| {
            when.method(GET).path(path).header_exists(header);
            then.status(200).json_body(body.clone());
        })
        .await
}

/// Sends `from` on `origin` to `to` on `target` with a 302.
async fn redirect(origin: &MockServer, from: &str, target: &MockServer, to: &str) {
    origin
        .mock_async(|when, then| {
            when.method(GET).path(from);
            then.status(302).header("location", target.url(to));
        })
        .await;
}

/// Sends `path` from `origin` to the same path on `target`, which answers
/// `body` to any request.
async fn redirect_off<'a>(
    origin: &MockServer,
    target: &'a MockServer,
    path: &str,
    body: &Value,
) -> Mock<'a> {
    redirect(origin, path, target, path).await;
    target
        .mock_async(|when, then| {
            when.method(GET).path(path);
            then.status(200).json_body(body.clone());
        })
        .await
}

async fn state(tracker: &dyn Tracker, text: &str) -> Result<ItemState, CheckError> {
    let item: ItemRef = text.parse().expect("valid ref");
    tracker.item_state(&item).await
}

/// The check fails naming the refused hop, and `target` saw nothing.
async fn assert_refused(result: Result<ItemState, CheckError>, target: &Mock<'_>) {
    assert_eq!(target.calls_async().await, 0, "{result:?}");
    let Err(CheckError::Network(message)) = result else {
        panic!("want a network error, got {result:?}");
    };
    assert!(
        message.contains("refused a redirect to another origin"),
        "{message}"
    );
}

#[tokio::test]
async fn gitlab_token_stays_off_a_redirect_to_another_origin() {
    let (origin, target) = (
        MockServer::start_async().await,
        MockServer::start_async().await,
    );
    let target = redirect_off(&origin, &target, GITLAB_ISSUE, &json!({"state": "opened"})).await;
    let gitlab =
        GitLab::new(Some(&origin.base_url()), keys(&origin, "gitlab.com")).expect("client");
    assert_refused(state(&gitlab, "gitlab:group/proj#1").await, &target).await;
}

#[tokio::test]
async fn redmine_key_stays_off_a_redirect_to_another_origin() {
    let (origin, target) = (
        MockServer::start_async().await,
        MockServer::start_async().await,
    );
    let body = json!({"issue": {"status": {"name": "New"}}});
    let target = redirect_off(&origin, &target, REDMINE_ISSUE, &body).await;
    let redmine = Redmine::new(
        Some(&origin.base_url()),
        keys(&origin, "redmine.example.org"),
    )
    .expect("client");
    assert_refused(
        state(&redmine, "redmine:redmine.example.org#1").await,
        &target,
    )
    .await;
}

#[tokio::test]
async fn gitea_token_stays_off_a_redirect_to_another_origin() {
    let (origin, target) = (
        MockServer::start_async().await,
        MockServer::start_async().await,
    );
    let target = redirect_off(&origin, &target, GITEA_ISSUE, &json!({"state": "open"})).await;
    let gitea =
        Gitea::new(Some(&origin.base_url()), keys(&origin, "codeberg.org")).expect("client");
    assert_refused(state(&gitea, "gitea:codeberg.org/o/r#1").await, &target).await;
}

#[tokio::test]
async fn a_redirect_within_the_origin_keeps_its_token() {
    let server = MockServer::start_async().await;
    redirect(
        &server,
        "/old/api/v4/projects/group%2Fproj/issues/1",
        &server,
        GITLAB_ISSUE,
    )
    .await;
    let answered = answer(
        &server,
        GITLAB_ISSUE,
        "private-token",
        &json!({"state": "opened"}),
    )
    .await;
    let mut tokens = keys(&server, "gitlab.com");
    tokens.insert(format!("{}/old", server.address()), "s3cret".to_owned());
    let gitlab = GitLab::new(Some(&server.url("/old")), tokens).expect("client");
    assert_eq!(
        state(&gitlab, "gitlab:group/proj#1").await,
        Ok(ItemState::Open)
    );
    assert_eq!(answered.calls_async().await, 1);
}

/// Asks for an issue through `hops` same-origin redirects.
async fn through(hops: usize) -> Result<ItemState, CheckError> {
    let server = MockServer::start_async().await;
    let at = |hop: usize| format!("/hop{hop}{GITLAB_ISSUE}");
    for hop in 0..hops {
        redirect(&server, &at(hop), &server, &at(hop + 1)).await;
    }
    answer(
        &server,
        &at(hops),
        "user-agent",
        &json!({"state": "opened"}),
    )
    .await;
    let gitlab = GitLab::new(Some(&server.url("/hop0")), HashMap::new()).expect("client");
    state(&gitlab, "gitlab:group/proj#1").await
}

#[tokio::test]
async fn follows_ten_redirects_and_refuses_the_eleventh() {
    assert_eq!(through(10).await, Ok(ItemState::Open));
    let result = through(11).await;
    let Err(CheckError::Network(message)) = result else {
        panic!("want a network error, got {result:?}");
    };
    assert_eq!(message, "gitlab:group/proj#1: too many redirects");
}

/// Asks `tracker` built with `server`'s own credential, then with only
/// `host`'s, for `item`, counting what reached `server` with `header`.
async fn assert_override_keyed_by_itself(
    server: &MockServer,
    answered: &Mock<'_>,
    host: &str,
    tracker: impl Fn(HashMap<String, String>) -> Box<dyn Tracker>,
    item: &str,
) {
    let own = HashMap::from([(server.address().to_string(), "s3cret".to_owned())]);
    let foreign = HashMap::from([(host.to_owned(), "s3cret".to_owned())]);
    for (credentials, want) in [(foreign, 0), (own, 1)] {
        let result = state(&*tracker(credentials), item).await;
        assert_eq!(answered.calls_async().await, want, "{result:?}");
    }
}

#[tokio::test]
async fn a_gitlab_override_gets_its_own_token_never_the_items_host() {
    let server = MockServer::start_async().await;
    let body = json!({"state": "opened"});
    let answered = answer(&server, GITLAB_ISSUE, "private-token", &body).await;
    let base = server.base_url();
    let gitlab =
        |tokens| Box::new(GitLab::new(Some(&base), tokens).expect("client")) as Box<dyn Tracker>;
    assert_override_keyed_by_itself(
        &server,
        &answered,
        "gitlab.com",
        gitlab,
        "gitlab:group/proj#1",
    )
    .await;
}

#[tokio::test]
async fn a_redmine_override_gets_its_own_key_never_the_items_host() {
    let server = MockServer::start_async().await;
    let body = json!({"issue": {"status": {"name": "New"}}});
    let answered = answer(&server, REDMINE_ISSUE, "x-redmine-api-key", &body).await;
    let base = server.base_url();
    let redmine =
        |keys| Box::new(Redmine::new(Some(&base), keys).expect("client")) as Box<dyn Tracker>;
    let item = "redmine:redmine.example.org#1";
    assert_override_keyed_by_itself(&server, &answered, "redmine.example.org", redmine, item).await;
}

#[test]
fn keys_a_credential_by_where_the_request_goes() {
    let credentials = HashMap::from([
        ("gitlab.com".to_owned(), 1),
        ("ghe.example.com:8443".to_owned(), 2),
        ("example.org/redmine".to_owned(), 3),
    ]);
    let url = |text: &str| Url::parse(text).expect("url");
    let at =
        |base: Option<&str>, host: &str| credential(&credentials, base.map(url).as_ref(), host);
    assert_eq!(at(None, "gitlab.com"), Some(&1));
    assert_eq!(at(None, "other.example.org"), None);
    assert_eq!(
        at(Some("https://ghe.example.com:8443/"), "gitlab.com"),
        Some(&2)
    );
    assert_eq!(
        at(Some("https://ghe.example.com"), "ghe.example.com:8443"),
        None
    );
    assert_eq!(
        at(Some("https://example.org/redmine/"), "gitlab.com"),
        Some(&3)
    );
    assert_eq!(at(Some("https://example.org"), "example.org/redmine"), None);
    assert_eq!(at(Some("https://gitlab.com:443"), "x"), Some(&1));
}
