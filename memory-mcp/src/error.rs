//! Error types for MCP operations

use thiserror::Error;

/// Error type for MCP operations
#[derive(Debug, Error)]
pub enum Error {
    /// Code execution error
    #[error("Execution error: {0}")]
    Execution(String),

    /// Sandbox configuration error
    #[error("Sandbox configuration error: {0}")]
    Configuration(String),

    /// Tool error
    #[error("Tool error: {0}")]
    Tool(String),

    /// Security violation
    #[error("Security violation: {0}")]
    Security(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// General error
    #[error("{0}")]
    General(String),
}

/// Result type for MCP operations
pub type Result<T> = std::result::Result<T, Error>;

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::General(err.to_string())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::General(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::General(s.to_string())
    }
}
