//! Turso Configuration
//!
//! This module contains the TursoConfig struct and its implementation.

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
    /// Cache configuration for performance optimization
    /// When None, caching is disabled (default: Some(CacheConfig::default()))
    pub cache_config: Option<crate::cache::CacheConfig>,
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
            // Cache configuration (enabled by default)
            cache_config: Some(crate::cache::CacheConfig::default()),
        }
    }
}
