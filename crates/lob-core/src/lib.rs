//! Core iterator library for lob data pipelines
//!
//! This crate provides the `Lob<I>` wrapper type that enables fluent, chainable
//! operations on iterators with lazy evaluation.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod fluent;
mod grouping;
mod joins;

pub use fluent::{Lob, LobExt};

// Re-export commonly used types
pub use std::collections::{HashMap, HashSet};
