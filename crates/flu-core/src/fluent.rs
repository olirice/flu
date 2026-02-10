//! Core Flu wrapper type and fluent API

use crate::grouping::{ChunkIterator, GroupByCollectIterator, WindowIterator};
use crate::joins::{InnerJoinIterator, LeftJoinIterator};
use std::collections::HashSet;
use std::hash::Hash;

/// Main wrapper type for fluent iterator operations
///
/// `Flu<I>` wraps any iterator and provides a chainable API for data transformations.
/// All operations are lazy and only execute when a terminal operation is called.
///
/// # Examples
///
/// ```
/// use flu_core::{Flu, FluExt};
///
/// let result: Vec<_> = vec![1, 2, 3, 4, 5]
///     .into_iter()
///     .flu()
///     .filter(|x| x % 2 == 0)
///     .map(|x| x * 2)
///     .collect();
///
/// assert_eq!(result, vec![4, 8]);
/// ```
#[derive(Debug, Clone)]
pub struct Flu<I> {
    iter: I,
}

impl<I: Iterator> Flu<I> {
    /// Create a new Flu wrapper from an iterator
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
    /// use flu_core::FluExt;
    ///
    /// let result: Vec<_> = (0..10)
    ///     .flu()
    ///     .filter(|x| x % 2 == 0)
    ///     .collect();
    ///
    /// assert_eq!(result, vec![0, 2, 4, 6, 8]);
    /// ```
    #[must_use]
    pub fn filter<F>(self, predicate: F) -> Flu<impl Iterator<Item = I::Item>>
    where
        F: FnMut(&I::Item) -> bool,
    {
        Flu::new(self.iter.filter(predicate))
    }

    /// Take the first n elements
    ///
    /// # Examples
    ///
    /// ```
    /// use flu_core::FluExt;
    ///
    /// let result: Vec<_> = (0..100)
    ///     .flu()
    ///     .take(3)
    ///     .collect();
    ///
    /// assert_eq!(result, vec![0, 1, 2]);
    /// ```
    #[must_use]
    pub fn take(self, n: usize) -> Flu<impl Iterator<Item = I::Item>> {
        Flu::new(self.iter.take(n))
    }

    /// Skip the first n elements
    ///
    /// # Examples
    ///
    /// ```
    /// use flu_core::FluExt;
    ///
    /// let result: Vec<_> = (0..5)
    ///     .flu()
    ///     .skip(2)
    ///     .collect();
    ///
    /// assert_eq!(result, vec![2, 3, 4]);
    /// ```
    #[must_use]
    pub fn skip(self, n: usize) -> Flu<impl Iterator<Item = I::Item>> {
        Flu::new(self.iter.skip(n))
    }

    /// Take elements while predicate is true
    ///
    /// # Examples
    ///
    /// ```
    /// use flu_core::FluExt;
    ///
    /// let result: Vec<_> = vec![1, 2, 3, 4, 1, 2]
    ///     .into_iter()
    ///     .flu()
    ///     .take_while(|x| *x < 4)
    ///     .collect();
    ///
    /// assert_eq!(result, vec![1, 2, 3]);
    /// ```
    #[must_use]
    pub fn take_while<F>(self, predicate: F) -> Flu<impl Iterator<Item = I::Item>>
    where
        F: FnMut(&I::Item) -> bool,
    {
        Flu::new(self.iter.take_while(predicate))
    }

    /// Drop elements while predicate is true
    ///
    /// # Examples
    ///
    /// ```
    /// use flu_core::FluExt;
    ///
    /// let result: Vec<_> = vec![1, 2, 3, 4, 5]
    ///     .into_iter()
    ///     .flu()
    ///     .drop_while(|x| *x < 3)
    ///     .collect();
    ///
    /// assert_eq!(result, vec![3, 4, 5]);
    /// ```
    #[must_use]
    pub fn drop_while<F>(self, predicate: F) -> Flu<impl Iterator<Item = I::Item>>
    where
        F: FnMut(&I::Item) -> bool,
    {
        Flu::new(self.iter.skip_while(predicate))
    }

    /// Keep only unique elements (using `HashSet`)
    ///
    /// # Examples
    ///
    /// ```
    /// use flu_core::FluExt;
    ///
    /// let result: Vec<_> = vec![1, 2, 2, 3, 1, 4]
    ///     .into_iter()
    ///     .flu()
    ///     .unique()
    ///     .collect();
    ///
    /// assert_eq!(result, vec![1, 2, 3, 4]);
    /// ```
    #[must_use]
    pub fn unique(self) -> Flu<impl Iterator<Item = I::Item>>
    where
        I::Item: Eq + Hash + Clone,
    {
        let mut seen = HashSet::new();
        Flu::new(self.iter.filter(move |item| seen.insert(item.clone())))
    }

    // ========== Transformation Operations (lazy) ==========

    /// Transform each element
    ///
    /// # Examples
    ///
    /// ```
    /// use flu_core::FluExt;
    ///
    /// let result: Vec<_> = vec![1, 2, 3]
    ///     .into_iter()
    ///     .flu()
    ///     .map(|x| x * 2)
    ///     .collect();
    ///
    /// assert_eq!(result, vec![2, 4, 6]);
    /// ```
    #[must_use]
    pub fn map<F, B>(self, f: F) -> Flu<impl Iterator<Item = B>>
    where
        F: FnMut(I::Item) -> B,
    {
        Flu::new(self.iter.map(f))
    }

    /// Add index to each element
    ///
    /// # Examples
    ///
    /// ```
    /// use flu_core::FluExt;
    ///
    /// let result: Vec<_> = vec!["a", "b", "c"]
    ///     .into_iter()
    ///     .flu()
    ///     .enumerate()
    ///     .collect();
    ///
    /// assert_eq!(result, vec![(0, "a"), (1, "b"), (2, "c")]);
    /// ```
    #[must_use]
    pub fn enumerate(self) -> Flu<impl Iterator<Item = (usize, I::Item)>> {
        Flu::new(self.iter.enumerate())
    }

    /// Zip with another iterator
    ///
    /// # Examples
    ///
    /// ```
    /// use flu_core::FluExt;
    ///
    /// let result: Vec<_> = vec![1, 2, 3]
    ///     .into_iter()
    ///     .flu()
    ///     .zip(vec!["a", "b", "c"])
    ///     .collect();
    ///
    /// assert_eq!(result, vec![(1, "a"), (2, "b"), (3, "c")]);
    /// ```
    #[must_use]
    pub fn zip<J>(self, other: J) -> Flu<impl Iterator<Item = (I::Item, J::Item)>>
    where
        J: IntoIterator,
    {
        Flu::new(self.iter.zip(other))
    }

    /// Flatten nested iterators
    ///
    /// # Examples
    ///
    /// ```
    /// use flu_core::FluExt;
    ///
    /// let result: Vec<_> = vec![vec![1, 2], vec![3, 4]]
    ///     .into_iter()
    ///     .flu()
    ///     .flatten()
    ///     .collect();
    ///
    /// assert_eq!(result, vec![1, 2, 3, 4]);
    /// ```
    #[must_use]
    pub fn flatten<T>(self) -> Flu<impl Iterator<Item = T>>
    where
        I::Item: IntoIterator<Item = T>,
    {
        Flu::new(self.iter.flatten())
    }

    // ========== Grouping Operations ==========

    /// Group elements into chunks of size n
    ///
    /// # Examples
    ///
    /// ```
    /// use flu_core::FluExt;
    ///
    /// let result: Vec<_> = (0..5)
    ///     .flu()
    ///     .chunk(2)
    ///     .collect();
    ///
    /// assert_eq!(result, vec![vec![0, 1], vec![2, 3], vec![4]]);
    /// ```
    #[must_use]
    pub fn chunk(self, n: usize) -> Flu<impl Iterator<Item = Vec<I::Item>>> {
        Flu::new(ChunkIterator::new(self.iter, n))
    }

    /// Create sliding windows of size n
    ///
    /// # Examples
    ///
    /// ```
    /// use flu_core::FluExt;
    ///
    /// let result: Vec<_> = (1..=4)
    ///     .flu()
    ///     .window(2)
    ///     .collect();
    ///
    /// assert_eq!(result, vec![vec![1, 2], vec![2, 3], vec![3, 4]]);
    /// ```
    #[must_use]
    pub fn window(self, n: usize) -> Flu<impl Iterator<Item = Vec<I::Item>>>
    where
        I::Item: Clone,
    {
        Flu::new(WindowIterator::new(self.iter, n))
    }

    /// Group elements by a key function
    ///
    /// # Examples
    ///
    /// ```
    /// use flu_core::FluExt;
    ///
    /// let result: Vec<_> = vec![1, 2, 3, 4, 5, 6]
    ///     .into_iter()
    ///     .flu()
    ///     .group_by(|x| x % 2)
    ///     .collect();
    ///
    /// // Result contains (key, group) pairs
    /// assert_eq!(result.len(), 2);
    /// ```
    #[must_use]
    pub fn group_by<K, F>(self, key_fn: F) -> Flu<impl Iterator<Item = (K, Vec<I::Item>)>>
    where
        K: Eq + Hash,
        F: FnMut(&I::Item) -> K,
    {
        Flu::new(GroupByCollectIterator::new(self.iter, key_fn))
    }

    // ========== Join Operations ==========

    /// Inner join with another iterator based on key functions
    ///
    /// # Examples
    ///
    /// ```
    /// use flu_core::FluExt;
    ///
    /// let left = vec![(1, "a"), (2, "b"), (3, "c")];
    /// let right = vec![(1, "x"), (2, "y"), (4, "z")];
    ///
    /// let result: Vec<_> = left
    ///     .into_iter()
    ///     .flu()
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
    ) -> Flu<impl Iterator<Item = (I::Item, J::Item)>>
    where
        I::Item: Clone,
        J: IntoIterator,
        J::Item: Clone,
        K: Eq + Hash,
        FL: Fn(&I::Item) -> K,
        FR: Fn(&J::Item) -> K,
    {
        Flu::new(InnerJoinIterator::new(
            self.iter, other, left_key, right_key,
        ))
    }

    /// Left join with another iterator based on key functions
    ///
    /// # Examples
    ///
    /// ```
    /// use flu_core::FluExt;
    ///
    /// let left = vec![(1, "a"), (2, "b"), (3, "c")];
    /// let right = vec![(1, "x"), (2, "y")];
    ///
    /// let result: Vec<_> = left
    ///     .into_iter()
    ///     .flu()
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
    ) -> Flu<impl Iterator<Item = (I::Item, Option<J::Item>)>>
    where
        I::Item: Clone,
        J: IntoIterator,
        J::Item: Clone,
        K: Eq + Hash,
        FL: Fn(&I::Item) -> K,
        FR: Fn(&J::Item) -> K,
    {
        Flu::new(LeftJoinIterator::new(self.iter, other, left_key, right_key))
    }

    // ========== Terminal Operations (consume iterator) ==========

    /// Collect into a collection
    ///
    /// # Examples
    ///
    /// ```
    /// use flu_core::FluExt;
    ///
    /// let result: Vec<_> = (0..5)
    ///     .flu()
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
    /// use flu_core::FluExt;
    ///
    /// let count = (0..10)
    ///     .flu()
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
    /// use flu_core::FluExt;
    ///
    /// let sum = (1..=5).flu().sum::<i32>();
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
    /// use flu_core::FluExt;
    ///
    /// let min = vec![3, 1, 4, 1, 5].into_iter().flu().min();
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
    /// use flu_core::FluExt;
    ///
    /// let max = vec![3, 1, 4, 1, 5].into_iter().flu().max();
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
    /// use flu_core::FluExt;
    ///
    /// let first = (1..10).flu().first();
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
    /// use flu_core::FluExt;
    ///
    /// let last = (1..10).flu().last();
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
    /// use flu_core::FluExt;
    ///
    /// let product = (1..=5).flu().reduce(|a, b| a * b);
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
    /// use flu_core::FluExt;
    ///
    /// let sum = (1..=5).flu().fold(0, |a, b| a + b);
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
    /// use flu_core::FluExt;
    ///
    /// let list = (0..5).flu().to_list();
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
    /// use flu_core::FluExt;
    ///
    /// let has_even = (1..10).flu().any(|x| x % 2 == 0);
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
    /// use flu_core::FluExt;
    ///
    /// let all_positive = (1..10).flu().all(|x| x > 0);
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

/// Extension trait to add `.flu()` method to all iterators
///
/// # Examples
///
/// ```
/// use flu_core::FluExt;
///
/// let result: Vec<_> = vec![1, 2, 3, 4, 5]
///     .into_iter()
///     .flu()  // Convert to Flu<I>
///     .filter(|x| x % 2 == 0)
///     .collect();
///
/// assert_eq!(result, vec![2, 4]);
/// ```
pub trait FluExt: Iterator + Sized {
    /// Convert an iterator into a `Flu` wrapper
    fn flu(self) -> Flu<Self> {
        Flu::new(self)
    }
}

impl<I: Iterator> FluExt for I {}

/// Implement `IntoIterator` for Flu to allow using it in for loops
impl<I: Iterator> IntoIterator for Flu<I> {
    type Item = I::Item;
    type IntoIter = I;

    fn into_iter(self) -> Self::IntoIter {
        self.iter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_filter() {
        let result: Vec<_> = (0..10).flu().filter(|x| x % 2 == 0).collect();
        assert_eq!(result, vec![0, 2, 4, 6, 8]);
    }

    #[test]
    fn chained_operations() {
        let result: Vec<_> = (0..10)
            .flu()
            .filter(|x| x % 2 == 0)
            .map(|x| x * 2)
            .take(3)
            .collect();
        assert_eq!(result, vec![0, 4, 8]);
    }

    #[test]
    fn terminal_operations() {
        assert_eq!((1..=5).flu().sum::<i32>(), 15);
        assert_eq!((1..=5).flu().count(), 5);
        assert_eq!((1..=5).flu().min(), Some(1));
        assert_eq!((1..=5).flu().max(), Some(5));
    }
}
