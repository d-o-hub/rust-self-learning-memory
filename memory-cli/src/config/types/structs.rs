//! Core configuration structures

use serde::{Deserialize, Serialize};

/// Main configuration struct that aggregates all configuration sections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Database configuration for storage backends
    pub database: DatabaseConfig,
    /// Storage configuration for memory system
    pub storage: StorageConfig,
    /// CLI-specific settings and preferences
    pub cli: CliConfig,
    /// Embeddings configuration for semantic search
    #[serde(default)]
    pub embeddings: EmbeddingsConfig,
}

/// Database configuration for storage backend setup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Turso database URL for remote storage
    pub turso_url: Option<String>,
    /// Turso authentication token for remote access
    pub turso_token: Option<String>,
    /// redb database path for local cache storage
    pub redb_path: Option<String>,
}

/// Storage configuration for memory system behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Maximum number of episodes to cache in memory
    pub max_episodes_cache: usize,
    /// Cache time-to-live in seconds
    pub cache_ttl_seconds: u64,
    /// Database connection pool size
    pub pool_size: usize,
}

/// CLI configuration for user interface preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
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
