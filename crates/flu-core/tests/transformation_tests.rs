//! Comprehensive tests for transformation operations

use flu_core::FluExt;

#[test]
fn map_basic() {
    let result: Vec<_> = vec![1, 2, 3].into_iter().flu().map(|x| x * 2).collect();
    assert_eq!(result, vec![2, 4, 6]);
}

#[test]
fn map_empty() {
    let result: Vec<i32> = vec![].into_iter().flu().map(|x: i32| x * 2).collect();
    assert!(result.is_empty());
}

#[test]
fn map_type_change() {
    let result: Vec<_> = vec![1, 2, 3]
        .into_iter()
        .flu()
        .map(|x| x.to_string())
        .collect();
    assert_eq!(result, vec!["1", "2", "3"]);
}

#[test]
fn map_strings() {
    let result: Vec<_> = vec!["hello", "world"]
        .into_iter()
        .flu()
        .map(str::to_uppercase)
        .collect();
    assert_eq!(result, vec!["HELLO", "WORLD"]);
}

#[test]
fn enumerate_basic() {
    let result: Vec<_> = vec!["a", "b", "c"].into_iter().flu().enumerate().collect();
    assert_eq!(result, vec![(0, "a"), (1, "b"), (2, "c")]);
}

#[test]
fn enumerate_empty() {
    let result: Vec<(usize, i32)> = vec![].into_iter().flu().enumerate().collect();
    assert!(result.is_empty());
}

#[test]
fn enumerate_after_skip() {
    let result: Vec<_> = (10..15).flu().skip(2).enumerate().collect();
    assert_eq!(result, vec![(0, 12), (1, 13), (2, 14)]);
}

#[test]
fn zip_basic() {
    let result: Vec<_> = vec![1, 2, 3]
        .into_iter()
        .flu()
        .zip(vec!["a", "b", "c"])
        .collect();
    assert_eq!(result, vec![(1, "a"), (2, "b"), (3, "c")]);
}

#[test]
fn zip_different_lengths_left_shorter() {
    let result: Vec<_> = vec![1, 2]
        .into_iter()
        .flu()
        .zip(vec!["a", "b", "c"])
        .collect();
    assert_eq!(result, vec![(1, "a"), (2, "b")]);
}

#[test]
fn zip_different_lengths_right_shorter() {
    let result: Vec<_> = vec![1, 2, 3]
        .into_iter()
        .flu()
        .zip(vec!["a", "b"])
        .collect();
    assert_eq!(result, vec![(1, "a"), (2, "b")]);
}

#[test]
fn zip_empty() {
    let result: Vec<(i32, &str)> = vec![].into_iter().flu().zip(vec!["a", "b"]).collect();
    assert!(result.is_empty());
}

#[test]
fn flatten_basic() {
    let result: Vec<_> = vec![vec![1, 2], vec![3, 4], vec![5]]
        .into_iter()
        .flu()
        .flatten()
        .collect();
    assert_eq!(result, vec![1, 2, 3, 4, 5]);
}

#[test]
fn flatten_empty_inner() {
    let result: Vec<_> = vec![vec![1, 2], vec![], vec![3]]
        .into_iter()
        .flu()
        .flatten()
        .collect();
    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn flatten_all_empty() {
    let result: Vec<i32> = vec![vec![], vec![], vec![]]
        .into_iter()
        .flu()
        .flatten()
        .collect();
    assert!(result.is_empty());
}

#[test]
fn chained_transformations() {
    let result: Vec<_> = (0..5)
        .flu()
        .map(|x| x * 2)
        .enumerate()
        .map(|(i, x)| (i, x + 1))
        .collect();
    assert_eq!(result, vec![(0, 1), (1, 3), (2, 5), (3, 7), (4, 9)]);
}
