use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;

use super::*;
use crate::condition::{Bound, ReachedCondition, StateCondition, Step, VersionCondition, Within};
use crate::package::{IndexKind, PackageRef};
use crate::provider::{CheckError, CodeHost, GitRef, ItemState, PackageIndex, Registry, Tracker};
use crate::refs::{ItemRef, Source, SourceKind};

#[derive(Default)]
struct Fake {
    states: HashMap<u64, Result<ItemState, CheckError>>,
    merge_commits: HashMap<u64, String>,
    containing: HashSet<String>,
    tags: Vec<String>,
    host_error: Option<CheckError>,
    contains_calls: AtomicUsize,
    state_calls: AtomicUsize,
    merge_calls: AtomicUsize,
}

#[async_trait]
impl Tracker for Fake {
    async fn item_state(&self, item: &ItemRef) -> Result<ItemState, CheckError> {
        self.state_calls.fetch_add(1, Ordering::SeqCst);
        self.states
            .get(&item.number)
            .cloned()
            .unwrap_or_else(|| Err(CheckError::NotFound(item.to_string())))
    }
}

#[async_trait]
impl CodeHost for Fake {
    async fn merge_commit(&self, item: &ItemRef) -> Result<Option<String>, CheckError> {
        self.merge_calls.fetch_add(1, Ordering::SeqCst);
        Ok(self.merge_commits.get(&item.number).cloned())
    }

    async fn contains(
        &self,
        _source: &Source,
        commit: &str,
        target: &GitRef,
    ) -> Result<bool, CheckError> {
        self.contains_calls.fetch_add(1, Ordering::SeqCst);
        if let Some(error) = &self.host_error {
            return Err(error.clone());
        }
        assert_eq!(commit, "abc1234def");
        Ok(self.containing.contains(target.name()))
    }

    async fn tags(&self, _source: &Source, prefix: &str) -> Result<Vec<String>, CheckError> {
        Ok(self
            .tags
            .iter()
            .filter(|tag| tag.starts_with(prefix))
            .cloned()
            .collect())
    }
}

fn registry(fake: Fake) -> (Registry, Arc<Fake>) {
    let fake = Arc::new(fake);
    let registry = Registry::default()
        .with_tracker(SourceKind::GitHub, fake.clone())
        .with_code_host(SourceKind::GitHub, fake.clone());
    (registry, fake)
}

async fn outcome_for(state: Result<ItemState, CheckError>) -> Outcome {
    let (registry, _) = registry(Fake {
        states: HashMap::from([(1, state)]),
        ..Fake::default()
    });
    let item: ItemRef = "github:o/r#1".parse().expect("valid ref");
    evaluate(&Condition::State(StateCondition { item }), &registry).await
}

#[tokio::test]
async fn open_waits() {
    let outcome = outcome_for(Ok(ItemState::Open)).await;
    assert_eq!(outcome.status, Status::Waiting);
    assert_eq!(outcome.detail, "github:o/r#1 is open");
}

#[tokio::test]
async fn merged_and_completed_are_bumpable() {
    let merged = outcome_for(Ok(ItemState::Merged)).await;
    assert_eq!(merged.status, Status::Bumpable);
    assert_eq!(merged.detail, "github:o/r#1 merged");
    let completed = outcome_for(Ok(ItemState::Completed)).await;
    assert_eq!(completed.status, Status::Bumpable);
    assert_eq!(completed.detail, "github:o/r#1 closed as completed");
}

#[tokio::test]
async fn rejections_are_dead_never_bumpable() {
    for (state, detail) in [
        (ItemState::ClosedUnmerged, "github:o/r#1 closed unmerged"),
        (ItemState::NotPlanned, "github:o/r#1 closed as not planned"),
        (ItemState::Duplicate, "github:o/r#1 closed as duplicate"),
        (
            ItemState::ClosedNoReason,
            "github:o/r#1 closed without a reason; check it by hand",
        ),
    ] {
        let outcome = outcome_for(Ok(state)).await;
        assert_eq!(outcome.status, Status::Dead, "{detail}");
        assert_eq!(outcome.detail, detail);
    }
}

#[tokio::test]
async fn a_provider_error_fails() {
    let outcome = outcome_for(Err(CheckError::RateLimited("github".to_owned()))).await;
    assert_eq!(outcome.status, Status::Failed);
    assert_eq!(outcome.detail, "github: rate limited");
}

#[tokio::test]
async fn a_missing_provider_fails() {
    let item: ItemRef = "github:o/r#1".parse().expect("valid ref");
    let outcome = evaluate(
        &Condition::State(StateCondition { item }),
        &Registry::default(),
    )
    .await;
    assert_eq!(outcome.status, Status::Failed);
    assert!(outcome.detail.contains("no provider"), "{}", outcome.detail);
}

fn merged_pr() -> Fake {
    Fake {
        states: HashMap::from([(1, Ok(ItemState::Merged))]),
        merge_commits: HashMap::from([(1, "abc1234def".to_owned())]),
        ..Fake::default()
    }
}

async fn reach(fake: Fake, target: &str, within: Within) -> (Outcome, Arc<Fake>) {
    let (registry, fake) = registry(fake);
    let condition = Condition::Reached(ReachedCondition {
        target: target.parse().expect("valid target"),
        within,
    });
    (evaluate(&condition, &registry).await, fake)
}

#[tokio::test]
async fn a_merged_pr_in_a_branch_is_bumpable() {
    let fake = Fake {
        containing: HashSet::from(["main".to_owned()]),
        ..merged_pr()
    };
    let (outcome, _) = reach(fake, "github:o/r#1", Within::Branch("main".to_owned())).await;
    assert_eq!(
        outcome,
        Outcome::new(Status::Bumpable, "github:o/r#1 (abc1234) is in main")
    );
    let (outcome, _) = reach(
        merged_pr(),
        "github:o/r#1",
        Within::Branch("main".to_owned()),
    )
    .await;
    assert_eq!(
        outcome,
        Outcome::new(Status::Waiting, "github:o/r#1 (abc1234) is not in main yet")
    );
}

#[tokio::test]
async fn an_unmerged_pr_waits_or_is_dead() {
    for (state, status, detail) in [
        (ItemState::Open, Status::Waiting, "github:o/r#1 is open"),
        (
            ItemState::ClosedUnmerged,
            Status::Dead,
            "github:o/r#1 closed unmerged",
        ),
    ] {
        let fake = Fake {
            states: HashMap::from([(1, Ok(state))]),
            ..Fake::default()
        };
        let (outcome, _) = reach(fake, "github:o/r#1", Within::Tag("v1".to_owned())).await;
        assert_eq!(outcome, Outcome::new(status, detail));
    }
}

#[tokio::test]
async fn an_issue_in_reached_fails_without_reading_history() {
    for state in [
        ItemState::Completed,
        ItemState::NotPlanned,
        ItemState::Duplicate,
        ItemState::ClosedNoReason,
    ] {
        let fake = Fake {
            states: HashMap::from([(1, Ok(state))]),
            ..Fake::default()
        };
        let (outcome, fake) = reach(fake, "github:o/r#1", Within::Tag("v1".to_owned())).await;
        assert_eq!(
            outcome,
            Outcome::new(
                Status::Failed,
                "github:o/r#1 is an issue, not a PR; wait on it with state"
            ),
            "{state:?}"
        );
        assert_eq!(fake.merge_calls.load(Ordering::SeqCst), 0, "{state:?}");
        assert_eq!(fake.contains_calls.load(Ordering::SeqCst), 0, "{state:?}");
    }
}

#[tokio::test]
async fn a_merged_pr_without_a_merge_commit_fails() {
    let fake = Fake {
        states: HashMap::from([(1, Ok(ItemState::Merged))]),
        ..Fake::default()
    };
    let (outcome, _) = reach(fake, "github:o/r#1", Within::Tag("v1".to_owned())).await;
    assert_eq!(outcome.status, Status::Failed);
}

#[tokio::test]
async fn a_commit_in_a_tag_is_bumpable() {
    let fake = Fake {
        containing: HashSet::from(["v1".to_owned()]),
        ..Fake::default()
    };
    let (outcome, _) = reach(fake, "github:o/r@abc1234def", Within::Tag("v1".to_owned())).await;
    assert_eq!(
        outcome,
        Outcome::new(Status::Bumpable, "github:o/r@abc1234 is in v1")
    );
}

fn released_from(first: u32) -> Fake {
    let tags: Vec<String> = (1..=40)
        .map(|minor| format!("v0.{minor}.0"))
        .chain(["b100".to_owned(), "b200".to_owned()])
        .collect();
    let containing = (first..=40).map(|minor| format!("v0.{minor}.0")).collect();
    Fake {
        tags,
        containing,
        ..merged_pr()
    }
}

fn tags(pattern: &str, since: Option<&str>) -> Within {
    Within::Tags {
        pattern: glob::Pattern::new(pattern).expect("glob"),
        since: since.map(str::to_owned),
    }
}

#[tokio::test]
async fn finds_the_first_matching_tag_by_binary_search() {
    let (outcome, fake) = reach(released_from(13), "github:o/r#1", tags("v*", None)).await;
    assert_eq!(
        outcome,
        Outcome::new(Status::Bumpable, "github:o/r#1 (abc1234) is in v0.13.0")
            .with_value("v0.13.0")
    );
    assert!(fake.contains_calls.load(Ordering::SeqCst) <= 7);
}

#[tokio::test]
async fn since_skips_older_tags() {
    let (outcome, _) = reach(
        released_from(13),
        "github:o/r#1",
        tags("v*", Some("0.20.0")),
    )
    .await;
    assert_eq!(outcome.detail, "github:o/r#1 (abc1234) is in v0.20.0");
}

#[tokio::test]
async fn waits_when_no_matching_tag_has_it() {
    let (outcome, _) = reach(released_from(41), "github:o/r#1", tags("v*", None)).await;
    assert_eq!(
        outcome,
        Outcome::new(
            Status::Waiting,
            "github:o/r#1 (abc1234) is in no v* tag yet"
        )
    );
    let (outcome, _) = reach(released_from(1), "github:o/r#1", tags("x*", None)).await;
    assert_eq!(outcome.status, Status::Waiting);
}

#[tokio::test]
async fn a_host_error_fails_the_reach() {
    let fake = Fake {
        host_error: Some(CheckError::Network("boom".to_owned())),
        ..released_from(1)
    };
    let (outcome, _) = reach(fake, "github:o/r#1", tags("v*", None)).await;
    assert_eq!(outcome, Outcome::new(Status::Failed, "boom"));
}

struct FakeIndex(Result<Vec<String>, CheckError>);

#[async_trait]
impl PackageIndex for FakeIndex {
    async fn versions(&self, _package: &PackageRef) -> Result<Vec<String>, CheckError> {
        self.0.clone()
    }
}

async fn version_outcome(versions: Result<Vec<&str>, CheckError>, at_least: &str) -> Outcome {
    let versions = versions.map(|list| list.into_iter().map(str::to_owned).collect());
    let registry =
        Registry::default().with_index(IndexKind::Repology, Arc::new(FakeIndex(versions)));
    let package = PackageRef::parse("repology:llama.cpp", Some("nix_unstable".to_owned()), None)
        .expect("valid");
    let condition = Condition::Version(VersionCondition {
        package,
        at_least: Bound::Version(at_least.to_owned()),
    });
    evaluate(&condition, &registry).await
}

#[tokio::test]
async fn a_version_at_or_above_the_bound_is_bumpable() {
    assert_eq!(
        version_outcome(Ok(vec!["0.5.0", "0.6.0"]), "0.6.0").await,
        Outcome::new(
            Status::Bumpable,
            "repology:llama.cpp in nix_unstable is at 0.6.0, at least 0.6.0"
        )
    );
    assert_eq!(
        version_outcome(Ok(vec!["b6981"]), "b7012").await,
        Outcome::new(
            Status::Waiting,
            "repology:llama.cpp in nix_unstable is at b6981, waiting for b7012"
        )
    );
}

#[tokio::test]
async fn a_package_missing_from_the_repo_waits() {
    assert_eq!(
        version_outcome(Ok(vec![]), "1").await,
        Outcome::new(
            Status::Waiting,
            "repology:llama.cpp in nix_unstable has no version yet"
        )
    );
    let failed = version_outcome(Err(CheckError::NotFound("x".to_owned())), "1").await;
    assert_eq!(failed.status, Status::Failed);
}

#[path = "eval_chain_tests.rs"]
mod chain;
