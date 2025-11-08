use uuid::Uuid;

/// Result type alias for memory operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for the self-learning memory system
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Learning error: {0}")]
    Learning(String),

    #[error("MCP error: {0}")]
    MCP(String),

    #[error("Episode not found: {0}")]
    NotFound(Uuid),

    #[error("Pattern error: {0}")]
    Pattern(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Execution timeout")]
    ExecutionTimeout,

    #[error("Circuit breaker open")]
    CircuitBreakerOpen,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl Error {
    /// Check if this error is recoverable (can retry with backoff)
    pub fn is_recoverable(&self) -> bool {
        match self {
            Error::Storage(_) => true,
            Error::Learning(_) => false,
            Error::MCP(_) => true,
            Error::NotFound(_) => false,
            Error::Pattern(_) => false,
            Error::Serialization(_) => false,
            Error::ExecutionTimeout => true,
            Error::CircuitBreakerOpen => true,
            Error::InvalidInput(_) => false,
            Error::InvalidState(_) => false,
            Error::Io(_) => true,
        }
    }
}
