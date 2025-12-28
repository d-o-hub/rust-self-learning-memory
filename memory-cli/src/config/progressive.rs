//! Progressive Configuration Modes
//!
//! This module implements progressive configuration modes that address the
//! "choice overload" issue by providing clear progression from simple to advanced.
//!
//! # Progressive Disclosure Design
//!
//! The system provides three distinct modes with clear progression:
//!
//! 1. **Ultra-Simple Mode (30-second setup)**: One function call for basic redb usage
//! 2. **Simple Mode (3-5 function calls)**: Clear preset selection with guided overrides  
//! 3. **Advanced Mode**: Interactive wizard for comprehensive setup
//!
//! # Usage Examples
//!
//! ## Ultra-Simple (30-second setup)
//!
//! ```no_run
//! use memory_cli::config::progressive::setup_quick_redb;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = setup_quick_redb().await?;
//!     println!("✅ Ready in under 30 seconds!");
//!     Ok(())
//! }
//! ```
//!
//! ## Simple Mode (Guided preset selection)
//!
//! ```no_run
//! use memory_cli::config::{SimpleSetup, ConfigPreset};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = SimpleSetup::preset(ConfigPreset::Local)
//!         .with_custom(|c| {
//!             // Easy customization
//!             c.storage.max_episodes_cache = 2000;
//!
//!             // Auto-validation with helpful error messages
//!         })
//!         .build()
//!         .await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Advanced Mode (Full configuration)
//!
//! ```no_run
//! use memory_cli::config::quick_setup;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = quick_setup().await?;
//!     Ok(())
//! }
//! ```

use super::types::{Config, ConfigPreset};
use super::{initialize_storage, validate_config, EnvironmentCheck};
use anyhow::{Context, Result};
use tracing::{info, warn};

/// Ultra-Simple Mode: 30-second setup for basic redb usage
///
/// This is the fastest way to get started with memory-cli. It automatically:
/// - Detects the best local storage path
/// - Creates a minimal configuration optimized for quick start
/// - Validates the environment and provides clear feedback
/// - Returns a ready-to-use configuration
///
/// # Success Criteria
/// - ✅ Setup time: < 30 seconds
/// - ✅ Clear success/error feedback
/// - ✅ No user decisions required
/// - ✅ Auto-detect optimal settings
///
/// # Example
///
/// ```no_run
/// use memory_cli::config::progressive::setup_quick_redb;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let config = setup_quick_redb().await?;
///     println!("🚀 Memory CLI ready for basic usage!");
///     Ok(())
/// }
/// ```
pub async fn setup_quick_redb() -> Result<Config> {
    info!("🚀 Starting 30-second quick setup for redb storage");

    // Step 1: Auto-detect environment and create minimal config
    let config = create_quick_config()?;

    // Step 2: Quick validation with helpful feedback
    let validation_result = validate_config(&config);

    if !validation_result.is_valid {
        let error_summary = validation_result
            .errors
            .iter()
            .map(|e| format!("  ❌ {}", e))
            .collect::<Vec<_>>()
            .join("\n");

        return Err(anyhow::anyhow!(
            "Quick setup failed - environment issues detected:\n{}\n\n💡 Tip: Run `setup_simple()` for guided configuration",
            error_summary
        ));
    }

    // Step 3: Provide clear success feedback
    if !validation_result.warnings.is_empty() {
        let warning_summary = validation_result
            .warnings
            .iter()
            .map(|w| format!("  ⚠️  {}", w))
            .collect::<Vec<_>>()
            .join("\n");

        warn!("Quick setup completed with warnings:\n{}", warning_summary);
    }

    info!("✅ Quick setup completed successfully - redb ready for use!");
    info!("📁 Data will be stored in: {:?}", config.database.redb_path);

    Ok(config)
}

/// Create minimal configuration optimized for quick start
fn create_quick_config() -> Result<Config> {
    // Use environment check to determine best approach
    let env_check = EnvironmentCheck::new();

    // For quick setup, prefer redb-only if available, otherwise use local preset
    let preset = if env_check.redb_available {
        ConfigPreset::Local
    } else {
        ConfigPreset::Memory // Fallback to in-memory if redb not available
    };

    let mut config = preset.create_config();

    // Optimize for quick start: smaller cache, shorter TTL, faster setup
    config.storage.max_episodes_cache = std::cmp::min(500, config.storage.max_episodes_cache);
    config.storage.cache_ttl_seconds = std::cmp::min(1800, config.storage.cache_ttl_seconds); // 30 min
    config.storage.pool_size = std::cmp::min(5, config.storage.pool_size);
    config.cli.batch_size = std::cmp::min(25, config.cli.batch_size);

    // Ensure we have a redb path for local storage
    if config.database.redb_path.is_none() {
        config.database.redb_path = Some(quick_redb_path_sync()?);
    }

    Ok(config)
}

/// Get a quick redb path for immediate use
fn quick_redb_path_sync() -> Result<String> {
    // Use a temporary directory that we can create quickly
    let temp_dir = std::env::temp_dir().join("memory-cli-quick");

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&temp_dir)
        .context("Failed to create temporary directory for quick setup")?;

    let redb_path = temp_dir.join("quick-setup.redb");
    Ok(redb_path.to_string_lossy().to_string())
}

/// Simple Mode: Clear preset selection with guided customization
///
/// This mode provides a guided experience for users who want more control
/// than ultra-simple mode but still want to avoid overwhelming choices.
///
/// # Features
/// - Clear preset descriptions and use cases
/// - Easy custom overrides with validation
/// - Helpful guidance and recommendations
/// - Progressive disclosure of advanced options
///
/// # Example
///
/// ```no_run
/// use memory_cli::config::{SimpleSetup, ConfigPreset};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let (config, storage) = SimpleSetup::preset(ConfigPreset::Cloud)
///         .with_custom(|c| {
///             // Easy customization with validation
///             c.storage.max_episodes_cache = 2000;
///             c.cli.default_format = "json".to_string();
///         })
///         .build_and_init()
///         .await?;
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct SimpleSetup {
    preset: ConfigPreset,
    custom_overrides: Option<Config>,
    validate: bool,
    show_guidance: bool,
}

impl SimpleSetup {
    /// Create a new simple setup with the specified preset
    ///
    /// Each preset comes with a clear description of when to use it:
    ///
    /// - **Local**: Best for development, testing, and personal use
    /// - **Cloud**: Best for production, sharing, and team collaboration  
    /// - **Memory**: Best for testing, temporary data, and CI/CD
    pub fn preset(preset: ConfigPreset) -> Self {
        Self {
            preset,
            custom_overrides: None,
            validate: true,
            show_guidance: true,
        }
    }

    /// Apply custom configuration overrides
    ///
    /// This allows easy customization of the preset with a closure.
    /// The configuration is validated after applying overrides.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use memory_cli::config::{SimpleSetup, ConfigPreset};
    /// # #[tokio::main]
    /// # async fn example() -> anyhow::Result<()> {
    /// let config = SimpleSetup::preset(ConfigPreset::Local)
    ///     .with_custom(|c| {
    ///         c.storage.max_episodes_cache = 1500;
    ///         c.cli.progress_bars = false;
    ///     })
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_custom<F>(mut self, customize: F) -> Self
    where
        F: FnOnce(&mut Config),
    {
        let mut config = self.preset.create_config();
        customize(&mut config);
        self.custom_overrides = Some(config);
        self
    }

    /// Skip validation for faster startup (advanced users only)
    ///
    /// ⚠️ **Warning**: Only use this if you understand the configuration
    /// requirements. Skipping validation may lead to runtime errors.
    pub fn skip_validation(mut self) -> Self {
        self.validate = false;
        self
    }

    /// Hide guidance messages (for automation/scripts)
    pub fn hide_guidance(mut self) -> Self {
        self.show_guidance = false;
        self
    }

    /// Build the configuration
    pub async fn build(self) -> Result<Config> {
        if self.show_guidance {
            info!("🎯 Building configuration with preset: {:?}", self.preset);
            info!("📋 Preset description: {}", self.preset_description());
        }

        let config = match self.custom_overrides {
            Some(custom) => {
                if self.show_guidance {
                    info!("🔧 Applying custom overrides");
                }
                custom
            }
            None => {
                if self.show_guidance {
                    info!("✨ Using preset defaults");
                }
                self.preset.create_config()
            }
        };

        if self.validate {
            if self.show_guidance {
                info!("🔍 Validating configuration");
            }

            let validation_result = validate_config(&config);
            if !validation_result.is_valid {
                let error_summary = validation_result
                    .errors
                    .iter()
                    .map(|e| format!("  ❌ {}", e))
                    .collect::<Vec<_>>()
                    .join("\n");

                return Err(anyhow::anyhow!(
                    "Configuration validation failed:\n{}\n\n💡 Tip: Check the configuration values or try a different preset",
                    error_summary
                ));
            }

            if self.show_guidance && !validation_result.warnings.is_empty() {
                let warning_summary = validation_result
                    .warnings
                    .iter()
                    .map(|w| format!("  ⚠️  {}", w))
                    .collect::<Vec<_>>()
                    .join("\n");

                warn!("Configuration warnings:\n{}", warning_summary);
            }
        }

        if self.show_guidance {
            info!("✅ Configuration built successfully");
        }

        Ok(config)
    }

    /// Build configuration and initialize storage
    pub async fn build_and_init(self) -> Result<(Config, super::storage::StorageInitResult)> {
        let show_guidance = self.show_guidance;
        let config = self.build().await?;
        let storage_result = initialize_storage(&config).await?;

        if show_guidance {
            info!("🚀 Storage initialized successfully");
            info!(
                "💾 Storage type: {:?}",
                storage_result.storage_info.primary_storage
            );
        }

        Ok((config, storage_result))
    }

    /// Get description of the current preset for user guidance
    fn preset_description(&self) -> String {
        match self.preset {
            ConfigPreset::Local => "Local development preset\n\
                 • Uses local redb storage\n\
                 • Optimized for development and testing\n\
                 • Good performance and reliability\n\
                 • Data stays on your machine"
                .to_string(),
            ConfigPreset::Cloud => "Cloud production preset\n\
                 • Uses Turso database with local redb cache\n\
                 • Optimized for production and team use\n\
                 • High performance and scalability\n\
                 • Data synced across devices"
                .to_string(),
            ConfigPreset::Memory => "Memory-only preset\n\
                 • Uses in-memory storage (no persistence)\n\
                 • Optimized for testing and temporary data\n\
                 • Fastest setup and usage\n\
                 • Data lost when program exits"
                .to_string(),
            ConfigPreset::Custom => "Custom configuration\n\
                 • Based on your specific requirements\n\
                 • Full control over all settings\n\
                 • May require more setup time\n\
                 • Use when presets don't fit your needs"
                .to_string(),
        }
    }
}

impl Default for SimpleSetup {
    fn default() -> Self {
        Self::preset(ConfigPreset::Local)
    }
}

/// Progressive disclosure helper: Check if current mode is appropriate
///
/// This function helps users understand if they should upgrade their
/// configuration mode based on their usage patterns and requirements.
pub struct ModeRecommendation {
    pub recommended_mode: ConfigurationMode,
    pub reasoning: String,
    pub upgrade_path: String,
}

/// Available configuration modes with clear progression
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigurationMode {
    /// Ultra-simple: 30-second setup, minimal decisions
    UltraSimple,
    /// Simple: Preset-based with guided customization
    Simple,
    /// Advanced: Full interactive wizard
    Advanced,
}

impl ConfigurationMode {
    /// Get description of this configuration mode
    pub fn description(&self) -> &'static str {
        match self {
            ConfigurationMode::UltraSimple => {
                "30-second setup for basic usage\n\
                 • One function call\n\
                 • Auto-detected settings\n\
                 • Minimal configuration\n\
                 • Best for: Quick testing, prototypes"
            }
            ConfigurationMode::Simple => {
                "Guided preset selection with customization\n\
                 • 3-5 function calls\n\
                 • Clear preset options\n\
                 • Easy customization\n\
                 • Best for: Development, specific use cases"
            }
            ConfigurationMode::Advanced => {
                "Full interactive configuration wizard\n\
                 • Comprehensive setup\n\
                 • All options available\n\
                 • Expert-level control\n\
                 • Best for: Production, complex requirements"
            }
        }
    }

    /// Get estimated setup time
    pub fn setup_time(&self) -> &'static str {
        match self {
            ConfigurationMode::UltraSimple => "30 seconds",
            ConfigurationMode::Simple => "2-5 minutes",
            ConfigurationMode::Advanced => "5-15 minutes",
        }
    }
}

/// Analyze current usage to recommend the best configuration mode
pub fn recommend_mode(usage_pattern: &UsagePattern) -> ModeRecommendation {
    match usage_pattern {
        UsagePattern::QuickTest => ModeRecommendation {
            recommended_mode: ConfigurationMode::UltraSimple,
            reasoning: "Quick testing detected - Ultra-Simple mode provides fastest setup".to_string(),
            upgrade_path: "Upgrade to Simple mode when you need custom settings".to_string(),
        },
        UsagePattern::Development => ModeRecommendation {
            recommended_mode: ConfigurationMode::Simple,
            reasoning: "Development usage detected - Simple mode offers best balance of convenience and control".to_string(),
            upgrade_path: "Upgrade to Advanced mode for production deployment".to_string(),
        },
        UsagePattern::Production => ModeRecommendation {
            recommended_mode: ConfigurationMode::Advanced,
            reasoning: "Production usage detected - Advanced mode provides comprehensive configuration".to_string(),
            upgrade_path: "Fine-tune settings using Simple mode for specific optimizations".to_string(),
        },
        UsagePattern::Testing => ModeRecommendation {
            recommended_mode: ConfigurationMode::UltraSimple,
            reasoning: "Testing workflow detected - Ultra-Simple mode minimizes setup overhead".to_string(),
            upgrade_path: "Use Simple mode for test configurations with specific requirements".to_string(),
        },
    }
}

/// Usage patterns to help determine appropriate configuration mode
#[derive(Debug, Clone)]
pub enum UsagePattern {
    /// Quick prototyping, experimentation, or one-time usage
    QuickTest,
    /// Ongoing development work with frequent iterations
    Development,
    /// Production deployment with reliability requirements
    Production,
    /// Automated testing, CI/CD, or batch processing
    Testing,
}

impl UsagePattern {
    /// Detect usage pattern from environment and usage hints
    pub fn detect() -> Self {
        // Check environment variables for hints
        if std::env::var("CI").is_ok() || std::env::var("GITHUB_ACTIONS").is_ok() {
            return UsagePattern::Testing;
        }

        if std::env::var("DEVELOPMENT").is_ok() || std::env::var("DEV").is_ok() {
            return UsagePattern::Development;
        }

        if std::env::var("PRODUCTION").is_ok() {
            return UsagePattern::Production;
        }

        // Default to development for unknown cases
        UsagePattern::Development
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_quick_redb_setup() {
        let config = setup_quick_redb().await.unwrap();

        assert!(config.database.redb_path.is_some());
        assert!(config.storage.max_episodes_cache > 0);
        assert!(config.storage.pool_size > 0);
    }

    #[tokio::test]
    async fn test_simple_setup_preset() {
        let setup = SimpleSetup::preset(ConfigPreset::Local);
        let config = setup.build().await.unwrap();

        // CI uses 100, development uses 500, production uses 1000
        let expected_values = [100, 500, 1000];
        assert!(
            expected_values.contains(&config.storage.max_episodes_cache),
            "max_episodes_cache should be one of {:?}, got {}",
            expected_values,
            config.storage.max_episodes_cache
        );
    }

    #[tokio::test]
    async fn test_simple_setup_custom_override() {
        let setup = SimpleSetup::preset(ConfigPreset::Local).with_custom(|c| {
            c.storage.max_episodes_cache = 2000;
        });

        let config = setup.build().await.unwrap();
        assert_eq!(config.storage.max_episodes_cache, 2000);
    }

    #[tokio::test]
    async fn test_mode_recommendation() {
        let recommendation = recommend_mode(&UsagePattern::Development);
        assert_eq!(recommendation.recommended_mode, ConfigurationMode::Simple);
        assert!(!recommendation.reasoning.is_empty());
    }

    #[test]
    fn test_usage_pattern_detection() {
        let pattern = UsagePattern::detect();
        // Should not panic and should return a valid pattern
        match pattern {
            UsagePattern::QuickTest
            | UsagePattern::Development
            | UsagePattern::Production
            | UsagePattern::Testing => {}
        }
    }
}
