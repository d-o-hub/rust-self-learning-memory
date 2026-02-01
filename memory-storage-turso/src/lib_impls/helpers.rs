//! Lib helper methods
//!
//! This module contains helper methods for TursoStorage.

use libsql::Connection;
use memory_core::{Error, Result};
use std::time::Instant;
use tracing::{debug, error, warn};

use super::storage::TursoStorage;
use crate::prepared::ConnectionId;

// These are extension methods for TursoStorage
// They are attached via the impl block below
impl TursoStorage {
    /// Check if a connection pool is available
    ///
    /// Returns true if any connection pool (standard, keepalive, or adaptive) is configured.
    ///
    /// The prepared statement cache is now connection-aware, so it can safely be used
    /// with connection pooling. Each connection has its own cache of prepared statements,
    /// and caches are cleared when connections are returned to the pool.
    pub(crate) fn has_connection_pool(&self) -> bool {
        // Connection-aware prepared statement cache is now implemented
        // The cache stores statements per-connection using ConnectionId,
        // ensuring statements are only used with the connection they were prepared on.
        self.pool.is_some() || self.adaptive_pool.is_some() || {
            #[cfg(feature = "keepalive-pool")]
            {
                self.keepalive_pool.is_some()
            }
            #[cfg(not(feature = "keepalive-pool"))]
            {
                false
            }
        }
    }

    /// Get a database connection
    ///
    /// If connection pooling is enabled, this will use a pooled connection.
    /// If keep-alive pool is enabled, it will be used for reduced overhead.
    /// If adaptive pool is enabled, it will be used for variable load optimization.
    /// Otherwise, it creates a new connection each time.
    pub async fn get_connection(&self) -> Result<Connection> {
        // Check adaptive pool first (highest priority for variable load)
        if let Some(ref adaptive_pool) = self.adaptive_pool {
            let adaptive_conn = adaptive_pool.get().await?;
            // Extract the connection from the pooled connection
            if let Some(conn) = adaptive_conn.into_inner() {
                return Ok(conn);
            }
            // Fallback if connection extraction fails
            return Err(Error::Storage(
                "Failed to extract connection from adaptive pool".to_string(),
            ));
        }

        #[cfg(feature = "keepalive-pool")]
        {
            if let Some(ref keepalive_pool) = self.keepalive_pool {
                // Use keep-alive pool for reduced connection overhead
                let keepalive_conn = keepalive_pool.get().await?;
                return Ok(keepalive_conn.connection().clone());
            }
        }

        if let Some(ref pool) = self.pool {
            // Use connection pool
            let pooled_conn = pool.get().await?;
            Ok(pooled_conn.into_inner()?)
        } else {
            // Create direct connection (legacy mode)
            self.db
                .connect()
                .map_err(|e| Error::Storage(format!("Failed to get connection: {}", e)))
        }
    }

    /// Get a database connection with its cache ID
    ///
    /// This method returns both the connection and a unique connection ID
    /// for use with the prepared statement cache. The ID should be passed
    /// to `prepare_cached` and `clear_prepared_cache` for proper cache management.
    ///
    /// # Returns
    ///
    /// A tuple of (Connection, ConnectionId)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use memory_storage_turso::TursoStorage;
    /// # async fn example(storage: &TursoStorage) -> anyhow::Result<()> {
    /// let (conn, conn_id) = storage.get_connection_with_id().await?;
    /// // Use conn and conn_id with prepare_cached
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_connection_with_id(&self) -> Result<(Connection, ConnectionId)> {
        let conn = self.get_connection().await?;
        let conn_id = self.prepared_cache.get_connection_id();
        Ok((conn, conn_id))
    }

    /// Get count of records in a table
    pub async fn get_count(&self, conn: &Connection, table: &str) -> Result<usize> {
        let sql = format!("SELECT COUNT(*) as count FROM {}", table);
        let mut rows = conn
            .query(&sql, ())
            .await
            .map_err(|e| Error::Storage(format!("Failed to count {}: {}", table, e)))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch count for {}: {}", table, e)))?
        {
            let count: i64 = row
                .get(0)
                .map_err(|e| Error::Storage(format!("Failed to parse count: {}", e)))?;
            Ok(count as usize)
        } else {
            Ok(0)
        }
    }

    /// Execute PRAGMA statements for database configuration
    ///
    /// PRAGMA statements may return rows, so we need to consume them before continuing.
    pub async fn execute_pragmas(&self, conn: &Connection) -> Result<()> {
        // Enable WAL mode for better concurrent access
        // WAL mode allows concurrent reads while writing, reducing "database locked" errors
        if let Ok(mut rows) = conn.query("PRAGMA journal_mode=WAL", ()).await {
            // Consume all rows to avoid "Execute returned rows" error
            while rows.next().await.is_ok_and(|r| r.is_some()) {
                // Consume each row
            }
        }

        // Increase busy timeout to allow more time for lock acquisition
        if let Ok(mut rows) = conn.query("PRAGMA busy_timeout=30000", ()).await {
            while rows.next().await.is_ok_and(|r| r.is_some()) {
                // Consume each row
            }
        }

        Ok(())
    }

    /// Execute a SQL statement with retry logic
    pub async fn execute_with_retry(&self, conn: &Connection, sql: &str) -> Result<()> {
        let mut attempts = 0;
        let mut delay = std::time::Duration::from_millis(self.config.retry_base_delay_ms);

        loop {
            match conn.execute(sql, ()).await {
                Ok(_) => {
                    if attempts > 0 {
                        debug!("SQL succeeded after {} retries", attempts);
                    }
                    return Ok(());
                }
                Err(e) => {
                    attempts += 1;
                    if attempts >= self.config.max_retries {
                        error!("SQL failed after {} attempts: {}", attempts, e);
                        return Err(Error::Storage(format!(
                            "SQL execution failed after {} retries: {}",
                            attempts, e
                        )));
                    }

                    warn!("SQL attempt {} failed: {}, retrying...", attempts, e);
                    tokio::time::sleep(delay).await;

                    // Exponential backoff
                    delay = std::cmp::min(
                        delay * 2,
                        std::time::Duration::from_millis(self.config.retry_max_delay_ms),
                    );
                }
            }
        }
    }

    /// Health check - verify database connectivity
    pub async fn health_check(&self) -> Result<bool> {
        let conn = self.get_connection().await?;
        match conn.query("SELECT 1", ()).await {
            Ok(_) => Ok(true),
            Err(e) => {
                error!("Health check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Wrap this storage with a cache layer using default cache configuration
    ///
    /// This provides transparent caching for episodes, patterns, and heuristics
    /// with adaptive TTL based on access patterns.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use memory_storage_turso::{TursoStorage, CacheConfig};
    /// # async fn example() -> anyhow::Result<()> {
    /// let storage = TursoStorage::new("file:test.db", "").await?;
    /// let cached = storage.with_cache_default();
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_cache_default(self) -> crate::cache::CachedTursoStorage {
        self.with_cache(crate::cache::CacheConfig::default())
    }

    /// Wrap this storage with a cache layer using custom cache configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Cache configuration to use
    ///
    /// # Returns
    ///
    /// A new `CachedTursoStorage` wrapping this storage
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use memory_storage_turso::{TursoStorage, CacheConfig};
    /// # use std::time::Duration;
    /// # async fn example() -> anyhow::Result<()> {
    /// let storage = TursoStorage::new("file:test.db", "").await?;
    /// let config = CacheConfig {
    ///     max_episodes: 1000,
    ///     episode_ttl: Duration::from_secs(3600),
    ///     ..Default::default()
    /// };
    /// let cached = storage.with_cache(config);
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_cache(
        self,
        cache_config: crate::cache::CacheConfig,
    ) -> crate::cache::CachedTursoStorage {
        crate::cache::CachedTursoStorage::new(self, cache_config)
    }

    /// Get the cache configuration if set
    pub fn cache_config(&self) -> Option<&crate::cache::CacheConfig> {
        self.config.cache_config.as_ref()
    }

    /// Get prepared statement cache statistics
    pub fn prepared_cache_stats(&self) -> crate::prepared::PreparedCacheStats {
        self.prepared_cache.stats()
    }

    /// Get a reference to the prepared statement cache
    pub fn prepared_cache(&self) -> &crate::prepared::PreparedStatementCache {
        &self.prepared_cache
    }

    /// Get database statistics
    pub async fn get_statistics(&self) -> Result<crate::trait_impls::StorageStatistics> {
        let conn = self.get_connection().await?;

        let episode_count = self.get_count(&conn, "episodes").await?;
        let pattern_count = self.get_count(&conn, "patterns").await?;
        let heuristic_count = self.get_count(&conn, "heuristics").await?;

        Ok(crate::trait_impls::StorageStatistics {
            episode_count,
            pattern_count,
            heuristic_count,
        })
    }

    /// Get pool statistics if pooling is enabled
    pub async fn pool_statistics(&self) -> Option<crate::pool::PoolStatistics> {
        if let Some(ref pool) = self.pool {
            Some(pool.statistics().await)
        } else {
            None
        }
    }

    /// Get pool utilization if pooling is enabled
    pub async fn pool_utilization(&self) -> Option<f32> {
        if let Some(ref pool) = self.pool {
            Some(pool.utilization().await)
        } else {
            self.adaptive_pool
                .as_ref()
                .map(|adaptive_pool| adaptive_pool.utilization() as f32)
        }
    }

    /// Get adaptive pool metrics if enabled
    pub fn adaptive_pool_metrics(&self) -> Option<crate::pool::AdaptivePoolMetrics> {
        self.adaptive_pool.as_ref().map(|pool| pool.metrics())
    }

    /// Get current adaptive pool size
    pub fn adaptive_pool_size(&self) -> Option<(u32, u32)> {
        self.adaptive_pool
            .as_ref()
            .map(|pool| (pool.active_connections(), pool.max_connections()))
    }

    /// Manually trigger adaptive pool scaling check
    pub async fn check_adaptive_pool_scale(&self) {
        if let Some(ref adaptive_pool) = self.adaptive_pool {
            adaptive_pool.check_and_scale().await;
        }
    }

    /// Get keep-alive pool statistics if enabled
    #[cfg(feature = "keepalive-pool")]
    pub fn keepalive_statistics(&self) -> Option<crate::pool::KeepAliveStatistics> {
        self.keepalive_pool.as_ref().map(|pool| pool.statistics())
    }

    /// Get keep-alive configuration if enabled
    #[cfg(feature = "keepalive-pool")]
    pub fn keepalive_config(&self) -> Option<&crate::pool::KeepAliveConfig> {
        self.keepalive_pool.as_ref().map(|pool| pool.config())
    }

    /// Prepare a SQL statement with cache tracking
    ///
    /// This method prepares a SQL statement and tracks cache statistics.
    /// If the statement is already cached for this connection, it's a cache hit.
    /// Otherwise, it's a cache miss and the statement is prepared and tracked.
    ///
    /// # Arguments
    ///
    /// * `conn_id` - Connection identifier for cache tracking
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
    pub async fn prepare_cached(
        &self,
        conn_id: ConnectionId,
        conn: &Connection,
        sql: &str,
    ) -> Result<libsql::Statement> {
        // Check if this is a cache hit
        if self.prepared_cache.is_cached(conn_id, sql) {
            self.prepared_cache.record_hit(conn_id, sql);
        }

        // Prepare the statement
        let start = Instant::now();
        let stmt = conn
            .prepare(sql)
            .await
            .map_err(|e| Error::Storage(format!("Failed to prepare statement: {}", e)))?;
        let prepare_time_us = start.elapsed().as_micros() as u64;

        // Record the miss (or re-record if it was a hit - tracks preparation time)
        self.prepared_cache
            .record_miss(conn_id, sql, prepare_time_us);

        Ok(stmt)
    }

    /// Clear the prepared statement cache for a connection
    ///
    /// This should be called when a connection is returned to the pool
    /// to prevent memory leaks and ensure proper cache management.
    ///
    /// # Arguments
    ///
    /// * `conn_id` - Connection identifier to clear
    ///
    /// # Returns
    ///
    /// Number of statements cleared from the cache
    pub fn clear_prepared_cache(&self, conn_id: ConnectionId) -> usize {
        self.prepared_cache.clear_connection(conn_id)
    }
}
