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
