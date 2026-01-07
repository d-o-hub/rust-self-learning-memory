use super::{Config, ConfigPreset, ConfigWizard, Result};

/// Helper function to format duration in human-readable format
pub fn format_duration(seconds: u64) -> String {
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
    let environment_check = super::super::simple::EnvironmentCheck::new();
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

    let template = super::super::generate_template()?;
    println!("{}", template);

    println!("\n💡 Tips:");
    println!("   • Save as 'memory-cli.toml' in your project directory");
    println!("   • Adjust values based on your use case");
    println!("   • Run 'memory-cli config validate' to check your config");
    println!("   • Use the wizard for interactive setup: 'memory-cli config wizard'");

    Ok(())
}
