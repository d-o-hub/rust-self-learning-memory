use super::{Config, ConfigWizard, Result};
use anyhow::Context;
use dialoguer::{Input, Select};
use std::path::Path;

impl ConfigWizard {
    /// Choose where to save configuration
    pub fn choose_save_path(&self) -> Result<String> {
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
    pub fn save_configuration(&self, config: &Config, path: &str) -> Result<()> {
        println!("\n💾 Saving configuration...");

        // Serialize configuration
        let content = toml::to_string_pretty(config).context(
            "Failed to serialize configuration to TOML format. This is an internal error.",
        )?;

        // Ensure parent directory exists
        if let Some(parent) = Path::new(path).parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent).context(format!(
                "Failed to create directory '{}'. Check write permissions.",
                parent.display()
            ))?;
            println!("✓ Created directory: {}", parent.display());
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
