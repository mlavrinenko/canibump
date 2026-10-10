//! Version order: semver when both sides parse, else a lenient order that
//! copes with tags like llama.cpp's `b7012`.
//!
//! Lenient order drops a leading letter prefix (`v`, `b`), splits the rest
//! into runs of digits and runs of letters, and compares run by run: numbers
//! numerically, letters alphabetically, and a number above letters at the
//! same position. When one side runs out, a letter run left on the other
//! side marks a pre-release (`1.0rc1` < `1.0`), a number run a later release
//! (`1.2` < `1.2.1`).

use std::cmp::Ordering;

#[derive(Debug, PartialEq, Eq)]
enum Part<'a> {
    Number(u128),
    Letters(&'a str),
}

impl Ord for Part<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Number(left), Self::Number(right)) => left.cmp(right),
            (Self::Letters(left), Self::Letters(right)) => left.cmp(right),
            (Self::Number(_), Self::Letters(_)) => Ordering::Greater,
            (Self::Letters(_), Self::Number(_)) => Ordering::Less,
        }
    }
}

impl PartialOrd for Part<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn semver(text: &str) -> Option<semver::Version> {
    semver::Version::parse(text.strip_prefix('v').unwrap_or(text)).ok()
}

fn parts(text: &str) -> Vec<Part<'_>> {
    let start = text
        .find(|char: char| char.is_ascii_digit())
        .unwrap_or(text.len());
    let rest = text.get(start..).unwrap_or_default();
    let mut parts = Vec::new();
    let mut chars = rest.char_indices().peekable();
    while let Some((begin, first)) = chars.next() {
        if !first.is_ascii_alphanumeric() {
            continue;
        }
        let digits = first.is_ascii_digit();
        let mut end = begin + first.len_utf8();
        while let Some((index, next)) = chars.peek().copied() {
            if !next.is_ascii_alphanumeric() || next.is_ascii_digit() != digits {
                break;
            }
            end = index + next.len_utf8();
            chars.next();
        }
        let run = rest.get(begin..end).unwrap_or_default();
        parts.push(if digits {
            Part::Number(run.parse().unwrap_or(u128::MAX))
        } else {
            Part::Letters(run)
        });
    }
    parts
}

fn lenient(left: &str, right: &str) -> Ordering {
    let (left, right) = (parts(left), parts(right));
    for (one, two) in left.iter().zip(&right) {
        let order = one.cmp(two);
        if order != Ordering::Equal {
            return order;
        }
    }
    let tail = |longer: &[Part<'_>], shorter: usize| match longer.get(shorter) {
        Some(Part::Letters(_)) => Ordering::Less,
        Some(Part::Number(_)) => Ordering::Greater,
        None => Ordering::Equal,
    };
    match left.len().cmp(&right.len()) {
        Ordering::Equal => Ordering::Equal,
        Ordering::Greater => tail(&left, right.len()),
        Ordering::Less => tail(&right, left.len()).reverse(),
    }
}

/// Orders two version strings.
#[must_use]
pub fn compare(left: &str, right: &str) -> Ordering {
    match (semver(left), semver(right)) {
        (Some(left), Some(right)) => left.cmp(&right),
        _ => lenient(left, right),
    }
}

/// Sorts version strings ascending.
pub fn sort<T: AsRef<str>>(versions: &mut [T]) {
    versions.sort_by(|left, right| compare(left.as_ref(), right.as_ref()));
}

#[cfg(test)]
#[path = "version_tests.rs"]
mod tests;
