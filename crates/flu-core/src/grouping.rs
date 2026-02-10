//! Grouping iterators: `chunk`, `window`, `group_by`

#![allow(clippy::missing_const_for_fn)]

use std::collections::HashMap;
use std::hash::Hash;

/// Iterator that groups elements into chunks of size n
pub struct ChunkIterator<I: Iterator> {
    iter: I,
    chunk_size: usize,
}

impl<I: Iterator> ChunkIterator<I> {
    pub fn new(iter: I, chunk_size: usize) -> Self {
        assert!(chunk_size > 0, "chunk size must be greater than 0");
        Self { iter, chunk_size }
    }
}

impl<I: Iterator> Iterator for ChunkIterator<I> {
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut chunk = Vec::with_capacity(self.chunk_size);

        for _ in 0..self.chunk_size {
            match self.iter.next() {
                Some(item) => chunk.push(item),
                None => break,
            }
        }

        if chunk.is_empty() {
            None
        } else {
            Some(chunk)
        }
    }
}

/// Iterator that creates sliding windows of size n
pub struct WindowIterator<I: Iterator> {
    iter: I,
    window_size: usize,
    buffer: Vec<I::Item>,
    started: bool,
}

impl<I: Iterator> WindowIterator<I>
where
    I::Item: Clone,
{
    pub fn new(iter: I, window_size: usize) -> Self {
        assert!(window_size > 0, "window size must be greater than 0");
        Self {
            iter,
            window_size,
            buffer: Vec::with_capacity(window_size),
            started: false,
        }
    }
}

impl<I: Iterator> Iterator for WindowIterator<I>
where
    I::Item: Clone,
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.started {
            // Fill initial buffer
            for _ in 0..self.window_size {
                match self.iter.next() {
                    Some(item) => self.buffer.push(item),
                    None => break,
                }
            }
            self.started = true;

            if self.buffer.len() == self.window_size {
                return Some(self.buffer.clone());
            }
            return None;
        }

        // Slide window: remove first, add new
        match self.iter.next() {
            Some(item) => {
                self.buffer.remove(0);
                self.buffer.push(item);
                Some(self.buffer.clone())
            }
            None => None,
        }
    }
}

/// Iterator that groups consecutive elements by a key function
#[allow(dead_code)]
pub struct GroupByIterator<I, K, F>
where
    I: Iterator,
    K: Eq + Hash,
    F: FnMut(&I::Item) -> K,
{
    iter: I,
    key_fn: F,
    done: bool,
}

impl<I, K, F> GroupByIterator<I, K, F>
where
    I: Iterator,
    K: Eq + Hash,
    F: FnMut(&I::Item) -> K,
{
    #[allow(dead_code)]
    pub fn new(iter: I, key_fn: F) -> Self {
        Self {
            iter,
            key_fn,
            done: false,
        }
    }
}

impl<I, K, F> Iterator for GroupByIterator<I, K, F>
where
    I: Iterator,
    K: Eq + Hash,
    F: FnMut(&I::Item) -> K,
{
    type Item = (K, Vec<I::Item>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        // Collect all items into groups
        let mut groups: HashMap<K, Vec<I::Item>> = HashMap::new();

        for item in &mut self.iter {
            let key = (self.key_fn)(&item);
            groups.entry(key).or_default().push(item);
        }

        self.done = true;

        // Return first group (deterministic iteration order not guaranteed)
        groups.into_iter().next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.done {
            (0, Some(0))
        } else {
            (0, None)
        }
    }
}

/// Specialized `group_by` that returns all groups at once (more useful in practice)
pub struct GroupByCollectIterator<I, K, F>
where
    I: Iterator,
    K: Eq + Hash,
    F: FnMut(&I::Item) -> K,
{
    groups: Option<std::collections::hash_map::IntoIter<K, Vec<I::Item>>>,
    iter: Option<I>,
    key_fn: Option<F>,
}

impl<I, K, F> GroupByCollectIterator<I, K, F>
where
    I: Iterator,
    K: Eq + Hash,
    F: FnMut(&I::Item) -> K,
{
    pub fn new(iter: I, key_fn: F) -> Self {
        Self {
            groups: None,
            iter: Some(iter),
            key_fn: Some(key_fn),
        }
    }
}

impl<I, K, F> Iterator for GroupByCollectIterator<I, K, F>
where
    I: Iterator,
    K: Eq + Hash,
    F: FnMut(&I::Item) -> K,
{
    type Item = (K, Vec<I::Item>);

    fn next(&mut self) -> Option<Self::Item> {
        // Lazy initialization: collect groups on first call
        if self.groups.is_none() {
            let mut groups: HashMap<K, Vec<I::Item>> = HashMap::new();
            let mut key_fn = self.key_fn.take().expect("key_fn should be Some");
            let iter = self.iter.take().expect("iter should be Some");

            for item in iter {
                let key = key_fn(&item);
                groups.entry(key).or_default().push(item);
            }

            self.groups = Some(groups.into_iter());
        }

        // Iterate through groups
        self.groups.as_mut().and_then(std::iter::Iterator::next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_basic() {
        let data = vec![1, 2, 3, 4, 5];
        let chunks: Vec<_> = ChunkIterator::new(data.into_iter(), 2).collect();
        assert_eq!(chunks, vec![vec![1, 2], vec![3, 4], vec![5]]);
    }

    #[test]
    fn chunk_exact() {
        let data = vec![1, 2, 3, 4];
        let chunks: Vec<_> = ChunkIterator::new(data.into_iter(), 2).collect();
        assert_eq!(chunks, vec![vec![1, 2], vec![3, 4]]);
    }

    #[test]
    fn chunk_empty() {
        let data: Vec<i32> = vec![];
        let chunks: Vec<_> = ChunkIterator::new(data.into_iter(), 2).collect();
        assert_eq!(chunks, Vec::<Vec<i32>>::new());
    }

    #[test]
    fn window_basic() {
        let data = vec![1, 2, 3, 4, 5];
        let windows: Vec<_> = WindowIterator::new(data.into_iter(), 3).collect();
        assert_eq!(windows, vec![vec![1, 2, 3], vec![2, 3, 4], vec![3, 4, 5]]);
    }

    #[test]
    fn window_size_2() {
        let data = vec![1, 2, 3, 4];
        let windows: Vec<_> = WindowIterator::new(data.into_iter(), 2).collect();
        assert_eq!(windows, vec![vec![1, 2], vec![2, 3], vec![3, 4]]);
    }

    #[test]
    fn window_too_small() {
        let data = vec![1, 2];
        let windows: Vec<_> = WindowIterator::new(data.into_iter(), 3).collect();
        assert_eq!(windows, Vec::<Vec<i32>>::new());
    }

    #[test]
    fn group_by_basic() {
        let data = vec![1, 2, 3, 4, 5, 6];
        let mut groups: Vec<_> = GroupByCollectIterator::new(data.into_iter(), |x| x % 2).collect();

        // Sort for deterministic testing
        groups.sort_by_key(|(k, _)| *k);

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].0, 0); // even
        assert_eq!(groups[0].1, vec![2, 4, 6]);
        assert_eq!(groups[1].0, 1); // odd
        assert_eq!(groups[1].1, vec![1, 3, 5]);
    }
}
