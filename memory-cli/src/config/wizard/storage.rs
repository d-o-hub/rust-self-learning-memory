use super::{ConfigPreset, ConfigWizard, Result, StorageConfig};
use dialoguer::Input;

impl ConfigWizard {
    /// Configure storage settings
    pub fn configure_storage(&self, preset: &ConfigPreset) -> Result<StorageConfig> {
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
    pub fn configure_storage_with_defaults(
        &self,
        mut config: StorageConfig,
    ) -> Result<StorageConfig> {
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
}
