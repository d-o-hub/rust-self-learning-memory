//! Default trait implementations for configuration structs

use super::defaults;
use super::structs::{CliConfig, Config, DatabaseConfig, EmbeddingsConfig, StorageConfig};

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            turso_url: defaults::detect_turso_url(),
            turso_token: defaults::detect_turso_token(),
            redb_path: Some(defaults::detect_redb_path()),
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            max_episodes_cache: defaults::suggest_cache_size(),
            cache_ttl_seconds: defaults::suggest_cache_ttl(),
            pool_size: defaults::suggest_pool_size(),
            quality_threshold: 0.7,
        }
    }
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            default_format: defaults::detect_default_format(),
            progress_bars: !defaults::get_system_info().is_ci, // Disable in CI
            batch_size: defaults::suggest_batch_size(),
        }
    }
}

impl Default for EmbeddingsConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default for backward compatibility
            provider: "local".to_string(),
            model: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            dimension: 384,
            api_key_env: None,
            base_url: None,
            similarity_threshold: 0.7,
            batch_size: 32,
            cache_embeddings: true,
            timeout_seconds: 30,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            storage: StorageConfig::default(),
            cli: CliConfig::default(),
            embeddings: EmbeddingsConfig::default(),
        }
    }
}
