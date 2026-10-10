use super::*;

use anstream::adapter::strip_str;
use jiff::SignedDuration;

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
    strip_str(&lines(entries, now())).to_string()
}

#[test]
fn each_status_word_gets_its_own_style() {
    let paint = |status| lines(&[entry("x", status)], now());
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
