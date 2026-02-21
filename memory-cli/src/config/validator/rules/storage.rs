//! Storage configuration validation rules.

use crate::config::ValidationError;
use crate::config::types::StorageConfig;

/// Validate storage configuration (errors only).
pub fn validate_storage_config_errors(config: &StorageConfig) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Validate cache size
    if config.max_episodes_cache == 0 {
        errors.push(ValidationError {
            field: "storage.max_episodes_cache".to_string(),
            message: "max_episodes_cache must be greater than 0".to_string(),
            suggestion: Some("Use a value between 1 and 10000".to_string()),
            context: Some("Cache management".to_string()),
        });
    }

    // Validate cache TTL
    if config.cache_ttl_seconds == 0 {
        errors.push(ValidationError {
            field: "storage.cache_ttl_seconds".to_string(),
            message: "cache_ttl_seconds must be greater than 0".to_string(),
            suggestion: Some("Use a value between 60 and 86400 (1 minute to 24 hours)".to_string()),
            context: Some("Cache expiration".to_string()),
        });
    }

    // Validate pool size
    if config.pool_size == 0 {
        errors.push(ValidationError {
            field: "storage.pool_size".to_string(),
            message: "pool_size must be greater than 0".to_string(),
            suggestion: Some("Use a value between 1 and 100".to_string()),
            context: Some("Database connections".to_string()),
        });
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_storage_config_valid() {
        let config = StorageConfig {
            max_episodes_cache: 1000,
            cache_ttl_seconds: 3600,
            pool_size: 5,
        };
        let errors = validate_storage_config_errors(&config);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_storage_config_zero_cache() {
        let config = StorageConfig {
            max_episodes_cache: 0,
            cache_ttl_seconds: 3600,
            pool_size: 5,
        };
        let errors = validate_storage_config_errors(&config);
        assert!(!errors.is_empty());
    }
}
