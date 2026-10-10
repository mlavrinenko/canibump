use super::*;

#[test]
fn prefers_github_token_then_gh_token_then_the_fallback() {
    let env = |vars: &'static [(&'static str, &'static str)]| {
        move |key: &str| {
            vars.iter()
                .find(|(name, _)| *name == key)
                .map(|(_, value)| (*value).to_owned())
        }
    };
    let fallback = || Some("from-gh".to_owned());
    assert_eq!(
        github_token_from(env(&[("GITHUB_TOKEN", "a"), ("GH_TOKEN", "b")]), fallback),
        Some("a".to_owned())
    );
    assert_eq!(
        github_token_from(env(&[("GITHUB_TOKEN", ""), ("GH_TOKEN", "b")]), fallback),
        Some("b".to_owned())
    );
    assert_eq!(
        github_token_from(env(&[]), fallback),
        Some("from-gh".to_owned())
    );
    assert_eq!(github_token_from(env(&[]), || None), None);
}

#[test]
fn trims_command_output() {
    assert_eq!(trimmed(b"tok\n".to_vec()), Some("tok".to_owned()));
    assert_eq!(trimmed(b" \n".to_vec()), None);
    assert_eq!(trimmed(vec![0xff]), None);
}
