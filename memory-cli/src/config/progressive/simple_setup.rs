//! Simple setup module.
//!
//! This module provides preset-based configuration with guided customization.

use crate::config::types::{Config, ConfigPreset};
use crate::config::{initialize_storage, validate_config};
use anyhow::Result;
use tracing::info;

/// Simple Mode: Clear preset selection with guided customization.
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
    /// Create a new simple setup with the specified preset.
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

    /// Apply custom configuration overrides.
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

    /// Skip validation for faster startup (advanced users only).
    ///
    /// ⚠️ **Warning**: Only use this if you understand the configuration
    /// requirements. Skipping validation may lead to runtime errors.
    pub fn skip_validation(mut self) -> Self {
        self.validate = false;
        self
    }

    /// Hide guidance messages (for automation/scripts).
    pub fn hide_guidance(mut self) -> Self {
        self.show_guidance = false;
        self
    }

    /// Build the configuration.
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

                tracing::warn!("Configuration warnings:\n{}", warning_summary);
            }
        }

        if self.show_guidance {
            info!("✅ Configuration built successfully");
        }

        Ok(config)
    }

    /// Build configuration and initialize storage.
    pub async fn build_and_init(
        self,
    ) -> Result<(Config, crate::config::storage::StorageInitResult)> {
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

    /// Get description of the current preset for user guidance.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_setup_preset() {
        let setup = SimpleSetup::preset(ConfigPreset::Local);
        let config = setup.build().await.unwrap();

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
}
