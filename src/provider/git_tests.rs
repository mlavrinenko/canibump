use gix::refs::transaction::PreviousValue;

use super::*;
use crate::refs::ItemKind;

/// A source repo: c1 - c2 (v0.1.0) - c3 (v0.2.0) on main, c4 on side off c2.
struct Upstream {
    _dir: tempfile::TempDir,
    url: String,
    c2: String,
    c3: String,
    c4: String,
}

fn upstream() -> Upstream {
    let dir = tempfile::tempdir().expect("tempdir");
    let repo = gix::init_bare(dir.path().join("up.git")).expect("init");
    let tree = repo
        .write_object(gix::objs::Tree::empty())
        .expect("tree")
        .detach();
    let when = gix::date::Time::new(1_700_000_000, 0);
    let signature = gix::actor::Signature {
        name: "t".into(),
        email: "t@t".into(),
        time: when,
    };
    let commit = |reference: &str, message: &str, parents: Vec<gix::ObjectId>| {
        repo.commit_as(
            signature.to_ref(&mut gix::date::parse::TimeBuf::default()),
            signature.to_ref(&mut gix::date::parse::TimeBuf::default()),
            reference,
            message,
            tree,
            parents,
        )
        .expect("commit")
        .detach()
    };
    let c1 = commit("refs/heads/main", "c1", vec![]);
    let c2 = commit("refs/heads/main", "c2", vec![c1]);
    let c3 = commit("refs/heads/main", "c3", vec![c2]);
    let c4 = commit("refs/heads/side", "c4", vec![c2]);
    for (tag, id) in [("v0.1.0", c2), ("v0.2.0", c3)] {
        repo.reference(
            format!("refs/tags/{tag}").as_str(),
            id,
            PreviousValue::Any,
            "tag",
        )
        .expect("tag");
    }
    Upstream {
        url: dir.path().join("up.git").display().to_string(),
        _dir: dir,
        c2: c2.to_string(),
        c3: c3.to_string(),
        c4: c4.to_string(),
    }
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
    let tag = |name: &str| GitRef::Tag(name.to_owned());
    let branch = |name: &str| GitRef::Branch(name.to_owned());
    assert_eq!(contains(&up.c2, tag("v0.1.0")).await, Ok(true));
    assert_eq!(contains(&up.c3, tag("v0.1.0")).await, Ok(false));
    assert_eq!(
        contains(up.c3.get(..7).expect("short"), branch("main")).await,
        Ok(true)
    );
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

#[test]
fn the_clone_dir_is_named_after_the_url() {
    assert_eq!(
        clone_dir(Path::new("/c"), "https://example.org/a/b.git"),
        PathBuf::from("/c/https___example.org_a_b.git.git")
    );
}
