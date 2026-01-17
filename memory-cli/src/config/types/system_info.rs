//! System information and smart configuration utilities

use std::path::PathBuf;

use super::defaults;
use super::Config;

/// System information for smart defaults
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub total_memory: u64,
    pub available_memory: u64,
    pub cpu_count: usize,
    pub is_ci: bool,
    pub is_development: bool,
}

/// Configuration recommendations for optimization
#[derive(Debug, Clone)]
pub struct ConfigRecommendations {
    pub suggested_pool_size: usize,
    pub suggested_cache_size: usize,
    pub suggested_cache_ttl: u64,
    pub suggested_batch_size: usize,
    pub data_directory: PathBuf,
    pub cache_directory: PathBuf,
    pub is_optimal_for_production: bool,
    pub memory_adequate: bool,
    pub cpu_adequate: bool,
}

/// Smart configuration utilities for advanced users
pub mod smart_config {
    use super::*;

    /// Create a configuration optimized for the current environment
    /// This is a more advanced version of Config::default() with additional context
    pub fn auto_detect_config() -> Config {
        let info = defaults::get_system_info();

        // Start with defaults and then apply smart enhancements
        let mut config = Config::default();

        // Apply environment-specific optimizations
        if info.is_ci {
            // CI optimizations
            config.cli.progress_bars = false;
            config.cli.batch_size = 10;
            config.storage.max_episodes_cache = 100;
        } else if info.is_development {
            // Development optimizations
            config.cli.default_format = "human".to_string();
            config.storage.cache_ttl_seconds = 1800; // 30 minutes
        }

        config
    }

    /// Get configuration recommendations based on current system
    pub fn get_recommendations() -> ConfigRecommendations {
        let info = defaults::get_system_info();

        ConfigRecommendations {
            suggested_pool_size: defaults::suggest_pool_size(),
            suggested_cache_size: defaults::suggest_cache_size(),
            suggested_cache_ttl: defaults::suggest_cache_ttl(),
            suggested_batch_size: defaults::suggest_batch_size(),
            data_directory: defaults::detect_data_directory(),
            cache_directory: defaults::detect_cache_directory(),
            is_optimal_for_production: !info.is_ci && !info.is_development,
            memory_adequate: info.available_memory > 1024 * 1024 * 1024, // 1GB
            cpu_adequate: info.cpu_count >= 2,
        }
    }
}
