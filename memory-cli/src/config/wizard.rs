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

    /// Choose configuration preset
    fn choose_preset(&self) -> Result<ConfigPreset> {
        println!("Choose a configuration preset to get started quickly.");
        println!("💡 Tip: Each preset provides optimized defaults for different use cases.\n");

        let presets = vec![
            "⭐ Local Development (Recommended) - SQLite + redb cache",
            "☁️  Cloud Setup - Remote Turso DB + local cache",
            "🧪 Memory Only - Testing/CI, no persistence",
            "⚙️  Custom Configuration - Full control",
        ];

        let selection = Select::with_theme(&self.theme)
            .with_prompt("Select configuration preset")
            .items(&presets)
            .default(0)
            .interact()?;

        let chosen_preset = match selection {
            0 => ConfigPreset::Local,
            1 => ConfigPreset::Cloud,
            2 => ConfigPreset::Memory,
            3 => ConfigPreset::Custom,
            _ => ConfigPreset::Custom,
        };

        // Show what this preset includes
        println!();
        match chosen_preset {
            ConfigPreset::Local => {
                println!("✓ Selected: Local Development");
                println!("  • Uses local SQLite database (file:./data/memory.db)");
                println!("  • Local redb cache for fast access");
                println!("  • Moderate cache size (1000 episodes)");
                println!("  • Perfect for development and testing");
            }
            ConfigPreset::Cloud => {
                println!("✓ Selected: Cloud Setup");
                println!("  • Uses remote Turso database");
                println!("  • Local redb cache for performance");
                println!("  • Large cache size (up to 5000 episodes)");
                println!("  • Optimized for production workloads");
            }
            ConfigPreset::Memory => {
                println!("✓ Selected: Memory Only");
                println!("  • In-memory storage only");
                println!("  • No persistent data (restarts clear all data)");
                println!("  • Minimal cache (100 episodes)");
                println!("  • Ideal for CI/CD and quick tests");
            }
            ConfigPreset::Custom => {
                println!("✓ Selected: Custom Configuration");
                println!("  • Full control over all settings");
                println!("  • You'll configure each option manually");
            }
        }

        Ok(chosen_preset)
    }

    /// Configure database settings
    fn configure_database(&self, preset: &ConfigPreset) -> Result<DatabaseConfig> {
        let mut config = preset.create_config().database;

        println!("Configure where your memory data will be stored.");
        println!("💡 Tip: You can use local storage, cloud storage, or both for redundancy.\n");

        // Configure Turso URL
        let default_turso = config.turso_url.is_some();
        if Confirm::with_theme(&self.theme)
            .with_prompt(if default_turso {
                "Configure Turso remote database? (Currently configured)"
            } else {
                "Do you want to configure Turso remote database? (Optional)"
            })
            .default(default_turso)
            .interact()?
        {
            println!("\n📡 Turso Database Setup");
            println!("   Example formats:");
            println!("   • libsql://your-database.turso.io/db  (Remote Turso)");
            println!("   • file:./data/memory.db               (Local SQLite)");

            let turso_url: String = Input::with_theme(&self.theme)
                .with_prompt("\n  Turso database URL")
                .default(
                    config
                        .turso_url
                        .clone()
                        .unwrap_or_else(|| "file:./data/memory.db".to_string()),
                )
                .validate_with(|input: &String| -> Result<(), &str> {
                    if input.trim().is_empty() {
                        return Err("URL cannot be empty");
                    }
                    if !input.starts_with("libsql://") && !input.starts_with("file:") {
                        return Err("URL must start with 'libsql://' or 'file:'");
                    }
                    if input.contains("..") {
                        return Err("Path traversal (..) is not allowed for security");
                    }
                    Ok(())
                })
                .interact_text()?;

            config.turso_url = Some(turso_url.clone());

            // Only ask for token if using remote Turso
            if turso_url.starts_with("libsql://") {
                println!("\n🔑 Authentication Token");
                println!("   Get your token from: https://turso.tech/");

                let turso_token: String = Input::with_theme(&self.theme)
                    .with_prompt("\n  Turso authentication token (or press Enter to skip)")
                    .default("".to_string())
                    .allow_empty(true)
                    .interact_text()?;

                config.turso_token = if turso_token.is_empty() {
                    if config.turso_url.as_ref().unwrap().starts_with("libsql://") {
                        println!(
                            "⚠️  Warning: Remote database without token - connection may fail!"
                        );
                    }
                    None
                } else {
                    Some(turso_token)
                };
            } else {
                println!("✓ Using local SQLite file - no authentication needed");
                config.turso_token = None;
            }
        }

        // Configure redb path
        println!("\n💾 Local Cache Configuration");
        println!("   The local cache provides fast access to recent episodes.");
        println!("   Example paths:");
        println!("   • ./data/cache.redb  (Recommended: Local file)");
        println!("   • :memory:           (In-memory only, no persistence)");

        let redb_path: String = Input::with_theme(&self.theme)
            .with_prompt("\n  Local cache database path")
            .default(
                config
                    .redb_path
                    .clone()
                    .unwrap_or_else(|| "./data/cache.redb".to_string()),
            )
            .validate_with(|input: &String| -> Result<(), &str> {
                if input.trim().is_empty() {
                    return Err("Path cannot be empty");
                }
                if input.contains("..") && input != ":memory:" {
                    return Err("Path traversal (..) is not allowed for security");
                }
                Ok(())
            })
            .interact_text()?;

        config.redb_path = Some(redb_path);

        println!("✓ Database configuration complete");
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

        println!("Configure how much data to cache and connection settings.");
        println!("💡 Tip: Larger cache = better performance, but uses more memory.\n");

        println!("📊 Cache Size Configuration");
        println!("   Recommended values:");
        println!("   • Testing/CI:    100-200 episodes   (~10MB memory)");
        println!("   • Development:   500-1000 episodes  (~50MB memory)");
        println!("   • Production:    1000-5000 episodes (~100-500MB memory)");

        let max_episodes: usize = Input::with_theme(&self.theme)
            .with_prompt(format!(
                "\n  Maximum episodes to cache (recommended: {})",
                config.max_episodes_cache
            ))
            .default(config.max_episodes_cache)
            .validate_with(|input: &usize| -> Result<(), &str> {
                if *input == 0 {
                    return Err("Cache size must be greater than 0");
                }
                if *input > 100000 {
                    return Err("Cache size too large (max: 100000)");
                }
                Ok(())
            })
            .interact_text()?;

        println!("\n⏰ Cache TTL (Time-To-Live)");
        println!("   How long cached episodes remain valid before refresh:");
        println!("   • Short (300s/5min):    Fresh data, more DB queries");
        println!("   • Medium (1800s/30min): Balanced (recommended for dev)");
        println!("   • Long (7200s/2hr):     Less queries (recommended for prod)");

        let cache_ttl: u64 = Input::with_theme(&self.theme)
            .with_prompt(format!(
                "\n  Cache time-to-live in seconds (recommended: {})",
                config.cache_ttl_seconds
            ))
            .default(config.cache_ttl_seconds)
            .validate_with(|input: &u64| -> Result<(), &str> {
                if *input == 0 {
                    return Err("TTL must be greater than 0");
                }
                if *input > 86400 {
                    return Err("TTL too long (max: 86400 seconds / 24 hours)");
                }
                Ok(())
            })
            .interact_text()?;

        println!("\n🔌 Connection Pool Size");
        println!("   Number of simultaneous database connections:");
        println!("   • Small (2-5):   Low concurrency, minimal resources");
        println!("   • Medium (5-10): Balanced (recommended for most uses)");
        println!("   • Large (10-20): High concurrency, more resources");

        let pool_size: usize = Input::with_theme(&self.theme)
            .with_prompt(format!(
                "\n  Database connection pool size (recommended: {})",
                config.pool_size
            ))
            .default(config.pool_size)
            .validate_with(|input: &usize| -> Result<(), &str> {
                if *input == 0 {
                    return Err("Pool size must be greater than 0");
                }
                if *input > 200 {
                    return Err("Pool size too large (max: 200)");
                }
                Ok(())
            })
            .interact_text()?;

        println!("✓ Storage configuration complete");
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

        println!("Configure how the CLI displays information and handles operations.");
        println!("💡 Tip: These settings affect the user interface, not functionality.\n");

        println!("🎨 Output Format");
        println!("   Choose how command results are displayed:");
        println!("   • human - Easy to read, colored output (recommended for interactive use)");
        println!("   • json  - Machine-readable, great for scripts and automation");
        println!("   • yaml  - Structured and readable, good for configs and logs");

        let formats = vec!["human (Recommended)", "json", "yaml"];
        let current_format_index = match config.default_format.as_str() {
            "json" => 1,
            "yaml" => 2,
            _ => 0, // default to human
        };

        let format_selection = Select::with_theme(&self.theme)
            .with_prompt("\n  Default output format")
            .items(&formats)
            .default(current_format_index)
            .interact()?;

        let default_format = match format_selection {
            0 => "human".to_string(),
            1 => "json".to_string(),
            2 => "yaml".to_string(),
            _ => "human".to_string(),
        };

        println!("\n📊 Progress Bars");
        println!("   Show progress bars for long-running operations?");
        println!("   • Yes: Visual feedback (recommended for interactive use)");
        println!("   • No:  Clean output (recommended for CI/scripts)");

        let progress_bars = Confirm::with_theme(&self.theme)
            .with_prompt("\n  Enable progress bars")
            .default(config.progress_bars)
            .interact()?;

        println!("\n📦 Batch Size");
        println!("   Number of items to process in a single batch operation:");
        println!("   • Small (10-50):    Safe, less memory, slower");
        println!("   • Medium (50-200):  Balanced (recommended)");
        println!("   • Large (200-1000): Fast, more memory");

        let batch_size: usize = Input::with_theme(&self.theme)
            .with_prompt(format!(
                "\n  Default batch size (recommended: {})",
                config.batch_size
            ))
            .default(config.batch_size)
            .validate_with(|input: &usize| -> Result<(), &str> {
                if *input == 0 {
                    return Err("Batch size must be greater than 0");
                }
                if *input > 10000 {
                    return Err("Batch size too large (max: 10000)");
                }
                Ok(())
            })
            .interact_text()?;

        println!("✓ CLI configuration complete");
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
        println!("Review your configuration before saving.\n");

        // Display configuration summary with visual indicators
        println!("📋 Configuration Summary");
        println!("════════════════════════");

        println!("\n💾 Database Configuration:");
        if let Some(turso_url) = &config.database.turso_url {
            let db_type = if turso_url.starts_with("libsql://") {
                "☁️  Remote Turso"
            } else if turso_url.starts_with("file:") {
                "📁 Local SQLite"
            } else {
                "❓ Unknown"
            };
            println!("  {} URL: {}", db_type, turso_url);
            if config.database.turso_token.is_some() && turso_url.starts_with("libsql://") {
                println!("  🔑 Token: ********** (configured)");
            } else if turso_url.starts_with("libsql://") {
                println!("  ⚠️  Token: Not configured (may cause connection issues)");
            }
        } else {
            println!("  ❌ No remote database configured");
        }

        if let Some(redb_path) = &config.database.redb_path {
            let cache_type = if redb_path == ":memory:" {
                "🧠 In-memory (no persistence)"
            } else {
                "💾 File-based cache"
            };
            println!("  {} Path: {}", cache_type, redb_path);
        } else {
            println!("  ❌ No local cache configured");
        }

        println!("\n⚙️  Storage Configuration:");
        println!(
            "  📊 Cache Size:     {} episodes (~{}MB memory)",
            config.storage.max_episodes_cache,
            (config.storage.max_episodes_cache / 10)
        ); // Rough estimate
        println!(
            "  ⏰ Cache TTL:      {} seconds ({})",
            config.storage.cache_ttl_seconds,
            format_duration(config.storage.cache_ttl_seconds)
        );
        println!(
            "  🔌 Pool Size:      {} connections",
            config.storage.pool_size
        );

        println!("\n🎨 CLI Configuration:");
        let format_icon = match config.cli.default_format.as_str() {
            "human" => "👤",
            "json" => "🤖",
            "yaml" => "📝",
            _ => "❓",
        };
        println!(
            "  {} Output Format:  {}",
            format_icon, config.cli.default_format
        );
        println!(
            "  📊 Progress Bars:  {}",
            if config.cli.progress_bars {
                "✓ Enabled"
            } else {
                "✗ Disabled"
            }
        );
        println!("  📦 Batch Size:     {} items", config.cli.batch_size);
        println!();

        // Validate configuration
        let validation_result = validate_config(config);

        if !validation_result.is_valid {
            println!("❌ Configuration Validation Failed");
            println!("══════════════════════════════════");
            println!("\nThe following errors must be fixed:\n");

            for (i, error) in validation_result.errors.iter().enumerate() {
                println!("{}. ❌ {}: {}", i + 1, error.field, error.message);
                if let Some(suggestion) = &error.suggestion {
                    println!("   💡 How to fix: {}", suggestion);
                }
                if let Some(context) = &error.context {
                    println!("   ℹ️  Context: {}", context);
                }
                println!();
            }

            if Confirm::with_theme(&self.theme)
                .with_prompt("⚠️  Configuration has errors. Do you want to continue anyway? (Not recommended)")
                .default(false)
                .interact()?
            {
                println!("⚠️  Continuing with invalid configuration - this may cause runtime errors!");
                return Ok(());
            } else {
                return Err(anyhow::anyhow!(
                    "Configuration validation failed. Please restart the wizard and fix the errors."
                ));
            }
        }

        if !validation_result.warnings.is_empty() {
            println!("⚠️  Configuration Warnings");
            println!("══════════════════════════");
            println!("\nThese won't prevent usage, but you may want to address them:\n");

            for (i, warning) in validation_result.warnings.iter().enumerate() {
                println!("{}. ⚠️  {}: {}", i + 1, warning.field, warning.message);
                if let Some(suggestion) = &warning.suggestion {
                    println!("   💡 Suggestion: {}", suggestion);
                }
                println!();
            }
        }

        println!("✅ Configuration Validation Passed!");
        println!("═══════════════════════════════════\n");

        // Ask for confirmation before saving
        if !Confirm::with_theme(&self.theme)
            .with_prompt("Does this configuration look correct?")
            .default(true)
            .interact()?
        {
            return Err(anyhow::anyhow!(
                "Configuration not confirmed. Please restart the wizard to make changes."
            ));
        }

        // Ask if user wants to save configuration
        if Confirm::with_theme(&self.theme)
            .with_prompt("Save this configuration to a file?")
            .default(true)
            .interact()?
        {
            let save_path = self.choose_save_path()?;
            self.save_configuration(config, &save_path)?;
        } else {
            println!("\n💡 Configuration not saved. You can:");
            println!("   • Use it for this session only");
            println!("   • Run the wizard again to save it later");
        }

        Ok(())
    }

    /// Choose where to save configuration
    fn choose_save_path(&self) -> Result<String> {
        println!("\n💾 Save Configuration");
        println!("Choose where to save your configuration file:\n");

        let paths = vec![
            "⭐ memory-cli.toml (Current directory - Recommended)",
            "🔒 .memory-cli.toml (Hidden file in current directory)",
            "📁 data/memory-cli.toml (Data directory)",
            "⚙️  Custom path (Specify your own location)",
        ];

        let selection = Select::with_theme(&self.theme)
            .with_prompt("Save location")
            .items(&paths)
            .default(0)
            .interact()?;

        match selection {
            0 => Ok("memory-cli.toml".to_string()),
            1 => Ok(".memory-cli.toml".to_string()),
            2 => {
                // Ensure data directory exists
                if let Err(e) = std::fs::create_dir_all("data") {
                    println!("⚠️  Warning: Could not create data directory: {}", e);
                }
                Ok("data/memory-cli.toml".to_string())
            }
            3 => {
                println!("\n💡 Tip: Use absolute paths or paths relative to current directory");
                println!("   Examples:");
                println!("   • ./config/memory.toml");
                println!("   • /etc/memory-cli/config.toml");
                println!("   • ~/.config/memory-cli.toml");

                let custom_path: String = Input::with_theme(&self.theme)
                    .with_prompt("\n  Enter custom path")
                    .validate_with(|input: &String| -> Result<(), &str> {
                        if input.trim().is_empty() {
                            return Err("Path cannot be empty");
                        }
                        if input.contains("..") {
                            return Err("Path traversal (..) not recommended");
                        }
                        // Check for valid file extension
                        if !input.ends_with(".toml")
                            && !input.ends_with(".json")
                            && !input.ends_with(".yaml")
                        {
                            return Err("File should have .toml, .json, or .yaml extension");
                        }
                        Ok(())
                    })
                    .interact_text()?;
                Ok(custom_path)
            }
            _ => Ok("memory-cli.toml".to_string()),
        }
    }

    /// Save configuration to file
    fn save_configuration(&self, config: &Config, path: &str) -> Result<()> {
        println!("\n💾 Saving configuration...");

        // Serialize configuration
        let content = toml::to_string_pretty(config).context(
            "Failed to serialize configuration to TOML format. This is an internal error.",
        )?;

        // Ensure parent directory exists
        if let Some(parent) = Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).context(format!(
                    "Failed to create directory '{}'. Check write permissions.",
                    parent.display()
                ))?;
                println!("✓ Created directory: {}", parent.display());
            }
        }

        // Write configuration file
        std::fs::write(path, &content).context(format!(
            "Failed to write configuration to '{}'. Check write permissions and disk space.",
            path
        ))?;

        println!("✅ Configuration successfully saved to: {}", path);
        println!("\n💡 Next steps:");
        println!("   • Test your configuration: memory-cli --config {}", path);
        println!("   • Edit manually if needed: {}", path);
        println!("   • Run the wizard again to update: memory-cli config wizard");

        Ok(())
    }
}

impl Default for ConfigWizard {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to format duration in human-readable format
fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        let mins = seconds / 60;
        let secs = seconds % 60;
        if secs == 0 {
            format!("{}min", mins)
        } else {
            format!("{}min {}s", mins, secs)
        }
    } else {
        let hours = seconds / 3600;
        let mins = (seconds % 3600) / 60;
        if mins == 0 {
            format!("{}hr", hours)
        } else {
            format!("{}hr {}min", hours, mins)
        }
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

    println!("\n🔍 Environment Detection");
    println!("═══════════════════════");
    println!("Recommended setup: {} ⭐", preset_suggestions);

    if !environment_check.warnings.is_empty() {
        println!("\n⚠️  Environment Warnings:");
        for warning in environment_check.warnings {
            println!("  • {}", warning);
        }
    }
    println!();

    wizard.run().await
}

/// Generate and display configuration template
pub fn show_template() -> Result<()> {
    println!("\n📄 Memory CLI Configuration Template");
    println!("════════════════════════════════════");
    println!("Copy and customize this template for your needs.\n");

    let template = generate_template()?;
    println!("{}", template);

    println!("\n💡 Tips:");
    println!("   • Save as 'memory-cli.toml' in your project directory");
    println!("   • Adjust values based on your use case");
    println!("   • Run 'memory-cli config validate' to check your config");
    println!("   • Use the wizard for interactive setup: 'memory-cli config wizard'");

    Ok(())
}
