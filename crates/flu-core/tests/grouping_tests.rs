//! Comprehensive tests for grouping operations

use flu_core::FluExt;

#[test]
fn chunk_basic() {
    let result: Vec<_> = (0..7).flu().chunk(3).collect();
    assert_eq!(result, vec![vec![0, 1, 2], vec![3, 4, 5], vec![6]]);
}

#[test]
fn chunk_exact_fit() {
    let result: Vec<_> = (0..6).flu().chunk(2).collect();
    assert_eq!(result, vec![vec![0, 1], vec![2, 3], vec![4, 5]]);
}

#[test]
fn chunk_size_one() {
    let result: Vec<_> = (0..3).flu().chunk(1).collect();
    assert_eq!(result, vec![vec![0], vec![1], vec![2]]);
}

#[test]
fn chunk_larger_than_input() {
    let result: Vec<_> = (0..3).flu().chunk(10).collect();
    assert_eq!(result, vec![vec![0, 1, 2]]);
}

#[test]
fn chunk_empty() {
    let result: Vec<Vec<i32>> = vec![].into_iter().flu().chunk(3).collect();
    assert!(result.is_empty());
}

#[test]
fn window_basic() {
    let result: Vec<_> = (1..=5).flu().window(3).collect();
    assert_eq!(result, vec![vec![1, 2, 3], vec![2, 3, 4], vec![3, 4, 5]]);
}

#[test]
fn window_size_two() {
    let result: Vec<_> = (1..=4).flu().window(2).collect();
    assert_eq!(result, vec![vec![1, 2], vec![2, 3], vec![3, 4]]);
}

#[test]
fn window_size_one() {
    let result: Vec<_> = (1..=3).flu().window(1).collect();
    assert_eq!(result, vec![vec![1], vec![2], vec![3]]);
}

#[test]
fn window_too_small() {
    let result: Vec<Vec<i32>> = vec![1, 2].into_iter().flu().window(3).collect();
    assert!(result.is_empty());
}

#[test]
fn window_empty() {
    let result: Vec<Vec<i32>> = vec![].into_iter().flu().window(2).collect();
    assert!(result.is_empty());
}

#[test]
fn window_exact_size() {
    let result: Vec<_> = vec![1, 2, 3].into_iter().flu().window(3).collect();
    assert_eq!(result, vec![vec![1, 2, 3]]);
}

#[test]
fn group_by_basic() {
    let data = vec![1, 2, 3, 4, 5, 6];
    let mut groups: Vec<_> = data.into_iter().flu().group_by(|x| x % 2).collect();

    // Sort for deterministic testing
    groups.sort_by_key(|(k, _)| *k);

    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].0, 0); // even
    assert_eq!(groups[0].1, vec![2, 4, 6]);
    assert_eq!(groups[1].0, 1); // odd
    assert_eq!(groups[1].1, vec![1, 3, 5]);
}

#[test]
fn group_by_strings() {
    let data = vec!["apple", "apricot", "banana", "blueberry", "cherry"];
    let mut groups: Vec<_> = data
        .into_iter()
        .flu()
        .group_by(|s| s.chars().next().unwrap())
        .collect();

    groups.sort_by_key(|(k, _)| *k);

    assert_eq!(groups.len(), 3);
    assert_eq!(groups[0].0, 'a');
    assert_eq!(groups[0].1, vec!["apple", "apricot"]);
    assert_eq!(groups[1].0, 'b');
    assert_eq!(groups[1].1, vec!["banana", "blueberry"]);
    assert_eq!(groups[2].0, 'c');
    assert_eq!(groups[2].1, vec!["cherry"]);
}

#[test]
fn group_by_single_group() {
    let data = vec![1, 1, 1, 1];
    let groups: Vec<_> = data.into_iter().flu().group_by(|_| "same").collect();

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].0, "same");
    assert_eq!(groups[0].1, vec![1, 1, 1, 1]);
}

#[test]
fn group_by_all_different() {
    let data = vec![1, 2, 3];
    let groups: Vec<_> = data.into_iter().flu().group_by(|x| *x).collect();

    assert_eq!(groups.len(), 3);
}

#[test]
fn group_by_empty() {
    let data: Vec<i32> = vec![];
    let groups: Vec<_> = data.into_iter().flu().group_by(|x| x % 2).collect();

    assert!(groups.is_empty());
}

#[test]
fn flatten_with_chunk() {
    let result: Vec<_> = (0..6).flu().chunk(2).flatten().collect();
    assert_eq!(result, vec![0, 1, 2, 3, 4, 5]);
}

#[test]
fn chunk_then_map() {
    let result: Vec<_> = (0..6).flu().chunk(2).map(|chunk| chunk.len()).collect();
    assert_eq!(result, vec![2, 2, 2]);
}
