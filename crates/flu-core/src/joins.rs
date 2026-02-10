//! Join operations: inner join, left join

use std::collections::HashMap;
use std::hash::Hash;

/// Inner join iterator
pub struct InnerJoinIterator<I, J, K, FL, FR>
where
    I: Iterator,
    J: IntoIterator,
    K: Eq + Hash,
    FL: Fn(&I::Item) -> K,
    FR: Fn(&J::Item) -> K,
{
    left: I,
    right_map: HashMap<K, Vec<J::Item>>,
    left_key: FL,
    current_left: Option<I::Item>,
    current_right_idx: usize,
    _right_key: std::marker::PhantomData<FR>,
}

impl<I, J, K, FL, FR> InnerJoinIterator<I, J, K, FL, FR>
where
    I: Iterator,
    J: IntoIterator,
    J::Item: Clone,
    K: Eq + Hash,
    FL: Fn(&I::Item) -> K,
    FR: Fn(&J::Item) -> K,
{
    pub fn new(left: I, right: J, left_key: FL, right_key: FR) -> Self {
        // Build hash map from right side
        let mut right_map: HashMap<K, Vec<J::Item>> = HashMap::new();
        for item in right {
            let key = right_key(&item);
            right_map.entry(key).or_default().push(item);
        }

        Self {
            left,
            right_map,
            left_key,
            current_left: None,
            current_right_idx: 0,
            _right_key: std::marker::PhantomData,
        }
    }
}

impl<I, J, K, FL, FR> Iterator for InnerJoinIterator<I, J, K, FL, FR>
where
    I: Iterator,
    I::Item: Clone,
    J: IntoIterator,
    J::Item: Clone,
    K: Eq + Hash,
    FL: Fn(&I::Item) -> K,
    FR: Fn(&J::Item) -> K,
{
    type Item = (I::Item, J::Item);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If we have a current left item, try to pair it with right items
            if let Some(left_item) = &self.current_left {
                let key = (self.left_key)(left_item);

                if let Some(right_items) = self.right_map.get(&key) {
                    if self.current_right_idx < right_items.len() {
                        let result = (
                            self.current_left.take().unwrap(),
                            right_items[self.current_right_idx].clone(),
                        );
                        self.current_right_idx += 1;

                        // Re-borrow left item if more right items remain
                        if self.current_right_idx < right_items.len() {
                            self.current_left = Some(result.0.clone());
                        }

                        return Some(result);
                    }
                }

                // No (more) matches for current left item, move to next
                self.current_left = None;
                self.current_right_idx = 0;
            }

            // Get next left item
            match self.left.next() {
                Some(left_item) => {
                    self.current_left = Some(left_item);
                    self.current_right_idx = 0;
                }
                None => return None,
            }
        }
    }
}

/// Left join iterator
pub struct LeftJoinIterator<I, J, K, FL, FR>
where
    I: Iterator,
    J: IntoIterator,
    K: Eq + Hash,
    FL: Fn(&I::Item) -> K,
    FR: Fn(&J::Item) -> K,
{
    left: I,
    right_map: HashMap<K, Vec<J::Item>>,
    left_key: FL,
    current_left: Option<I::Item>,
    current_right_idx: usize,
    emitted_current: bool,
    _right_key: std::marker::PhantomData<FR>,
}

impl<I, J, K, FL, FR> LeftJoinIterator<I, J, K, FL, FR>
where
    I: Iterator,
    J: IntoIterator,
    J::Item: Clone,
    K: Eq + Hash,
    FL: Fn(&I::Item) -> K,
    FR: Fn(&J::Item) -> K,
{
    pub fn new(left: I, right: J, left_key: FL, right_key: FR) -> Self {
        // Build hash map from right side
        let mut right_map: HashMap<K, Vec<J::Item>> = HashMap::new();
        for item in right {
            let key = right_key(&item);
            right_map.entry(key).or_default().push(item);
        }

        Self {
            left,
            right_map,
            left_key,
            current_left: None,
            current_right_idx: 0,
            emitted_current: false,
            _right_key: std::marker::PhantomData,
        }
    }
}

impl<I, J, K, FL, FR> Iterator for LeftJoinIterator<I, J, K, FL, FR>
where
    I: Iterator,
    I::Item: Clone,
    J: IntoIterator,
    J::Item: Clone,
    K: Eq + Hash,
    FL: Fn(&I::Item) -> K,
    FR: Fn(&J::Item) -> K,
{
    type Item = (I::Item, Option<J::Item>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If we have a current left item, try to pair it with right items
            if let Some(left_item) = &self.current_left {
                let key = (self.left_key)(left_item);

                if let Some(right_items) = self.right_map.get(&key) {
                    if self.current_right_idx < right_items.len() {
                        let result = (
                            self.current_left.take().unwrap(),
                            Some(right_items[self.current_right_idx].clone()),
                        );
                        self.current_right_idx += 1;
                        self.emitted_current = true;

                        // Re-borrow left item if more right items remain
                        if self.current_right_idx < right_items.len() {
                            self.current_left = Some(result.0.clone());
                        }

                        return Some(result);
                    }
                }

                // No matches for current left item - emit with None if not emitted yet
                if !self.emitted_current {
                    self.emitted_current = true;
                    return Some((self.current_left.take().unwrap(), None));
                }

                // Move to next left item
                self.current_left = None;
                self.current_right_idx = 0;
                self.emitted_current = false;
            }

            // Get next left item
            match self.left.next() {
                Some(left_item) => {
                    self.current_left = Some(left_item);
                    self.current_right_idx = 0;
                    self.emitted_current = false;
                }
                None => return None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inner_join_basic() {
        let left = vec![(1, "a"), (2, "b"), (3, "c")];
        let right = vec![(1, "x"), (2, "y"), (4, "z")];

        let mut result: Vec<_> =
            InnerJoinIterator::new(left.into_iter(), right, |x| x.0, |x| x.0).collect();
        result.sort_unstable();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ((1, "a"), (1, "x")));
        assert_eq!(result[1], ((2, "b"), (2, "y")));
    }

    #[test]
    fn inner_join_no_matches() {
        let left = vec![(1, "a"), (2, "b")];
        let right = vec![(3, "x"), (4, "y")];

        let count = InnerJoinIterator::new(left.into_iter(), right, |x| x.0, |x| x.0).count();

        assert_eq!(count, 0);
    }

    #[test]
    fn inner_join_multiple_matches() {
        let left = vec![(1, "a"), (1, "b")];
        let right = vec![(1, "x"), (1, "y")];

        let count = InnerJoinIterator::new(left.into_iter(), right, |x| x.0, |x| x.0).count();

        // 2 left * 2 right = 4 results
        assert_eq!(count, 4);
    }

    #[test]
    fn left_join_basic() {
        let left = vec![(1, "a"), (2, "b"), (3, "c")];
        let right = vec![(1, "x"), (2, "y")];

        let result: Vec<_> =
            LeftJoinIterator::new(left.into_iter(), right, |x| x.0, |x| x.0).collect();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], ((1, "a"), Some((1, "x"))));
        assert_eq!(result[1], ((2, "b"), Some((2, "y"))));
        assert_eq!(result[2], ((3, "c"), None));
    }

    #[test]
    fn left_join_all_match() {
        let left = vec![(1, "a"), (2, "b")];
        let right = vec![(1, "x"), (2, "y")];

        let result: Vec<_> =
            LeftJoinIterator::new(left.into_iter(), right, |x| x.0, |x| x.0).collect();

        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|(_, r)| r.is_some()));
    }

    #[test]
    fn left_join_no_matches() {
        let left = vec![(1, "a"), (2, "b")];
        let right = vec![(3, "x"), (4, "y")];

        let result: Vec<_> =
            LeftJoinIterator::new(left.into_iter(), right, |x| x.0, |x| x.0).collect();

        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|(_, r)| r.is_none()));
    }
}
