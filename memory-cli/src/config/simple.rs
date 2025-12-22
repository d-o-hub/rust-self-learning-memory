//! Simple configuration module
//!
//! This module provides simplified configuration setup for common use cases,
//! allowing users to get started quickly with minimal configuration complexity.

use super::types::{Config, ConfigPreset};
use super::{initialize_storage, load_config, validate_config};
use anyhow::Context;
use anyhow::Result;
use std::path::Path;

/// Simple configuration builder for common use cases
pub struct SimpleConfig {
    preset: ConfigPreset,
    custom_overrides: Option<Config>,
    config_path: Option<String>,
    validate: bool,
}

impl SimpleConfig {
    /// Create a new simple configuration builder
    pub fn new() -> Self {
        Self {
            preset: ConfigPreset::Local,
            custom_overrides: None,
            config_path: None,
            validate: true,
        }
    }

    /// Use a preset configuration
    pub fn preset(preset: ConfigPreset) -> Self {
        Self {
            preset,
            ..Default::default()
        }
    }

    /// Override with custom configuration
    pub fn override_with(mut self, config: Config) -> Self {
        self.custom_overrides = Some(config);
        self
    }

    /// Load configuration from file
    pub fn from_file<P: Into<String>>(path: P) -> Result<Self> {
        let path_str = path.into();
        let path = Path::new(&path_str);

        let config = load_config(Some(path))?;

        Ok(Self {
            preset: ConfigPreset::Custom,
            custom_overrides: Some(config),
            config_path: Some(path_str),
            validate: true,
        })
    }

    /// Skip validation for faster startup
    pub fn skip_validation(mut self) -> Self {
        self.validate = false;
        self
    }

    /// Build the configuration
    pub fn build(self) -> Result<Config> {
        let config = match self.custom_overrides {
            Some(custom) => custom,
            None => self.preset.create_config(),
        };

        // If validate is true, run validation and return detailed errors
        if self.validate {
            let validation_result = validate_config(&config);
            if !validation_result.is_valid {
                return Err(anyhow::anyhow!(
                    "Configuration validation failed:\n{}",
                    validation_result
                        .errors
                        .iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>()
                        .join("\n")
                ));
            }
        }

        Ok(config)
    }

    /// Build configuration and initialize storage in one call
    pub async fn build_and_init(self) -> Result<(Config, super::storage::StorageInitResult)> {
        let config = self.build()?;
        let storage_result = initialize_storage(&config).await?;
        Ok((config, storage_result))
    }
}

impl Default for SimpleConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Quick setup functions for common scenarios

/// Set up local development configuration
///
/// This creates a simple configuration suitable for local development
/// with SQLite storage and redb caching.
pub async fn setup_local() -> Result<(Config, super::storage::StorageInitResult)> {
    let simple_config = SimpleConfig::preset(ConfigPreset::Local);
    simple_config.build_and_init().await
}

/// Set up cloud configuration
///
/// This creates a configuration suitable for cloud deployment with
/// remote database and local caching.
pub async fn setup_cloud() -> Result<(Config, super::storage::StorageInitResult)> {
    let simple_config = SimpleConfig::preset(ConfigPreset::Cloud);
    simple_config.build_and_init().await
}

/// Set up in-memory configuration for testing
///
/// This creates a configuration suitable for testing with in-memory
/// storage only.
pub async fn setup_memory() -> Result<(Config, super::storage::StorageInitResult)> {
    let simple_config = SimpleConfig::preset(ConfigPreset::Memory);
    simple_config.build_and_init().await
}

/// Set up configuration from file with automatic detection
///
/// This loads configuration from a file with automatic format detection
/// and validation.
pub async fn setup_from_file<P: AsRef<Path>>(
    path: P,
) -> Result<(Config, super::storage::StorageInitResult)> {
    let simple_config = SimpleConfig::from_file(path.as_ref().display().to_string())?;
    simple_config.build_and_init().await
}

/// Set up configuration with custom overrides
///
/// This applies custom configuration overrides to a preset.
pub async fn setup_with_overrides(
    preset: ConfigPreset,
    overrides: Config,
) -> Result<(Config, super::storage::StorageInitResult)> {
    let simple_config = SimpleConfig::preset(preset).override_with(overrides);
    simple_config.build_and_init().await
}

/// Environment-based automatic configuration
///
/// This automatically selects the best configuration based on environment
/// variables and system capabilities.
pub async fn setup_auto() -> Result<(Config, super::storage::StorageInitResult)> {
    let mut config = Config::default();

    // Check for Turso environment variables
    if let (Ok(url), Ok(token)) = (std::env::var("TURSO_URL"), std::env::var("TURSO_TOKEN")) {
        config.database.turso_url = Some(url);
        config.database.turso_token = Some(token);

        // Check for custom redb path
        if let Ok(redb_path) = std::env::var("REDB_PATH") {
            config.database.redb_path = Some(redb_path);
        }

        // Use cloud preset for Turso configurations
        let simple_config = SimpleConfig::preset(ConfigPreset::Cloud).override_with(config);
        return simple_config.build_and_init().await;
    }

    // Check for local database configuration
    if let Ok(local_db) = std::env::var("LOCAL_DATABASE_URL") {
        config.database.turso_url = Some(local_db);

        // Use local preset for local database configurations
        let simple_config = SimpleConfig::preset(ConfigPreset::Local).override_with(config);
        return simple_config.build_and_init().await;
    }

    // Default to local configuration
    let simple_config = SimpleConfig::preset(ConfigPreset::Local);
    simple_config.build_and_init().await
}

/// Quick validation check for current environment
///
/// This performs a quick check to see what configuration would work
/// best in the current environment.
pub struct EnvironmentCheck {
    pub turso_available: bool,
    pub redb_available: bool,
    pub local_db_available: bool,
    pub recommended_preset: ConfigPreset,
    pub warnings: Vec<String>,
}

impl EnvironmentCheck {
    /// Perform environment check
    pub fn new() -> Self {
        let mut warnings = Vec::new();

        // Check for Turso configuration
        let turso_available =
            std::env::var("TURSO_URL").is_ok() && std::env::var("TURSO_TOKEN").is_ok();

        // Check for redb availability (feature flag)
        let redb_available = cfg!(feature = "redb");

        // Check for local database configuration
        let local_db_available = std::env::var("LOCAL_DATABASE_URL").is_ok();

        // Determine recommended preset
        let recommended_preset = if turso_available {
            ConfigPreset::Cloud
        } else if local_db_available {
            ConfigPreset::Local
        } else if redb_available {
            ConfigPreset::Local
        } else {
            ConfigPreset::Memory
        };

        // Add warnings for missing features
        if !turso_available && std::env::var("TURSO_URL").is_ok() {
            warnings.push(
                "TURSO_URL set but TURSO_TOKEN missing - authentication required".to_string(),
            );
        }

        if !redb_available {
            warnings
                .push("redb feature not enabled - cache performance may be reduced".to_string());
        }

        Self {
            turso_available,
            redb_available,
            local_db_available,
            recommended_preset,
            warnings,
        }
    }

    /// Get summary of environment capabilities
    pub fn summary(&self) -> String {
        let mut summary = String::new();
        summary.push_str(&format!("Turso available: {}\n", self.turso_available));
        summary.push_str(&format!("redb available: {}\n", self.redb_available));
        summary.push_str(&format!(
            "Local DB available: {}\n",
            self.local_db_available
        ));
        summary.push_str(&format!(
            "Recommended preset: {:?}\n",
            self.recommended_preset
        ));

        if !self.warnings.is_empty() {
            summary.push_str("\nWarnings:\n");
            for warning in &self.warnings {
                summary.push_str(&format!("  - {}\n", warning));
            }
        }

        summary
    }
}

impl Default for EnvironmentCheck {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate configuration template
///
/// This creates a configuration template file that users can customize.
pub fn generate_template() -> Result<String> {
    let template = Config::default();
    let toml =
        toml::to_string_pretty(&template).context("Failed to generate configuration template")?;

    Ok(format!(
        "# Memory CLI Configuration Template\n# Copy this to memory-cli.toml and customize as needed\n\n{}",
        toml
    ))
}

/// Validate environment readiness
///
/// This checks if the current environment is ready for memory-cli usage.
pub struct ReadinessCheck {
    pub is_ready: bool,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
}

impl ReadinessCheck {
    /// Perform readiness check
    pub fn new() -> Self {
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();

        // Check for required environment variables for Turso
        if std::env::var("TURSO_URL").is_ok() && std::env::var("TURSO_TOKEN").is_err() {
            issues.push("TURSO_URL is set but TURSO_TOKEN is missing".to_string());
            recommendations
                .push("Set TURSO_TOKEN environment variable for authentication".to_string());
        }

        // Check file permissions for common paths
        let common_paths = ["data/", "./", "/tmp/"];
        for path_str in &common_paths {
            let path = std::path::Path::new(path_str);
            if path.exists() && !path.read_dir().is_ok() {
                issues.push(format!("Cannot read directory: {}", path_str));
                recommendations.push(format!("Check permissions for directory: {}", path_str));
            }
        }

        // Check for Rust features
        if !cfg!(feature = "redb") && !cfg!(feature = "turso") {
            issues.push("No storage features enabled (redb, turso)".to_string());
            recommendations
                .push("Enable redb and/or turso features for storage functionality".to_string());
        }

        let is_ready = issues.is_empty();

        Self {
            is_ready,
            issues,
            recommendations,
        }
    }

    /// Get detailed readiness report
    pub fn report(&self) -> String {
        let mut report = String::new();

        if self.is_ready {
            report.push_str("✅ Environment is ready for memory-cli\n");
        } else {
            report.push_str("❌ Environment has issues that need to be resolved\n\n");

            report.push_str("Issues:\n");
            for issue in &self.issues {
                report.push_str(&format!("  - {}\n", issue));
            }

            if !self.recommendations.is_empty() {
                report.push_str("\nRecommendations:\n");
                for rec in &self.recommendations {
                    report.push_str(&format!("  - {}\n", rec));
                }
            }
        }

        report
    }
}

impl Default for ReadinessCheck {
    fn default() -> Self {
        Self::new()
    }
}
