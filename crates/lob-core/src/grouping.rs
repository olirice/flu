//! Grouping iterators: `chunk`, `window`, `group_by`

#![allow(clippy::missing_const_for_fn)]

use std::collections::{HashMap, VecDeque};
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
    buffer: VecDeque<I::Item>,
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
            buffer: VecDeque::with_capacity(window_size),
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
                    Some(item) => self.buffer.push_back(item),
                    None => break,
                }
            }
            self.started = true;

            if self.buffer.len() == self.window_size {
                return Some(self.buffer.iter().cloned().collect());
            }
            return None;
        }

        // Slide window: remove first, add new
        match self.iter.next() {
            Some(item) => {
                self.buffer.pop_front();
                self.buffer.push_back(item);
                Some(self.buffer.iter().cloned().collect())
            }
            None => None,
        }
    }
}

/// Specialized `group_by` that returns all groups at once
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
