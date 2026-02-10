//! Error types for flu CLI

use thiserror::Error;

/// Errors that can occur during flu execution
#[derive(Error, Debug)]
pub enum FluError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Compilation error
    #[error("Compilation failed:\n{0}")]
    Compilation(String),

    /// Cache error
    #[error("Cache error: {0}")]
    Cache(String),

    /// Code generation error
    #[allow(dead_code)]
    #[error("Code generation error: {0}")]
    CodeGen(String),

    /// Toolchain error
    #[error("Toolchain error: {0}")]
    Toolchain(String),

    /// Invalid expression
    #[error("Invalid expression: {0}")]
    InvalidExpression(String),
}

/// Result type for flu operations
pub type Result<T> = std::result::Result<T, FluError>;
