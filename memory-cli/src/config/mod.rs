//! Memory CLI Configuration Module
//!
//! This module provides a clean, modular configuration system for memory-cli
//! with enhanced validation, simplified setup, and interactive configuration.
//!
//! # Quick Start
//!
//! ```no_run
//! use memory_cli::config::*;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Simple local setup
//!     let (config, storage) = setup_local().await?;
//!     
//!     // Or use auto-detection
//!     let (config, storage) = setup_auto().await?;
//!     
//!     // Or run interactive wizard
//!     let config = quick_setup().await?;
//!     # Ok(())
//! }
//! ```
//!
//! # Features
//!
//! - **Modular Design**: Clean separation of concerns
//! - **Enhanced Validation**: Rich error messages and suggestions
//! - **Simple Mode**: One-call setup for common use cases
//! - **Configuration Wizard**: Interactive setup experience
//! - **Multiple Formats**: Support for TOML, JSON, YAML
//! - **Environment Detection**: Automatic configuration based on environment

pub mod loader;
pub mod progressive;
pub mod simple;
pub mod storage;
pub mod types;
pub mod validator;
pub mod wizard;

// Re-export main types and functions for convenient access
pub use types::{
    CliConfig, Config, ConfigPreset, DatabaseConfig, StorageConfig, ValidationError,
    ValidationResult, ValidationWarning,
};

pub use loader::{
    detect_format, load_config, load_config_verbose, load_config_with_format, ConfigFormat,
    ConfigWriter,
};

pub use validator::{
    format_validation_result, quick_validation_check, validate_cli_config, validate_config,
    validate_config_path, validate_database_config, validate_storage_config,
};

pub use storage::{initialize_storage, StorageInfo, StorageInitResult, StorageType};

pub use simple::{
    generate_template, setup_auto, setup_cloud, setup_from_file, setup_local, setup_memory,
    setup_with_overrides, EnvironmentCheck, ReadinessCheck, SimpleConfig,
};

pub use progressive::{
    recommend_mode, setup_quick_redb, ConfigurationMode, ModeRecommendation, SimpleSetup,
    UsagePattern,
};

pub use wizard::{quick_setup, show_template, ConfigWizard};

/// Load and validate configuration from file or use defaults
///
/// This is the main entry point for loading configuration with automatic
/// format detection and validation.
pub fn load_config_with_validation(
    path: Option<&std::path::Path>,
) -> Result<Config, anyhow::Error> {
    let config = load_config(path)?;
    let validation_result = validate_config(&config);

    if !validation_result.is_valid {
        let error_messages = validation_result
            .errors
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("\n");

        return Err(anyhow::anyhow!(
            "Configuration validation failed:\n{}",
            error_messages
        ));
    }

    if !validation_result.warnings.is_empty() {
        tracing::warn!(
            "Configuration has warnings:\n{}",
            validation_result
                .warnings
                .iter()
                .map(|w| w.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    Ok(config)
}

/// Load configuration and initialize storage in one call
///
/// This convenience function combines configuration loading and storage
/// initialization for a complete setup.
pub async fn load_and_init(
    path: Option<&std::path::Path>,
) -> Result<(Config, StorageInitResult), anyhow::Error> {
    let config = load_config_with_validation(path)?;
    let storage_result = initialize_storage(&config).await?;
    Ok((config, storage_result))
}

/// Validate existing configuration and return detailed results
///
/// This provides detailed validation results with errors and warnings
/// for better error reporting and debugging.
pub fn validate_detailed(config: &Config) -> ValidationResult {
    validate_config(config)
}

/// Create configuration writer for saving configurations
///
/// This allows saving configurations in various formats with automatic
/// directory creation and pretty formatting.
pub fn create_writer(config: Config) -> ConfigWriter {
    ConfigWriter::new(config)
}

/// Save configuration to file with automatic format detection
///
/// This convenience function saves a configuration to a file with
/// automatic format detection based on file extension.
pub fn save_config(config: &Config, path: &std::path::Path) -> Result<(), anyhow::Error> {
    let writer = create_writer(config.clone());
    writer.write_to_file(path)
}

/// Environment-based configuration loading
///
/// This function automatically selects the best configuration approach
/// based on environment variables and system capabilities.
pub async fn auto_configure() -> Result<(Config, StorageInitResult), anyhow::Error> {
    let environment_check = EnvironmentCheck::new();

    tracing::info!("Environment check results: {}", environment_check.summary());

    setup_with_environment(&environment_check).await
}

/// Internal function to setup based on environment check
async fn setup_with_environment(
    environment_check: &EnvironmentCheck,
) -> Result<(Config, StorageInitResult), anyhow::Error> {
    match environment_check.recommended_preset {
        ConfigPreset::Local => setup_local().await,
        ConfigPreset::Cloud => setup_cloud().await,
        ConfigPreset::Memory => setup_memory().await,
        ConfigPreset::Custom => {
            // For custom, try to load from file first
            let default_paths = [
                "memory-cli.toml",
                "memory-cli.json",
                "memory-cli.yaml",
                ".memory-cli.toml",
                ".memory-cli.json",
                ".memory-cli.yaml",
            ];

            for path_str in &default_paths {
                if std::path::Path::new(path_str).exists() {
                    match load_and_init(Some(std::path::Path::new(path_str))).await {
                        Ok(result) => {
                            tracing::info!("Loaded configuration from: {}", path_str);
                            return Ok(result);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to load {}: {}, trying next", path_str, e);
                            continue;
                        }
                    }
                }
            }

            // If no files found, use defaults
            setup_local().await
        }
    }
}

/// Check if configuration is ready for use
///
/// This performs a comprehensive check of the environment and
/// configuration to determine if memory-cli can run successfully.
pub async fn check_readiness() -> ReadinessCheck {
    let check = ReadinessCheck::new();

    if !check.is_ready {
        tracing::warn!("Environment readiness issues detected:\n{}", check.report());
    } else {
        tracing::info!("Environment is ready for memory-cli");
    }

    check
}

/// Convert old configuration format to new format
///
/// This helper function can be used to migrate from older configuration
/// formats if needed.
pub fn migrate_config(old_config: &Config) -> Config {
    // For now, just return the config as-is since we're maintaining
    // backward compatibility. In the future, this could implement
    // migrations from different configuration formats.
    old_config.clone()
}

/// Get configuration summary for logging and debugging
///
/// This provides a human-readable summary of the configuration
/// without exposing sensitive information.
pub fn get_config_summary(config: &Config) -> String {
    format!(
        "Config Summary:\n\
         - Database: {}\n\
         - Storage: {} episodes, {}s TTL, pool={}\n\
         - CLI: {} format, batch_size={}",
        match (&config.database.turso_url, &config.database.redb_path) {
            (Some(_), Some(_)) => "Turso + redb".to_string(),
            (Some(_), None) => "Turso only".to_string(),
            (None, Some(_)) => "redb only".to_string(),
            (None, None) => "No storage".to_string(),
        },
        config.storage.max_episodes_cache,
        config.storage.cache_ttl_seconds,
        config.storage.pool_size,
        config.cli.default_format,
        config.cli.batch_size
    )
}
