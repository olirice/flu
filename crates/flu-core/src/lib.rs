//! Core iterator library for flu data pipelines
//!
//! This crate provides the `Flu<I>` wrapper type that enables fluent, chainable
//! operations on iterators with lazy evaluation.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod fluent;
mod grouping;
mod joins;
mod selection;
mod terminal;
mod transformation;

pub use fluent::{Flu, FluExt};

// Re-export commonly used types
pub use std::collections::{HashMap, HashSet};
