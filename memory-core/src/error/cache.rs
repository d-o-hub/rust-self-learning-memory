//! Error types for cache operations.
//!
//! This module defines specific error types for cache management,
//! providing detailed context for failures during cache operations.

use std::fmt;

/// Errors that can occur during cache operations.
#[derive(Debug, Clone, PartialEq)]
pub enum CacheError {
    /// Cache entry exceeds maximum allowed size.
    EntryTooLarge {
        /// The cache key.
        key: String,
        /// The actual size of the entry.
        size: usize,
        /// The maximum allowed size.
        max_size: usize,
    },
    /// Failed to serialize cache entry.
    SerializationFailed {
        /// The cache key.
        key: String,
        /// The serialization error message.
        error: String,
    },
    /// Failed to deserialize cache entry.
    DeserializationFailed {
        /// The cache key.
        key: String,
        /// The deserialization error message.
        error: String,
    },
    /// Cache eviction failed.
    EvictionFailed {
        /// The reason for eviction failure.
        reason: String,
    },
    /// Invalid cache configuration.
    InvalidConfiguration {
        /// The configuration field that is invalid.
        field: String,
        /// The invalid value.
        value: String,
    },
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EntryTooLarge {
                key,
                size,
                max_size,
            } => {
                write!(
                    f,
                    "Cache entry too large: key='{key}', size={size}, max_size={max_size}"
                )
            }
            Self::SerializationFailed { key, error } => {
                write!(f, "Failed to serialize cache entry '{key}': {error}")
            }
            Self::DeserializationFailed { key, error } => {
                write!(f, "Failed to deserialize cache entry '{key}': {error}")
            }
            Self::EvictionFailed { reason } => {
                write!(f, "Cache eviction failed: {reason}")
            }
            Self::InvalidConfiguration { field, value } => {
                write!(
                    f,
                    "Invalid cache configuration: field='{field}', value='{value}'"
                )
            }
        }
    }
}

impl std::error::Error for CacheError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_too_large_error() {
        let err = CacheError::EntryTooLarge {
            key: "test_key".to_string(),
            size: 1024,
            max_size: 512,
        };
        let msg = err.to_string();
        assert!(msg.contains("too large"));
        assert!(msg.contains("test_key"));
        assert!(msg.contains("1024"));
        assert!(msg.contains("512"));
    }

    #[test]
    fn test_serialization_failed_error() {
        let err = CacheError::SerializationFailed {
            key: "my_key".to_string(),
            error: "JSON parse error".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("serialize"));
        assert!(msg.contains("my_key"));
        assert!(msg.contains("JSON parse error"));
    }

    #[test]
    fn test_deserialization_failed_error() {
        let err = CacheError::DeserializationFailed {
            key: "cached_data".to_string(),
            error: "Invalid format".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("deserialize"));
        assert!(msg.contains("cached_data"));
        assert!(msg.contains("Invalid format"));
    }

    #[test]
    fn test_eviction_failed_error() {
        let err = CacheError::EvictionFailed {
            reason: "Lock poisoned".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("eviction failed"));
        assert!(msg.contains("Lock poisoned"));
    }

    #[test]
    fn test_invalid_configuration_error() {
        let err = CacheError::InvalidConfiguration {
            field: "max_entries".to_string(),
            value: "0".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Invalid cache configuration"));
        assert!(msg.contains("max_entries"));
        assert!(msg.contains('0'));
    }

    #[test]
    fn test_error_equality() {
        let err1 = CacheError::EntryTooLarge {
            key: "key1".to_string(),
            size: 100,
            max_size: 50,
        };
        let err2 = CacheError::EntryTooLarge {
            key: "key1".to_string(),
            size: 100,
            max_size: 50,
        };
        let err3 = CacheError::EntryTooLarge {
            key: "key2".to_string(),
            size: 100,
            max_size: 50,
        };

        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }

    #[test]
    fn test_error_is_error_trait() {
        let err: Box<dyn std::error::Error> = Box::new(CacheError::EvictionFailed {
            reason: "Test error".to_string(),
        });
        assert!(err.to_string().contains("eviction failed"));
    }
}
