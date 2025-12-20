//! Configuration validator module
//!
//! This module provides comprehensive validation for configuration with
//! contextual error messages and suggestions for fixing issues.

use super::types::{
    CliConfig, Config, DatabaseConfig, StorageConfig, ValidationError, ValidationResult,
    ValidationWarning,
};
use std::path::Path;

/// Validate the entire configuration
pub fn validate_config(config: &Config) -> ValidationResult {
    let mut errors: Vec<ValidationError> = Vec::new();
    let mut warnings: Vec<ValidationWarning> = Vec::new();

    // Validate database configuration
    let db_validation = validate_database_config(&config.database);
    errors.extend(db_validation.errors);
    warnings.extend(db_validation.warnings);

    // Validate storage configuration
    let storage_validation = validate_storage_config(&config.storage);
    errors.extend(storage_validation.errors);
    warnings.extend(storage_validation.warnings);

    // Validate CLI configuration
    let cli_validation = validate_cli_config(&config.cli);
    errors.extend(cli_validation.errors);
    warnings.extend(cli_validation.warnings);

    // Cross-configuration validation
    let cross_validation = validate_cross_config(config);
    errors.extend(cross_validation.errors);
    warnings.extend(cross_validation.warnings);

    let is_valid = errors.is_empty();
    let mut result = if is_valid {
        ValidationResult::ok()
    } else {
        ValidationResult::with_errors(errors)
    };

    if !warnings.is_empty() {
        result = result.with_warnings(warnings);
    }

    result
}

/// Validate database configuration
pub fn validate_database_config(config: &DatabaseConfig) -> ValidationResult {
    let mut errors: Vec<ValidationError> = Vec::new();
    let mut warnings: Vec<ValidationWarning> = Vec::new();

    // Check if at least one storage option is configured
    if config.turso_url.is_none() && config.redb_path.is_none() {
        errors.push(ValidationError {
            field: "database".to_string(),
            message: "At least one database configuration is required".to_string(),
            suggestion: Some("Configure either turso_url or redb_path".to_string()),
            context: Some("No durable storage backend configured".to_string()),
        });
    }

    // Validate Turso URL if provided
    if let Some(turso_url) = &config.turso_url {
        if turso_url.trim().is_empty() {
            errors.push(ValidationError {
                field: "database.turso_url".to_string(),
                message: "Turso URL cannot be empty".to_string(),
                suggestion: Some("Provide a valid Turso database URL".to_string()),
                context: Some("Remote database access".to_string()),
            });
        } else if !is_valid_turso_url(turso_url) {
            warnings.push(ValidationWarning {
                field: "database.turso_url".to_string(),
                message: format!("Turso URL format may be invalid: {}", turso_url),
                suggestion: Some(
                    "Ensure URL follows format: libsql://host/db or file:path".to_string(),
                ),
            });
        }
    }

    // Validate redb path if provided
    if let Some(redb_path) = &config.redb_path {
        if redb_path.trim().is_empty() {
            errors.push(ValidationError {
                field: "database.redb_path".to_string(),
                message: "redb path cannot be empty".to_string(),
                suggestion: Some(
                    "Provide a valid file path or use ':memory:' for in-memory storage".to_string(),
                ),
                context: Some("Local cache storage".to_string()),
            });
        }
    }

    // Check for suspicious combinations
    if config.turso_url.is_some() && config.turso_token.is_none() {
        warnings.push(ValidationWarning {
            field: "database.turso_token".to_string(),
            message: "Turso URL provided without authentication token".to_string(),
            suggestion: Some("Add turso_token for secure database access".to_string()),
        });
    }

    if config.turso_url.is_none() && config.redb_path.is_some() {
        let redb_path = config.redb_path.as_ref().unwrap();
        if redb_path.starts_with("libsql://") {
            warnings.push(ValidationWarning {
                field: "database.redb_path".to_string(),
                message: "redb_path contains what looks like a remote URL".to_string(),
                suggestion: Some(
                    "Use turso_url for remote databases and redb_path for local files".to_string(),
                ),
            });
        }
    }

    let is_valid = errors.is_empty();
    let mut result = if is_valid {
        ValidationResult::ok()
    } else {
        ValidationResult::with_errors(errors)
    };

    if !warnings.is_empty() {
        result = result.with_warnings(warnings);
    }

    result
}

/// Validate storage configuration
pub fn validate_storage_config(config: &StorageConfig) -> ValidationResult {
    let mut errors: Vec<ValidationError> = Vec::new();
    let mut warnings: Vec<ValidationWarning> = Vec::new();

    // Validate cache size
    if config.max_episodes_cache == 0 {
        errors.push(ValidationError {
            field: "storage.max_episodes_cache".to_string(),
            message: "max_episodes_cache must be greater than 0".to_string(),
            suggestion: Some("Use a value between 1 and 10000".to_string()),
            context: Some("Cache management".to_string()),
        });
    } else if config.max_episodes_cache > 100000 {
        warnings.push(ValidationWarning {
            field: "storage.max_episodes_cache".to_string(),
            message: format!("Large cache size: {} episodes", config.max_episodes_cache),
            suggestion: Some("Consider reducing cache size for better memory usage".to_string()),
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
    } else if config.cache_ttl_seconds < 60 {
        warnings.push(ValidationWarning {
            field: "storage.cache_ttl_seconds".to_string(),
            message: format!("Short cache TTL: {} seconds", config.cache_ttl_seconds),
            suggestion: Some("Consider longer TTL for better cache efficiency".to_string()),
        });
    } else if config.cache_ttl_seconds > 86400 {
        warnings.push(ValidationWarning {
            field: "storage.cache_ttl_seconds".to_string(),
            message: format!("Long cache TTL: {} seconds", config.cache_ttl_seconds),
            suggestion: Some("Consider shorter TTL for fresher data".to_string()),
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
    } else if config.pool_size > 200 {
        warnings.push(ValidationWarning {
            field: "storage.pool_size".to_string(),
            message: format!("Large connection pool: {} connections", config.pool_size),
            suggestion: Some("Consider smaller pool size to avoid resource exhaustion".to_string()),
        });
    }

    let is_valid = errors.is_empty();
    let mut result = if is_valid {
        ValidationResult::ok()
    } else {
        ValidationResult::with_errors(errors)
    };

    if !warnings.is_empty() {
        result = result.with_warnings(warnings);
    }

    result
}

/// Validate CLI configuration
pub fn validate_cli_config(config: &CliConfig) -> ValidationResult {
    let mut errors: Vec<ValidationError> = Vec::new();
    let mut warnings: Vec<ValidationWarning> = Vec::new();

    // Validate output format
    match config.default_format.as_str() {
        "human" | "json" | "yaml" => {
            // Valid format
        }
        _ => {
            errors.push(ValidationError {
                field: "cli.default_format".to_string(),
                message: format!("Invalid output format: {}", config.default_format),
                suggestion: Some("Use 'human', 'json', or 'yaml'".to_string()),
                context: Some("Output formatting".to_string()),
            });
        }
    }

    // Validate batch size
    if config.batch_size == 0 {
        errors.push(ValidationError {
            field: "cli.batch_size".to_string(),
            message: "batch_size must be greater than 0".to_string(),
            suggestion: Some("Use a value between 1 and 1000".to_string()),
            context: Some("Bulk operations".to_string()),
        });
    } else if config.batch_size > 10000 {
        warnings.push(ValidationWarning {
            field: "cli.batch_size".to_string(),
            message: format!("Large batch size: {}", config.batch_size),
            suggestion: Some("Consider smaller batches for better responsiveness".to_string()),
        });
    }

    let is_valid = errors.is_empty();
    let mut result = if is_valid {
        ValidationResult::ok()
    } else {
        ValidationResult::with_errors(errors)
    };

    if !warnings.is_empty() {
        result = result.with_warnings(warnings);
    }

    result
}

/// Cross-configuration validation
fn validate_cross_config(config: &Config) -> ValidationResult {
    let mut errors: Vec<ValidationError> = Vec::new();
    let mut warnings: Vec<ValidationWarning> = Vec::new();

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
    let total_memory_estimate = config.storage.max_episodes_cache * 1024; // Rough estimate
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

    ValidationResult::ok().with_warnings(warnings)
}

/// Check if Turso URL format is valid
fn is_valid_turso_url(url: &str) -> bool {
    // Basic validation for Turso/libSQL URL format
    url.starts_with("libsql://") || url.starts_with("file:")
}

/// Validate configuration file path
pub fn validate_config_path(path: &Path) -> Result<(), ValidationError> {
    if !path.exists() {
        return Err(ValidationError {
            field: "config_path".to_string(),
            message: format!("Configuration file does not exist: {}", path.display()),
            suggestion: Some("Check file path or create configuration file".to_string()),
            context: Some("File system validation".to_string()),
        });
    }

    if !path.is_file() {
        return Err(ValidationError {
            field: "config_path".to_string(),
            message: format!("Configuration path is not a file: {}", path.display()),
            suggestion: Some("Provide a valid file path".to_string()),
            context: Some("File system validation".to_string()),
        });
    }

    // Check file extension
    match path.extension().and_then(|s| s.to_str()) {
        Some("toml") | Some("json") | Some("yaml") | Some("yml") => Ok(()),
        Some(ext) => Err(ValidationError {
            field: "config_path".to_string(),
            message: format!("Unsupported configuration file format: .{}", ext),
            suggestion: Some("Use .toml, .json, .yaml, or .yml".to_string()),
            context: Some("File format validation".to_string()),
        }),
        None => Err(ValidationError {
            field: "config_path".to_string(),
            message: "Configuration file has no extension".to_string(),
            suggestion: Some("Add .toml, .json, .yaml, or .yml extension".to_string()),
            context: Some("File format validation".to_string()),
        }),
    }
}

/// Helper function to format validation results
pub fn format_validation_result(result: &ValidationResult) -> String {
    let mut output = String::new();

    if result.is_valid {
        output.push_str("✅ Configuration is valid\n");
    } else {
        output.push_str(&format!(
            "❌ Configuration has {} error(s):\n",
            result.errors.len()
        ));
        for (i, error) in result.errors.iter().enumerate() {
            output.push_str(&format!("  {}. {}\n", i + 1, error));
        }
    }

    if !result.warnings.is_empty() {
        output.push_str(&format!(
            "\n⚠️  Configuration has {} warning(s):\n",
            result.warnings.len()
        ));
        for (i, warning) in result.warnings.iter().enumerate() {
            output.push_str(&format!("  {}. {}\n", i + 1, warning));
        }
    }

    output
}

/// Quick validation check for common issues
pub fn quick_validation_check(config: &Config) -> Vec<String> {
    let mut issues = Vec::new();

    // Check for missing database configuration
    if config.database.turso_url.is_none() && config.database.redb_path.is_none() {
        issues
            .push("No database configuration found. Configure turso_url or redb_path.".to_string());
    }

    // Check for suspicious values
    if config.storage.max_episodes_cache == 0 {
        issues.push("Cache size is 0, which will cause errors.".to_string());
    }

    if config.storage.pool_size == 0 {
        issues.push("Connection pool size is 0, which will cause errors.".to_string());
    }

    if config.cli.batch_size == 0 {
        issues.push("Batch size is 0, which will cause errors.".to_string());
    }

    // Check for performance issues
    if config.storage.max_episodes_cache > 50000 {
        issues.push("Very large cache size may cause memory issues.".to_string());
    }

    issues
}
