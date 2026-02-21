//! Configuration validator module
//!
//! This module provides comprehensive validation for configuration with
//! contextual error messages and suggestions for fixing issues.

mod messages;
mod rules;

pub use messages::{
    ValidationError, ValidationResult, ValidationWarning, format_validation_result,
    validate_config, validate_config_path,
};
pub use rules::{
    quick_validation_check, validate_cli_config, validate_cross_config, validate_database_config,
    validate_environment_fitness, validate_storage_config,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{
        CliConfig, Config, DatabaseConfig, EmbeddingsConfig, StorageConfig,
    };

    fn create_test_config() -> Config {
        Config {
            database: DatabaseConfig {
                turso_url: None,
                turso_token: None,
                redb_path: Some("/tmp/test.redb".to_string()),
            },
            storage: StorageConfig {
                max_episodes_cache: 1000,
                cache_ttl_seconds: 3600,
                pool_size: 5,
            },
            cli: CliConfig {
                default_format: "json".to_string(),
                progress_bars: false,
                batch_size: 100,
            },
            embeddings: EmbeddingsConfig::default(),
        }
    }

    #[test]
    fn test_validate_config_valid() {
        let config = create_test_config();
        let result = validate_config(&config);
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_config_invalid() {
        let mut config = create_test_config();
        config.database.redb_path = None;
        config.database.turso_url = None;
        let result = validate_config(&config);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_validate_config_with_warnings() {
        let mut config = create_test_config();
        config.storage.max_episodes_cache = 100000; // Large cache
        let result = validate_config(&config);
        assert!(result.is_valid);
        assert!(!result.warnings.is_empty());
    }
}
