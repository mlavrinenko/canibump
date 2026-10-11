//! Version order: one total order for semver, `v`-tags, dates and builds like
//! llama.cpp's `b7012` alike.
//!
//! A version string becomes a [`Key`] once: build metadata after `+` is
//! ignored, a leading letter prefix (`v`, `b`) drops, and the rest splits into
//! runs of digits and runs of letters. Keys compare run by run: numbers
//! numerically, letters alphabetically, a number above letters at the same
//! position. A zero run right before the end or a letter run drops, so a
//! missing trailing component equals zero (`1.0` = `1.0.0`). Every key ends in
//! a sentinel that ranks above letters and below numbers, so a letter run left
//! over on the longer side marks a pre-release (`1.0rc1` < `1.0`) and a number
//! run a later release (`1.2` < `1.2.1`).
//! Because the order is lexicographic over one sequence type, it is total:
//! no mix of tags can make a sort or a binary search disagree with itself.

use std::cmp::Ordering;

/// One run of a version. The variant order is the rank order.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Part {
    Letters(String),
    End,
    Number(u128),
}

/// A version string parsed once into a totally ordered key.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key(Vec<Part>);

impl Key {
    /// Builds the key of a version string.
    #[must_use]
    pub fn new(text: &str) -> Self {
        let release = text.split_once('+').map_or(text, |(release, _)| release);
        let mut parts = parts(release);
        parts.push(Part::End);
        Self(without_padding_zeros(parts))
    }
}

/// Drops each zero run whose next run, after the drops, is the end or letters:
/// `1.0.0` and `1.0.0rc1` become `1` and `1rc1`. Walks from the right so a
/// chain of zeros goes in one pass.
fn without_padding_zeros(parts: Vec<Part>) -> Vec<Part> {
    let mut kept: Vec<Part> = Vec::with_capacity(parts.len());
    for part in parts.into_iter().rev() {
        let padding = matches!(part, Part::Number(0))
            && matches!(kept.last(), Some(Part::End | Part::Letters(_)));
        if !padding {
            kept.push(part);
        }
    }
    kept.reverse();
    kept
}

fn parts(text: &str) -> Vec<Part> {
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
            Part::Letters(run.to_owned())
        });
    }
    parts
}

/// Orders two version strings.
#[must_use]
pub fn compare(left: &str, right: &str) -> Ordering {
    Key::new(left).cmp(&Key::new(right))
}

/// Sorts version strings ascending.
pub fn sort<T: AsRef<str>>(versions: &mut [T]) {
    versions.sort_by_cached_key(|version| Key::new(version.as_ref()));
}

#[cfg(test)]
#[path = "version_tests.rs"]
mod tests;
