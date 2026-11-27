//! Turso Configuration
//!
//! This module contains the TursoConfig struct and its implementation.

use crate::prepared::PreparedCacheConfig;
use std::path::PathBuf;

/// Storage mode for Turso storage
#[derive(Debug, Clone)]
pub enum StorageMode {
    /// Local file-based SQLite via libsql
    Local {
        /// Path to the SQLite database file
        path: PathBuf,
    },
    /// In-memory SQLite (tests, ephemeral agents)
    InMemory,
    /// Remote Turso Cloud / libSQL server
    Remote {
        /// Database URL
        url: String,
        /// Authentication token
        auth_token: String,
    },
}

impl Default for StorageMode {
    fn default() -> Self {
        // Default to local file-based SQLite using the workspace's default
        // database path. The previous default of `Remote { url: "", ... }`
        // silently produced an empty URL that failed at runtime; choosing
        // `Local` keeps `Default::default()` safe to use as a placeholder
        // and matches the new first-class local mode path.
        StorageMode::Local {
            path: do_memory_core::memory::default_db_path(),
        }
    }
}

/// Configuration for Turso storage
#[derive(Debug, Clone)]
pub struct TursoConfig {
    /// Maximum retry attempts for failed operations
    pub max_retries: u32,
    /// Base delay for exponential backoff (milliseconds)
    pub retry_base_delay_ms: u64,
    /// Maximum delay for exponential backoff (milliseconds)
    pub retry_max_delay_ms: u64,
    /// Enable connection pooling
    pub enable_pooling: bool,
    /// Enable keep-alive connection pool (reduces connection overhead)
    #[cfg(feature = "keepalive-pool")]
    pub enable_keepalive: bool,
    /// Keep-alive interval (seconds)
    #[cfg(feature = "keepalive-pool")]
    pub keepalive_interval_secs: u64,
    /// Stale connection threshold (seconds)
    #[cfg(feature = "keepalive-pool")]
    pub stale_threshold_secs: u64,
    /// Compression threshold in bytes (default: 1024 = 1KB)
    /// Payloads smaller than this won't be compressed
    /// Only used when compression feature is enabled
    pub compression_threshold: usize,
    /// Enable compression for episodes (default: true)
    /// Only used when compression feature is enabled
    pub compress_episodes: bool,
    /// Enable compression for patterns (default: true)
    /// Only used when compression feature is enabled
    pub compress_patterns: bool,
    /// Enable compression for embeddings (default: true)
    /// Only used when compression feature is enabled
    pub compress_embeddings: bool,
    /// Compression level for zstd (1-22, default: 3)
    /// 1 = fastest, 22 = best compression
    /// Only used when compression feature is enabled
    pub compression_level: i32,
    /// Enable transport compression for network operations (default: true)
    ///
    /// **Architectural Note**: This flag is reserved for future custom transport implementations.
    /// It is NOT currently wired into TursoStorage because libsql handles its own transport layer
    /// internally. Compression IS applied at the data layer (embedding storage) via the
    /// `compress()`/`decompress()` functions.
    ///
    /// For details, see `tests/runtime_wiring_transport_compression.rs` which documents:
    /// - CompressedTransport is a standalone utility for custom Transport implementations
    /// - TursoStorage uses libsql::Database directly, not the Transport trait
    /// - Data-layer compression (at embedding level) is the correct architectural approach
    #[cfg(feature = "compression")]
    pub enable_transport_compression: bool,
    /// Cache configuration for performance optimization
    /// When None, caching is disabled (default: Some(CacheConfig::default()))
    pub cache_config: Option<crate::cache::CacheConfig>,
    /// Prepared-statement cache configuration
    /// When None, [`PreparedCacheConfig::default`] applies (capacity 100)
    pub prepared_cache_config: Option<PreparedCacheConfig>,
}

impl Default for TursoConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_base_delay_ms: 100,
            retry_max_delay_ms: 5000,
            enable_pooling: true,
            #[cfg(feature = "keepalive-pool")]
            enable_keepalive: true,
            #[cfg(feature = "keepalive-pool")]
            keepalive_interval_secs: 30,
            #[cfg(feature = "keepalive-pool")]
            stale_threshold_secs: 60,
            // Compression settings (always present, only used when compression feature is enabled)
            compression_threshold: 1024,
            compress_episodes: true,
            compress_patterns: true,
            compress_embeddings: true,
            compression_level: 3, // Default zstd level (good balance of speed/ratio)
            #[cfg(feature = "compression")]
            enable_transport_compression: true,
            // Cache configuration (enabled by default)
            cache_config: Some(crate::cache::CacheConfig::default()),
            prepared_cache_config: Some(PreparedCacheConfig::default()),
        }
    }
}

impl TursoConfig {
    /// Prepared-statement cache configuration, falling back to
    /// [`PreparedCacheConfig::default`] when the field is `None`.
    pub fn prepared_cache_config_or_default(&self) -> PreparedCacheConfig {
        self.prepared_cache_config.clone().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TursoStorage;

    fn in_memory_config(prepared_cache_config: Option<PreparedCacheConfig>) -> TursoConfig {
        TursoConfig {
            // A single direct connection keeps the test off the pool path.
            enable_pooling: false,
            prepared_cache_config,
            ..TursoConfig::default()
        }
    }

    #[test]
    fn default_prepared_cache_capacity_is_100() {
        assert_eq!(
            TursoConfig::default()
                .prepared_cache_config_or_default()
                .max_size,
            100
        );
    }

    #[test]
    fn absent_prepared_cache_config_falls_back_to_default() {
        let config = in_memory_config(None);

        assert_eq!(
            config.prepared_cache_config_or_default().max_size,
            PreparedCacheConfig::default().max_size
        );
    }

    #[tokio::test]
    async fn storage_honours_configured_prepared_cache_capacity() {
        let config = in_memory_config(Some(PreparedCacheConfig {
            max_size: 7,
            ..Default::default()
        }));

        let storage = TursoStorage::with_config(":memory:", "", config)
            .await
            .expect("in-memory storage should build");

        assert_eq!(storage.prepared_cache.max_size(), 7);
    }

    #[tokio::test]
    async fn storage_defaults_prepared_cache_capacity_to_100() {
        let storage = TursoStorage::with_config(":memory:", "", in_memory_config(None))
            .await
            .expect("in-memory storage should build");

        assert_eq!(storage.prepared_cache.max_size(), 100);
    }
}
