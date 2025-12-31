//! Configuration validator module
//!
//! This module provides comprehensive validation for configuration with
//! contextual error messages and suggestions for fixing issues.

use super::types::{CliConfig, Config, DatabaseConfig, StorageConfig};
use std::path::Path;

/// Validation result with enhanced error context
#[derive(Debug)]
pub struct ValidationResult {
    /// Whether validation passed
    pub is_valid: bool,
    /// List of validation errors with context
    pub errors: Vec<ValidationError>,
    /// List of warnings that don't prevent usage
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    /// Create a successful validation result
    pub fn ok() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Create a failed validation result with errors
    pub fn with_errors(errors: Vec<ValidationError>) -> Self {
        Self {
            is_valid: false,
            errors,
            warnings: Vec::new(),
        }
    }

    /// Add warnings to a validation result
    pub fn with_warnings(mut self, warnings: Vec<ValidationWarning>) -> Self {
        self.warnings.extend(warnings);
        self
    }
}

/// Enhanced validation error with context and suggestions
#[derive(Debug)]
pub struct ValidationError {
    /// Field that failed validation
    pub field: String,
    /// Human-readable error message
    pub message: String,
    /// Suggested value or fix
    pub suggestion: Option<String>,
    /// Context for the error
    pub context: Option<String>,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(suggestion) = &self.suggestion {
            write!(f, " Suggestion: {}", suggestion)?;
        }
        if let Some(context) = &self.context {
            write!(f, " Context: {}", context)?;
        }
        Ok(())
    }
}

/// Non-blocking validation warnings
#[derive(Debug)]
pub struct ValidationWarning {
    /// Field that generated the warning
    pub field: String,
    /// Warning message
    pub message: String,
    /// Suggested improvement
    pub suggestion: Option<String>,
}

impl std::fmt::Display for ValidationWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(suggestion) = &self.suggestion {
            write!(f, " Suggestion: {}", suggestion)?;
        }
        Ok(())
    }
}

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
        } else {
            // Security check for file: URLs
            if let Err(security_error) = validate_file_url_security(turso_url) {
                errors.push(security_error);
            }
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

/// Validate that a file URL doesn't contain path traversal or access sensitive paths
fn validate_file_url_security(url: &str) -> Result<(), ValidationError> {
    if !url.starts_with("file:") {
        // Not a file URL, no security check needed
        return Ok(());
    }

    // Extract the path from the file: URL
    let path = url.strip_prefix("file:").unwrap_or(url);

    // Check for path traversal attempts
    if path.contains("..") {
        return Err(ValidationError {
            field: "database.turso_url".to_string(),
            message: "Storage error: Path traversal detected in database URL".to_string(),
            suggestion: Some("Use an absolute path without '..' components".to_string()),
            context: Some("Security: Path traversal attacks are blocked".to_string()),
        });
    }

    // Check for access to sensitive system paths
    // Note: /tmp/ is excluded as it's commonly used for temporary/test databases
    let sensitive_paths = [
        "/etc/",
        "/bin/",
        "/sbin/",
        "/usr/bin/",
        "/usr/sbin/",
        "/sys/",
        "/proc/",
        "/dev/",
        "/boot/",
        "/root/",
        "/var/log/",
        "/var/run/",
    ];

    for sensitive_path in &sensitive_paths {
        if path.starts_with(sensitive_path) {
            return Err(ValidationError {
                field: "database.turso_url".to_string(),
                message: format!(
                    "Storage error: Access to sensitive system path is not allowed: {}",
                    path
                ),
                suggestion: Some(
                    "Use a path in your home directory or project directory".to_string(),
                ),
                context: Some("Security: Access to system paths is blocked".to_string()),
            });
        }
    }

    // Additional check for specific sensitive files
    let sensitive_files = ["/etc/passwd", "/etc/shadow", "/etc/hosts", "/etc/sudoers"];

    for sensitive_file in &sensitive_files {
        if path == *sensitive_file || path.ends_with(sensitive_file) {
            return Err(ValidationError {
                field: "database.turso_url".to_string(),
                message: format!(
                    "Storage error: Access to sensitive file is not allowed: {}",
                    sensitive_file
                ),
                suggestion: Some("Use a database file in your project directory".to_string()),
                context: Some("Security: Access to sensitive files is blocked".to_string()),
            });
        }
    }

    Ok(())
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

/// Validate if the current configuration is suitable for the environment
pub fn validate_environment_fitness(config: &Config) -> ValidationResult {
    // Import the smart_defaults module functions
    let mut system = sysinfo::System::new_all();
    system.refresh_all();

    let total_memory = system.total_memory();
    let available_memory = system.available_memory();
    let cpu_count = sysinfo::System::physical_core_count().unwrap_or(1);
    let is_ci = std::env::var("CI").is_ok();
    let is_development = std::env::var("DEVELOPMENT").is_ok() || std::env::var("DEV").is_ok();

    let mut result = ValidationResult::ok();
    let mut warnings = Vec::new();

    // Check cache size vs available memory
    let cache_memory_estimate = config.storage.max_episodes_cache * 1024; // Rough estimate in bytes
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

    if !warnings.is_empty() {
        result = result.with_warnings(warnings);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::EmbeddingsConfig;

    fn create_test_config() -> Config {
        Config {
            embeddings: EmbeddingsConfig::default(),
            database: DatabaseConfig {
                turso_url: Some("libsql://test.turso.io/test".to_string()),
                turso_token: Some("test_token".to_string()),
                redb_path: Some("./test.redb".to_string()),
            },
            storage: StorageConfig {
                max_episodes_cache: 1000,
                cache_ttl_seconds: 3600,
                pool_size: 10,
            },
            cli: CliConfig {
                default_format: "json".to_string(),
                batch_size: 100,
                progress_bars: true,
            },
        }
    }
    #[test]
    fn test_valid_configuration() {
        let config = create_test_config();
        let result = validate_config(&config);
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_missing_database_configuration() {
        let mut config = create_test_config();
        config.database.turso_url = None;
        config.database.redb_path = None;

        let result = validate_database_config(&config.database);
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].field, "database");
    }

    #[test]
    fn test_empty_turso_url() {
        let mut config = create_test_config();
        config.database.turso_url = Some("".to_string());

        let result = validate_database_config(&config.database);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.field == "database.turso_url"));
    }

    #[test]
    fn test_invalid_turso_url_format() {
        let mut config = create_test_config();
        config.database.turso_url = Some("http://invalid.url".to_string());

        let result = validate_database_config(&config.database);
        assert!(result.is_valid);
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_path_traversal_in_file_url() {
        let mut config = create_test_config();
        config.database.turso_url = Some("file:../../../etc/passwd".to_string());

        let result = validate_database_config(&config.database);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.message.contains("traversal")));
    }

    #[test]
    fn test_sensitive_path_access() {
        let mut config = create_test_config();
        config.database.turso_url = Some("file:/etc/passwd".to_string());

        let result = validate_database_config(&config.database);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.message.contains("sensitive")));
    }

    #[test]
    fn test_zero_cache_size() {
        let mut config = create_test_config();
        config.storage.max_episodes_cache = 0;

        let result = validate_storage_config(&config.storage);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.field == "storage.max_episodes_cache"));
    }

    #[test]
    fn test_zero_cache_ttl() {
        let mut config = create_test_config();
        config.storage.cache_ttl_seconds = 0;

        let result = validate_storage_config(&config.storage);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.field == "storage.cache_ttl_seconds"));
    }

    #[test]
    fn test_zero_pool_size() {
        let mut config = create_test_config();
        config.storage.pool_size = 0;

        let result = validate_storage_config(&config.storage);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.field == "storage.pool_size"));
    }

    #[test]
    fn test_large_cache_size_warning() {
        let mut config = create_test_config();
        config.storage.max_episodes_cache = 150000;

        let result = validate_storage_config(&config.storage);
        assert!(result.is_valid);
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_invalid_output_format() {
        let mut config = create_test_config();
        config.cli.default_format = "xml".to_string();

        let result = validate_cli_config(&config.cli);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.field == "cli.default_format"));
    }

    #[test]
    fn test_zero_batch_size() {
        let mut config = create_test_config();
        config.cli.batch_size = 0;

        let result = validate_cli_config(&config.cli);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.field == "cli.batch_size"));
    }

    #[test]
    fn test_cross_validation_cache_smaller_than_batch() {
        let mut config = create_test_config();
        config.storage.max_episodes_cache = 50;
        config.cli.batch_size = 100;

        let result = validate_cross_config(&config);
        assert!(result.is_valid);
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_validation_error_display() {
        let error = ValidationError {
            field: "test.field".to_string(),
            message: "Test error message".to_string(),
            suggestion: Some("Try this fix".to_string()),
            context: Some("Additional context".to_string()),
        };

        let display = format!("{}", error);
        assert!(display.contains("Test error message"));
        assert!(display.contains("Try this fix"));
        assert!(display.contains("Additional context"));
    }

    #[test]
    fn test_validation_warning_display() {
        let warning = ValidationWarning {
            field: "test.field".to_string(),
            message: "Test warning message".to_string(),
            suggestion: Some("Consider this improvement".to_string()),
        };

        let display = format!("{}", warning);
        assert!(display.contains("Test warning message"));
        assert!(display.contains("Consider this improvement"));
    }

    #[test]
    fn test_format_validation_result_valid() {
        let result = ValidationResult::ok();
        let formatted = format_validation_result(&result);
        assert!(formatted.contains("✅"));
        assert!(formatted.contains("valid"));
    }

    #[test]
    fn test_format_validation_result_with_errors() {
        let errors = vec![ValidationError {
            field: "test".to_string(),
            message: "Test error".to_string(),
            suggestion: None,
            context: None,
        }];
        let result = ValidationResult::with_errors(errors);
        let formatted = format_validation_result(&result);
        assert!(formatted.contains("❌"));
        assert!(formatted.contains("Test error"));
    }

    #[test]
    fn test_quick_validation_check() {
        let mut config = create_test_config();
        config.database.turso_url = None;
        config.database.redb_path = None;
        config.storage.max_episodes_cache = 0;

        let issues = quick_validation_check(&config);
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.contains("database")));
        assert!(issues.iter().any(|i| i.contains("Cache size")));
    }

    #[test]
    fn test_is_valid_turso_url() {
        assert!(is_valid_turso_url("libsql://test.turso.io/test"));
        assert!(is_valid_turso_url("file:/tmp/test.db"));
        assert!(!is_valid_turso_url("http://test.com"));
        assert!(!is_valid_turso_url("invalid"));
    }

    #[test]
    fn test_validation_result_ok() {
        let result = ValidationResult::ok();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_validation_result_with_warnings() {
        let warnings = vec![ValidationWarning {
            field: "test".to_string(),
            message: "Test warning".to_string(),
            suggestion: None,
        }];
        let result = ValidationResult::ok().with_warnings(warnings);
        assert!(result.is_valid);
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_turso_token_warning() {
        let mut config = create_test_config();
        config.database.turso_token = None;

        let result = validate_database_config(&config.database);
        assert!(result.is_valid);
        assert!(!result.warnings.is_empty());
        assert!(result.warnings.iter().any(|w| w.field == "database.turso_token"));
    }

    #[test]
    fn test_all_valid_output_formats() {
        let formats = vec!["human", "json", "yaml"];
        for format in formats {
            let mut config = create_test_config();
            config.cli.default_format = format.to_string();
            let result = validate_cli_config(&config.cli);
            assert!(result.is_valid, "Format {} should be valid", format);
        }
    }

    #[test]
    fn test_contextual_error_messages() {
        let mut config = create_test_config();
        config.storage.max_episodes_cache = 0;

        let result = validate_storage_config(&config.storage);
        assert!(!result.is_valid);
        
        let error = &result.errors[0];
        assert!(error.suggestion.is_some());
        assert!(error.context.is_some());
    }

    #[test]
    fn test_performance_recommendations() {
        let mut config = create_test_config();
        config.storage.cache_ttl_seconds = 30;

        let result = validate_storage_config(&config.storage);
        assert!(result.is_valid);
        assert!(!result.warnings.is_empty());
        assert!(result.warnings.iter().any(|w| w.suggestion.is_some()));
    }

    #[test]
    fn test_security_validation() {
        let mut config = create_test_config();
        config.database.turso_url = Some("file:../../secret.db".to_string());

        let result = validate_database_config(&config.database);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.context.as_ref()
            .map(|c| c.contains("Security"))
            .unwrap_or(false)));
    }
}
