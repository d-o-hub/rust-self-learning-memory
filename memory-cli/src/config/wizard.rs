//! Configuration wizard module
//!
//! This module provides an interactive configuration wizard that guides
//! users through setting up memory-cli with sensible defaults and validation.

use super::types::{CliConfig, Config, ConfigPreset, DatabaseConfig, StorageConfig};
use super::{generate_template, validate_config};
use anyhow::Context;
use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use std::path::Path;

/// Interactive configuration wizard
pub struct ConfigWizard {
    theme: ColorfulTheme,
}

impl ConfigWizard {
    /// Create a new configuration wizard
    pub fn new() -> Self {
        Self {
            theme: ColorfulTheme::default(),
        }
    }

    /// Run the interactive configuration wizard
    pub async fn run(&self) -> Result<Config> {
        println!("🚀 Memory CLI Configuration Wizard");
        println!("==================================");
        println!();

        // Step 1: Choose configuration preset
        let preset = self.choose_preset()?;

        // Step 2: Customize database configuration
        let database_config = self.configure_database(&preset)?;

        // Step 3: Customize storage configuration
        let storage_config = self.configure_storage(&preset)?;

        // Step 4: Customize CLI configuration
        let cli_config = self.configure_cli(&preset)?;

        // Step 5: Review and validate
        let config = Config {
            database: database_config,
            storage: storage_config,
            cli: cli_config,
        };

        self.review_and_validate(&config)?;

        Ok(config)
    }

    /// Run wizard with custom starting configuration
    pub async fn run_with_config(&self, initial_config: Config) -> Result<Config> {
        println!("🚀 Memory CLI Configuration Wizard");
        println!("==================================");
        println!();

        // Use initial config as starting point
        let mut database_config = initial_config.database;
        let mut storage_config = initial_config.storage;
        let mut cli_config = initial_config.cli;

        // Step 1: Customize database configuration
        database_config = self.configure_database_with_defaults(database_config)?;

        // Step 2: Customize storage configuration
        storage_config = self.configure_storage_with_defaults(storage_config)?;

        // Step 3: Customize CLI configuration
        cli_config = self.configure_cli_with_defaults(cli_config)?;

        let config = Config {
            database: database_config,
            storage: storage_config,
            cli: cli_config,
        };

        self.review_and_validate(&config)?;

        Ok(config)
    }

    /// Choose configuration preset
    fn choose_preset(&self) -> Result<ConfigPreset> {
        println!("Step 1: Choose a configuration preset");
        println!("This will determine sensible defaults for your use case.\n");

        let presets = vec![
            "Local Development (SQLite + redb)",
            "Cloud Setup (Remote DB + local cache)",
            "Memory Only (Testing, no persistence)",
            "Custom Configuration",
        ];

        let selection = Select::with_theme(&self.theme)
            .with_prompt("Select configuration preset")
            .items(&presets)
            .default(0)
            .interact()?;

        match selection {
            0 => Ok(ConfigPreset::Local),
            1 => Ok(ConfigPreset::Cloud),
            2 => Ok(ConfigPreset::Memory),
            3 => Ok(ConfigPreset::Custom),
            _ => Ok(ConfigPreset::Custom),
        }
    }

    /// Configure database settings
    fn configure_database(&self, preset: &ConfigPreset) -> Result<DatabaseConfig> {
        let mut config = preset.create_config().database;

        println!("\nStep 2: Database Configuration");
        println!("=============================");

        // Configure Turso URL
        if Confirm::with_theme(&self.theme)
            .with_prompt("Do you want to configure Turso remote database?")
            .default(false)
            .interact()?
        {
            let turso_url: String = Input::with_theme(&self.theme)
                .with_prompt("Turso database URL")
                .default("libsql://your-db.turso.io/db".to_string())
                .interact_text()?;

            config.turso_url = Some(turso_url);

            let turso_token: String = Input::with_theme(&self.theme)
                .with_prompt("Turso authentication token")
                .default("".to_string())
                .interact_text()?;

            config.turso_token = if turso_token.is_empty() {
                None
            } else {
                Some(turso_token)
            };
        }

        // Configure redb path
        let redb_path: String = Input::with_theme(&self.theme)
            .with_prompt("Local cache database path")
            .default(config.redb_path.clone().unwrap_or_default())
            .interact_text()?;

        config.redb_path = Some(redb_path);

        Ok(config)
    }

    /// Configure database with existing config as defaults
    fn configure_database_with_defaults(
        &self,
        mut config: DatabaseConfig,
    ) -> Result<DatabaseConfig> {
        println!("\nStep 2: Database Configuration");
        println!("=============================");

        // Configure Turso URL
        let configure_turso = if config.turso_url.is_some() {
            Confirm::with_theme(&self.theme)
                .with_prompt("Configure Turso remote database?")
                .default(true)
                .interact()?
        } else {
            Confirm::with_theme(&self.theme)
                .with_prompt("Do you want to configure Turso remote database?")
                .default(false)
                .interact()?
        };

        if configure_turso {
            let turso_url: String = Input::with_theme(&self.theme)
                .with_prompt("Turso database URL")
                .default(
                    config
                        .turso_url
                        .clone()
                        .unwrap_or_else(|| "libsql://your-db.turso.io/db".to_string()),
                )
                .interact_text()?;

            config.turso_url = Some(turso_url);

            let turso_token: String = Input::with_theme(&self.theme)
                .with_prompt("Turso authentication token (optional)")
                .default(config.turso_token.clone().unwrap_or_default())
                .interact_text()?;

            config.turso_token = if turso_token.is_empty() {
                None
            } else {
                Some(turso_token)
            };
        }

        // Configure redb path
        let redb_path: String = Input::with_theme(&self.theme)
            .with_prompt("Local cache database path")
            .default(config.redb_path.clone().unwrap_or_default())
            .interact_text()?;

        config.redb_path = Some(redb_path);

        Ok(config)
    }

    /// Configure storage settings
    fn configure_storage(&self, preset: &ConfigPreset) -> Result<StorageConfig> {
        let config = preset.create_config().storage;

        println!("\nStep 3: Storage Configuration");
        println!("============================");

        let max_episodes: usize = Input::with_theme(&self.theme)
            .with_prompt("Maximum episodes to cache")
            .default(config.max_episodes_cache)
            .interact_text()?;

        let cache_ttl: u64 = Input::with_theme(&self.theme)
            .with_prompt("Cache time-to-live (seconds)")
            .default(config.cache_ttl_seconds)
            .interact_text()?;

        let pool_size: usize = Input::with_theme(&self.theme)
            .with_prompt("Database connection pool size")
            .default(config.pool_size)
            .interact_text()?;

        Ok(StorageConfig {
            max_episodes_cache: max_episodes,
            cache_ttl_seconds: cache_ttl,
            pool_size,
        })
    }

    /// Configure storage with existing config as defaults
    fn configure_storage_with_defaults(&self, mut config: StorageConfig) -> Result<StorageConfig> {
        println!("\nStep 3: Storage Configuration");
        println!("============================");

        let max_episodes: usize = Input::with_theme(&self.theme)
            .with_prompt("Maximum episodes to cache")
            .default(config.max_episodes_cache)
            .interact_text()?;

        config.max_episodes_cache = max_episodes;

        let cache_ttl: u64 = Input::with_theme(&self.theme)
            .with_prompt("Cache time-to-live (seconds)")
            .default(config.cache_ttl_seconds)
            .interact_text()?;

        config.cache_ttl_seconds = cache_ttl;

        let pool_size: usize = Input::with_theme(&self.theme)
            .with_prompt("Database connection pool size")
            .default(config.pool_size)
            .interact_text()?;

        config.pool_size = pool_size;

        Ok(config)
    }

    /// Configure CLI settings
    fn configure_cli(&self, preset: &ConfigPreset) -> Result<CliConfig> {
        let config = preset.create_config().cli;

        println!("\nStep 4: CLI Configuration");
        println!("========================");

        let formats = vec!["human", "json", "yaml"];
        let format_selection = Select::with_theme(&self.theme)
            .with_prompt("Default output format")
            .items(&formats)
            .default(0)
            .interact()?;

        let default_format = formats[format_selection].to_string();

        let progress_bars = Confirm::with_theme(&self.theme)
            .with_prompt("Enable progress bars")
            .default(config.progress_bars)
            .interact()?;

        let batch_size: usize = Input::with_theme(&self.theme)
            .with_prompt("Default batch size")
            .default(config.batch_size)
            .interact_text()?;

        Ok(CliConfig {
            default_format,
            progress_bars,
            batch_size,
        })
    }

    /// Configure CLI with existing config as defaults
    fn configure_cli_with_defaults(&self, mut config: CliConfig) -> Result<CliConfig> {
        println!("\nStep 4: CLI Configuration");
        println!("========================");

        let formats = vec!["human", "json", "yaml"];
        let format_selection = Select::with_theme(&self.theme)
            .with_prompt("Default output format")
            .items(&formats)
            .default(
                formats
                    .iter()
                    .position(|&f| f == config.default_format)
                    .unwrap_or(0),
            )
            .interact()?;

        config.default_format = formats[format_selection].to_string();

        let progress_bars = Confirm::with_theme(&self.theme)
            .with_prompt("Enable progress bars")
            .default(config.progress_bars)
            .interact()?;

        config.progress_bars = progress_bars;

        let batch_size: usize = Input::with_theme(&self.theme)
            .with_prompt("Default batch size")
            .default(config.batch_size)
            .interact_text()?;

        config.batch_size = batch_size;

        Ok(config)
    }

    /// Review and validate final configuration
    fn review_and_validate(&self, config: &Config) -> Result<()> {
        println!("\nStep 5: Review Configuration");
        println!("============================");

        // Display configuration summary
        println!("Configuration Summary:");
        println!("---------------------");
        println!("Database:");
        println!(
            "  Turso URL: {}",
            config.database.turso_url.as_deref().unwrap_or("None")
        );
        println!(
            "  redb Path: {}",
            config.database.redb_path.as_deref().unwrap_or("None")
        );
        println!("Storage:");
        println!(
            "  Max Episodes Cache: {}",
            config.storage.max_episodes_cache
        );
        println!("  Cache TTL: {} seconds", config.storage.cache_ttl_seconds);
        println!("  Pool Size: {}", config.storage.pool_size);
        println!("CLI:");
        println!("  Default Format: {}", config.cli.default_format);
        println!("  Progress Bars: {}", config.cli.progress_bars);
        println!("  Batch Size: {}", config.cli.batch_size);
        println!();

        // Validate configuration
        let validation_result = validate_config(config);

        if !validation_result.is_valid {
            println!("⚠️  Configuration validation failed:");
            for error in &validation_result.errors {
                println!("  - {}", error);
            }
            println!();

            if Confirm::with_theme(&self.theme)
                .with_prompt("Configuration has errors. Continue anyway?")
                .default(false)
                .interact()?
            {
                return Ok(());
            } else {
                return Err(anyhow::anyhow!("Configuration validation failed"));
            }
        }

        if !validation_result.warnings.is_empty() {
            println!("⚠️  Configuration warnings:");
            for warning in &validation_result.warnings {
                println!("  - {}", warning);
            }
            println!();
        }

        println!("✅ Configuration validation passed!");

        // Ask if user wants to save configuration
        if Confirm::with_theme(&self.theme)
            .with_prompt("Save configuration to file?")
            .default(true)
            .interact()?
        {
            let save_path = self.choose_save_path()?;
            self.save_configuration(config, &save_path)?;
        }

        Ok(())
    }

    /// Choose where to save configuration
    fn choose_save_path(&self) -> Result<String> {
        let paths = vec![
            "memory-cli.toml (current directory)",
            ".memory-cli.toml (current directory)",
            "data/memory-cli.toml (data directory)",
            "Custom path",
        ];

        let selection = Select::with_theme(&self.theme)
            .with_prompt("Where to save configuration?")
            .items(&paths)
            .default(0)
            .interact()?;

        match selection {
            0 => Ok("memory-cli.toml".to_string()),
            1 => Ok(".memory-cli.toml".to_string()),
            2 => {
                // Ensure data directory exists
                std::fs::create_dir_all("data").ok();
                Ok("data/memory-cli.toml".to_string())
            }
            3 => {
                let custom_path: String = Input::with_theme(&self.theme)
                    .with_prompt("Enter custom path")
                    .interact_text()?;
                Ok(custom_path)
            }
            _ => Ok("memory-cli.toml".to_string()),
        }
    }

    /// Save configuration to file
    fn save_configuration(&self, config: &Config, path: &str) -> Result<()> {
        let toml = toml::to_string_pretty(config).context("Failed to serialize configuration")?;

        // Ensure parent directory exists
        if let Some(parent) = Path::new(path).parent() {
            std::fs::create_dir_all(parent).context("Failed to create directory")?;
        }

        std::fs::write(path, toml).context("Failed to write configuration file")?;

        println!("✅ Configuration saved to: {}", path);
        Ok(())
    }
}

impl Default for ConfigWizard {
    fn default() -> Self {
        Self::new()
    }
}

/// Quick wizard for common scenarios
pub async fn quick_setup() -> Result<Config> {
    let wizard = ConfigWizard::new();

    // Detect environment and suggest preset
    let environment_check = super::simple::EnvironmentCheck::new();
    let preset_suggestions = match environment_check.recommended_preset {
        ConfigPreset::Local => "Local Development",
        ConfigPreset::Cloud => "Cloud Setup",
        ConfigPreset::Memory => "Memory Only",
        ConfigPreset::Custom => "Custom",
    };

    println!("🔍 Environment Check Results:");
    println!("Recommended setup: {}", preset_suggestions);

    if !environment_check.warnings.is_empty() {
        println!("\nWarnings:");
        for warning in environment_check.warnings {
            println!("  - {}", warning);
        }
    }
    println!();

    wizard.run().await
}

/// Generate and display configuration template
pub fn show_template() -> Result<()> {
    let template = generate_template()?;
    println!("Configuration Template:");
    println!("======================");
    println!("{}", template);
    Ok(())
}
