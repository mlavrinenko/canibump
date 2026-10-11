use gix::refs::transaction::PreviousValue;

use super::*;
use crate::refs::ItemKind;

/// Builds commits over one empty tree in a bare repo.
struct Builder {
    repo: gix::Repository,
    tree: gix::ObjectId,
    signature: gix::actor::Signature,
}

impl Builder {
    fn new(path: &Path) -> Self {
        let repo = gix::init_bare(path).expect("init");
        let tree = repo
            .write_object(gix::objs::Tree::empty())
            .expect("tree")
            .detach();
        let signature = gix::actor::Signature {
            name: "t".into(),
            email: "t@t".into(),
            time: gix::date::Time::new(1_700_000_000, 0),
        };
        Self {
            repo,
            tree,
            signature,
        }
    }

    /// A commit on `reference`, which must point at the first parent.
    fn commit(&self, reference: &str, message: &str, parents: Vec<gix::ObjectId>) -> gix::ObjectId {
        let (mut author, mut committer) = Default::default();
        self.repo
            .commit_as(
                self.signature.to_ref(&mut author),
                self.signature.to_ref(&mut committer),
                reference,
                message,
                self.tree,
                parents,
            )
            .expect("commit")
            .detach()
    }
}

/// A source repo: c1 - c2 (v0.1.0) - c3 (v0.2.0) on main, c4 on side off c2,
/// and c5 alone on orphan.
struct Upstream {
    _dir: tempfile::TempDir,
    url: String,
    tree: String,
    c2: String,
    c3: String,
    c4: String,
    c5: String,
}

fn upstream() -> Upstream {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("up.git");
    let build = Builder::new(&path);
    let c1 = build.commit("refs/heads/main", "c1", vec![]);
    let c2 = build.commit("refs/heads/main", "c2", vec![c1]);
    let c3 = build.commit("refs/heads/main", "c3", vec![c2]);
    let c4 = build.commit("refs/heads/side", "c4", vec![c2]);
    let c5 = build.commit("refs/heads/orphan", "c5", vec![]);
    for (tag, id) in [("v0.1.0", c2), ("v0.2.0", c3)] {
        build
            .repo
            .reference(
                format!("refs/tags/{tag}").as_str(),
                id,
                PreviousValue::Any,
                "tag",
            )
            .expect("tag");
    }
    Upstream {
        url: path.display().to_string(),
        _dir: dir,
        tree: build.tree.to_string(),
        c2: c2.to_string(),
        c3: c3.to_string(),
        c4: c4.to_string(),
        c5: c5.to_string(),
    }
}

/// A source repo whose main holds two commits sharing a four-digit prefix.
struct Collision {
    _dir: tempfile::TempDir,
    url: String,
    prefix: String,
}

/// Commits pile onto main until two ids start alike. An id is a pure function
/// of the fixed message and time, so every run builds the same few hundred
/// commits.
fn collision() -> Collision {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("up.git");
    let build = Builder::new(&path);
    let mut seen = HashSet::new();
    let mut parents = vec![];
    let mut shared = None;
    for number in 0_u32.. {
        let id = build.commit("refs/heads/main", &number.to_string(), parents);
        let prefix = id.to_string().get(..4).expect("hex").to_owned();
        if !seen.insert(prefix.clone()) {
            shared = Some(prefix);
            break;
        }
        parents = vec![id];
    }
    Collision {
        url: path.display().to_string(),
        _dir: dir,
        prefix: shared.expect("the loop breaks on a collision"),
    }
}

fn tag(name: &str) -> GitRef {
    GitRef::Tag(name.to_owned())
}

fn branch(name: &str) -> GitRef {
    GitRef::Branch(name.to_owned())
}

#[tokio::test]
async fn answers_containment_for_branches_and_tags() {
    let up = upstream();
    let cache = tempfile::tempdir().expect("cache");
    let remotes = GitRemotes::new(cache.path().join("not/yet"));
    let source = Source::Git {
        url: up.url.clone(),
    };
    let contains = |sha: &str, target: GitRef| {
        let (remotes, source, sha) = (remotes.clone(), source.clone(), sha.to_owned());
        async move { remotes.contains(&source, &sha, &target).await }
    };
    assert_eq!(contains(&up.c2, tag("v0.1.0")).await, Ok(true));
    assert_eq!(contains(&up.c3, tag("v0.1.0")).await, Ok(false));
    assert_eq!(
        contains(up.c3.get(..7).expect("short"), branch("main")).await,
        Ok(true)
    );
    assert_eq!(
        contains(&up.c3.to_ascii_uppercase(), branch("main")).await,
        Ok(true)
    );
    assert_eq!(contains(&up.c2, branch("main")).await, Ok(true));
    assert_eq!(contains(&up.c4, branch("main")).await, Ok(false));
    assert_eq!(contains(&up.c4, branch("side")).await, Ok(true));
    let missing = contains(&up.c2, tag("v9")).await;
    assert!(
        matches!(missing, Err(CheckError::NotFound(_))),
        "{missing:?}"
    );
    assert_eq!(
        remotes.tags(&source, "v").await,
        Ok(vec!["v0.1.0".to_owned(), "v0.2.0".to_owned()])
    );
}

/// A remote over `url`, cloned into a cache that lives as long as the dir.
fn remote(url: &str) -> (tempfile::TempDir, GitRemotes, Source) {
    let cache = tempfile::tempdir().expect("cache");
    let remotes = GitRemotes::new(cache.path().to_owned());
    let source = Source::Git {
        url: url.to_owned(),
    };
    (cache, remotes, source)
}

#[tokio::test]
async fn blockers_asking_one_remote_together_do_not_race_the_clone_or_the_fetch() {
    let up = upstream();
    let (_cache, remotes, source) = remote(&up.url);
    let asks = [
        (&up.c2, tag("v0.1.0")),
        (&up.c3, tag("v0.2.0")),
        (&up.c3, branch("main")),
        (&up.c4, branch("side")),
        (&up.c5, branch("orphan")),
    ];
    let contains = asks
        .iter()
        .map(|(sha, target)| remotes.contains(&source, sha, target));
    let (answers, tags) = tokio::join!(
        futures_util::future::join_all(contains),
        remotes.tags(&source, "v")
    );
    assert_eq!(answers, vec![Ok(true); asks.len()]);
    assert_eq!(tags.map(|names| names.len()), Ok(2));
}

#[tokio::test]
async fn a_sha_the_history_lacks_reads_waiting() {
    let up = upstream();
    let (_cache, remotes, source) = remote(&up.url);
    let main = branch("main");
    // Unknown digits, and a tree that no commit history can contain.
    for sha in ["deadbeef", up.tree.as_str()] {
        assert_eq!(remotes.contains(&source, sha, &main).await, Ok(false));
    }
    // A commit that is here but shares no ancestor with main.
    assert_eq!(
        remotes.contains(&source, &up.c5, &branch("orphan")).await,
        Ok(true)
    );
    assert_eq!(remotes.contains(&source, &up.c5, &main).await, Ok(false));
}

#[tokio::test]
async fn a_short_sha_shared_by_two_commits_asks_for_more_digits() {
    let up = collision();
    let (_cache, remotes, source) = remote(&up.url);
    let result = remotes.contains(&source, &up.prefix, &branch("main")).await;
    let Err(CheckError::Unsupported(message)) = result else {
        panic!("expected an ambiguity, got {result:?}");
    };
    assert!(message.contains(&up.prefix), "{message}");
    assert!(message.contains("more digits"), "{message}");
}

#[tokio::test]
async fn an_unreachable_remote_is_a_network_error() {
    let cache = tempfile::tempdir().expect("cache");
    let remotes = GitRemotes::new(cache.path().to_owned());
    let source = Source::Git {
        url: cache.path().join("nope.git").display().to_string(),
    };
    let result = remotes.tags(&source, "").await;
    assert!(matches!(result, Err(CheckError::Network(_))), "{result:?}");
}

#[tokio::test]
async fn a_git_remote_has_no_merge_commits() {
    let cache = tempfile::tempdir().expect("cache");
    let remotes = GitRemotes::new(cache.path().to_owned());
    let item = ItemRef {
        source: Source::Git {
            url: "x".to_owned(),
        },
        number: 1,
        kind: ItemKind::Issue,
    };
    assert!(matches!(
        remotes.merge_commit(&item).await,
        Err(CheckError::Unsupported(_))
    ));
}

/// Deletes `name` in the upstream repo at `url`.
fn delete_upstream(url: &str, name: &str) {
    let repo = gix::open(url).expect("open upstream");
    repo.find_reference(name)
        .expect("ref")
        .delete()
        .expect("delete");
}

#[tokio::test]
async fn a_branch_deleted_upstream_since_an_earlier_run_reads_not_found() {
    let up = upstream();
    let (cache, earlier, source) = remote(&up.url);
    let side = branch("side");
    assert_eq!(earlier.contains(&source, &up.c4, &side).await, Ok(true));
    delete_upstream(&up.url, "refs/heads/side");
    let later = GitRemotes::new(cache.path().to_owned());
    let result = later.contains(&source, &up.c4, &side).await;
    assert!(matches!(result, Err(CheckError::NotFound(_))), "{result:?}");
}

#[tokio::test]
async fn a_tag_retracted_upstream_since_an_earlier_run_is_gone() {
    let up = upstream();
    let (cache, earlier, source) = remote(&up.url);
    assert_eq!(
        earlier.tags(&source, "v").await.map(|tags| tags.len()),
        Ok(2)
    );
    assert_eq!(
        earlier.contains(&source, &up.c3, &tag("v0.2.0")).await,
        Ok(true)
    );
    delete_upstream(&up.url, "refs/tags/v0.2.0");
    let later = GitRemotes::new(cache.path().to_owned());
    assert_eq!(
        later.tags(&source, "v").await,
        Ok(vec!["v0.1.0".to_owned()])
    );
    let result = later.contains(&source, &up.c3, &tag("v0.2.0")).await;
    assert!(matches!(result, Err(CheckError::NotFound(_))), "{result:?}");
}

#[tokio::test]
async fn a_remote_with_no_tags_left_lists_none() {
    let up = upstream();
    let (cache, earlier, source) = remote(&up.url);
    assert_eq!(
        earlier.tags(&source, "v").await.map(|tags| tags.len()),
        Ok(2)
    );
    delete_upstream(&up.url, "refs/tags/v0.1.0");
    delete_upstream(&up.url, "refs/tags/v0.2.0");
    let later = GitRemotes::new(cache.path().to_owned());
    assert_eq!(later.tags(&source, "v").await, Ok(vec![]));
}

#[test]
fn the_clone_dir_is_named_after_the_url_and_its_hash() {
    assert_eq!(
        clone_dir(Path::new("/c"), "https://example.org/a/b.git").expect("hash"),
        PathBuf::from("/c/https___example.org_a_b.git-ae9f785a26a8581a.git")
    );
}

#[test]
fn urls_that_differ_only_in_punctuation_get_their_own_clone_dirs() {
    let cache = Path::new("/c");
    assert_ne!(
        clone_dir(cache, "/srv/a_b.git").expect("hash"),
        clone_dir(cache, "/srv/a/b.git").expect("hash")
    );
}

#[test]
fn the_clone_dir_holds_no_credentials() {
    assert_eq!(
        clone_dir(Path::new("/c"), "https://user:s3cret@example.org/a/b.git").expect("hash"),
        PathBuf::from("/c/https___user_____example.org_a_b.git-cadd697c4c03abd6.git")
    );
}

#[test]
fn a_long_url_names_a_clone_dir_a_file_system_takes() {
    let url = format!("https://example.org/{}.git", "a".repeat(400));
    let dir = clone_dir(Path::new("/c"), &url).expect("hash");
    let name = dir.file_name().expect("name").len();
    assert_eq!(name, SLUG_CHARS + 1 + HASH_DIGITS + ".git".len());
}
