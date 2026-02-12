//! Input source handling (files and stdin)

use crate::error::{LobError, Result};
use std::path::PathBuf;

/// Input format for parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputFormat {
    /// Plain text lines
    Lines,
    /// CSV with headers
    Csv,
    /// TSV with headers
    Tsv,
    /// JSON lines (one JSON object per line)
    JsonLines,
}

/// Input source configuration
#[derive(Debug, Clone)]
pub struct InputSource {
    /// Files to read (empty = stdin)
    pub files: Vec<PathBuf>,
    /// Input format
    pub format: InputFormat,
}

impl InputSource {
    /// Create new input source from files
    pub fn new(files: Vec<PathBuf>, format: InputFormat) -> Self {
        Self { files, format }
    }

    /// Check if reading from stdin
    pub fn is_stdin(&self) -> bool {
        self.files.is_empty()
    }

    /// Validate that files exist
    pub fn validate(&self) -> Result<()> {
        for file in &self.files {
            if !file.exists() {
                return Err(LobError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("File not found: {}", file.display()),
                )));
            }
        }
        Ok(())
    }
}
