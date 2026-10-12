use super::*;

use anstream::adapter::strip_str;
use std::collections::BTreeMap;

use jiff::SignedDuration;

use crate::scan::{Findings, Hit, Site};

fn now() -> Timestamp {
    Timestamp::from_second(1_800_000_000).expect("in range")
}

fn entry(id: &str, status: Status) -> Entry {
    Entry {
        id: id.to_owned(),
        note: "drop it".to_owned(),
        status,
        detail: "d".to_owned(),
        checked_at: now(),
        fresh_until: Timestamp::MAX,
        spec: String::new(),
    }
}

/// A stale entry checked `seconds` ago; a negative count is a clock set back.
fn stale(seconds: i64) -> Entry {
    let checked_at = now()
        .checked_sub(SignedDuration::from_secs(seconds))
        .expect("in range");
    Entry {
        checked_at,
        fresh_until: Timestamp::UNIX_EPOCH,
        ..entry("x", Status::Waiting)
    }
}

fn plain(entries: &[Entry]) -> String {
    strip_str(&lines(entries, &BTreeMap::new(), now())).to_string()
}

#[test]
fn each_status_word_gets_its_own_style() {
    let paint = |status| lines(&[entry("x", status)], &BTreeMap::new(), now());
    assert_eq!(paint(Status::Waiting), "\x1b[2mwaiting\x1b[0m   x  d\n");
    assert_eq!(
        paint(Status::Bumpable),
        "\x1b[32mbumpable\x1b[0m  x  d (drop it)\n"
    );
    assert_eq!(
        paint(Status::Dead),
        "\x1b[35mdead\x1b[0m      x  d (drop it)\n"
    );
    assert_eq!(paint(Status::Failed), "\x1b[31mfailed\x1b[0m    x  d\n");
}

#[test]
fn stripped_lines_are_the_plain_report() {
    let entries = [
        entry("a", Status::Waiting),
        entry("bb", Status::Bumpable),
        entry("c", Status::Dead),
        entry("d", Status::Failed),
    ];
    assert_eq!(
        plain(&entries),
        "waiting   a   d\n\
         bumpable  bb  d (drop it)\n\
         dead      c   d (drop it)\n\
         failed    d   d\n"
    );
}

#[test]
fn a_dead_line_names_the_workaround_like_a_bumpable_one() {
    assert_eq!(
        plain(&[entry("x", Status::Dead)]),
        "dead      x  d (drop it)\n"
    );
}

#[test]
fn columns_count_chars_not_bytes() {
    let entries = [entry("día", Status::Waiting), entry("ab", Status::Waiting)];
    assert_eq!(plain(&entries), "waiting   día  d\nwaiting   ab   d\n");
}

#[test]
fn a_stale_line_says_how_long_ago_in_the_largest_whole_unit() {
    let cases = [
        (-5, "just now"),
        (0, "just now"),
        (59, "just now"),
        (60, "1m ago"),
        (3_599, "59m ago"),
        (3_600, "1h ago"),
        (7 * 3_600 + 5, "7h ago"),
        (86_399, "23h ago"),
        (86_400, "1d ago"),
        (3 * 86_400 + 3_600, "3d ago"),
    ];
    for (seconds, ago) in cases {
        assert_eq!(
            plain(&[stale(seconds)]),
            format!("waiting   x  d (stale, checked {ago})\n"),
            "{seconds}s"
        );
    }
}

#[test]
fn the_exit_code_is_8_plus_a_bit_per_counted_status_that_showed_up() {
    let code = |statuses: &[Status], counted: &[Actionable]| {
        let entries: Vec<Entry> = statuses.iter().map(|status| entry("x", *status)).collect();
        exit_code(&entries, &Findings::default(), counted)
    };
    let all = &Actionable::ALL;
    assert_eq!(code(&[], all), EXIT_OK);
    assert_eq!(code(&[Status::Waiting], all), EXIT_OK);
    assert_eq!(code(&[Status::Bumpable], all), 9);
    assert_eq!(code(&[Status::Dead], all), 10);
    assert_eq!(code(&[Status::Failed], all), 12);
    assert_eq!(code(&[Status::Dead, Status::Bumpable], all), 11);
    let every = [
        Status::Waiting,
        Status::Bumpable,
        Status::Dead,
        Status::Failed,
    ];
    assert_eq!(code(&every, all), 15);
}

#[test]
fn only_a_counted_status_adds_its_bit() {
    let entries = [entry("a", Status::Bumpable), entry("b", Status::Dead)];
    assert_eq!(
        exit_code(&entries, &Findings::default(), &[Actionable::Dead]),
        10
    );
    assert_eq!(
        exit_code(&entries, &Findings::default(), &[Actionable::Failed]),
        EXIT_OK
    );
    assert_eq!(exit_code(&entries, &Findings::default(), &[]), EXIT_OK);
    let twice = [Actionable::Dead, Actionable::Dead];
    assert_eq!(exit_code(&entries, &Findings::default(), &twice), 10);
}

#[test]
fn a_blocker_answers_0_bumpable_1_waiting_and_2_for_the_rest() {
    assert_eq!(blocker_exit_code(Status::Bumpable), 0);
    assert_eq!(blocker_exit_code(Status::Waiting), 1);
    assert_eq!(blocker_exit_code(Status::Dead), EXIT_UNKNOWN);
    assert_eq!(blocker_exit_code(Status::Failed), EXIT_UNKNOWN);
}

#[test]
fn an_error_exits_with_its_own_code_or_the_usual_one() {
    let plain = anyhow::anyhow!("no luck");
    assert_eq!(Exit::code_of(&plain), EXIT_ERROR);
    let coded: anyhow::Error = Exit::new(EXIT_UNKNOWN, plain).into();
    assert_eq!(Exit::code_of(&coded), EXIT_UNKNOWN);
}

#[test]
fn an_exit_prints_as_the_error_it_wraps_with_its_chain() {
    let inner = anyhow::anyhow!("root cause").context("while reading");
    let coded: anyhow::Error = Exit::new(EXIT_UNKNOWN, inner).into();
    assert_eq!(format!("{coded:#}"), "while reading: root cause");
}

/// Blocker `x`'s sites: `flake.nix:29`, then `Justfile:26`.
fn sites_of_x() -> BTreeMap<String, Vec<Site>> {
    let site = |file: &str, line| Site {
        file: file.into(),
        line,
    };
    BTreeMap::from([(
        "x".to_owned(),
        vec![site("flake.nix", 29), site("Justfile", 26)],
    )])
}

#[test]
fn a_bumpable_or_dead_line_lists_its_sites_under_it() {
    let sites = sites_of_x();
    let plain = |status| strip_str(&lines(&[entry("x", status)], &sites, now())).to_string();
    assert_eq!(
        plain(Status::Bumpable),
        "bumpable  x  d (drop it)\n          at flake.nix:29, Justfile:26\n"
    );
    assert_eq!(
        plain(Status::Dead),
        "dead      x  d (drop it)\n          at flake.nix:29, Justfile:26\n"
    );
    assert_eq!(plain(Status::Waiting), "waiting   x  d\n");
    assert_eq!(plain(Status::Failed), "failed    x  d\n");
}

#[test]
fn the_json_report_gives_every_row_its_sites() {
    let entries = [entry("x", Status::Waiting), entry("y", Status::Waiting)];
    let findings = Findings {
        hits: Vec::new(),
        sites: sites_of_x(),
        unmarked: Vec::new(),
    };
    let report =
        Report::new(Path::new("m"), Path::new("c"), &entries, now(), 0).with_scan(&findings);
    let json = serde_json::to_value(&report).expect("serializes");
    assert_eq!(
        json.pointer("/entries/0/sites"),
        Some(&serde_json::json!([
            { "file": "flake.nix", "line": 29 },
            { "file": "Justfile", "line": 26 },
        ]))
    );
    assert_eq!(
        json.pointer("/entries/1/sites"),
        Some(&serde_json::json!([]))
    );
}

/// What a scan found: one unlisted link when `unlisted`, and these
/// `unmarked` blockers.
fn findings(unlisted: bool, unmarked: &[&str]) -> Findings {
    let hit = Hit {
        file: "hosts/gpu.nix".into(),
        line: 41,
        reference: "github:NixOS/nixpkgs#412345".to_owned(),
        catcher: "github".to_owned(),
    };
    Findings {
        hits: if unlisted { vec![hit] } else { Vec::new() },
        sites: BTreeMap::new(),
        unmarked: unmarked.iter().map(|id| (*id).to_owned()).collect(),
    }
}

#[test]
fn an_unlisted_link_adds_16_when_counted() {
    let entries = [entry("a", Status::Bumpable)];
    let unlisted = findings(true, &[]);
    assert_eq!(exit_code(&[], &unlisted, &Actionable::ALL), 24);
    assert_eq!(exit_code(&entries, &unlisted, &Actionable::ALL), 25);
    assert_eq!(exit_code(&entries, &unlisted, &[Actionable::Bumpable]), 9);
    let none = findings(false, &[]);
    assert_eq!(exit_code(&entries, &none, &[Actionable::Unlisted]), EXIT_OK);
}

#[test]
fn an_unmarked_blocker_adds_32_when_counted() {
    let unmarked = findings(false, &["a"]);
    assert_eq!(exit_code(&[], &unmarked, &Actionable::ALL), 40);
    assert_eq!(
        exit_code(&[], &findings(true, &["a"]), &Actionable::ALL),
        56
    );
    assert_eq!(exit_code(&[], &unmarked, &[Actionable::Unlisted]), EXIT_OK);
}

#[test]
fn prints_an_unlisted_line_per_hit_then_an_unmarked_line_per_blocker() {
    let plain = |found: &Findings| strip_str(&scan_lines(found)).to_string();
    assert_eq!(
        plain(&findings(true, &[])),
        "unlisted  hosts/gpu.nix:41  github:NixOS/nixpkgs#412345 has no blocker\n"
    );
    assert_eq!(
        plain(&findings(true, &["a", "long-id"])),
        "unlisted  hosts/gpu.nix:41  github:NixOS/nixpkgs#412345 has no blocker\n\
         unmarked  a        no canibump:a in the scanned files\n\
         unmarked  long-id  no canibump:long-id in the scanned files\n"
    );
    assert_eq!(plain(&findings(false, &[])), "");
}
