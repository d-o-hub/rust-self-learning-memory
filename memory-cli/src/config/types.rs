//! Configuration types for memory-cli
//!
//! This module defines the core configuration structures used throughout
//! the memory-cli application, providing a clean separation of concerns
//! from loading, validation, and storage initialization logic.

use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use sysinfo::System;

/// Helper functions for smart defaults and platform-aware configuration
mod smart_defaults {
    use super::*;

    /// Detect the appropriate data directory based on platform and environment
    pub fn detect_data_directory() -> PathBuf {
        // Check environment variable first
        if let Ok(data_dir) = env::var("MEMORY_DATA_DIR") {
            return PathBuf::from(data_dir);
        }

        // Platform-appropriate default directories
        if let Some(mut data_dir) = dirs::data_dir() {
            data_dir.push("memory-cli");
            data_dir
        } else if let Some(mut home_dir) = dirs::home_dir() {
            home_dir.push(".memory-cli");
            home_dir
        } else {
            // Fallback to current directory
            PathBuf::from("./data")
        }
    }

    /// Detect cache directory for redb storage
    pub fn detect_cache_directory() -> PathBuf {
        // Check environment variable first
        if let Ok(cache_dir) = env::var("MEMORY_CACHE_DIR") {
            return PathBuf::from(cache_dir);
        }

        let mut data_dir = detect_data_directory();
        data_dir.push("cache");
        data_dir
    }

    /// Get system resource information for smart configuration
    pub fn get_system_info() -> SystemInfo {
        let mut system = System::new_all();
        system.refresh_all();

        SystemInfo {
            total_memory: system.total_memory(),
            available_memory: system.available_memory(),
            cpu_count: system.physical_core_count().unwrap_or(1),
            is_ci: env::var("CI").is_ok(),
            is_development: env::var("DEVELOPMENT").is_ok() || env::var("DEV").is_ok(),
        }
    }

    /// Smart database path detection
    pub fn detect_redb_path() -> String {
        // Check environment variable
        if let Ok(path) = env::var("REDB_PATH") {
            return path;
        }

        // Use cache directory with memory-specific filename
        let mut cache_dir = detect_cache_directory();
        cache_dir.push("memory.redb");
        cache_dir.to_string_lossy().to_string()
    }

    /// Smart pool size based on system resources
    pub fn suggest_pool_size() -> usize {
        let info = get_system_info();

        if info.is_ci || info.is_development {
            5 // Conservative for CI/dev
        } else {
            // Scale with CPU cores, max 20, min 3
            std::cmp::max(3, std::cmp::min(20, info.cpu_count * 2))
        }
    }

    /// Smart cache size based on available memory
    pub fn suggest_cache_size() -> usize {
        let info = get_system_info();

        if info.is_ci {
            100 // Minimal for CI
        } else if info.is_development {
            500 // Moderate for development
        } else {
            // Scale with available memory: 1GB = ~200 episodes
            let gb_available = info.available_memory / (1024 * 1024 * 1024);
            std::cmp::min(5000, std::cmp::max(1000, (gb_available * 200) as usize))
        }
    }

    /// Smart cache TTL based on system and usage patterns
    pub fn suggest_cache_ttl() -> u64 {
        let info = get_system_info();

        if info.is_ci {
            300 // 5 minutes for CI
        } else if info.is_development {
            1800 // 30 minutes for development
        } else {
            7200 // 2 hours for production
        }
    }

    /// Detect default output format based on environment
    pub fn detect_default_format() -> String {
        if let Ok(format) = env::var("MEMORY_FORMAT") {
            return format;
        }

        if env::var("CI").is_ok() || env::var("GITHUB_ACTIONS").is_ok() {
            return "json".to_string(); // Machine-readable for CI
        }

        "human".to_string() // Human-readable by default
    }

    /// Smart batch size for operations
    pub fn suggest_batch_size() -> usize {
        let info = get_system_info();

        if info.is_ci {
            10 // Small batches for CI
        } else if info.is_development {
            50 // Moderate for development
        } else {
            200 // Larger for production
        }
    }

    /// Check if running in cloud environment
    pub fn is_cloud_environment() -> bool {
        env::var("TURSO_URL").is_ok() && env::var("TURSO_TOKEN").is_ok()
    }

    /// Smart Turso URL detection
    pub fn detect_turso_url() -> Option<String> {
        if let Ok(url) = env::var("TURSO_URL") {
            return Some(url);
        }

        // Check for common cloud indicators
        if env::var("RENDER").is_ok() || env::var("HEROKU").is_ok() || env::var("FLY_IO").is_ok() {
            return Some("file:./data/memory.db".to_string());
        }

        None
    }

    /// Smart Turso token detection
    pub fn detect_turso_token() -> Option<String> {
        if let Ok(token) = env::var("TURSO_TOKEN") {
            return Some(token);
        }
        None
    }
}

/// System information for smart defaults
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub total_memory: u64,
    pub available_memory: u64,
    pub cpu_count: usize,
    pub is_ci: bool,
    pub is_development: bool,
}

/// Smart configuration utilities for advanced users
pub mod smart_config {
    use super::*;

    /// Create a configuration optimized for the current environment
    /// This is a more advanced version of Config::default() with additional context
    pub fn auto_detect_config() -> Config {
        let info = smart_defaults::get_system_info();

        // Start with defaults and then apply smart enhancements
        let mut config = Config::default();

        // Apply environment-specific optimizations
        if info.is_ci {
            // CI optimizations
            config.cli.progress_bars = false;
            config.cli.batch_size = 10;
            config.storage.max_episodes_cache = 100;
        } else if info.is_development {
            // Development optimizations
            config.cli.default_format = "human".to_string();
            config.storage.cache_ttl_seconds = 1800; // 30 minutes
        }

        config
    }

    /// Get configuration recommendations based on current system
    pub fn get_recommendations() -> ConfigRecommendations {
        let info = smart_defaults::get_system_info();

        ConfigRecommendations {
            suggested_pool_size: smart_defaults::suggest_pool_size(),
            suggested_cache_size: smart_defaults::suggest_cache_size(),
            suggested_cache_ttl: smart_defaults::suggest_cache_ttl(),
            suggested_batch_size: smart_defaults::suggest_batch_size(),
            data_directory: smart_defaults::detect_data_directory(),
            cache_directory: smart_defaults::detect_cache_directory(),
            is_optimal_for_production: !info.is_ci && !info.is_development,
            memory_adequate: info.available_memory > 1024 * 1024 * 1024, // 1GB
            cpu_adequate: info.cpu_count >= 2,
        }
    }

    /// Validate if the current configuration is suitable for the environment
    pub fn validate_environment_fitness(config: &Config) -> ValidationResult {
        let info = smart_defaults::get_system_info();
        let mut result = ValidationResult::ok();
        let mut warnings = Vec::new();

        // Check cache size vs available memory
        let cache_memory_estimate = config.storage.max_episodes_cache * 1024; // Rough estimate in bytes
        if cache_memory_estimate > info.available_memory as usize {
            warnings.push(ValidationWarning {
                field: "storage.max_episodes_cache".to_string(),
                message: format!(
                    "Cache size estimate ({}MB) may exceed available memory ({}MB)",
                    cache_memory_estimate / (1024 * 1024),
                    info.available_memory / (1024 * 1024)
                ),
                suggestion: Some(format!(
                    "Consider reducing cache size to {}",
                    info.available_memory / (1024 * 1024)
                )),
            });
        }

        // Check pool size vs CPU cores
        if config.storage.pool_size > info.cpu_count * 4 {
            warnings.push(ValidationWarning {
                field: "storage.pool_size".to_string(),
                message: format!(
                    "Pool size ({}) may be too high for {} CPU cores",
                    config.storage.pool_size, info.cpu_count
                ),
                suggestion: Some(format!("Consider reducing to {}", info.cpu_count * 2)),
            });
        }

        // CI-specific checks
        if info.is_ci {
            if config.storage.max_episodes_cache > 200 {
                warnings.push(ValidationWarning {
                    field: "storage.max_episodes_cache".to_string(),
                    message: "CI environment detected - large cache may impact performance"
                        .to_string(),
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
}

/// Configuration recommendations for optimization
#[derive(Debug, Clone)]
pub struct ConfigRecommendations {
    pub suggested_pool_size: usize,
    pub suggested_cache_size: usize,
    pub suggested_cache_ttl: u64,
    pub suggested_batch_size: usize,
    pub data_directory: PathBuf,
    pub cache_directory: PathBuf,
    pub is_optimal_for_production: bool,
    pub memory_adequate: bool,
    pub cpu_adequate: bool,
}

/// Main configuration struct that aggregates all configuration sections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Database configuration for storage backends
    pub database: DatabaseConfig,
    /// Storage configuration for memory system
    pub storage: StorageConfig,
    /// CLI-specific settings and preferences
    pub cli: CliConfig,
}

/// Database configuration for storage backend setup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Turso database URL for remote storage
    pub turso_url: Option<String>,
    /// Turso authentication token for remote access
    pub turso_token: Option<String>,
    /// redb database path for local cache storage
    pub redb_path: Option<String>,
}

/// Storage configuration for memory system behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Maximum number of episodes to cache in memory
    pub max_episodes_cache: usize,
    /// Cache time-to-live in seconds
    pub cache_ttl_seconds: u64,
    /// Database connection pool size
    pub pool_size: usize,
}

/// CLI configuration for user interface preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    /// Default output format for CLI commands
    pub default_format: String,
    /// Enable progress bars for long-running operations
    pub progress_bars: bool,
    /// Batch size for bulk operations
    pub batch_size: usize,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            turso_url: smart_defaults::detect_turso_url(),
            turso_token: smart_defaults::detect_turso_token(),
            redb_path: Some(smart_defaults::detect_redb_path()),
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            max_episodes_cache: smart_defaults::suggest_cache_size(),
            cache_ttl_seconds: smart_defaults::suggest_cache_ttl(),
            pool_size: smart_defaults::suggest_pool_size(),
        }
    }
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            default_format: smart_defaults::detect_default_format(),
            progress_bars: !smart_defaults::get_system_info().is_ci, // Disable in CI
            batch_size: smart_defaults::suggest_batch_size(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            storage: StorageConfig::default(),
            cli: CliConfig::default(),
        }
    }
}

/// Configuration preset types for quick setup
#[derive(Debug, Clone)]
pub enum ConfigPreset {
    /// Minimal local development setup with SQLite and redb
    Local,
    /// Cloud setup with Turso and local cache
    Cloud,
    /// In-memory only setup for testing
    Memory,
    /// Custom configuration
    Custom,
}

impl ConfigPreset {
    /// Create a configuration from this preset
    pub fn create_config(&self) -> Config {
        match self {
            ConfigPreset::Local => {
                // Enhanced local development preset
                let info = smart_defaults::get_system_info();

                Config {
                    database: DatabaseConfig {
                        turso_url: None, // Local only
                        turso_token: None,
                        redb_path: Some(smart_defaults::detect_redb_path()),
                    },
                    storage: StorageConfig {
                        max_episodes_cache: if info.is_development { 500 } else { 1000 },
                        cache_ttl_seconds: if info.is_development { 1800 } else { 3600 },
                        pool_size: 5, // Conservative for local development
                    },
                    cli: CliConfig {
                        default_format: "human".to_string(),
                        progress_bars: true,
                        batch_size: if info.is_development { 50 } else { 100 },
                    },
                }
            }
            ConfigPreset::Cloud => {
                // Enhanced cloud production preset
                let info = smart_defaults::get_system_info();

                Config {
                    database: DatabaseConfig {
                        turso_url: smart_defaults::detect_turso_url()
                            .or(Some("file:./data/memory.db".to_string())),
                        turso_token: smart_defaults::detect_turso_token(),
                        redb_path: Some({
                            let mut cache_dir = smart_defaults::detect_cache_directory();
                            cache_dir.push("cache.redb");
                            cache_dir.to_string_lossy().to_string()
                        }),
                    },
                    storage: StorageConfig {
                        max_episodes_cache: std::cmp::min(
                            5000,
                            smart_defaults::suggest_cache_size(),
                        ),
                        cache_ttl_seconds: smart_defaults::suggest_cache_ttl(),
                        pool_size: smart_defaults::suggest_pool_size(),
                    },
                    cli: CliConfig {
                        default_format: "json".to_string(), // Machine-readable for automation
                        progress_bars: !info.is_ci,
                        batch_size: std::cmp::max(200, smart_defaults::suggest_batch_size()),
                    },
                }
            }
            ConfigPreset::Memory => Config {
                database: DatabaseConfig {
                    turso_url: None,
                    turso_token: None,
                    redb_path: Some(":memory:".to_string()),
                },
                storage: StorageConfig {
                    max_episodes_cache: 100, // Minimal for testing
                    cache_ttl_seconds: 300,  // 5 minutes
                    pool_size: 2,            // Minimal connections
                },
                cli: CliConfig {
                    default_format: "human".to_string(),
                    progress_bars: false, // Disable for tests
                    batch_size: 10,       // Small batches for testing
                },
            },
            ConfigPreset::Custom => Config::default(),
        }
    }
}

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
