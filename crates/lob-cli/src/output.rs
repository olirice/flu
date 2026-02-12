//! Output format handling

use std::io::{stdout, IsTerminal};

/// Output format for results
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Rust debug format (current default)
    Debug,
    /// JSON array
    Json,
    /// JSON lines (newline-delimited)
    JsonLines,
    /// CSV (requires CSV input)
    Csv,
    /// Table (requires CSV/JSON input)
    Table,
}

impl OutputFormat {
    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "debug" => Some(Self::Debug),
            "json" => Some(Self::Json),
            "jsonl" | "jsonlines" => Some(Self::JsonLines),
            "csv" => Some(Self::Csv),
            "table" => Some(Self::Table),
            _ => None,
        }
    }

    /// Get default format based on context
    pub fn default(is_terminal: bool) -> Self {
        if is_terminal {
            Self::Debug
        } else {
            Self::JsonLines
        }
    }
}

/// Detect if stdout is a terminal
pub fn is_terminal() -> bool {
    stdout().is_terminal()
}
