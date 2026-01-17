use super::{CliConfig, ConfigPreset, ConfigWizard, Result};
use dialoguer::{Confirm, Input, Select};

impl ConfigWizard {
    /// Configure CLI settings
    pub fn configure_cli(&self, preset: &ConfigPreset) -> Result<CliConfig> {
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
    pub fn configure_cli_with_defaults(&self, mut config: CliConfig) -> Result<CliConfig> {
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
}
