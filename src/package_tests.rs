use super::*;

#[test]
fn parses_package_refs() {
    let nix = PackageRef::parse("repology:llama.cpp", Some("nix_unstable".to_owned()), None)
        .expect("valid");
    assert_eq!(
        nix,
        PackageRef::Repology {
            project: "llama.cpp".to_owned(),
            repo: "nix_unstable".to_owned(),
            srcname: None,
        }
    );
    assert_eq!(nix.to_string(), "repology:llama.cpp in nix_unstable");
    assert_eq!(nix.index(), IndexKind::Repology);
    let crate_ref = PackageRef::parse("deps.dev:cargo/serde", None, None).expect("valid");
    assert_eq!(
        crate_ref,
        PackageRef::DepsDev {
            system: "cargo".to_owned(),
            name: "serde".to_owned(),
        }
    );
    assert_eq!(crate_ref.to_string(), "deps.dev:cargo/serde");
    assert_eq!(crate_ref.index(), IndexKind::DepsDev);
    let go = PackageRef::parse("deps.dev:go/github.com/a/b", None, None).expect("valid");
    assert_eq!(go.to_string(), "deps.dev:go/github.com/a/b");
}

#[test]
fn rejects_bad_package_refs() {
    for (text, repo, srcname, message) in [
        ("repology:llama.cpp", None, None, "needs `repo`"),
        ("deps.dev:cargo/serde", Some("x"), None, "takes no `repo`"),
        (
            "deps.dev:cargo/serde",
            None,
            Some("x"),
            "takes no `srcname`",
        ),
        ("deps.dev:serde", None, None, "deps.dev:<system>/<name>"),
        ("deps.dev:cobol/x", None, None, "cobol"),
        ("repology:", Some("r"), None, "repology:<project>"),
        ("pypi:x", None, None, "unknown scheme"),
    ] {
        let error = PackageRef::parse(text, repo.map(str::to_owned), srcname.map(str::to_owned))
            .expect_err(text)
            .to_string();
        assert!(error.contains(message), "{text}: {error}");
    }
}

#[test]
fn an_index_displays_as_its_manifest_scheme() {
    assert_eq!(IndexKind::Repology.to_string(), "repology");
    assert_eq!(IndexKind::DepsDev.to_string(), "deps.dev");
}
