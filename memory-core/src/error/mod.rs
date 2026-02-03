use uuid::Uuid;

pub mod cache;
pub mod relationship;

pub use cache::CacheError;
pub use relationship::RelationshipError;

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

    /// Relationship operation error with detailed context.
    #[error("{0}")]
    Relationship(#[from] RelationshipError),

    /// Cache operation error with detailed context.
    #[error("{0}")]
    Cache(#[from] CacheError),
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
            // Relationship errors - generally non-recoverable
            Error::Relationship(rel_err) => {
                matches!(rel_err, RelationshipError::ValidationFailed { .. })
            }
            // Cache errors - some are recoverable
            Error::Cache(cache_err) => matches!(
                cache_err,
                CacheError::EvictionFailed { .. } | CacheError::SerializationFailed { .. }
            ),
        }
    }

    /// Check if this is a relationship error
    #[must_use]
    pub fn is_relationship_error(&self) -> bool {
        matches!(self, Error::Relationship(_))
    }

    /// Check if this is a cache error
    #[must_use]
    pub fn is_cache_error(&self) -> bool {
        matches!(self, Error::Cache(_))
    }

    /// Get the relationship error if this is one
    #[must_use]
    pub fn as_relationship_error(&self) -> Option<&RelationshipError> {
        match self {
            Error::Relationship(e) => Some(e),
            _ => None,
        }
    }

    /// Get the cache error if this is one
    #[must_use]
    pub fn as_cache_error(&self) -> Option<&CacheError> {
        match self {
            Error::Cache(e) => Some(e),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_from_relationship_error() {
        let rel_err = RelationshipError::SelfReference {
            episode_id: Uuid::new_v4(),
        };
        let err: Error = rel_err.into();
        assert!(err.is_relationship_error());
        assert!(!err.is_recoverable());
    }

    #[test]
    fn test_error_from_cache_error() {
        let cache_err = CacheError::EntryTooLarge {
            key: "test".to_string(),
            size: 100,
            max_size: 50,
        };
        let err: Error = cache_err.into();
        assert!(err.is_cache_error());
        assert!(err.is_recoverable());
    }

    #[test]
    fn test_as_relationship_error() {
        let rel_err = RelationshipError::NotFound {
            relationship_id: Uuid::new_v4(),
        };
        let err = Error::Relationship(rel_err.clone());

        assert!(err.as_relationship_error().is_some());
        assert_eq!(err.as_relationship_error(), Some(&rel_err));

        let other_err = Error::NotFound(Uuid::new_v4());
        assert!(other_err.as_relationship_error().is_none());
    }

    #[test]
    fn test_as_cache_error() {
        let cache_err = CacheError::EvictionFailed {
            reason: "test".to_string(),
        };
        let err = Error::Cache(cache_err.clone());

        assert!(err.as_cache_error().is_some());
        assert_eq!(err.as_cache_error(), Some(&cache_err));

        let other_err = Error::Storage("test".to_string());
        assert!(other_err.as_cache_error().is_none());
    }

    #[test]
    fn test_recoverable_cache_errors() {
        let eviction_err = Error::Cache(CacheError::EvictionFailed {
            reason: "test".to_string(),
        });
        assert!(eviction_err.is_recoverable());

        let serialization_err = Error::Cache(CacheError::SerializationFailed {
            key: "key".to_string(),
            error: "test".to_string(),
        });
        assert!(serialization_err.is_recoverable());

        let config_err = Error::Cache(CacheError::InvalidConfiguration {
            field: "test".to_string(),
            value: "invalid".to_string(),
        });
        assert!(!config_err.is_recoverable());
    }

    #[test]
    fn test_recoverable_relationship_errors() {
        let validation_err = Error::Relationship(RelationshipError::ValidationFailed {
            reason: "test".to_string(),
        });
        assert!(validation_err.is_recoverable());

        let self_ref_err = Error::Relationship(RelationshipError::SelfReference {
            episode_id: Uuid::new_v4(),
        });
        assert!(!self_ref_err.is_recoverable());
    }

    #[test]
    fn test_error_display() {
        let rel_err = RelationshipError::SelfReference {
            episode_id: Uuid::new_v4(),
        };
        let err = Error::Relationship(rel_err);
        let msg = err.to_string();
        assert!(msg.contains("self-referencing"));
    }
}
