use super::{ConfigPreset, ConfigWizard, Result, StorageConfig};
use dialoguer::Input;

/// Validation error messages for storage configuration
pub mod validation {
    pub const CACHE_SIZE_ZERO: &str = "Cache size must be greater than 0";
    pub const CACHE_SIZE_TOO_LARGE: &str = "Cache size too large (max: 100_000)";
    pub const TTL_ZERO: &str = "TTL must be greater than 0";
    pub const TTL_TOO_LONG: &str = "TTL too long (max: 86400 seconds / 24 hours)";
    pub const POOL_SIZE_ZERO: &str = "Pool size must be greater than 0";
    pub const POOL_SIZE_TOO_LARGE: &str = "Pool size too large (max: 200)";

    /// Validate cache size (max_episodes_cache)
    pub fn validate_cache_size(input: usize) -> std::result::Result<(), &'static str> {
        if input == 0 {
            return Err(CACHE_SIZE_ZERO);
        }
        if input > 100_000 {
            return Err(CACHE_SIZE_TOO_LARGE);
        }
        Ok(())
    }

    /// Validate cache TTL (cache_ttl_seconds)
    pub fn validate_cache_ttl(input: u64) -> std::result::Result<(), &'static str> {
        if input == 0 {
            return Err(TTL_ZERO);
        }
        if input > 86400 {
            return Err(TTL_TOO_LONG);
        }
        Ok(())
    }

    /// Validate pool size
    pub fn validate_pool_size(input: usize) -> std::result::Result<(), &'static str> {
        if input == 0 {
            return Err(POOL_SIZE_ZERO);
        }
        if input > 200 {
            return Err(POOL_SIZE_TOO_LARGE);
        }
        Ok(())
    }
}

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
                validation::validate_cache_size(*input)
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
                validation::validate_cache_ttl(*input)
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
                validation::validate_pool_size(*input)
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

#[cfg(test)]
mod storage_validation_tests {
    use super::validation::*;

    #[test]
    fn test_validate_cache_size_valid() {
        assert!(validate_cache_size(100).is_ok());
        assert!(validate_cache_size(1000).is_ok());
        assert!(validate_cache_size(100_000).is_ok());
    }

    #[test]
    fn test_validate_cache_size_zero() {
        let result = validate_cache_size(0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CACHE_SIZE_ZERO);
    }

    #[test]
    fn test_validate_cache_size_too_large() {
        let result = validate_cache_size(100_001);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CACHE_SIZE_TOO_LARGE);
    }

    #[test]
    fn test_validate_cache_ttl_valid() {
        assert!(validate_cache_ttl(300).is_ok());
        assert!(validate_cache_ttl(1800).is_ok());
        assert!(validate_cache_ttl(86400).is_ok());
    }

    #[test]
    fn test_validate_cache_ttl_zero() {
        let result = validate_cache_ttl(0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TTL_ZERO);
    }

    #[test]
    fn test_validate_cache_ttl_too_long() {
        let result = validate_cache_ttl(86401);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TTL_TOO_LONG);
    }

    #[test]
    fn test_validate_pool_size_valid() {
        assert!(validate_pool_size(5).is_ok());
        assert!(validate_pool_size(10).is_ok());
        assert!(validate_pool_size(200).is_ok());
    }

    #[test]
    fn test_validate_pool_size_zero() {
        let result = validate_pool_size(0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), POOL_SIZE_ZERO);
    }

    #[test]
    fn test_validate_pool_size_too_large() {
        let result = validate_pool_size(201);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), POOL_SIZE_TOO_LARGE);
    }
}
