//! Cache configuration types

use std::time::Duration;

/// Configuration for Turso storage caching
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Enable episode caching
    pub enable_episode_cache: bool,

    /// Enable pattern caching
    pub enable_pattern_cache: bool,

    /// Enable query result caching
    pub enable_query_cache: bool,

    /// Maximum number of cached episodes
    pub max_episodes: usize,

    /// Maximum number of cached patterns
    pub max_patterns: usize,

    /// Maximum number of cached query results
    pub max_query_results: usize,

    /// Default TTL for episodes
    pub episode_ttl: Duration,

    /// Default TTL for patterns
    pub pattern_ttl: Duration,

    /// Default TTL for query results
    pub query_ttl: Duration,

    /// Minimum TTL for adaptive cache
    pub min_ttl: Duration,

    /// Maximum TTL for adaptive cache
    pub max_ttl: Duration,

    /// Hot threshold for adaptive TTL (accesses to be considered "hot")
    pub hot_threshold: usize,

    /// Cold threshold for adaptive TTL
    pub cold_threshold: usize,

    /// TTL adaptation rate (0.0 - 1.0)
    pub adaptation_rate: f64,

    /// Enable background cleanup
    pub enable_background_cleanup: bool,

    /// Cleanup interval in seconds
    pub cleanup_interval_secs: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enable_episode_cache: true,
            enable_pattern_cache: true,
            enable_query_cache: true,
            max_episodes: 10_000,
            max_patterns: 5_000,
            max_query_results: 1_000,
            episode_ttl: Duration::from_secs(1800), // 30 minutes
            pattern_ttl: Duration::from_secs(3600), // 1 hour
            query_ttl: Duration::from_secs(300),    // 5 minutes
            min_ttl: Duration::from_secs(60),       // 1 minute
            max_ttl: Duration::from_secs(7200),     // 2 hours
            hot_threshold: 10,
            cold_threshold: 2,
            adaptation_rate: 0.25,
            enable_background_cleanup: true,
            cleanup_interval_secs: 60,
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Episode cache hits
    pub episode_hits: u64,

    /// Episode cache misses
    pub episode_misses: u64,

    /// Pattern cache hits
    pub pattern_hits: u64,

    /// Pattern cache misses
    pub pattern_misses: u64,

    /// Query cache hits
    pub query_hits: u64,

    /// Query cache misses
    pub query_misses: u64,

    /// Total cache evictions
    pub evictions: u64,

    /// Total cache expirations
    pub expirations: u64,
}

impl CacheStats {
    /// Calculate overall cache hit rate
    pub fn hit_rate(&self) -> f64 {
        let total_hits = self.episode_hits + self.pattern_hits + self.query_hits;
        let total_requests =
            total_hits + self.episode_misses + self.pattern_misses + self.query_misses;

        if total_requests == 0 {
            0.0
        } else {
            total_hits as f64 / total_requests as f64
        }
    }

    /// Calculate episode cache hit rate
    pub fn episode_hit_rate(&self) -> f64 {
        let total = self.episode_hits + self.episode_misses;
        if total == 0 {
            0.0
        } else {
            self.episode_hits as f64 / total as f64
        }
    }

    /// Calculate pattern cache hit rate
    pub fn pattern_hit_rate(&self) -> f64 {
        let total = self.pattern_hits + self.pattern_misses;
        if total == 0 {
            0.0
        } else {
            self.pattern_hits as f64 / total as f64
        }
    }

    /// Calculate query cache hit rate
    pub fn query_hit_rate(&self) -> f64 {
        let total = self.query_hits + self.query_misses;
        if total == 0 {
            0.0
        } else {
            self.query_hits as f64 / total as f64
        }
    }
}
