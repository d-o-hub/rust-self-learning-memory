//! Configuration preset implementations

use super::defaults;
use super::structs::{CliConfig, Config, DatabaseConfig, EmbeddingsConfig, StorageConfig};

/// Configuration preset types for quick setup
#[derive(Debug, Clone)]
pub enum ConfigPreset {
    /// Minimal local development setup with SQLite and redb
    Local,
    /// Cloud setup with Turso and local cache
    Cloud,
    /// In-memory only setup for testing
    Memory,
    /// Custom configuration
    Custom,
}

impl ConfigPreset {
    /// Create a configuration from this preset
    pub fn create_config(&self) -> Config {
        match self {
            ConfigPreset::Local => {
                // Enhanced local development preset with SQLite + redb
                let info = defaults::get_system_info();

                Config {
                    database: DatabaseConfig {
                        turso_url: Some("file:./data/memory.db".to_string()), // SQLite support
                        turso_token: None,
                        redb_path: Some(if info.is_ci {
                            ":memory:".to_string()
                        } else {
                            "./data/cache.redb".to_string()
                        }), // CI uses in-memory
                    },
                    storage: StorageConfig {
                        max_episodes_cache: if info.is_ci {
                            100
                        } else if info.is_development {
                            500
                        } else {
                            1000
                        },
                        cache_ttl_seconds: if info.is_development { 1800 } else { 3600 },
                        pool_size: 5, // Conservative for local development
                        quality_threshold: 0.7,
                    },
                    cli: CliConfig {
                        default_format: if info.is_ci {
                            "json".to_string()
                        } else {
                            "human".to_string()
                        },
                        progress_bars: !info.is_ci,
                        batch_size: if info.is_ci {
                            10
                        } else if info.is_development {
                            50
                        } else {
                            100
                        },
                    },
                    embeddings: EmbeddingsConfig::default(),
                }
            }
            ConfigPreset::Cloud => {
                // Enhanced cloud production preset
                let info = defaults::get_system_info();

                Config {
                    database: DatabaseConfig {
                        turso_url: defaults::detect_turso_url()
                            .or(Some("file:./data/memory.db".to_string())),
                        turso_token: defaults::detect_turso_token(),
                        redb_path: Some("./data/cache.redb".to_string()), // Consistent path
                    },
                    storage: StorageConfig {
                        max_episodes_cache: std::cmp::min(5000, defaults::suggest_cache_size()),
                        cache_ttl_seconds: defaults::suggest_cache_ttl(),
                        pool_size: defaults::suggest_pool_size(),
                        quality_threshold: 0.7,
                    },
                    cli: CliConfig {
                        default_format: "json".to_string(), // Machine-readable for automation
                        progress_bars: !info.is_ci,
                        batch_size: std::cmp::max(200, defaults::suggest_batch_size()),
                    },
                    embeddings: EmbeddingsConfig::default(),
                }
            }
            ConfigPreset::Memory => Config {
                database: DatabaseConfig {
                    turso_url: None,
                    turso_token: None,
                    redb_path: Some(":memory:".to_string()),
                },
                storage: StorageConfig {
                    max_episodes_cache: 100, // Minimal for testing
                    cache_ttl_seconds: 300,  // 5 minutes
                    pool_size: 2,            // Minimal connections
                    quality_threshold: 0.0,
                },
                cli: CliConfig {
                    default_format: "human".to_string(),
                    progress_bars: false, // Disable for tests
                    batch_size: 10,       // Small batches for testing
                },
                embeddings: EmbeddingsConfig::default(),
            },
            ConfigPreset::Custom => Config::default(),
        }
    }
}
