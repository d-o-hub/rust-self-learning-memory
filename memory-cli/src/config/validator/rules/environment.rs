//! Environment-based validation rules.

use crate::config::ValidationWarning;
use crate::config::types::Config;

/// Quick validation check for common issues.
pub fn quick_validation_check(config: &Config) -> Vec<String> {
    let mut issues = Vec::new();

    if config.database.turso_url.is_none() && config.database.redb_path.is_none() {
        issues
            .push("No database configuration found. Configure turso_url or redb_path.".to_string());
    }

    if config.storage.max_episodes_cache == 0 {
        issues.push("Cache size is 0, which will cause errors.".to_string());
    }

    if config.storage.pool_size == 0 {
        issues.push("Connection pool size is 0, which will cause errors.".to_string());
    }

    if config.cli.batch_size == 0 {
        issues.push("Batch size is 0, which will cause errors.".to_string());
    }

    if config.storage.max_episodes_cache > 50000 {
        issues.push("Very large cache size may cause memory issues.".to_string());
    }

    issues
}

/// Validate if the current configuration is suitable for the environment.
pub fn validate_environment_fitness(config: &Config) -> Vec<ValidationWarning> {
    let mut system = sysinfo::System::new_all();
    system.refresh_all();

    let total_memory = system.total_memory();
    let available_memory = system.available_memory();
    let cpu_count = sysinfo::System::physical_core_count().unwrap_or(1);
    let is_ci = std::env::var("CI").is_ok();

    let mut warnings = Vec::new();

    // Check cache size vs available memory
    let cache_memory_estimate = config.storage.max_episodes_cache * 1024;
    if cache_memory_estimate > available_memory as usize {
        warnings.push(ValidationWarning {
            field: "storage.max_episodes_cache".to_string(),
            message: format!(
                "Cache size estimate ({}MB) may exceed available memory ({}MB)",
                cache_memory_estimate / (1024 * 1024),
                available_memory / (1024 * 1024)
            ),
            suggestion: Some(format!(
                "Consider reducing cache size to {}",
                available_memory / (1024 * 1024)
            )),
        });
    }

    // Check pool size vs CPU cores
    if config.storage.pool_size > cpu_count * 4 {
        warnings.push(ValidationWarning {
            field: "storage.pool_size".to_string(),
            message: format!(
                "Pool size ({}) may be too high for {} CPU cores",
                config.storage.pool_size, cpu_count
            ),
            suggestion: Some(format!("Consider reducing to {}", cpu_count * 2)),
        });
    }

    // CI-specific checks
    if is_ci {
        if config.storage.max_episodes_cache > 200 {
            warnings.push(ValidationWarning {
                field: "storage.max_episodes_cache".to_string(),
                message: "CI environment detected - large cache may impact performance".to_string(),
                suggestion: Some("Consider reducing to 100-200".to_string()),
            });
        }

        if config.cli.progress_bars {
            warnings.push(ValidationWarning {
                field: "cli.progress_bars".to_string(),
                message: "Progress bars may not work properly in CI environments".to_string(),
                suggestion: Some("Set to false for CI".to_string()),
            });
        }
    }

    // Unused variable warning suppression
    let _ = total_memory;

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quick_validation_check_valid() {
        let config = Config {
            database: crate::config::types::DatabaseConfig {
                turso_url: None,
                turso_token: None,
                redb_path: Some("/tmp/test.redb".to_string()),
            },
            storage: crate::config::types::StorageConfig {
                max_episodes_cache: 1000,
                cache_ttl_seconds: 3600,
                pool_size: 5,
            },
            cli: crate::config::types::CliConfig {
                default_format: "json".to_string(),
                progress_bars: false,
                batch_size: 100,
            },
            embeddings: crate::config::types::EmbeddingsConfig::default(),
        };
        let issues = quick_validation_check(&config);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_quick_validation_check_no_storage() {
        let mut config = Config {
            database: crate::config::types::DatabaseConfig {
                turso_url: None,
                turso_token: None,
                redb_path: None,
            },
            storage: crate::config::types::StorageConfig {
                max_episodes_cache: 1000,
                cache_ttl_seconds: 3600,
                pool_size: 5,
            },
            cli: crate::config::types::CliConfig {
                default_format: "json".to_string(),
                progress_bars: false,
                batch_size: 100,
            },
            embeddings: crate::config::types::EmbeddingsConfig::default(),
        };
        config.database.redb_path = None;
        let issues = quick_validation_check(&config);
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.contains("database")));
    }
}
