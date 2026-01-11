//! Configuration wizard module
//!
//! This module provides an interactive configuration wizard that guides
//! users through setting up memory-cli with sensible defaults and validation.

use super::types::{CliConfig, Config, ConfigPreset, DatabaseConfig, StorageConfig};
use super::validate_config;
use anyhow::Result;
use dialoguer::theme::ColorfulTheme;

mod cli;
mod database;
mod helpers;
mod presets;
mod save;
mod storage;
mod validation;

// These are used internally by the wizard module for code organization
#[allow(unused)]
pub use cli::*;

#[allow(unused)]
pub use database::*;

#[allow(unused)]
pub use helpers::*;

#[allow(unused)]
pub use presets::*;

#[allow(unused)]
pub use save::*;

#[allow(unused)]
pub use storage::*;

#[allow(unused)]
pub use validation::*;

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
    #[allow(clippy::excessive_nesting)]
    pub async fn run(&self) -> Result<Config> {
        println!("\n🚀 Memory CLI Configuration Wizard");
        println!("===================================");
        println!("This wizard will guide you through setting up memory-cli with optimal defaults.");
        println!("You can customize each setting or press Enter to accept recommended values.\n");

        // Step 1: Choose configuration preset
        println!("📋 Step 1 of 5: Configuration Preset");
        println!("────────────────────────────────────");
        let preset = self.choose_preset()?;

        // Step 2: Customize database configuration
        println!("\n💾 Step 2 of 5: Database Configuration");
        println!("───────────────────────────────────────");
        let database_config = self.configure_database(&preset)?;

        // Step 3: Customize storage configuration
        println!("\n⚙️  Step 3 of 5: Storage Configuration");
        println!("──────────────────────────────────────");
        let storage_config = self.configure_storage(&preset)?;

        // Step 4: Customize CLI configuration
        println!("\n🎨 Step 4 of 5: CLI Configuration");
        println!("──────────────────────────────────");
        let cli_config = self.configure_cli(&preset)?;

        // Step 5: Review and validate
        let config = Config {
            database: database_config,
            storage: storage_config,
            cli: cli_config,
            embeddings: crate::config::types::EmbeddingsConfig::default(),
        };

        println!("\n✅ Step 5 of 5: Review & Validate");
        println!("──────────────────────────────────");
        self.review_and_validate(&config)?;

        Ok(config)
    }

    /// Run wizard with custom starting configuration
    pub async fn run_with_config(&self, initial_config: Config) -> Result<Config> {
        println!("\n🚀 Memory CLI Configuration Wizard (Update Mode)");
        println!("═════════════════════════════════════════════════");
        println!("Updating existing configuration with new values.");
        println!("Press Enter to keep current values, or type new ones.\n");

        // Use initial config as starting point
        let mut database_config = initial_config.database;
        let mut storage_config = initial_config.storage;
        let mut cli_config = initial_config.cli;

        // Step 1: Customize database configuration
        println!("💾 Step 1 of 4: Database Configuration");
        println!("───────────────────────────────────────");
        database_config = self.configure_database_with_defaults(database_config)?;

        // Step 2: Customize storage configuration
        println!("\n⚙️  Step 2 of 4: Storage Configuration");
        println!("──────────────────────────────────────");
        storage_config = self.configure_storage_with_defaults(storage_config)?;

        // Step 3: Customize CLI configuration
        println!("\n🎨 Step 3 of 4: CLI Configuration");
        println!("──────────────────────────────────");
        cli_config = self.configure_cli_with_defaults(cli_config)?;

        let config = Config {
            database: database_config,
            storage: storage_config,
            cli: cli_config,
            embeddings: crate::config::types::EmbeddingsConfig::default(),
        };

        println!("\n✅ Step 4 of 4: Review & Validate");
        println!("──────────────────────────────────");
        self.review_and_validate(&config)?;

        Ok(config)
    }
}

impl Default for ConfigWizard {
    fn default() -> Self {
        Self::new()
    }
}
