//! Comprehensive tests for join operations

use flu_core::FluExt;

#[test]
fn inner_join_basic() {
    let left = vec![(1, "a"), (2, "b"), (3, "c")];
    let right = vec![(1, "x"), (2, "y"), (4, "z")];

    let result: Vec<_> = left
        .into_iter()
        .flu()
        .join_inner(right, |x| x.0, |x| x.0)
        .collect();

    assert_eq!(result.len(), 2);
    assert!(result.contains(&((1, "a"), (1, "x"))));
    assert!(result.contains(&((2, "b"), (2, "y"))));
}

#[test]
fn inner_join_no_matches() {
    let left = vec![(1, "a"), (2, "b")];
    let right = vec![(3, "x"), (4, "y")];

    let result: Vec<_> = left
        .into_iter()
        .flu()
        .join_inner(right, |x| x.0, |x| x.0)
        .collect();

    assert_eq!(result.len(), 0);
}

#[test]
fn inner_join_all_match() {
    let left = vec![(1, "a"), (2, "b")];
    let right = vec![(1, "x"), (2, "y")];

    let result: Vec<_> = left
        .into_iter()
        .flu()
        .join_inner(right, |x| x.0, |x| x.0)
        .collect();

    assert_eq!(result.len(), 2);
}

#[test]
fn inner_join_multiple_matches() {
    let left = vec![(1, "a"), (1, "b")];
    let right = vec![(1, "x"), (1, "y")];

    let result: Vec<_> = left
        .into_iter()
        .flu()
        .join_inner(right, |x| x.0, |x| x.0)
        .collect();

    // 2 left * 2 right = 4 results
    assert_eq!(result.len(), 4);
}

#[test]
fn inner_join_empty_left() {
    let left: Vec<(i32, &str)> = vec![];
    let right = vec![(1, "x"), (2, "y")];

    let result: Vec<_> = left
        .into_iter()
        .flu()
        .join_inner(right, |x| x.0, |x| x.0)
        .collect();

    assert_eq!(result.len(), 0);
}

#[test]
fn inner_join_empty_right() {
    let left = vec![(1, "a"), (2, "b")];
    let right: Vec<(i32, &str)> = vec![];

    let result: Vec<_> = left
        .into_iter()
        .flu()
        .join_inner(right, |x| x.0, |x| x.0)
        .collect();

    assert_eq!(result.len(), 0);
}

#[test]
fn inner_join_strings() {
    let left = vec![("key1", 1), ("key2", 2)];
    let right = vec![("key1", "a"), ("key2", "b"), ("key3", "c")];

    let result: Vec<_> = left
        .into_iter()
        .flu()
        .join_inner(right, |x| x.0, |x| x.0)
        .collect();

    assert_eq!(result.len(), 2);
}

#[test]
fn left_join_basic() {
    let left = vec![(1, "a"), (2, "b"), (3, "c")];
    let right = vec![(1, "x"), (2, "y")];

    let result: Vec<_> = left
        .into_iter()
        .flu()
        .join_left(right, |x| x.0, |x| x.0)
        .collect();

    assert_eq!(result.len(), 3);
    assert_eq!(result[0], ((1, "a"), Some((1, "x"))));
    assert_eq!(result[1], ((2, "b"), Some((2, "y"))));
    assert_eq!(result[2], ((3, "c"), None));
}

#[test]
fn left_join_all_match() {
    let left = vec![(1, "a"), (2, "b")];
    let right = vec![(1, "x"), (2, "y")];

    let result: Vec<_> = left
        .into_iter()
        .flu()
        .join_left(right, |x| x.0, |x| x.0)
        .collect();

    assert_eq!(result.len(), 2);
    assert!(result.iter().all(|(_, r)| r.is_some()));
}

#[test]
fn left_join_no_matches() {
    let left = vec![(1, "a"), (2, "b")];
    let right = vec![(3, "x"), (4, "y")];

    let result: Vec<_> = left
        .into_iter()
        .flu()
        .join_left(right, |x| x.0, |x| x.0)
        .collect();

    assert_eq!(result.len(), 2);
    assert!(result.iter().all(|(_, r)| r.is_none()));
}

#[test]
fn left_join_empty_left() {
    let left: Vec<(i32, &str)> = vec![];
    let right = vec![(1, "x"), (2, "y")];

    let result: Vec<_> = left
        .into_iter()
        .flu()
        .join_left(right, |x| x.0, |x| x.0)
        .collect();

    assert_eq!(result.len(), 0);
}

#[test]
fn left_join_empty_right() {
    let left = vec![(1, "a"), (2, "b")];
    let right: Vec<(i32, &str)> = vec![];

    let result: Vec<_> = left
        .into_iter()
        .flu()
        .join_left(right, |x| x.0, |x| x.0)
        .collect();

    assert_eq!(result.len(), 2);
    assert!(result.iter().all(|(_, r)| r.is_none()));
}

#[test]
fn left_join_multiple_matches() {
    let left = vec![(1, "a")];
    let right = vec![(1, "x"), (1, "y"), (1, "z")];

    let result: Vec<_> = left
        .into_iter()
        .flu()
        .join_left(right, |x| x.0, |x| x.0)
        .collect();

    // One left item matched with 3 right items
    assert_eq!(result.len(), 3);
    assert!(result.iter().all(|(_, r)| r.is_some()));
}

#[test]
fn join_with_filter() {
    let left = vec![(1, "a"), (2, "b"), (3, "c"), (4, "d")];
    let right = vec![(1, "x"), (2, "y"), (3, "z")];

    let result: Vec<_> = left
        .into_iter()
        .flu()
        .filter(|x| x.0 <= 3)
        .join_inner(right, |x| x.0, |x| x.0)
        .collect();

    assert_eq!(result.len(), 3);
}
