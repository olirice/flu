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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_from_str() {
        assert_eq!(OutputFormat::from_str("debug"), Some(OutputFormat::Debug));
        assert_eq!(OutputFormat::from_str("json"), Some(OutputFormat::Json));
        assert_eq!(
            OutputFormat::from_str("jsonl"),
            Some(OutputFormat::JsonLines)
        );
        assert_eq!(
            OutputFormat::from_str("jsonlines"),
            Some(OutputFormat::JsonLines)
        );
        assert_eq!(OutputFormat::from_str("csv"), Some(OutputFormat::Csv));
        assert_eq!(OutputFormat::from_str("table"), Some(OutputFormat::Table));
        assert_eq!(OutputFormat::from_str("invalid"), None);
    }

    #[test]
    fn test_output_format_default() {
        assert_eq!(OutputFormat::default(true), OutputFormat::Debug);
        assert_eq!(OutputFormat::default(false), OutputFormat::JsonLines);
    }
}
