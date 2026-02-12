//! Core Lob wrapper type and fluent API

use crate::grouping::{ChunkIterator, GroupByCollectIterator, WindowIterator};
use crate::joins::{InnerJoinIterator, LeftJoinIterator};
use std::collections::HashSet;
use std::hash::Hash;

/// Main wrapper type for fluent iterator operations
///
/// `Lob<I>` wraps any iterator and provides a chainable API for data transformations.
/// All operations are lazy and only execute when a terminal operation is called.
///
/// # Examples
///
/// ```
/// use lob_core::{Lob, LobExt};
///
/// let result: Vec<_> = vec![1, 2, 3, 4, 5]
///     .into_iter()
///     .lob()
///     .filter(|x| x % 2 == 0)
///     .map(|x| x * 2)
///     .collect();
///
/// assert_eq!(result, vec![4, 8]);
/// ```
#[derive(Debug, Clone)]
pub struct Lob<I> {
    iter: I,
}

impl<I: Iterator> Lob<I> {
    /// Create a new Lob wrapper from an iterator
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(iter: I) -> Self {
        Self { iter }
    }

    // ========== Selection Operations (lazy) ==========

    /// Filter elements based on a predicate
    ///
    /// # Examples
    ///
    /// ```
    /// use lob_core::LobExt;
    ///
    /// let result: Vec<_> = (0..10)
    ///     .lob()
    ///     .filter(|x| x % 2 == 0)
    ///     .collect();
    ///
    /// assert_eq!(result, vec![0, 2, 4, 6, 8]);
    /// ```
    #[must_use]
    pub fn filter<F>(self, predicate: F) -> Lob<impl Iterator<Item = I::Item>>
    where
        F: FnMut(&I::Item) -> bool,
    {
        Lob::new(self.iter.filter(predicate))
    }

    /// Take the first n elements
    ///
    /// # Examples
    ///
    /// ```
    /// use lob_core::LobExt;
    ///
    /// let result: Vec<_> = (0..100)
    ///     .lob()
    ///     .take(3)
    ///     .collect();
    ///
    /// assert_eq!(result, vec![0, 1, 2]);
    /// ```
    #[must_use]
    pub fn take(self, n: usize) -> Lob<impl Iterator<Item = I::Item>> {
        Lob::new(self.iter.take(n))
    }

    /// Skip the first n elements
    ///
    /// # Examples
    ///
    /// ```
    /// use lob_core::LobExt;
    ///
    /// let result: Vec<_> = (0..5)
    ///     .lob()
    ///     .skip(2)
    ///     .collect();
    ///
    /// assert_eq!(result, vec![2, 3, 4]);
    /// ```
    #[must_use]
    pub fn skip(self, n: usize) -> Lob<impl Iterator<Item = I::Item>> {
        Lob::new(self.iter.skip(n))
    }

    /// Take elements while predicate is true
    ///
    /// # Examples
    ///
    /// ```
    /// use lob_core::LobExt;
    ///
    /// let result: Vec<_> = vec![1, 2, 3, 4, 1, 2]
    ///     .into_iter()
    ///     .lob()
    ///     .take_while(|x| *x < 4)
    ///     .collect();
    ///
    /// assert_eq!(result, vec![1, 2, 3]);
    /// ```
    #[must_use]
    pub fn take_while<F>(self, predicate: F) -> Lob<impl Iterator<Item = I::Item>>
    where
        F: FnMut(&I::Item) -> bool,
    {
        Lob::new(self.iter.take_while(predicate))
    }

    /// Drop elements while predicate is true
    ///
    /// # Examples
    ///
    /// ```
    /// use lob_core::LobExt;
    ///
    /// let result: Vec<_> = vec![1, 2, 3, 4, 5]
    ///     .into_iter()
    ///     .lob()
    ///     .drop_while(|x| *x < 3)
    ///     .collect();
    ///
    /// assert_eq!(result, vec![3, 4, 5]);
    /// ```
    #[must_use]
    pub fn drop_while<F>(self, predicate: F) -> Lob<impl Iterator<Item = I::Item>>
    where
        F: FnMut(&I::Item) -> bool,
    {
        Lob::new(self.iter.skip_while(predicate))
    }

    /// Keep only unique elements (using `HashSet`)
    ///
    /// # Examples
    ///
    /// ```
    /// use lob_core::LobExt;
    ///
    /// let result: Vec<_> = vec![1, 2, 2, 3, 1, 4]
    ///     .into_iter()
    ///     .lob()
    ///     .unique()
    ///     .collect();
    ///
    /// assert_eq!(result, vec![1, 2, 3, 4]);
    /// ```
    #[must_use]
    pub fn unique(self) -> Lob<impl Iterator<Item = I::Item>>
    where
        I::Item: Eq + Hash + Clone,
    {
        let mut seen = HashSet::new();
        Lob::new(self.iter.filter(move |item| seen.insert(item.clone())))
    }

    // ========== Transformation Operations (lazy) ==========

    /// Transform each element
    ///
    /// # Examples
    ///
    /// ```
    /// use lob_core::LobExt;
    ///
    /// let result: Vec<_> = vec![1, 2, 3]
    ///     .into_iter()
    ///     .lob()
    ///     .map(|x| x * 2)
    ///     .collect();
    ///
    /// assert_eq!(result, vec![2, 4, 6]);
    /// ```
    #[must_use]
    pub fn map<F, B>(self, f: F) -> Lob<impl Iterator<Item = B>>
    where
        F: FnMut(I::Item) -> B,
    {
        Lob::new(self.iter.map(f))
    }

    /// Add index to each element
    ///
    /// # Examples
    ///
    /// ```
    /// use lob_core::LobExt;
    ///
    /// let result: Vec<_> = vec!["a", "b", "c"]
    ///     .into_iter()
    ///     .lob()
    ///     .enumerate()
    ///     .collect();
    ///
    /// assert_eq!(result, vec![(0, "a"), (1, "b"), (2, "c")]);
    /// ```
    #[must_use]
    pub fn enumerate(self) -> Lob<impl Iterator<Item = (usize, I::Item)>> {
        Lob::new(self.iter.enumerate())
    }

    /// Zip with another iterator
    ///
    /// # Examples
    ///
    /// ```
    /// use lob_core::LobExt;
    ///
    /// let result: Vec<_> = vec![1, 2, 3]
    ///     .into_iter()
    ///     .lob()
    ///     .zip(vec!["a", "b", "c"])
    ///     .collect();
    ///
    /// assert_eq!(result, vec![(1, "a"), (2, "b"), (3, "c")]);
    /// ```
    #[must_use]
    pub fn zip<J>(self, other: J) -> Lob<impl Iterator<Item = (I::Item, J::Item)>>
    where
        J: IntoIterator,
    {
        Lob::new(self.iter.zip(other))
    }

    /// Flatten nested iterators
    ///
    /// # Examples
    ///
    /// ```
    /// use lob_core::LobExt;
    ///
    /// let result: Vec<_> = vec![vec![1, 2], vec![3, 4]]
    ///     .into_iter()
    ///     .lob()
    ///     .flatten()
    ///     .collect();
    ///
    /// assert_eq!(result, vec![1, 2, 3, 4]);
    /// ```
    #[must_use]
    pub fn flatten<T>(self) -> Lob<impl Iterator<Item = T>>
    where
        I::Item: IntoIterator<Item = T>,
    {
        Lob::new(self.iter.flatten())
    }

    // ========== Grouping Operations ==========

    /// Group elements into chunks of size n
    ///
    /// # Examples
    ///
    /// ```
    /// use lob_core::LobExt;
    ///
    /// let result: Vec<_> = (0..5)
    ///     .lob()
    ///     .chunk(2)
    ///     .collect();
    ///
    /// assert_eq!(result, vec![vec![0, 1], vec![2, 3], vec![4]]);
    /// ```
    #[must_use]
    pub fn chunk(self, n: usize) -> Lob<impl Iterator<Item = Vec<I::Item>>> {
        Lob::new(ChunkIterator::new(self.iter, n))
    }

    /// Create sliding windows of size n
    ///
    /// # Examples
    ///
    /// ```
    /// use lob_core::LobExt;
    ///
    /// let result: Vec<_> = (1..=4)
    ///     .lob()
    ///     .window(2)
    ///     .collect();
    ///
    /// assert_eq!(result, vec![vec![1, 2], vec![2, 3], vec![3, 4]]);
    /// ```
    #[must_use]
    pub fn window(self, n: usize) -> Lob<impl Iterator<Item = Vec<I::Item>>>
    where
        I::Item: Clone,
    {
        Lob::new(WindowIterator::new(self.iter, n))
    }

    /// Group elements by a key function
    ///
    /// # Examples
    ///
    /// ```
    /// use lob_core::LobExt;
    ///
    /// let result: Vec<_> = vec![1, 2, 3, 4, 5, 6]
    ///     .into_iter()
    ///     .lob()
    ///     .group_by(|x| x % 2)
    ///     .collect();
    ///
    /// // Result contains (key, group) pairs
    /// assert_eq!(result.len(), 2);
    /// ```
    #[must_use]
    pub fn group_by<K, F>(self, key_fn: F) -> Lob<impl Iterator<Item = (K, Vec<I::Item>)>>
    where
        K: Eq + Hash,
        F: FnMut(&I::Item) -> K,
    {
        Lob::new(GroupByCollectIterator::new(self.iter, key_fn))
    }

    // ========== Join Operations ==========

    /// Inner join with another iterator based on key functions
    ///
    /// # Examples
    ///
    /// ```
    /// use lob_core::LobExt;
    ///
    /// let left = vec![(1, "a"), (2, "b"), (3, "c")];
    /// let right = vec![(1, "x"), (2, "y"), (4, "z")];
    ///
    /// let result: Vec<_> = left
    ///     .into_iter()
    ///     .lob()
    ///     .join_inner(right, |x| x.0, |x| x.0)
    ///     .collect();
    ///
    /// assert_eq!(result, vec![((1, "a"), (1, "x")), ((2, "b"), (2, "y"))]);
    /// ```
    #[must_use]
    pub fn join_inner<J, K, FL, FR>(
        self,
        other: J,
        left_key: FL,
        right_key: FR,
    ) -> Lob<impl Iterator<Item = (I::Item, J::Item)>>
    where
        I::Item: Clone,
        J: IntoIterator,
        J::Item: Clone,
        K: Eq + Hash,
        FL: Fn(&I::Item) -> K,
        FR: Fn(&J::Item) -> K,
    {
        Lob::new(InnerJoinIterator::new(
            self.iter, other, left_key, right_key,
        ))
    }

    /// Left join with another iterator based on key functions
    ///
    /// # Examples
    ///
    /// ```
    /// use lob_core::LobExt;
    ///
    /// let left = vec![(1, "a"), (2, "b"), (3, "c")];
    /// let right = vec![(1, "x"), (2, "y")];
    ///
    /// let result: Vec<_> = left
    ///     .into_iter()
    ///     .lob()
    ///     .join_left(right, |x| x.0, |x| x.0)
    ///     .collect();
    ///
    /// assert_eq!(result.len(), 3);  // All left items preserved
    /// ```
    #[must_use]
    pub fn join_left<J, K, FL, FR>(
        self,
        other: J,
        left_key: FL,
        right_key: FR,
    ) -> Lob<impl Iterator<Item = (I::Item, Option<J::Item>)>>
    where
        I::Item: Clone,
        J: IntoIterator,
        J::Item: Clone,
        K: Eq + Hash,
        FL: Fn(&I::Item) -> K,
        FR: Fn(&J::Item) -> K,
    {
        Lob::new(LeftJoinIterator::new(self.iter, other, left_key, right_key))
    }

    // ========== Terminal Operations (consume iterator) ==========

    /// Collect into a collection
    ///
    /// # Examples
    ///
    /// ```
    /// use lob_core::LobExt;
    ///
    /// let result: Vec<_> = (0..5)
    ///     .lob()
    ///     .filter(|x| x % 2 == 0)
    ///     .collect();
    ///
    /// assert_eq!(result, vec![0, 2, 4]);
    /// ```
    pub fn collect<B: FromIterator<I::Item>>(self) -> B {
        self.iter.collect()
    }

    /// Count the number of elements
    ///
    /// # Examples
    ///
    /// ```
    /// use lob_core::LobExt;
    ///
    /// let count = (0..10)
    ///     .lob()
    ///     .filter(|x| x % 2 == 0)
    ///     .count();
    ///
    /// assert_eq!(count, 5);
    /// ```
    pub fn count(self) -> usize {
        self.iter.count()
    }

    /// Sum all elements
    ///
    /// # Examples
    ///
    /// ```
    /// use lob_core::LobExt;
    ///
    /// let sum = (1..=5).lob().sum::<i32>();
    ///
    /// assert_eq!(sum, 15);
    /// ```
    pub fn sum<S>(self) -> S
    where
        S: std::iter::Sum<I::Item>,
    {
        self.iter.sum()
    }

    /// Find the minimum element
    ///
    /// # Examples
    ///
    /// ```
    /// use lob_core::LobExt;
    ///
    /// let min = vec![3, 1, 4, 1, 5].into_iter().lob().min();
    ///
    /// assert_eq!(min, Some(1));
    /// ```
    pub fn min(self) -> Option<I::Item>
    where
        I::Item: Ord,
    {
        self.iter.min()
    }

    /// Find the maximum element
    ///
    /// # Examples
    ///
    /// ```
    /// use lob_core::LobExt;
    ///
    /// let max = vec![3, 1, 4, 1, 5].into_iter().lob().max();
    ///
    /// assert_eq!(max, Some(5));
    /// ```
    pub fn max(self) -> Option<I::Item>
    where
        I::Item: Ord,
    {
        self.iter.max()
    }

    /// Get the first element
    ///
    /// # Examples
    ///
    /// ```
    /// use lob_core::LobExt;
    ///
    /// let first = (1..10).lob().first();
    ///
    /// assert_eq!(first, Some(1));
    /// ```
    pub fn first(mut self) -> Option<I::Item> {
        self.iter.next()
    }

    /// Get the last element
    ///
    /// # Examples
    ///
    /// ```
    /// use lob_core::LobExt;
    ///
    /// let last = (1..10).lob().last();
    ///
    /// assert_eq!(last, Some(9));
    /// ```
    pub fn last(self) -> Option<I::Item> {
        self.iter.last()
    }

    /// Reduce to a single value
    ///
    /// # Examples
    ///
    /// ```
    /// use lob_core::LobExt;
    ///
    /// let product = (1..=5).lob().reduce(|a, b| a * b);
    ///
    /// assert_eq!(product, Some(120));
    /// ```
    pub fn reduce<F>(self, f: F) -> Option<I::Item>
    where
        F: FnMut(I::Item, I::Item) -> I::Item,
    {
        self.iter.reduce(f)
    }

    /// Fold with an initial value
    ///
    /// # Examples
    ///
    /// ```
    /// use lob_core::LobExt;
    ///
    /// let sum = (1..=5).lob().fold(0, |a, b| a + b);
    ///
    /// assert_eq!(sum, 15);
    /// ```
    pub fn fold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, I::Item) -> B,
    {
        self.iter.fold(init, f)
    }

    /// Collect into a Vec
    ///
    /// # Examples
    ///
    /// ```
    /// use lob_core::LobExt;
    ///
    /// let list = (0..5).lob().to_list();
    ///
    /// assert_eq!(list, vec![0, 1, 2, 3, 4]);
    /// ```
    pub fn to_list(self) -> Vec<I::Item> {
        self.iter.collect()
    }

    /// Check if any element matches a predicate
    ///
    /// # Examples
    ///
    /// ```
    /// use lob_core::LobExt;
    ///
    /// let has_even = (1..10).lob().any(|x| x % 2 == 0);
    ///
    /// assert!(has_even);
    /// ```
    pub fn any<F>(mut self, f: F) -> bool
    where
        F: FnMut(I::Item) -> bool,
    {
        self.iter.any(f)
    }

    /// Check if all elements match a predicate
    ///
    /// # Examples
    ///
    /// ```
    /// use lob_core::LobExt;
    ///
    /// let all_positive = (1..10).lob().all(|x| x > 0);
    ///
    /// assert!(all_positive);
    /// ```
    pub fn all<F>(mut self, f: F) -> bool
    where
        F: FnMut(I::Item) -> bool,
    {
        self.iter.all(f)
    }
}

/// Extension trait to add `.lob()` method to all iterators
///
/// # Examples
///
/// ```
/// use lob_core::LobExt;
///
/// let result: Vec<_> = vec![1, 2, 3, 4, 5]
///     .into_iter()
///     .lob()  // Convert to Lob<I>
///     .filter(|x| x % 2 == 0)
///     .collect();
///
/// assert_eq!(result, vec![2, 4]);
/// ```
pub trait LobExt: Iterator + Sized {
    /// Convert an iterator into a `Lob` wrapper
    fn lob(self) -> Lob<Self> {
        Lob::new(self)
    }
}

impl<I: Iterator> LobExt for I {}

/// Implement `IntoIterator` for Lob to allow using it in for loops
impl<I: Iterator> IntoIterator for Lob<I> {
    type Item = I::Item;
    type IntoIter = I;

    fn into_iter(self) -> Self::IntoIter {
        self.iter
    }
}
