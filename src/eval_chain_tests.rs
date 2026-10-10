//! all-of and any-of chains: steps in order, values handed on.

use super::*;

fn step(id: Option<&str>, condition: Condition) -> Step {
    Step {
        id: id.map(str::to_owned),
        condition,
    }
}

fn state_step(id: Option<&str>, number: u64) -> Step {
    let item: ItemRef = format!("github:o/r#{number}").parse().expect("valid ref");
    step(id, Condition::State(StateCondition { item }))
}

fn chain_registry(states: &[(u64, ItemState)], packaged: &[&str]) -> Registry {
    let fake = Fake {
        states: states
            .iter()
            .map(|(number, state)| (*number, Ok(*state)))
            .collect(),
        ..released_from(13)
    };
    let (registry, _) = registry(fake);
    let versions = packaged
        .iter()
        .map(|version| (*version).to_owned())
        .collect();
    registry.with_index(IndexKind::Repology, Arc::new(FakeIndex(Ok(versions))))
}

fn chain() -> Condition {
    let package =
        PackageRef::parse("repology:x", Some("nix_unstable".to_owned()), None).expect("valid");
    Condition::All(vec![
        state_step(Some("merged"), 1),
        step(
            Some("released"),
            Condition::Reached(ReachedCondition {
                target: "github:o/r#1".parse().expect("valid"),
                within: tags("v*", None),
            }),
        ),
        step(
            None,
            Condition::Version(VersionCondition {
                package,
                at_least: Bound::Step("released".to_owned()),
            }),
        ),
    ])
}

#[tokio::test]
async fn an_all_of_resolves_only_when_every_step_holds() {
    let merged = [(1, ItemState::Merged)];
    let outcome = evaluate(&chain(), &chain_registry(&merged, &["0.13.0"])).await;
    assert_eq!(
        outcome,
        Outcome::new(
            Status::Bumpable,
            "all 3 steps hold; repology:x in nix_unstable is at 0.13.0, at least v0.13.0"
        )
    );
    let outcome = evaluate(&chain(), &chain_registry(&merged, &["0.12.0"])).await;
    assert_eq!(
        outcome,
        Outcome::new(
            Status::Waiting,
            "step 3: repology:x in nix_unstable is at 0.12.0, waiting for v0.13.0"
        )
    );
}

#[tokio::test]
async fn a_chain_naming_one_item_in_two_steps_asks_about_it_once() {
    let (registry, fake) = registry(released_from(13));
    let released = ReachedCondition {
        target: "github:o/r#1".parse().expect("valid"),
        within: tags("v*", None),
    };
    let chain = Condition::All(vec![
        state_step(Some("merged"), 1),
        step(Some("released"), Condition::Reached(released)),
    ]);
    assert_eq!(evaluate(&chain, &registry).await.status, Status::Bumpable);
    assert_eq!(fake.state_calls.load(Ordering::SeqCst), 1);
    assert_eq!(fake.merge_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn an_all_of_stops_at_the_first_step_that_does_not_hold() {
    for (state, status, detail) in [
        (
            ItemState::Open,
            Status::Waiting,
            "merged: github:o/r#1 is open",
        ),
        (
            ItemState::ClosedUnmerged,
            Status::Dead,
            "merged: github:o/r#1 closed unmerged",
        ),
    ] {
        let outcome = evaluate(&chain(), &chain_registry(&[(1, state)], &[])).await;
        assert_eq!(outcome, Outcome::new(status, detail));
    }
    let failing = Condition::All(vec![state_step(None, 9), state_step(None, 1)]);
    let outcome = evaluate(&failing, &chain_registry(&[(1, ItemState::Merged)], &[])).await;
    assert_eq!(
        outcome,
        Outcome::new(Status::Failed, "step 1: github:o/r#9: not found")
    );
}

#[tokio::test]
async fn an_any_of_resolves_on_its_first_step_that_holds() {
    let states = [
        (1, ItemState::Open),
        (2, ItemState::Merged),
        (3, ItemState::NotPlanned),
        (4, ItemState::Duplicate),
    ];
    let registry = chain_registry(&states, &[]);
    let any = |numbers: &[u64]| {
        Condition::Any(
            numbers
                .iter()
                .map(|number| state_step(None, *number))
                .collect(),
        )
    };
    assert_eq!(
        evaluate(&any(&[1, 2]), &registry).await,
        Outcome::new(Status::Bumpable, "step 2: github:o/r#2 merged")
    );
    assert_eq!(
        evaluate(&any(&[3, 1]), &registry).await,
        Outcome::new(Status::Waiting, "step 2: github:o/r#1 is open")
    );
    assert_eq!(
        evaluate(&any(&[3, 4]), &registry).await,
        Outcome::new(Status::Dead, "step 1: github:o/r#3 closed as not planned")
    );
    assert_eq!(
        evaluate(&any(&[1, 9]), &registry).await,
        Outcome::new(Status::Failed, "step 2: github:o/r#9: not found")
    );
}

#[tokio::test]
async fn a_step_bound_without_a_found_tag_fails() {
    let package = PackageRef::parse("repology:x", Some("r".to_owned()), None).expect("valid");
    let condition = Condition::Version(VersionCondition {
        package,
        at_least: Bound::Step("released".to_owned()),
    });
    let outcome = evaluate(&condition, &chain_registry(&[], &["1"])).await;
    assert_eq!(
        outcome,
        Outcome::new(
            Status::Failed,
            "step `released` found no tag to compare against"
        )
    );
}
