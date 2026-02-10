//! Comprehensive tests for terminal operations

use flu_core::FluExt;

#[test]
fn collect_to_vec() {
    let result: Vec<_> = (0..5).flu().collect();
    assert_eq!(result, vec![0, 1, 2, 3, 4]);
}

#[test]
fn count_basic() {
    let count = (0..10).flu().count();
    assert_eq!(count, 10);
}

#[test]
fn count_empty() {
    let empty: Vec<i32> = vec![];
    let count = empty.into_iter().flu().count();
    assert_eq!(count, 0);
}

#[test]
fn count_after_filter() {
    let count = (0..10).flu().filter(|x| x % 2 == 0).count();
    assert_eq!(count, 5);
}

#[test]
fn sum_integers() {
    let sum = (1..=5).flu().sum::<i32>();
    assert_eq!(sum, 15);
}

#[test]
fn sum_empty() {
    let empty: Vec<i32> = vec![];
    let sum: i32 = empty.into_iter().flu().sum();
    assert_eq!(sum, 0);
}

#[test]
fn sum_floats() {
    let sum = vec![1.5, 2.5, 3.0].into_iter().flu().sum::<f64>();
    assert!((sum - 7.0).abs() < f64::EPSILON);
}

#[test]
fn min_basic() {
    let min = vec![3, 1, 4, 1, 5].into_iter().flu().min();
    assert_eq!(min, Some(1));
}

#[test]
fn min_empty() {
    let min: Option<i32> = vec![].into_iter().flu().min();
    assert_eq!(min, None);
}

#[test]
fn min_single() {
    let min = vec![42].into_iter().flu().min();
    assert_eq!(min, Some(42));
}

#[test]
fn max_basic() {
    let max = vec![3, 1, 4, 1, 5].into_iter().flu().max();
    assert_eq!(max, Some(5));
}

#[test]
fn max_empty() {
    let max: Option<i32> = vec![].into_iter().flu().max();
    assert_eq!(max, None);
}

#[test]
fn max_single() {
    let max = vec![42].into_iter().flu().max();
    assert_eq!(max, Some(42));
}

#[test]
fn first_basic() {
    let first = (1..10).flu().first();
    assert_eq!(first, Some(1));
}

#[test]
fn first_empty() {
    let first: Option<i32> = vec![].into_iter().flu().first();
    assert_eq!(first, None);
}

#[test]
fn first_after_filter() {
    let first = (0..10).flu().filter(|x| x % 3 == 0).first();
    assert_eq!(first, Some(0));
}

#[test]
fn last_basic() {
    let last = (1..10).flu().last();
    assert_eq!(last, Some(9));
}

#[test]
fn last_empty() {
    let last: Option<i32> = vec![].into_iter().flu().last();
    assert_eq!(last, None);
}

#[test]
fn last_after_take() {
    let last = (1..100).flu().take(5).last();
    assert_eq!(last, Some(5));
}

#[test]
fn reduce_basic() {
    let product = (1..=5).flu().reduce(|a, b| a * b);
    assert_eq!(product, Some(120));
}

#[test]
fn reduce_empty() {
    let result: Option<i32> = vec![].into_iter().flu().reduce(|a, b| a + b);
    assert_eq!(result, None);
}

#[test]
fn reduce_single() {
    let result = vec![42].into_iter().flu().reduce(|a, b| a + b);
    assert_eq!(result, Some(42));
}

#[test]
fn fold_basic() {
    let sum = (1..=5).flu().fold(0, |a, b| a + b);
    assert_eq!(sum, 15);
}

#[test]
fn fold_with_initial() {
    let sum = (1..=5).flu().fold(10, |a, b| a + b);
    assert_eq!(sum, 25);
}

#[test]
fn fold_empty() {
    let empty: Vec<i32> = vec![];
    let sum: i32 = empty.into_iter().flu().fold(42, |a, b| a + b);
    assert_eq!(sum, 42);
}

#[test]
fn to_list_basic() {
    let list = (0..5).flu().to_list();
    assert_eq!(list, vec![0, 1, 2, 3, 4]);
}

#[test]
fn to_list_empty() {
    let list: Vec<i32> = vec![].into_iter().flu().to_list();
    assert!(list.is_empty());
}

#[test]
fn any_true() {
    let result = (1..10).flu().any(|x| x > 5);
    assert!(result);
}

#[test]
fn any_false() {
    let result = (1..10).flu().any(|x| x > 100);
    assert!(!result);
}

#[test]
fn any_empty() {
    let empty: Vec<i32> = vec![];
    let result: bool = empty.into_iter().flu().any(|x| x > 0);
    assert!(!result);
}

#[test]
fn all_true() {
    let result = (1..10).flu().all(|x| x > 0);
    assert!(result);
}

#[test]
fn all_false() {
    let result = (1..10).flu().all(|x| x > 5);
    assert!(!result);
}

#[test]
fn all_empty() {
    let empty: Vec<i32> = vec![];
    let result: bool = empty.into_iter().flu().all(|x| x > 0);
    assert!(result); // Vacuous truth
}
