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

    #[error("Security validation failed: {0}")]
    Security(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Quota exceeded: {0}")]
    QuotaExceeded(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Embedding error: {0}")]
    Embedding(#[from] anyhow::Error),
}

impl Error {
    /// Check if this error is recoverable (can retry with backoff)
    #[must_use]
    pub fn is_recoverable(&self) -> bool {
        match self {
            // Recoverable errors (can retry with backoff)
            Error::Storage(_)
            | Error::MCP(_)
            | Error::ExecutionTimeout
            | Error::CircuitBreakerOpen
            | Error::RateLimitExceeded(_)
            | Error::Io(_)
            | Error::Embedding(_) => true,
            // Non-recoverable errors
            Error::Learning(_)
            | Error::NotFound(_)
            | Error::Pattern(_)
            | Error::Serialization(_)
            | Error::InvalidInput(_)
            | Error::InvalidState(_)
            | Error::Security(_)
            | Error::ValidationFailed(_)
            | Error::QuotaExceeded(_)
            | Error::Configuration(_) => false,
        }
    }
}
