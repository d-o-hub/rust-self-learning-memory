use super::{ConfigPreset, ConfigWizard, DatabaseConfig, Result};
use dialoguer::{Confirm, Input};

impl ConfigWizard {
    /// Configure database settings
    #[allow(clippy::excessive_nesting)]
    pub fn configure_database(&self, preset: &ConfigPreset) -> Result<DatabaseConfig> {
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
                    if config
                        .turso_url
                        .as_ref()
                        .expect("turso_url is Some: just set on line 50")
                        .starts_with("libsql://")
                    {
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
    pub fn configure_database_with_defaults(
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
}
