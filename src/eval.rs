//! Turns a blocker's condition into an [`Outcome`].

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use crate::condition::{
    Bound, Condition, ReachedCondition, StateCondition, Step, VersionCondition, Within,
};
use crate::outcome::{Outcome, Status};
use crate::provider::{CheckError, CodeHost, GitRef, ItemKind, ItemState, Registry};
use crate::refs::{ItemRef, Reachable, Source};
use crate::version;

/// Checks a condition through the registered providers.
///
/// A provider error becomes a [`Status::Failed`] outcome, never a resolved
/// one.
pub async fn evaluate(condition: &Condition, registry: &Registry) -> Outcome {
    evaluate_in(condition, registry, &mut Outputs::new()).await
}

/// Values named steps hand to later ones, by step id.
type Outputs = HashMap<String, String>;

type Pending<'a> = Pin<Box<dyn Future<Output = Outcome> + Send + 'a>>;

fn evaluate_in<'a>(
    condition: &'a Condition,
    registry: &'a Registry,
    outputs: &'a mut Outputs,
) -> Pending<'a> {
    Box::pin(async move {
        let result = match condition {
            Condition::State(state) => evaluate_state(state, registry).await,
            Condition::Reached(reached) => evaluate_reached(reached, registry).await,
            Condition::Version(version) => evaluate_version(version, registry, outputs).await,
            Condition::All(steps) => Ok(all_of(steps, registry, outputs).await),
            Condition::Any(steps) => Ok(any_of(steps, registry, outputs).await),
        };
        result.unwrap_or_else(|error| Outcome::new(Status::Failed, error.to_string()))
    })
}

/// Runs one step and records its value under its id.
async fn run_step(step: &Step, registry: &Registry, outputs: &mut Outputs) -> Outcome {
    let outcome = evaluate_in(&step.condition, registry, outputs).await;
    if let (Some(id), Some(value)) = (&step.id, &outcome.value) {
        outputs.insert(id.clone(), value.clone());
    }
    outcome
}

/// Prefixes an outcome's detail with its step's id, or its 1-based position.
fn labelled(step: &Step, index: usize, mut outcome: Outcome) -> Outcome {
    let label = step
        .id
        .clone()
        .unwrap_or_else(|| format!("step {}", index + 1));
    outcome.detail = format!("{label}: {}", outcome.detail);
    outcome
}

/// Every step must hold; the first that does not decides the outcome.
async fn all_of(steps: &[Step], registry: &Registry, outputs: &mut Outputs) -> Outcome {
    let mut last = String::new();
    for (index, step) in steps.iter().enumerate() {
        let outcome = run_step(step, registry, outputs).await;
        if outcome.status != Status::Bumpable {
            return labelled(step, index, outcome);
        }
        last = outcome.detail;
    }
    Outcome::new(
        Status::Bumpable,
        format!("all {} steps hold; {last}", steps.len()),
    )
}

/// The first step that holds resolves it. Otherwise a failure wins over a
/// wait, and a wait over a dead step, so it reads dead only when all are.
async fn any_of(steps: &[Step], registry: &Registry, outputs: &mut Outputs) -> Outcome {
    let (mut failed, mut waiting, mut dead) = (None, None, None);
    for (index, step) in steps.iter().enumerate() {
        let outcome = labelled(step, index, run_step(step, registry, outputs).await);
        let slot = match outcome.status {
            Status::Bumpable => return outcome,
            Status::Failed => &mut failed,
            Status::Waiting => &mut waiting,
            Status::Dead => &mut dead,
        };
        slot.get_or_insert(outcome);
    }
    failed
        .or(waiting)
        .or(dead)
        .unwrap_or_else(|| Outcome::new(Status::Waiting, "no steps"))
}

async fn item_state(item: &ItemRef, registry: &Registry) -> Result<ItemState, CheckError> {
    registry.tracker(item.source.kind())?.item_state(item).await
}

fn state_outcome(item: &ItemRef, state: ItemState) -> Outcome {
    let (status, what) = describe(state);
    Outcome::new(status, format!("{item} {what}"))
}

async fn evaluate_state(
    condition: &StateCondition,
    registry: &Registry,
) -> Result<Outcome, CheckError> {
    let item = &condition.item;
    Ok(state_outcome(item, item_state(item, registry).await?))
}

/// The status an item state maps to, and how to say it.
const fn describe(state: ItemState) -> (Status, &'static str) {
    match state {
        ItemState::Open => (Status::Waiting, "is open"),
        ItemState::Merged => (Status::Bumpable, "merged"),
        ItemState::Completed => (Status::Bumpable, "closed as completed"),
        ItemState::ClosedUnmerged => (Status::Dead, "closed unmerged"),
        ItemState::NotPlanned => (Status::Dead, "closed as not planned"),
        ItemState::Duplicate => (Status::Dead, "closed as duplicate"),
        ItemState::ClosedNoReason => (Status::Dead, "closed without a reason; check it by hand"),
    }
}

fn short(sha: &str) -> &str {
    sha.get(..7).unwrap_or(sha)
}

/// The commit to follow and how to name it, or the outcome that ends the
/// check before any history is read (an open or rejected PR, or an issue).
async fn resolve_commit(
    target: &Reachable,
    registry: &Registry,
) -> Result<Result<(String, String), Outcome>, CheckError> {
    let item = match target {
        Reachable::Commit(commit) => {
            let label = format!("{}@{}", commit.source, short(&commit.sha));
            return Ok(Ok((commit.sha.clone(), label)));
        }
        Reachable::Item(item) => item,
    };
    let found = registry.tracker(item.source.kind())?.item(item).await?;
    // An issue has no merge commit to follow, open or closed. Only an issue
    // closes with a reason, so those states tell one even with no kind.
    match (found.state, found.kind) {
        (_, Some(ItemKind::Issue))
        | (
            ItemState::Completed
            | ItemState::NotPlanned
            | ItemState::Duplicate
            | ItemState::ClosedNoReason,
            _,
        ) => {
            return Ok(Err(Outcome::new(
                Status::Failed,
                format!("{item} is an issue, not a PR; wait on it with state"),
            )));
        }
        (ItemState::Merged, _) => {}
        (state @ (ItemState::Open | ItemState::ClosedUnmerged), _) => {
            return Ok(Err(state_outcome(item, state)));
        }
    }
    let host = registry.code_host(item.source.kind())?;
    let sha = host
        .merge_commit(item)
        .await?
        .ok_or_else(|| CheckError::Network(format!("{item} merged but has no merge commit")))?;
    let label = format!("{item} ({})", short(&sha));
    Ok(Ok((sha, label)))
}

async fn evaluate_reached(
    condition: &ReachedCondition,
    registry: &Registry,
) -> Result<Outcome, CheckError> {
    let (sha, label) = match resolve_commit(&condition.target, registry).await? {
        Ok(found) => found,
        Err(outcome) => return Ok(outcome),
    };
    let source = condition.target.source();
    let host = registry.code_host(source.kind())?;
    let found = Found {
        host,
        source,
        sha: &sha,
        label: &label,
    };
    match &condition.within {
        Within::Branch(branch) => found.within(GitRef::Branch(branch.clone())).await,
        Within::Tag(tag) => found.within(GitRef::Tag(tag.clone())).await,
        Within::Tags { pattern, since } => {
            let tags = candidate_tags(host, source, pattern, since.as_deref()).await?;
            Ok(match found.first_containing(&tags).await? {
                Some(tag) => {
                    Outcome::new(Status::Bumpable, format!("{label} is in {tag}")).with_value(tag)
                }
                None => Outcome::new(
                    Status::Waiting,
                    format!("{label} is in no {} tag yet", pattern.as_str()),
                ),
            })
        }
    }
}

/// A resolved commit on its host.
struct Found<'a> {
    host: &'a dyn CodeHost,
    source: &'a Source,
    sha: &'a str,
    label: &'a str,
}

impl Found<'_> {
    async fn holds(&self, tag: &str) -> Result<bool, CheckError> {
        self.host
            .contains(self.source, self.sha, &GitRef::Tag(tag.to_owned()))
            .await
    }

    /// The first of `tags` holding the commit, taking every tag after one
    /// that holds it to hold it too.
    async fn first_in<'t>(&self, tags: &'t [String]) -> Result<Option<&'t String>, CheckError> {
        let (mut low, mut high) = (0, tags.len());
        while low < high {
            let middle = low + (high - low) / 2;
            match tags.get(middle) {
                Some(tag) if self.holds(tag).await? => high = middle,
                Some(_) => low = middle + 1,
                None => break,
            }
        }
        Ok(tags.get(low))
    }

    /// The first of `tags`, in version order, holding the commit.
    ///
    /// Containment need not grow with the version across lines: a fix merged
    /// into 1.x only is in `v1.5.0` but not `v2.0.0`. Within a line it is
    /// taken to, so this asks each line's newest tag, lowest line first, and
    /// binary-searches the first line that holds it: one call per line plus
    /// about log2 of that line, not one per tag.
    async fn first_containing(&self, tags: &[String]) -> Result<Option<String>, CheckError> {
        for line in tags.chunk_by(|left, right| release_line(left) == release_line(right)) {
            let Some((newest, older)) = line.split_last() else {
                continue;
            };
            if self.holds(newest).await? {
                return Ok(Some(self.first_in(older).await?.unwrap_or(newest).clone()));
            }
        }
        Ok(None)
    }

    async fn within(&self, target: GitRef) -> Result<Outcome, CheckError> {
        let (label, name) = (self.label, target.name());
        Ok(
            if self.host.contains(self.source, self.sha, &target).await? {
                Outcome::new(Status::Bumpable, format!("{label} is in {name}"))
            } else {
                Outcome::new(Status::Waiting, format!("{label} is not in {name} yet"))
            },
        )
    }
}

async fn evaluate_version(
    condition: &VersionCondition,
    registry: &Registry,
    outputs: &Outputs,
) -> Result<Outcome, CheckError> {
    let package = &condition.package;
    let bound = match &condition.at_least {
        Bound::Version(version) => version,
        Bound::Step(step) => match outputs.get(step) {
            Some(found) => found,
            // A loaded manifest cannot reach this: `check_steps` refuses a
            // reference to a step that may not have held.
            None => {
                return Ok(Outcome::new(
                    Status::Failed,
                    format!("step `{step}` found no tag to compare against"),
                ));
            }
        },
    };
    let versions = registry.index(package.index())?.versions(package).await?;
    let highest = versions
        .iter()
        .max_by_key(|version| version::Key::new(version));
    Ok(match highest {
        None => Outcome::new(Status::Waiting, format!("{package} has no version yet")),
        Some(found) if version::compare(found, bound).is_ge() => Outcome::new(
            Status::Bumpable,
            format!("{package} is at {found}, at least {bound}"),
        ),
        Some(found) => Outcome::new(
            Status::Waiting,
            format!("{package} is at {found}, waiting for {bound}"),
        ),
    })
}

/// Tags matching `pattern`, at or after `since`, in ascending version order.
async fn candidate_tags(
    host: &dyn CodeHost,
    source: &Source,
    pattern: &glob::Pattern,
    since: Option<&str>,
) -> Result<Vec<String>, CheckError> {
    let text = pattern.as_str();
    let prefix = text
        .find(['*', '?', '[', '\\'])
        .and_then(|end| text.get(..end))
        .unwrap_or(text);
    let mut tags: Vec<String> = host
        .tags(source, prefix)
        .await?
        .into_iter()
        .filter(|tag| pattern.matches(tag))
        .filter(|tag| since.is_none_or(|since| version::compare(tag, since).is_ge()))
        .collect();
    version::sort(&mut tags);
    Ok(tags)
}

/// The release line of a tag: its first two number runs (`v1.4.2-rc1` is on
/// `1.4`), the first alone for a two-run tag (`v1.5` is on `1`), and none for
/// a build number (every `b7012` shares one line).
fn release_line(tag: &str) -> Vec<&str> {
    let mut runs: Vec<&str> = tag
        .split(|char: char| !char.is_ascii_digit())
        .filter(|run| !run.is_empty())
        .collect();
    runs.truncate(runs.len().saturating_sub(1).min(2));
    runs
}

#[cfg(test)]
#[path = "eval_tests.rs"]
mod tests;
