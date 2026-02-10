//! Comprehensive tests for selection operations

use flu_core::FluExt;

#[test]
fn filter_basic() {
    let result: Vec<_> = (0..10).flu().filter(|x| x % 2 == 0).collect();
    assert_eq!(result, vec![0, 2, 4, 6, 8]);
}

#[test]
fn filter_empty_result() {
    let result: Vec<_> = (0..10).flu().filter(|_| false).collect();
    assert!(result.is_empty());
}

#[test]
fn filter_all_pass() {
    let result: Vec<_> = (0..5).flu().filter(|_| true).collect();
    assert_eq!(result, vec![0, 1, 2, 3, 4]);
}

#[test]
fn filter_strings() {
    let data = vec!["hello", "world", "test", "hello world"];
    let result: Vec<_> = data.into_iter().flu().filter(|s| s.len() > 4).collect();
    assert_eq!(result, vec!["hello", "world", "hello world"]);
}

#[test]
fn take_basic() {
    let result: Vec<_> = (0..10).flu().take(3).collect();
    assert_eq!(result, vec![0, 1, 2]);
}

#[test]
fn take_more_than_available() {
    let result: Vec<_> = (0..3).flu().take(10).collect();
    assert_eq!(result, vec![0, 1, 2]);
}

#[test]
fn take_zero() {
    let result: Vec<_> = (0..10).flu().take(0).collect();
    assert!(result.is_empty());
}

#[test]
fn skip_basic() {
    let result: Vec<_> = (0..5).flu().skip(2).collect();
    assert_eq!(result, vec![2, 3, 4]);
}

#[test]
fn skip_more_than_available() {
    let result: Vec<_> = (0..3).flu().skip(10).collect();
    assert!(result.is_empty());
}

#[test]
fn skip_zero() {
    let result: Vec<_> = (0..3).flu().skip(0).collect();
    assert_eq!(result, vec![0, 1, 2]);
}

#[test]
fn take_while_basic() {
    let result: Vec<_> = vec![1, 2, 3, 4, 1, 2]
        .into_iter()
        .flu()
        .take_while(|x| *x < 4)
        .collect();
    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn take_while_none() {
    let result: Vec<_> = (1..5).flu().take_while(|x| *x > 10).collect();
    assert!(result.is_empty());
}

#[test]
fn take_while_all() {
    let result: Vec<_> = (1..5).flu().take_while(|x| *x < 10).collect();
    assert_eq!(result, vec![1, 2, 3, 4]);
}

#[test]
fn drop_while_basic() {
    let result: Vec<_> = vec![1, 2, 3, 4, 5]
        .into_iter()
        .flu()
        .drop_while(|x| *x < 3)
        .collect();
    assert_eq!(result, vec![3, 4, 5]);
}

#[test]
fn drop_while_none() {
    let result: Vec<_> = (1..5).flu().drop_while(|x| *x > 10).collect();
    assert_eq!(result, vec![1, 2, 3, 4]);
}

#[test]
fn drop_while_all() {
    let result: Vec<_> = (1..5).flu().drop_while(|x| *x < 10).collect();
    assert!(result.is_empty());
}

#[test]
fn unique_basic() {
    let result: Vec<_> = vec![1, 2, 2, 3, 1, 4, 3]
        .into_iter()
        .flu()
        .unique()
        .collect();
    assert_eq!(result, vec![1, 2, 3, 4]);
}

#[test]
fn unique_all_same() {
    let result: Vec<_> = vec![1, 1, 1, 1].into_iter().flu().unique().collect();
    assert_eq!(result, vec![1]);
}

#[test]
fn unique_empty() {
    let result: Vec<i32> = vec![].into_iter().flu().unique().collect();
    assert!(result.is_empty());
}

#[test]
fn unique_strings() {
    let result: Vec<_> = vec!["a", "b", "a", "c", "b"]
        .into_iter()
        .flu()
        .unique()
        .collect();
    assert_eq!(result, vec!["a", "b", "c"]);
}

#[test]
fn chained_selection() {
    let result: Vec<_> = (0..20)
        .flu()
        .filter(|x| x % 2 == 0)
        .skip(2)
        .take(3)
        .collect();
    assert_eq!(result, vec![4, 6, 8]);
}
