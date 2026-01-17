use super::{ConfigPreset, ConfigWizard, Result};
use dialoguer::Select;

impl ConfigWizard {
    /// Choose configuration preset
    pub fn choose_preset(&self) -> Result<ConfigPreset> {
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
}
