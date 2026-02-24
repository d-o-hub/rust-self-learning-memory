//! Connection-Aware Prepared Statement Cache
//!
//! This module provides a connection-aware prepared statement cache that:
//! - Tracks prepared statements per connection (using connection ID)
//! - Handles connection lifecycle (cleanup on connection close)
//! - Implements LRU eviction tracking
//! - Provides thread-safe operations
//! - Tracks cache statistics (hits, misses, evictions)
//!
//! ## Architecture
//!
//! The cache uses a two-level structure:
//! ```text
//! ConnectionId -> { SQL -> CachedStatementMetadata }
//! ```
//!
//! Note: Due to libsql::Statement not implementing Clone or Send, we cannot
//! actually cache the statement objects. Instead, we cache metadata about
//! prepared statements and track statistics. The real performance benefit
//! comes from SQLite's internal statement cache.
//!
//! Each connection has its own cache of prepared statement metadata.
//! When a connection is closed/returned to pool, its cache is cleared.

use crate::pool::ConnectionId;
#[path = "cache_types.rs"]
mod types;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, trace, warn};
use types::{CachedStatementMetadata, ConnectionCache};
pub use types::{PreparedCacheConfig, PreparedCacheStats};

/// Maximum age for cached statements before forced refresh (1 hour)
const MAX_STATEMENT_AGE: Duration = Duration::from_secs(3600);

// ConnectionId is imported from crate::pool::ConnectionId (type alias for u64)

/// Connection-aware prepared statement cache
///
/// This cache tracks prepared statements per connection, ensuring that
/// statement metadata is associated with the correct connection.
///
/// ## Thread Safety
///
/// The cache uses `RwLock` for interior mutability, allowing concurrent
/// reads and exclusive writes. All operations are thread-safe.
///
/// ## Connection Lifecycle
///
/// When a connection is returned to the pool or closed, its cache should
/// be cleared by calling `clear_connection()`.
pub struct PreparedStatementCache {
    /// The cache storage: ConnectionId -> ConnectionCache
    cache: RwLock<HashMap<ConnectionId, ConnectionCache>>,
    /// Cache configuration
    config: PreparedCacheConfig,
    /// Statistics
    stats: RwLock<PreparedCacheStats>,
}

impl PreparedStatementCache {
    /// Create a new connection-aware prepared statement cache
    ///
    /// # Arguments
    ///
    /// * `max_size` - Maximum number of statements to cache per connection
    ///
    /// # Example
    ///
    /// ```rust
    /// use memory_storage_turso::prepared::PreparedStatementCache;
    ///
    /// let cache = PreparedStatementCache::new(100);
    /// ```
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            config: PreparedCacheConfig {
                max_size,
                ..Default::default()
            },
            stats: RwLock::new(PreparedCacheStats::default()),
        }
    }

    /// Create a cache with custom configuration
    pub fn with_config(config: PreparedCacheConfig) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            config,
            stats: RwLock::new(PreparedCacheStats::default()),
        }
    }

    /// Get or create a connection ID for a connection
    ///
    /// This generates a unique ID for tracking the connection in the cache.
    /// The ID should be stored alongside the connection and used for all
    /// cache operations with that connection.
    pub fn get_connection_id(&self) -> ConnectionId {
        use std::sync::atomic::AtomicU64;
        use std::sync::atomic::Ordering;

        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        NEXT_ID.fetch_add(1, Ordering::Relaxed)
    }

    /// Record a cache hit for a statement
    ///
    /// This should be called when a statement is found in SQLite's internal cache
    /// or when the application determines a statement was reused.
    ///
    /// # Arguments
    ///
    /// * `conn_id` - Connection identifier
    /// * `sql` - SQL statement that was hit
    pub fn record_hit(&self, conn_id: ConnectionId, sql: &str) {
        let mut cache = self.cache.write();

        // Get or create connection cache
        let conn_cache = cache.entry(conn_id).or_insert_with(|| {
            debug!("Creating new connection cache for {:?}", conn_id);
            ConnectionCache::new()
        });

        // Update the statement metadata if it exists
        if let Some(stmt) = conn_cache.get(sql) {
            stmt.increment_use();
        }

        drop(cache);
        self.stats.write().record_hit();
    }

    /// Record a cache miss for a statement
    ///
    /// This should be called when a statement needs to be prepared.
    ///
    /// # Arguments
    ///
    /// * `conn_id` - Connection identifier
    /// * `sql` - SQL statement that was missed
    /// * `prepare_time_us` - Time taken to prepare the statement (microseconds)
    pub fn record_miss(&self, conn_id: ConnectionId, sql: &str, prepare_time_us: u64) {
        let mut cache = self.cache.write();

        // Check if we need to evict at connection level
        if cache.len() >= self.config.max_connections && !cache.contains_key(&conn_id) {
            self.evict_lru_connection(&mut cache);
        }

        // Get or create connection cache
        let conn_cache = cache.entry(conn_id).or_insert_with(|| {
            debug!("Creating new connection cache for {:?}", conn_id);
            ConnectionCache::new()
        });

        // Check if we need to evict at statement level
        if conn_cache.len() >= self.config.max_size && !conn_cache.statements.contains_key(sql) {
            if let Some(evicted) = conn_cache.evict_lru() {
                debug!("Evicted cached statement: {}", evicted);
                self.stats.write().record_eviction();
            }
        }

        // Insert the new statement metadata
        let metadata = CachedStatementMetadata::new(sql.to_string());
        conn_cache.insert(sql.to_string(), metadata);

        // Calculate sizes before dropping the lock
        let total_size = cache.values().map(|c| c.len()).sum();
        let connection_count = cache.len();

        drop(cache);

        // Update stats
        let mut stats = self.stats.write();
        stats.record_miss();
        stats.record_prepared(prepare_time_us);
        stats.update_size(total_size);
        stats.update_active_connections(connection_count);

        trace!("Recorded cache miss for SQL on {:?}: {}", conn_id, sql);
    }

    /// Check if a statement is cached for a connection
    ///
    /// # Arguments
    ///
    /// * `conn_id` - Connection identifier
    /// * `sql` - SQL statement to check
    ///
    /// # Returns
    ///
    /// true if the statement is cached and doesn't need refresh
    pub fn is_cached(&self, conn_id: ConnectionId, sql: &str) -> bool {
        let mut cache = self.cache.write();

        if let Some(conn_cache) = cache.get_mut(&conn_id) {
            if let Some(stmt) = conn_cache.get(sql) {
                return !stmt.needs_refresh(&self.config);
            }
        }

        false
    }

    /// Get a prepared statement or prepare it if not cached
    ///
    /// This is a convenience method that generates a new connection ID for each call.
    /// For proper connection-aware caching, use `get_connection_id()` and the
    /// connection-specific methods instead.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection to prepare on
    /// * `sql` - SQL statement to prepare
    ///
    /// # Returns
    ///
    /// The prepared statement
    ///
    /// # Errors
    ///
    /// Returns error if statement preparation fails
    pub async fn get_or_prepare(
        &self,
        conn: &libsql::Connection,
        sql: &str,
    ) -> Result<libsql::Statement, libsql::Error> {
        let conn_id = self.get_connection_id();

        // Check if this is a cache hit
        if self.is_cached(conn_id, sql) {
            self.record_hit(conn_id, sql);
        }

        // Prepare the statement
        let start = Instant::now();
        let stmt = conn.prepare(sql).await?;
        let prepare_time_us = start.elapsed().as_micros() as u64;

        // Record the miss (tracks metadata)
        self.record_miss(conn_id, sql, prepare_time_us);

        Ok(stmt)
    }

    /// Evict the least recently used connection cache
    fn evict_lru_connection(&self, cache: &mut HashMap<ConnectionId, ConnectionCache>) {
        if cache.is_empty() {
            return;
        }

        // Find the connection with the oldest last access time
        let mut oldest = None;
        let mut oldest_time = Instant::now();

        for (id, conn_cache) in cache.iter() {
            if conn_cache.last_accessed < oldest_time {
                oldest_time = conn_cache.last_accessed;
                oldest = Some(*id);
            }
        }

        if let Some(id) = oldest {
            if cache.remove(&id).is_some() {
                warn!(
                    "Evicted connection cache for {:?} (max connections exceeded)",
                    id
                );
                self.stats.write().record_connection_eviction();
            }
        }
    }

    /// Clear all cached statements for a specific connection
    ///
    /// This should be called when a connection is returned to the pool
    /// or closed to prevent memory leaks.
    ///
    /// # Arguments
    ///
    /// * `conn_id` - Connection identifier to clear
    ///
    /// # Returns
    ///
    /// Number of statements cleared
    pub fn clear_connection(&self, conn_id: ConnectionId) -> usize {
        let mut cache = self.cache.write();
        let cleared = if let Some(conn_cache) = cache.remove(&conn_id) {
            let count = conn_cache.len();
            debug!(
                "Cleared {} cached statements for connection {:?}",
                count, conn_id
            );
            count
        } else {
            0
        };

        // Update stats - calculate size while still holding the write lock to avoid deadlock
        let total_size = cache.values().map(|c| c.len()).sum();
        let active_connections = cache.len();
        drop(cache);
        let mut stats = self.stats.write();
        stats.update_size(total_size);
        stats.update_active_connections(active_connections);

        cleared
    }

    /// Clear all cached statements across all connections
    pub fn clear(&self) {
        let mut cache = self.cache.write();
        let total_statements: usize = cache.values().map(|c| c.len()).sum();
        cache.clear();

        let mut stats = self.stats.write();
        stats.update_size(0);
        stats.update_active_connections(0);

        debug!(
            "Cleared {} cached statements from {} connections",
            total_statements,
            cache.len()
        );
    }

    /// Get current cache statistics
    pub fn stats(&self) -> PreparedCacheStats {
        self.stats.read().clone()
    }

    /// Get current total cache size (across all connections)
    pub fn total_size(&self) -> usize {
        self.cache.read().values().map(|c| c.len()).sum()
    }

    /// Get cache size for a specific connection
    pub fn connection_size(&self, conn_id: ConnectionId) -> usize {
        self.cache
            .read()
            .get(&conn_id)
            .map(|c| c.len())
            .unwrap_or(0)
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.total_size() == 0
    }

    /// Get number of tracked connections
    pub fn connection_count(&self) -> usize {
        self.cache.read().len()
    }

    /// Remove a specific statement from a connection's cache
    pub fn remove(&self, conn_id: ConnectionId, sql: &str) -> bool {
        let mut cache = self.cache.write();
        let removed = if let Some(conn_cache) = cache.get_mut(&conn_id) {
            conn_cache.remove(sql)
        } else {
            false
        };

        if removed {
            // Calculate size while still holding the write lock to avoid deadlock
            let total_size = cache.values().map(|c| c.len()).sum();
            let active_connections = cache.len();
            drop(cache);
            let mut stats = self.stats.write();
            stats.update_size(total_size);
            stats.update_active_connections(active_connections);
        }

        removed
    }

    /// Clean up idle connection caches
    ///
    /// Removes connection caches that haven't been accessed for longer than the threshold.
    /// This helps prevent memory leaks from abandoned connection IDs.
    ///
    /// # Arguments
    ///
    /// * `max_idle_duration` - Maximum idle time before cleanup
    ///
    /// # Returns
    ///
    /// Number of connections cleaned up
    pub fn cleanup_idle_connections(&self, max_idle_duration: Duration) -> usize {
        let mut cache = self.cache.write();
        let mut to_remove = Vec::new();

        for (id, conn_cache) in cache.iter() {
            if conn_cache.idle_time() > max_idle_duration {
                to_remove.push(*id);
            }
        }

        let count = to_remove.len();
        for id in to_remove {
            cache.remove(&id);
            debug!("Cleaned up idle connection cache for {:?}", id);
        }

        if count > 0 {
            // Calculate size while still holding the write lock to avoid deadlock
            let total_size = cache.values().map(|c| c.len()).sum();
            let active_connections = cache.len();
            drop(cache);
            let mut stats = self.stats.write();
            stats.update_size(total_size);
            stats.update_active_connections(active_connections);
            stats.connection_evictions += count as u64;
        }

        count
    }
}

impl Default for PreparedStatementCache {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
#[path = "cache_tests.rs"]
mod tests;
