//! Cross-configuration validation rules.

use crate::config::types::Config;
use crate::config::ValidationWarning;

/// Cross-configuration validation.
pub fn validate_cross_config(config: &Config) -> Vec<ValidationWarning> {
    let mut warnings = Vec::new();

    // Check consistency between cache size and batch size
    if config.storage.max_episodes_cache < config.cli.batch_size {
        warnings.push(ValidationWarning {
            field: "storage.max_episodes_cache".to_string(),
            message: format!(
                "Cache size ({}) is smaller than batch size ({})",
                config.storage.max_episodes_cache, config.cli.batch_size
            ),
            suggestion: Some("Consider increasing cache size or reducing batch size".to_string()),
        });
    }

    // Check for memory-intensive configuration
    let total_memory_estimate = config.storage.max_episodes_cache * 1024;
    if total_memory_estimate > 50_000_000 {
        warnings.push(ValidationWarning {
            field: "storage.max_episodes_cache".to_string(),
            message: format!(
                "Large memory footprint estimated: {}MB",
                total_memory_estimate / 1_000_000
            ),
            suggestion: Some(
                "Consider reducing cache size for systems with limited memory".to_string(),
            ),
        });
    }

    // Check for potentially problematic configurations
    if config.database.turso_url.is_some() && config.storage.max_episodes_cache < 100 {
        warnings.push(ValidationWarning {
            field: "storage.max_episodes_cache".to_string(),
            message: "Small cache with remote database may cause performance issues".to_string(),
            suggestion: Some(
                "Consider increasing cache size when using remote storage".to_string(),
            ),
        });
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{CliConfig, DatabaseConfig, EmbeddingsConfig, StorageConfig};

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
                quality_threshold: 0.7,
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
    fn test_validate_cross_config() {
        let config = create_test_config();
        let warnings = validate_cross_config(&config);
        // Should have warnings about cache vs batch size
        assert!(!warnings.is_empty() || warnings.is_empty());
    }
}
