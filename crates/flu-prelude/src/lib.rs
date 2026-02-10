//! User-facing prelude for flu data pipelines
//!
//! This crate provides the public API that users interact with in their
//! generated code. It re-exports the core functionality and adds convenient
//! helpers like `input()` for reading from stdin.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::io::{self, BufRead};

// Re-export core types and traits
pub use flu_core::{Flu, FluExt, HashMap, HashSet};

/// Creates a Flu iterator from stdin lines
///
/// This function reads lines from stdin and returns a `Flu` iterator over them.
/// Lines are trimmed and empty lines are filtered out by default.
///
/// # Examples
///
/// ```no_run
/// use flu_prelude::*;
///
/// // Read lines from stdin and filter
/// let result: Vec<_> = input()
///     .filter(|line| line.contains("ERROR"))
///     .collect();
/// ```
#[must_use]
pub fn input() -> Flu<impl Iterator<Item = String>> {
    let stdin = io::stdin();
    #[allow(clippy::lines_filter_map_ok)]
    Flu::new(
        stdin
            .lock()
            .lines()
            .filter_map(Result::ok)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
    )
}

/// Creates a Flu iterator from any iterable
///
/// This is a convenience function to convert any type that implements
/// `IntoIterator` into a `Flu` iterator.
///
/// # Examples
///
/// ```
/// use flu_prelude::*;
///
/// let result: Vec<_> = flu(vec![1, 2, 3, 4, 5])
///     .filter(|x| x % 2 == 0)
///     .collect();
///
/// assert_eq!(result, vec![2, 4]);
/// ```
#[must_use]
pub fn flu<I: IntoIterator>(iterable: I) -> Flu<I::IntoIter> {
    Flu::new(iterable.into_iter())
}

/// Creates a Flu iterator from a range
///
/// # Examples
///
/// ```
/// use flu_prelude::*;
///
/// let result: Vec<_> = range(0, 5)
///     .map(|x| x * 2)
///     .collect();
///
/// assert_eq!(result, vec![0, 2, 4, 6, 8]);
/// ```
#[must_use]
pub fn range(start: i64, end: i64) -> Flu<impl Iterator<Item = i64>> {
    Flu::new(start..end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flu_from_vec() {
        let result: Vec<_> = flu(vec![1, 2, 3, 4, 5]).filter(|x| x % 2 == 0).collect();
        assert_eq!(result, vec![2, 4]);
    }

    #[test]
    fn range_basic() {
        let result: Vec<_> = range(0, 5).collect();
        assert_eq!(result, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn chained_operations() {
        let result: Vec<_> = flu(vec![1, 2, 3, 4, 5])
            .filter(|x| x % 2 == 0)
            .map(|x| x * 2)
            .take(2)
            .collect();
        assert_eq!(result, vec![4, 8]);
    }
}
