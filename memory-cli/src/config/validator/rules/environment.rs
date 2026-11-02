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
    // sysinfo >=0.40 probing is fallible; skip the memory check rather than the
    // CPU and CI checks, which do not need a live system snapshot.
    let memory_totals = sysinfo::System::new_all().ok().map(|mut system| {
        system.refresh_all();
        (system.total_memory(), system.available_memory())
    });
    let cpu_count = sysinfo::System::physical_core_count().unwrap_or(1);
    let is_ci = std::env::var("CI").is_ok();

    let mut warnings = Vec::new();

    // Check cache size vs available memory
    if let Some((_total_memory, available_memory)) = memory_totals {
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
                storage_mode: None,
                db_path: None,
            },
            storage: crate::config::types::StorageConfig {
                max_episodes_cache: 1000,
                cache_ttl_seconds: 3600,
                pool_size: 5,
                storage_mode: None,
            },
            cli: crate::config::types::CliConfig {
                default_format: "json".to_string(),
                progress_bars: false,
                batch_size: 100,
            },
            embeddings: crate::config::types::EmbeddingsConfig::default(),
        };
        let issues = quick_validation_check(&config);
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn test_quick_validation_check_no_storage() {
        let mut config = Config {
            database: crate::config::types::DatabaseConfig {
                turso_url: None,
                turso_token: None,
                redb_path: None,
                storage_mode: None,
                db_path: None,
            },
            storage: crate::config::types::StorageConfig {
                max_episodes_cache: 1000,
                cache_ttl_seconds: 3600,
                pool_size: 5,
                storage_mode: None,
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
        assert_ne!(issues.len(), 0);
        assert!(issues.iter().any(|i| i.contains("database")));
    }

    fn config_with(max_episodes_cache: usize, pool_size: usize, progress_bars: bool) -> Config {
        Config {
            database: crate::config::types::DatabaseConfig {
                turso_url: None,
                turso_token: None,
                redb_path: Some("/tmp/test.redb".to_string()),
                storage_mode: None,
                db_path: None,
            },
            storage: crate::config::types::StorageConfig {
                max_episodes_cache,
                cache_ttl_seconds: 3600,
                pool_size,
                storage_mode: None,
            },
            cli: crate::config::types::CliConfig {
                default_format: "json".to_string(),
                progress_bars,
                batch_size: 100,
            },
            embeddings: crate::config::types::EmbeddingsConfig::default(),
        }
    }

    /// `validate_environment_fitness` reads `CI` from the process environment, so
    /// the assertions follow whatever the runner provides instead of mutating env
    /// (which is `unsafe` in edition 2024 and would race other tests).
    fn in_ci() -> bool {
        std::env::var("CI").is_ok()
    }

    #[test]
    fn test_environment_fitness_flags_oversized_cache_and_pool() {
        // usize::MAX / 2048 keeps the * 1024 estimate inside the type while
        // guaranteeing it exceeds any real machine's available memory.
        let warnings =
            validate_environment_fitness(&config_with(usize::MAX / 2048, 100_000, false));

        assert!(
            warnings.iter().any(|w| w.field == "storage.pool_size"),
            "pool_size far above cpu_count * 4 must warn: {warnings:?}"
        );

        // sysinfo 0.40 constructors are fallible and the validator skips the memory
        // check when they fail. Asserting the same predicate keeps the line covered on
        // both kinds of runner, where an `if` would sit uncovered on whichever side the
        // probe did not take.
        let probeable = sysinfo::System::new_all().is_ok();
        let has_memory_warning = warnings
            .iter()
            .any(|w| w.message.contains("may exceed available memory"));
        assert_eq!(
            has_memory_warning, probeable,
            "an estimate above available memory must warn iff sysinfo can probe: {warnings:?}"
        );
    }

    #[test]
    fn test_environment_fitness_accepts_a_small_config() {
        // pool_size 1 stays under cpu_count * 4 even on a single-core runner, the cache
        // is under CI's 200 threshold, and progress bars are off — so no branch of the
        // validator may warn, on either kind of runner.
        let warnings = validate_environment_fitness(&config_with(100, 1, false));
        assert!(
            warnings.is_empty(),
            "expected no warnings, got {warnings:?}"
        );
    }

    #[test]
    fn test_environment_fitness_ci_checks_track_the_ci_env_var() {
        let warnings = validate_environment_fitness(&config_with(1000, 1, true));

        assert_eq!(
            warnings
                .iter()
                .any(|w| w.message.contains("CI environment detected")),
            in_ci(),
            "CI cache check ran off the env var"
        );
        assert_eq!(
            warnings.iter().any(|w| w.field == "cli.progress_bars"),
            in_ci(),
            "CI progress check ran off the env var"
        );
        assert!(
            warnings.iter().all(|w| w.suggestion.is_some()),
            "a warning the user cannot act on is not a suggestion: {warnings:?}"
        );
    }
}
