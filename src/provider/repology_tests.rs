use std::time::{Duration, Instant};

use httpmock::Method::GET;
use httpmock::MockServer;
use serde_json::json;

use super::*;

fn nix(project: &str, srcname: Option<&str>) -> PackageRef {
    PackageRef::parse(
        &format!("repology:{project}"),
        Some("nix_unstable".to_owned()),
        srcname.map(str::to_owned),
    )
    .expect("valid")
}

#[tokio::test]
async fn filters_by_repo_and_srcname_and_asks_once_per_project() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(GET).path("/project/llama.cpp").header_matches(
                "user-agent",
                "^canibump/.+ \\(\\+https://github.com/mlavrinenko/canibump\\)$",
            );
            then.status(200).json_body(json!([
                {"repo": "nix_unstable", "srcname": "llama-cpp", "version": "0.6.0"},
                {"repo": "nix_unstable", "srcname": "llama-cpp-rocm", "version": "0.5.0"},
                {"repo": "nix_stable_25_11", "srcname": "llama-cpp", "version": "b6981"},
            ]));
        })
        .await;
    let repology = Repology::new(&server.base_url(), Duration::ZERO).expect("client");
    assert_eq!(
        repology.versions(&nix("llama.cpp", None)).await,
        Ok(vec!["0.6.0".to_owned(), "0.5.0".to_owned()])
    );
    assert_eq!(
        repology
            .versions(&nix("llama.cpp", Some("llama-cpp-rocm")))
            .await,
        Ok(vec!["0.5.0".to_owned()])
    );
    mock.assert_calls_async(1).await;
}

#[tokio::test]
async fn an_unknown_project_is_not_found() {
    let server = MockServer::start_async().await;
    server
        .mock_async(|when, then| {
            when.method(GET).path("/project/nope");
            then.status(200).json_body(json!([]));
        })
        .await;
    let repology = Repology::new(&server.base_url(), Duration::ZERO).expect("client");
    assert_eq!(
        repology.versions(&nix("nope", None)).await,
        Err(CheckError::NotFound("repology:nope".to_owned()))
    );
}

#[tokio::test]
async fn spaces_requests_by_the_interval() {
    let server = MockServer::start_async().await;
    server
        .mock_async(|when, then| {
            when.method(GET);
            then.status(200)
                .json_body(json!([{"repo": "nix_unstable", "version": "1"}]));
        })
        .await;
    let repology = Repology::new(&server.base_url(), Duration::from_millis(300)).expect("client");
    let started = Instant::now();
    for project in ["a", "b", "c"] {
        repology
            .versions(&nix(project, None))
            .await
            .expect("versions");
    }
    assert!(
        started.elapsed() >= Duration::from_millis(600),
        "{:?}",
        started.elapsed()
    );
}

#[tokio::test]
async fn refuses_a_deps_dev_package() {
    let repology = Repology::new("http://127.0.0.1:9", Duration::ZERO).expect("client");
    let package = PackageRef::parse("deps.dev:cargo/serde", None, None).expect("valid");
    assert!(matches!(
        repology.versions(&package).await,
        Err(CheckError::Unsupported(_))
    ));
}
