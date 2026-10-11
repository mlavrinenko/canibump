use super::*;

fn condition(text: &str) -> Result<Condition, String> {
    parse_condition(text.parse::<Table>().expect("toml"))
}

fn reached(target: &str) -> Result<Condition, String> {
    condition(&format!(
        "reached = \"{target}\"\nin = {{ tag = \"v1\" }}\n"
    ))
}

#[test]
fn reached_takes_github_and_git_targets() {
    for target in [
        "github:o/r#1",
        "github:o/r@abc1234",
        "git:https://example.com/r.git@abc1234",
    ] {
        assert!(reached(target).is_ok(), "{target}");
    }
}

#[test]
fn reached_refuses_a_target_with_no_history_provider() {
    for target in [
        "gitlab:g/p!12",
        "gitlab:g/p#12",
        "gitlab:g/p@abc1234",
        "codeberg:o/r#1",
        "gitea:git.example.com/o/r#1",
        "forgejo:git.example.com/o/r@abc1234",
        "redmine:redmine.example.com#1",
        "jira:jira.example.com#CORE-1",
    ] {
        assert_eq!(
            reached(target),
            Err(format!(
                "`reached` takes github:owner/repo#N, github:owner/repo@<sha> or git:<url>@<sha>, not `{target}`"
            )),
        );
    }
}
