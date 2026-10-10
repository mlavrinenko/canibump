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
