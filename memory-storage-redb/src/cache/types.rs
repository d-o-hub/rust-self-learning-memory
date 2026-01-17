//! Cache type definitions
//!
//! This module contains the core types used by the LRU cache:
//! - CacheConfig: Configuration settings
//! - CacheEntry: Individual cache entries with TTL
//! - CacheMetrics: Performance metrics tracking

use chrono::{DateTime, Duration, Utc};

/// Configuration for the LRU cache
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of items in cache
    pub max_size: usize,
    /// Default TTL in seconds (0 = no expiration)
    pub default_ttl_secs: u64,
    /// Background cleanup interval in seconds
    pub cleanup_interval_secs: u64,
    /// Enable background cleanup task
    pub enable_background_cleanup: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 1000,
            default_ttl_secs: 3600,     // 1 hour
            cleanup_interval_secs: 300, // 5 minutes
            enable_background_cleanup: true,
        }
    }
}

/// A cache entry with metadata
#[derive(Debug, Clone)]
pub(crate) struct CacheEntry {
    pub last_access: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub size_bytes: usize,
}

impl CacheEntry {
    /// Create a new cache entry
    pub fn new(ttl_secs: u64, size_bytes: usize) -> Self {
        let now = Utc::now();
        let expires_at = if ttl_secs > 0 {
            Some(now + Duration::seconds(ttl_secs as i64))
        } else {
            None
        };

        Self {
            last_access: now,
            expires_at,
            size_bytes,
        }
    }

    /// Check if entry has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() >= expires_at
        } else {
            false
        }
    }

    /// Update last access time
    pub fn touch(&mut self) {
        self.last_access = Utc::now();
    }
}

/// Cache performance metrics
#[derive(Debug, Clone, Default)]
pub struct CacheMetrics {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub expirations: u64,
    pub item_count: usize,
    pub total_size_bytes: usize,
    pub hit_rate: f64,
}

impl CacheMetrics {
    /// Calculate hit rate
    pub fn calculate_hit_rate(&mut self) {
        let total = self.hits + self.misses;
        self.hit_rate = if total > 0 {
            self.hits as f64 / total as f64
        } else {
            0.0
        };
    }

    /// Check if cache is effective (>40% hit rate)
    pub fn is_effective(&self) -> bool {
        self.hit_rate > 0.4
    }
}
