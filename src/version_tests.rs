use std::cmp::Ordering;

use super::*;

#[test]
fn semver_orders_releases_and_prereleases() {
    assert_eq!(compare("1.2.3", "1.10.0"), Ordering::Less);
    assert_eq!(compare("v1.0.0", "1.0.0"), Ordering::Equal);
    assert_eq!(compare("1.0.0-rc.1", "1.0.0"), Ordering::Less);
    assert_eq!(compare("0.6.0", "0.5.0"), Ordering::Greater);
}

#[test]
fn lenient_order_drops_a_letter_prefix_and_compares_numbers() {
    assert_eq!(compare("b7012", "7012"), Ordering::Equal);
    assert_eq!(compare("b999", "b1000"), Ordering::Less);
    assert_eq!(compare("2024.01", "2024.1"), Ordering::Equal);
    assert_eq!(compare("1.2", "1.2.1"), Ordering::Less);
}

#[test]
fn a_letter_suffix_is_a_prerelease() {
    assert_eq!(compare("1.0rc1", "1.0"), Ordering::Less);
    assert_eq!(compare("1.0a", "1.0b"), Ordering::Less);
    assert_eq!(compare("1.0", "1.0rc1"), Ordering::Greater);
}

#[test]
fn numbers_sort_after_letters_at_the_same_position() {
    assert_eq!(compare("1.a", "1.0"), Ordering::Less);
    assert_eq!(compare("1.0", "1.a"), Ordering::Greater);
}

#[test]
fn sorts_tags_ascending() {
    let mut tags = vec!["v0.10.0", "v0.2.0", "v0.6.0", "v0.6.0-rc1"];
    sort(&mut tags);
    assert_eq!(tags, ["v0.2.0", "v0.6.0-rc1", "v0.6.0", "v0.10.0"]);
}

#[test]
fn mixed_pairs_no_longer_cycle() {
    assert_eq!(compare("1.0.0", "1.0.0+zz"), Ordering::Equal);
    assert_eq!(compare("1.0.0+zz", "1.0.0zzz"), Ordering::Greater);
    assert_eq!(compare("1.0.0zzz", "1.0.0"), Ordering::Less);
}

#[test]
fn numbered_prereleases_order_numerically() {
    assert_eq!(compare("1.0.0-rc9", "1.0.0-rc10"), Ordering::Less);
    assert_eq!(compare("1.0.0-rc.9", "1.0.0-rc.10"), Ordering::Less);
    assert_eq!(compare("1.0rc9", "1.0rc10"), Ordering::Less);
}

#[test]
fn numbers_outrank_letters_inside_a_prerelease_too() {
    assert_eq!(compare("1.0.0-1", "1.0.0-rc"), Ordering::Greater);
    assert_eq!(
        compare("1.0.0-alpha.1", "1.0.0-alpha.beta"),
        Ordering::Greater
    );
}

#[test]
fn build_metadata_is_ignored() {
    assert_eq!(compare("1.0.0+zz", "1.0.0+aa"), Ordering::Equal);
    assert_eq!(compare("v1.0.0+build.5", "1.0.0"), Ordering::Equal);
    assert_eq!(compare("1.0.0+zz", "1.0.1"), Ordering::Less);
}

/// Semver, `v`-tags, `b`-builds, rc streams, `+build` and dates, mixed.
const CORPUS: [&str; 24] = [
    "1.0.0",
    "v1.0.0",
    "1.0.0+zz",
    "1.0.0zzz",
    "1.0.0-rc.1",
    "1.0.0-rc.10",
    "1.0.0-rc9",
    "1.0rc1",
    "1.0rc10",
    "1.0",
    "1.0.1",
    "1.2",
    "1.2.1",
    "v1.10.0+build.7",
    "b999",
    "b1000",
    "b7012",
    "7012",
    "2024.01",
    "2024.1",
    "2024.01.15",
    "1.a",
    "latest",
    "",
];

#[test]
fn corpus_is_antisymmetric() {
    for left in CORPUS {
        for right in CORPUS {
            assert_eq!(
                compare(left, right),
                compare(right, left).reverse(),
                "{left:?} vs {right:?}"
            );
        }
    }
}

#[test]
fn corpus_is_transitive() {
    for one in CORPUS {
        for two in CORPUS {
            for three in CORPUS {
                if compare(one, two).is_le() && compare(two, three).is_le() {
                    assert!(
                        compare(one, three).is_le(),
                        "{one:?} <= {two:?} <= {three:?} but not {one:?} <= {three:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn sorting_the_corpus_ignores_input_order() {
    let mut forward = CORPUS.to_vec();
    sort(&mut forward);
    let mut backward: Vec<&str> = CORPUS.iter().rev().copied().collect();
    sort(&mut backward);
    assert!(
        forward
            .iter()
            .zip(forward.iter().skip(1))
            .all(|(low, high)| compare(low, high).is_le())
    );
    assert!(
        forward
            .iter()
            .zip(&backward)
            .all(|(left, right)| compare(left, right).is_eq())
    );
}
