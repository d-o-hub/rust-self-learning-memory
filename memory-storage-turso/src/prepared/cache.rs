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

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tracing::{debug, trace, warn};

/// Maximum age for cached statements before forced refresh (1 hour)
const MAX_STATEMENT_AGE: Duration = Duration::from_secs(3600);

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

/// Statistics for prepared statement cache operations
#[derive(Debug, Default, Clone)]
pub struct PreparedCacheStats {
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Total statements prepared
    pub prepared: u64,
    /// Total evictions
    pub evictions: u64,
    /// Current total cache size (across all connections)
    pub current_size: usize,
    /// Maximum cache size reached
    pub max_size_reached: usize,
    /// Total refreshes performed
    pub refreshes: u64,
    /// Total time spent preparing statements (microseconds)
    pub preparation_time_us: u64,
    /// Average preparation time (microseconds)
    pub avg_preparation_time_us: f64,
    /// Number of active connections being tracked
    pub active_connections: usize,
    /// Number of connection evictions (when max_connections exceeded)
    pub connection_evictions: u64,
}

impl PreparedCacheStats {
    /// Calculate cache hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Record a cache hit
    pub fn record_hit(&mut self) {
        self.hits += 1;
    }

    /// Record a cache miss
    pub fn record_miss(&mut self) {
        self.misses += 1;
    }

    /// Record a prepared statement
    pub fn record_prepared(&mut self, time_us: u64) {
        self.prepared += 1;
        self.preparation_time_us += time_us;
        self.avg_preparation_time_us = self.preparation_time_us as f64 / self.prepared as f64;
    }

    /// Record an eviction
    pub fn record_eviction(&mut self) {
        self.evictions += 1;
    }

    /// Update cache size
    pub fn update_size(&mut self, size: usize) {
        self.current_size = size;
        if size > self.max_size_reached {
            self.max_size_reached = size;
        }
    }

    /// Record a refresh
    pub fn record_refresh(&mut self) {
        self.refreshes += 1;
    }

    /// Update active connection count
    pub fn update_active_connections(&mut self, count: usize) {
        self.active_connections = count;
    }

    /// Record a connection eviction
    pub fn record_connection_eviction(&mut self) {
        self.connection_evictions += 1;
    }
}

/// Unique identifier for a database connection
///
/// This is a wrapper type that provides a unique, hashable identifier
/// for tracking connections in the cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionId(u64);

impl ConnectionId {
    /// Generate a unique connection ID
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    /// Create from a raw value (for testing)
    #[cfg(test)]
    pub fn from_raw(id: u64) -> Self {
        Self(id)
    }
}

impl Default for ConnectionId {
    fn default() -> Self {
        Self::new()
    }
}

/// Metadata for a cached prepared statement
struct CachedStatementMetadata {
    /// When the statement was first prepared
    prepared_at: Instant,
    /// Number of times this statement was used
    use_count: AtomicU64,
    /// The SQL string (for reference)
    sql: String,
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
    fn new(sql: String) -> Self {
        Self {
            prepared_at: Instant::now(),
            use_count: AtomicU64::new(0),
            sql,
        }
    }

    /// Increment use count
    fn increment_use(&self) {
        self.use_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Get use count
    fn use_count(&self) -> u64 {
        self.use_count.load(Ordering::Relaxed)
    }

    /// Check if statement needs refresh
    fn needs_refresh(&self, config: &PreparedCacheConfig) -> bool {
        config.enable_refresh
            && (self.use_count() > config.refresh_threshold
                || self.prepared_at.elapsed() > MAX_STATEMENT_AGE)
    }
}

/// Per-connection statement cache with LRU eviction tracking
struct ConnectionCache {
    /// The statements metadata for this connection
    statements: HashMap<String, CachedStatementMetadata>,
    /// Access order for LRU eviction (most recent at end)
    access_order: Vec<String>,
    /// When this connection cache was created
    created_at: Instant,
    /// Last access time
    last_accessed: Instant,
}

impl ConnectionCache {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            statements: HashMap::new(),
            access_order: Vec::new(),
            created_at: now,
            last_accessed: now,
        }
    }

    /// Get a statement from the cache
    fn get(&mut self, sql: &str) -> Option<&CachedStatementMetadata> {
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
    fn insert(&mut self, sql: String, stmt: CachedStatementMetadata) {
        self.statements.insert(sql.clone(), stmt);
        self.access_order.push(sql);
        self.last_accessed = Instant::now();
    }

    /// Remove a specific statement
    fn remove(&mut self, sql: &str) -> bool {
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
    fn evict_lru(&mut self) -> Option<String> {
        if self.access_order.is_empty() {
            return None;
        }

        let lru_key = self.access_order.remove(0);
        self.statements.remove(&lru_key);
        Some(lru_key)
    }

    /// Get the number of cached statements
    fn len(&self) -> usize {
        self.statements.len()
    }

    /// Check if cache is empty
    fn is_empty(&self) -> bool {
        self.statements.is_empty()
    }

    /// Clear all statements
    fn clear(&mut self) {
        self.statements.clear();
        self.access_order.clear();
    }

    /// Get idle time
    fn idle_time(&self) -> Duration {
        self.last_accessed.elapsed()
    }
}

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
        ConnectionId::new()
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

        drop(cache);

        // Update stats
        let mut stats = self.stats.write();
        stats.record_miss();
        stats.record_prepared(prepare_time_us);
        stats.update_size(self.total_size());
        stats.update_active_connections(self.cache.read().len());

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

        // Update stats
        let mut stats = self.stats.write();
        stats.update_size(self.total_size());
        stats.update_active_connections(cache.len());

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
            let mut stats = self.stats.write();
            stats.update_size(self.total_size());
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
            let mut stats = self.stats.write();
            stats.update_size(self.total_size());
            stats.update_active_connections(cache.len());
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
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic_operations() {
        let cache = PreparedStatementCache::new(10);
        let conn_id = cache.get_connection_id();

        // Record a miss (preparing a statement)
        cache.record_miss(conn_id, "SELECT 1", 100);

        // Record a hit
        cache.record_hit(conn_id, "SELECT 1");

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.prepared, 1);
        assert_eq!(stats.active_connections, 1);
    }

    #[test]
    fn test_cache_per_connection() {
        let cache = PreparedStatementCache::new(10);
        let conn_id1 = cache.get_connection_id();
        let conn_id2 = cache.get_connection_id();

        let sql = "SELECT 1";

        // Record on connection 1
        cache.record_miss(conn_id1, sql, 100);

        // Record on connection 2
        cache.record_miss(conn_id2, sql, 100);

        // Should have 2 separate caches
        assert_eq!(cache.connection_count(), 2);
        assert_eq!(cache.connection_size(conn_id1), 1);
        assert_eq!(cache.connection_size(conn_id2), 1);

        let stats = cache.stats();
        assert_eq!(stats.active_connections, 2);
    }

    #[test]
    fn test_cache_eviction() {
        let cache = PreparedStatementCache::new(3);
        let conn_id = cache.get_connection_id();

        // Record 4 statements (will trigger eviction)
        for i in 0..4 {
            let sql = format!("SELECT {}", i);
            cache.record_miss(conn_id, &sql, 100);
        }

        let stats = cache.stats();
        assert_eq!(stats.evictions, 1);
        assert_eq!(cache.connection_size(conn_id), 3);
    }

    #[test]
    fn test_clear_connection() {
        let cache = PreparedStatementCache::new(10);
        let conn_id = cache.get_connection_id();

        // Record some statements
        for i in 0..5 {
            let sql = format!("SELECT {}", i);
            cache.record_miss(conn_id, &sql, 100);
        }

        assert_eq!(cache.connection_size(conn_id), 5);

        // Clear connection cache
        let cleared = cache.clear_connection(conn_id);
        assert_eq!(cleared, 5);
        assert_eq!(cache.connection_size(conn_id), 0);

        let stats = cache.stats();
        assert_eq!(stats.active_connections, 0);
    }

    #[test]
    fn test_cache_clear() {
        let cache = PreparedStatementCache::new(10);
        let conn_id1 = cache.get_connection_id();
        let conn_id2 = cache.get_connection_id();

        // Record some statements on both connections
        for i in 0..5 {
            let sql = format!("SELECT {}", i);
            cache.record_miss(conn_id1, &sql, 100);
            cache.record_miss(conn_id2, &sql, 100);
        }

        assert_eq!(cache.total_size(), 10);

        // Clear all caches
        cache.clear();
        assert!(cache.is_empty());

        let stats = cache.stats();
        assert_eq!(stats.current_size, 0);
        assert_eq!(stats.active_connections, 0);
    }

    #[test]
    fn test_connection_eviction() {
        let config = PreparedCacheConfig {
            max_size: 10,
            max_connections: 2,
            ..Default::default()
        };
        let cache = PreparedStatementCache::with_config(config);

        // Create 3 connections (will trigger eviction of 1)
        let conn_id1 = cache.get_connection_id();
        let conn_id2 = cache.get_connection_id();
        let conn_id3 = cache.get_connection_id();

        cache.record_miss(conn_id1, "SELECT 1", 100);
        cache.record_miss(conn_id2, "SELECT 2", 100);
        cache.record_miss(conn_id3, "SELECT 3", 100);

        // Should have evicted one connection
        assert_eq!(cache.connection_count(), 2);

        let stats = cache.stats();
        assert_eq!(stats.connection_evictions, 1);
    }

    #[test]
    fn test_cleanup_idle_connections() {
        let cache = PreparedStatementCache::new(10);
        let conn_id = cache.get_connection_id();

        cache.record_miss(conn_id, "SELECT 1", 100);

        assert_eq!(cache.connection_count(), 1);

        // Cleanup with zero duration should remove the connection
        let cleaned = cache.cleanup_idle_connections(Duration::from_secs(0));
        assert_eq!(cleaned, 1);
        assert_eq!(cache.connection_count(), 0);
    }

    #[test]
    fn test_cache_hit_rate() {
        let cache = PreparedStatementCache::new(10);
        let conn_id = cache.get_connection_id();

        let sql = "SELECT 1";

        // First call - miss
        cache.record_miss(conn_id, sql, 100);

        // Second call - hit
        cache.record_hit(conn_id, sql);

        // Third call - hit
        cache.record_hit(conn_id, sql);

        let stats = cache.stats();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert!((stats.hit_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_remove_statement() {
        let cache = PreparedStatementCache::new(10);
        let conn_id = cache.get_connection_id();

        let sql = "SELECT 1";
        cache.record_miss(conn_id, sql, 100);

        assert!(!cache.is_empty());

        // Remove the statement
        assert!(cache.remove(conn_id, sql));
        assert!(cache.is_empty());

        // Try to remove again (should return false)
        assert!(!cache.remove(conn_id, sql));
    }

    #[test]
    fn test_is_cached() {
        let cache = PreparedStatementCache::new(10);
        let conn_id = cache.get_connection_id();

        let sql = "SELECT 1";

        // Not cached initially
        assert!(!cache.is_cached(conn_id, sql));

        // Record miss (caches the statement)
        cache.record_miss(conn_id, sql, 100);

        // Now it should be cached
        assert!(cache.is_cached(conn_id, sql));
    }
}
