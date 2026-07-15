//! Core configuration structures

use serde::{Deserialize, Serialize};

/// Main configuration struct that aggregates all configuration sections.
///
/// All sections implement [`Default`] and accept partial TOML/JSON/YAML via
/// `#[serde(default)]` so users can start with a minimal file (issue #829):
///
/// ```toml
/// [database]
/// redb_path = "./data/memory.redb"
/// storage_mode = "local"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Database configuration for storage backends
    pub database: DatabaseConfig,
    /// Storage configuration for memory system
    pub storage: StorageConfig,
    /// CLI-specific settings and preferences
    pub cli: CliConfig,
    /// Embeddings configuration for semantic search
    pub embeddings: EmbeddingsConfig,
}

impl Config {
    /// Normalize cross-section aliases after deserializing a config file.
    ///
    /// Users often put `storage_mode` under `[storage]` (issue #832). The
    /// canonical field is `[database].storage_mode`; this copies the alias
    /// when the database field is unset.
    pub fn normalize_storage_mode(&mut self) {
        if self.database.storage_mode.is_none() {
            if let Some(mode) = self.storage.storage_mode.take() {
                self.database.storage_mode = Some(mode);
            }
        } else {
            // Prefer [database].storage_mode; drop the alias so show/init stay clean.
            self.storage.storage_mode = None;
        }
    }
}

/// Database configuration for storage backend setup
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DatabaseConfig {
    /// Turso database URL for remote storage
    pub turso_url: Option<String>,
    /// Turso authentication token for remote access
    pub turso_token: Option<String>,
    /// redb database path for local cache storage
    pub redb_path: Option<String>,
    /// Storage mode: "remote", "local", or "memory" (canonical location)
    pub storage_mode: Option<String>,
    /// Explicit database path for local Turso SQLite (`storage_mode = "local"`).
    /// Prefer `--db-path` / `MEMORY_DB_PATH` which also set `redb_path`.
    pub db_path: Option<String>,
}

/// Storage configuration for memory system behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StorageConfig {
    /// Maximum number of episodes to cache in memory
    pub max_episodes_cache: usize,
    /// Cache time-to-live in seconds
    pub cache_ttl_seconds: u64,
    /// Database connection pool size
    pub pool_size: usize,
    /// UX alias for [`DatabaseConfig::storage_mode`] (issue #832).
    /// Prefer setting `storage_mode` under `[database]`. Not serialized by
    /// `config init` / `show-template` so the canonical location stays clear.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage_mode: Option<String>,
}

/// CLI configuration for user interface preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CliConfig {
    /// Default output format for CLI commands
    pub default_format: String,
    /// Enable progress bars for long-running operations
    pub progress_bars: bool,
    /// Batch size for bulk operations
    pub batch_size: usize,
}

/// Embeddings configuration for semantic search
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EmbeddingsConfig {
    /// Enable semantic embeddings
    pub enabled: bool,
    /// Embedding provider: "local", "openai", "mistral", "azure", or "custom"
    pub provider: String,
    /// Model name or identifier
    pub model: String,
    /// Embedding dimension
    pub dimension: usize,
    /// API key environment variable (e.g., "OPENAI_API_KEY")
    pub api_key_env: Option<String>,
    /// Base URL for custom providers
    pub base_url: Option<String>,
    /// Similarity threshold for search (0.0 - 1.0)
    pub similarity_threshold: f32,
    /// Batch size for embedding generation
    pub batch_size: usize,
    /// Cache embeddings to avoid regeneration
    pub cache_embeddings: bool,
    /// Timeout for embedding requests (seconds)
    pub timeout_seconds: u64,
}
