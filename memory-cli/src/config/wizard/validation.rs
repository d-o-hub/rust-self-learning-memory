use super::{Config, ConfigWizard, Result};
use dialoguer::Confirm;

impl ConfigWizard {
    /// Review and validate final configuration
    pub fn review_and_validate(&self, config: &Config) -> Result<()> {
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
            super::format_duration(config.storage.cache_ttl_seconds)
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
        let validation_result = super::validate_config(config);

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
}
