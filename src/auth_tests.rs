use std::collections::HashMap;

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

#[test]
fn gitlab_tokens_come_from_glab_and_the_env_by_host() {
    let config = "host: gitlab.example.org\nhosts:\n    gitlab.example.org:\n        api_host: gitlab.example.org\n        token: from-glab\n    nope.example.org:\n        api_host: nope.example.org\n";
    let env = |vars: &'static [(&'static str, &'static str)]| {
        move |key: &str| {
            vars.iter()
                .find(|(name, _)| *name == key)
                .map(|(_, value)| (*value).to_owned())
        }
    };
    let tokens = gitlab_tokens_from(env(&[("GITLAB_TOKEN", "from-env")]), Some(config));
    assert_eq!(
        tokens.get("gitlab.example.org").map(String::as_str),
        Some("from-glab")
    );
    assert_eq!(
        tokens.get("gitlab.com").map(String::as_str),
        Some("from-env")
    );
    assert_eq!(tokens.len(), 2);
    let tokens = gitlab_tokens_from(
        env(&[
            ("GITLAB_TOKEN", "e"),
            ("GITLAB_HOST", "https://gitlab.example.org/"),
        ]),
        Some(config),
    );
    assert_eq!(
        tokens.get("gitlab.example.org").map(String::as_str),
        Some("e")
    );
    assert!(gitlab_tokens_from(env(&[]), Some("not: [yaml")).is_empty());
}

#[test]
fn gitea_tokens_come_from_tea_and_the_env_by_host() {
    let config = "logins:\n    - name: home\n      url: https://git.example.org\n      token: from-tea\n    - name: empty\n      url: https://empty.example.org\n      token: \"\"\n";
    let env = |vars: &'static [(&'static str, &'static str)]| {
        move |key: &str| {
            vars.iter()
                .find(|(name, _)| *name == key)
                .map(|(_, value)| (*value).to_owned())
        }
    };
    let tokens = gitea_tokens_from(env(&[("GITEA_TOKEN", "lost")]), Some(config));
    assert_eq!(
        tokens,
        HashMap::from([("git.example.org".to_owned(), "from-tea".to_owned())])
    );
    let tokens = gitea_tokens_from(
        env(&[("GITEA_TOKEN", "e"), ("GITEA_HOST", "codeberg.org")]),
        None,
    );
    assert_eq!(
        tokens,
        HashMap::from([("codeberg.org".to_owned(), "e".to_owned())])
    );
}

#[test]
fn redmine_and_jira_credentials_go_to_their_own_host() {
    let env = |vars: &'static [(&'static str, &'static str)]| {
        move |key: &str| {
            vars.iter()
                .find(|(name, _)| *name == key)
                .map(|(_, value)| (*value).to_owned())
        }
    };
    assert_eq!(
        redmine_keys_from(env(&[
            ("REDMINE_URL", "https://rm.example.org/"),
            ("REDMINE_API_KEY", "k")
        ])),
        HashMap::from([("rm.example.org".to_owned(), "k".to_owned())])
    );
    assert!(redmine_keys_from(env(&[("REDMINE_API_KEY", "k")])).is_empty());
    assert_eq!(
        jira_auth_from(env(&[
            ("JIRA_URL", "https://j.example.org"),
            ("JIRA_TOKEN", "t")
        ])),
        HashMap::from([("j.example.org".to_owned(), JiraAuth::Bearer("t".to_owned()))])
    );
    assert_eq!(
        jira_auth_from(env(&[
            ("JIRA_URL", "j.example.org"),
            ("JIRA_USER", "me"),
            ("JIRA_API_TOKEN", "s"),
        ])),
        HashMap::from([(
            "j.example.org".to_owned(),
            JiraAuth::Basic("me".to_owned(), "s".to_owned())
        )])
    );
    assert!(jira_auth_from(env(&[("JIRA_TOKEN", "t")])).is_empty());
}
