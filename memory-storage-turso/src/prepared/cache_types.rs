//! Prepared statement cache types and metadata
//!
//! This module provides core types for prepared statement caching.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Maximum age for cached statements before forced refresh (1 hour)
pub const MAX_STATEMENT_AGE: Duration = Duration::from_secs(3600);

/// Configuration for prepared statement cache
#[derive(Debug, Clone)]
pub struct PreparedCacheConfig {
    /// Maximum number of statements to cache per connection
    pub max_size: usize,
    /// Enable automatic statement refresh
    pub enable_refresh: bool,
    /// Statement refresh threshold (number of uses)
    pub refresh_threshold: u64,
    /// Maximum number of connections to track
    pub max_connections: usize,
}

impl Default for PreparedCacheConfig {
    fn default() -> Self {
        Self {
            max_size: 100,
            enable_refresh: true,
            refresh_threshold: 1000,
            max_connections: 100,
        }
    }
}

/// Metadata for a cached prepared statement
pub struct CachedStatementMetadata {
    /// When the statement was first prepared
    pub prepared_at: Instant,
    /// Number of times this statement was used
    use_count: AtomicU64,
    /// The SQL string (for reference)
    pub sql: String,
}

impl Clone for CachedStatementMetadata {
    fn clone(&self) -> Self {
        Self {
            prepared_at: self.prepared_at,
            use_count: AtomicU64::new(self.use_count.load(Ordering::Relaxed)),
            sql: self.sql.clone(),
        }
    }
}

impl CachedStatementMetadata {
    /// Create new cached statement metadata
    pub fn new(sql: String) -> Self {
        Self {
            prepared_at: Instant::now(),
            use_count: AtomicU64::new(0),
            sql,
        }
    }

    /// Increment use count
    pub fn increment_use(&self) {
        self.use_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Get use count
    pub fn use_count(&self) -> u64 {
        self.use_count.load(Ordering::Relaxed)
    }

    /// Check if statement needs refresh
    pub fn needs_refresh(&self, config: &PreparedCacheConfig) -> bool {
        config.enable_refresh
            && (self.use_count() > config.refresh_threshold
                || self.prepared_at.elapsed() > MAX_STATEMENT_AGE)
    }
}

/// Per-connection statement cache with LRU eviction tracking
#[allow(dead_code)]
pub struct ConnectionCache {
    /// The statements metadata for this connection
    pub statements: HashMap<String, CachedStatementMetadata>,
    /// Access order for LRU eviction (most recent at end)
    pub access_order: Vec<String>,
    /// When this connection cache was created
    pub created_at: Instant,
    /// Last access time
    pub last_accessed: Instant,
}

impl ConnectionCache {
    /// Create a new connection cache
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            statements: HashMap::new(),
            access_order: Vec::new(),
            created_at: now,
            last_accessed: now,
        }
    }

    /// Get a statement from the cache
    pub fn get(&mut self, sql: &str) -> Option<&CachedStatementMetadata> {
        if let Some(stmt) = self.statements.get(sql) {
            // Update access order
            if let Some(pos) = self.access_order.iter().position(|s| s == sql) {
                let key = self.access_order.remove(pos);
                self.access_order.push(key);
            }
            self.last_accessed = Instant::now();
            Some(stmt)
        } else {
            None
        }
    }

    /// Insert a statement into the cache
    pub fn insert(&mut self, sql: String, stmt: CachedStatementMetadata) {
        self.statements.insert(sql.clone(), stmt);
        self.access_order.push(sql);
        self.last_accessed = Instant::now();
    }

    /// Remove a specific statement
    pub fn remove(&mut self, sql: &str) -> bool {
        if self.statements.remove(sql).is_some() {
            if let Some(pos) = self.access_order.iter().position(|s| s == sql) {
                self.access_order.remove(pos);
            }
            true
        } else {
            false
        }
    }

    /// Evict the least recently used statement
    pub fn evict_lru(&mut self) -> Option<String> {
        if self.access_order.is_empty() {
            return None;
        }

        let lru_key = self.access_order.remove(0);
        self.statements.remove(&lru_key);
        Some(lru_key)
    }

    /// Get the number of cached statements
    pub fn len(&self) -> usize {
        self.statements.len()
    }

    /// Check if cache is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.statements.is_empty()
    }

    /// Clear all statements
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.statements.clear();
        self.access_order.clear();
    }

    /// Get idle time
    pub fn idle_time(&self) -> Duration {
        self.last_accessed.elapsed()
    }
}

impl Default for ConnectionCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cached_statement_metadata() {
        let meta = CachedStatementMetadata::new("SELECT 1".to_string());
        assert_eq!(meta.use_count(), 0);
        meta.increment_use();
        assert_eq!(meta.use_count(), 1);
    }

    #[test]
    fn test_connection_cache_operations() {
        let mut cache = ConnectionCache::new();

        let stmt = CachedStatementMetadata::new("SELECT 1".to_string());
        cache.insert("SELECT 1".to_string(), stmt);

        assert!(cache.get("SELECT 1").is_some());
        assert!(cache.get("SELECT 2").is_none());
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_connection_cache_eviction() {
        let mut cache = ConnectionCache::new();

        cache.insert(
            "SELECT 1".to_string(),
            CachedStatementMetadata::new("SELECT 1".to_string()),
        );
        cache.insert(
            "SELECT 2".to_string(),
            CachedStatementMetadata::new("SELECT 2".to_string()),
        );

        // Access first one
        cache.get("SELECT 1");

        // Evict LRU (should be SELECT 2 since SELECT 1 was accessed more recently)
        let evicted = cache.evict_lru();
        assert_eq!(evicted, Some("SELECT 2".to_string()));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_prepared_cache_config_default() {
        let config = PreparedCacheConfig::default();
        assert_eq!(config.max_size, 100);
        assert!(config.enable_refresh);
        assert_eq!(config.refresh_threshold, 1000);
    }
}
