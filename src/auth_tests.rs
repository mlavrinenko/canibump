use std::collections::HashMap;

use super::*;

fn env(vars: &'static [(&'static str, &'static str)]) -> impl Fn(&str) -> Option<String> {
    move |key: &str| {
        vars.iter()
            .find(|(name, _)| *name == key)
            .map(|(_, value)| (*value).to_owned())
    }
}

const GITHUB: &str = "https://api.github.com";
const ENTERPRISE: &str = "https://ghe.example.com:8443/api/v3";

#[test]
fn prefers_github_token_then_gh_token_then_the_fallback() {
    let fallback = |host: &str| Some(format!("from-gh for {host}"));
    assert_eq!(
        github_token_from(
            GITHUB,
            env(&[("GITHUB_TOKEN", "a"), ("GH_TOKEN", "b")]),
            fallback
        ),
        Some("a".to_owned())
    );
    assert_eq!(
        github_token_from(
            GITHUB,
            env(&[("GITHUB_TOKEN", ""), ("GH_TOKEN", "b")]),
            fallback
        ),
        Some("b".to_owned())
    );
    assert_eq!(
        github_token_from(GITHUB, env(&[]), fallback),
        Some("from-gh for github.com".to_owned())
    );
    assert_eq!(github_token_from(GITHUB, env(&[]), |_| None), None);
}

#[test]
fn an_enterprise_api_gets_its_own_hosts_token_never_github_coms() {
    let fallback = |host: &str| Some(format!("from-gh for {host}"));
    let github_com = &[("GITHUB_TOKEN", "a"), ("GH_TOKEN", "b")];
    assert_eq!(
        github_token_from(ENTERPRISE, env(github_com), |_| None),
        None
    );
    assert_eq!(
        github_token_from(ENTERPRISE, env(github_com), fallback),
        Some("from-gh for ghe.example.com:8443".to_owned())
    );
    let both = &[
        ("GH_ENTERPRISE_TOKEN", "e"),
        ("GITHUB_ENTERPRISE_TOKEN", "f"),
    ];
    assert_eq!(
        github_token_from(ENTERPRISE, env(both), fallback),
        Some("e".to_owned())
    );
    let second = &[
        ("GH_ENTERPRISE_TOKEN", ""),
        ("GITHUB_ENTERPRISE_TOKEN", "f"),
    ];
    assert_eq!(
        github_token_from(ENTERPRISE, env(second), fallback),
        Some("f".to_owned())
    );
    assert_eq!(
        github_token_from("https://ghe.example.com/api/v3", env(&[]), fallback),
        Some("from-gh for ghe.example.com".to_owned())
    );
}

#[test]
fn a_github_api_that_is_not_a_url_gets_no_token() {
    let asked = std::cell::Cell::new(false);
    let fallback = |_: &str| {
        asked.set(true);
        Some("t".to_owned())
    };
    assert_eq!(
        github_token_from("not-a-url", env(&[("GH_TOKEN", "b")]), fallback),
        None
    );
    assert!(!asked.get(), "gh never runs for a bad URL");
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
