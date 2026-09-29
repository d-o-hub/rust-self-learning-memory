//! True connection pool with stable IDs for prepared statement caching
//!
//! This pool maintains actual reusable connections with stable IDs,
//! enabling effective prepared statement caching.

use super::connection_wrapper::PooledConnection;
use do_memory_core::{Error, Result};
use libsql::Database;
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::{debug, info};

/// Configuration for the caching-aware connection pool
#[derive(Debug, Clone)]
pub struct CachingPoolConfig {
    /// Maximum number of connections to maintain
    pub max_connections: usize,
    /// Minimum number of connections to maintain
    pub min_connections: usize,
    /// Maximum time to wait for a connection
    pub connection_timeout: Duration,
    /// Maximum idle time before a connection is eligible for eviction
    pub max_idle_time: Duration,
    /// Maximum connection age before forcing recreation
    pub max_connection_age: Duration,
    /// Enable connection health validation
    pub enable_health_check: bool,
}

impl Default for CachingPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 2,
            connection_timeout: Duration::from_secs(5),
            max_idle_time: Duration::from_secs(300),
            max_connection_age: Duration::from_secs(3600),
            enable_health_check: true,
        }
    }
}

/// Statistics for the caching pool
#[derive(Debug, Default, Clone)]
pub struct CachingPoolStats {
    /// Total connections created
    pub total_created: u64,
    /// Total connections checked out
    pub total_checkouts: u64,
    /// Total connections returned
    pub total_returns: u64,
    /// Cache hits (reused connections)
    pub cache_hits: u64,
    /// Cache misses (new connections created)
    pub cache_misses: u64,
    /// Current active connections (checked out)
    pub active_connections: usize,
    /// Current idle connections (available in pool)
    pub idle_connections: usize,
    /// Connections evicted due to age/idle time
    pub evictions: u64,
}

/// Shared state for the caching pool.
///
/// This state is held behind an `Arc` by both the `CachingPool` and every
/// `ConnectionGuard` it hands out. A guard therefore owns a reference to the
/// return path and to the idle/active/statistics bookkeeping, so returning a
/// connection is safe even after the owning `CachingPool` value has been
/// dropped.
struct CachingPoolState {
    idle_connections: Mutex<Vec<PooledConnection>>,
    active_connection_ids: Mutex<std::collections::HashSet<u64>>,
    stats: Mutex<CachingPoolStats>,
    cleanup_callback: Mutex<Option<Arc<dyn Fn(u64) + Send + Sync>>>,
}

impl CachingPoolState {
    fn new() -> Self {
        Self {
            idle_connections: Mutex::new(Vec::new()),
            active_connection_ids: Mutex::new(std::collections::HashSet::new()),
            stats: Mutex::new(CachingPoolStats::default()),
            cleanup_callback: Mutex::new(None),
        }
    }

    /// Return a connection to the idle pool and update bookkeeping.
    fn return_connection(&self, mut connection: PooledConnection) {
        let conn_id = connection.id();

        debug!("Returning connection {} to pool", conn_id);

        // Mark as no longer active
        self.active_connection_ids.lock().remove(&conn_id);
        self.stats.lock().total_returns += 1;
        self.stats.lock().active_connections = self.active_connection_ids.lock().len();
        self.stats.lock().idle_connections = self.idle_connections.lock().len() + 1;

        // Update last-used time
        connection.touch();

        // Return to idle pool
        self.idle_connections.lock().push(connection);
    }

    /// Permanently destroy a connection and notify the cleanup callback.
    fn destroy_connection(&self, connection: PooledConnection) {
        let conn_id = connection.id();

        debug!("Destroying connection {}", conn_id);

        // Mark as no longer active
        self.active_connection_ids.lock().remove(&conn_id);
        self.stats.lock().evictions += 1;

        // Invoke cleanup callback to clear prepared statement cache
        if let Some(callback) = self.cleanup_callback.lock().as_ref() {
            callback(conn_id);
        }

        // Connection is dropped here
    }

    /// Clean up idle connections that exceed max age or idle time.
    fn cleanup_idle_connections(&self, config: &CachingPoolConfig) -> usize {
        let mut idle = self.idle_connections.lock();
        let original_len = idle.len();

        // Retain only connections that are within limits
        idle.retain(|conn| {
            let age = conn.age();
            let idle_time = conn.idle_time();

            let should_keep = age < config.max_connection_age && idle_time < config.max_idle_time;

            if !should_keep {
                // Invoke cleanup callback for evicted connections
                if let Some(callback) = self.cleanup_callback.lock().as_ref() {
                    callback(conn.id());
                }
                self.stats.lock().evictions += 1;
            }

            should_keep
        });

        let evicted = original_len - idle.len();
        if evicted > 0 {
            info!(
                "Cleaned up {} idle connections (remaining: {})",
                evicted,
                idle.len()
            );
        }

        self.stats.lock().idle_connections = idle.len();
        evicted
    }

    fn stats(&self) -> CachingPoolStats {
        self.stats.lock().clone()
    }

    fn cache_hit_rate(&self) -> f64 {
        let stats = self.stats.lock();
        let total = stats.cache_hits + stats.cache_misses;
        if total == 0 {
            0.0
        } else {
            stats.cache_hits as f64 / total as f64
        }
    }

    fn available_connections(&self) -> usize {
        self.idle_connections.lock().len()
    }

    fn active_connections(&self) -> usize {
        self.active_connection_ids.lock().len()
    }
}

/// A connection pool that maintains reusable connections with stable IDs
///
/// # Architecture
///
/// ```text
/// CachingPool {
///     db: Arc<Database>,
///     idle_connections: Vec<PooledConnection>,  // Available connections
///     active_connections: HashSet<u64>,          // IDs of checked-out connections
///     semaphore: Semaphore,                      // Limits concurrent checkouts
///     cleanup_callback: Arc<dyn Fn(u64)>,       // Called on connection drop
/// }
/// ```
///
/// # Connection Lifecycle
///
/// 1. **Creation**: Connection created with unique stable ID
/// 2. **Checkout**: Connection taken from idle pool or created new
/// 3. **Return**: Connection returned to idle pool (not destroyed)
/// 4. **Eviction**: Old/idle connections destroyed periodically
/// 5. **Drop**: Cleanup callback invoked to clear prepared statement cache
pub struct CachingPool {
    db: Arc<Database>,
    config: CachingPoolConfig,
    semaphore: Arc<Semaphore>,
    /// Shared with every `ConnectionGuard` handed out by `get`.
    state: Arc<CachingPoolState>,
}

impl CachingPool {
    /// Create a new caching-aware connection pool
    ///
    /// # Arguments
    ///
    /// * `db` - The libsql database
    /// * `config` - Pool configuration
    ///
    /// # Errors
    ///
    /// Returns error if database validation fails
    pub async fn new(db: Arc<Database>, config: CachingPoolConfig) -> Result<Self> {
        info!(
            "Creating caching pool: min={}, max={}",
            config.min_connections, config.max_connections
        );

        // Validate database connectivity
        let conn = db
            .connect()
            .map_err(|e| Error::Storage(format!("Failed to connect: {}", e)))?;
        conn.query("SELECT 1", ())
            .await
            .map_err(|e| Error::Storage(format!("Database validation failed: {}", e)))?;

        let semaphore = Arc::new(Semaphore::new(config.max_connections));

        let pool = Self {
            db,
            config,
            semaphore,
            state: Arc::new(CachingPoolState::new()),
        };

        // Pre-create minimum connections
        pool.pre_create_connections().await?;

        info!("Caching pool created successfully");
        Ok(pool)
    }

    /// Set the cleanup callback for connection lifecycle events
    ///
    /// This callback is invoked when a connection is permanently destroyed,
    /// allowing the prepared statement cache to clean up entries for that connection.
    pub fn set_cleanup_callback<F>(&self, callback: F)
    where
        F: Fn(u64) + Send + Sync + 'static,
    {
        *self.state.cleanup_callback.lock() = Some(Arc::new(callback));
    }

    /// Pre-create the minimum number of connections
    async fn pre_create_connections(&self) -> Result<()> {
        let current_count = self.state.idle_connections.lock().len();
        let needed = self.config.min_connections.saturating_sub(current_count);

        for _ in 0..needed {
            let conn = self.create_connection().await?;
            self.state.idle_connections.lock().push(conn);
        }

        debug!("Pre-created {} connections", needed);
        Ok(())
    }

    /// Create a new physical database connection
    async fn create_connection(&self) -> Result<PooledConnection> {
        let conn = self
            .db
            .connect()
            .map_err(|e| Error::Storage(format!("Failed to create connection: {}", e)))?;

        let pooled_conn = PooledConnection::new(conn);

        // Validate if enabled
        if self.config.enable_health_check {
            pooled_conn
                .validate()
                .await
                .map_err(|e| Error::Storage(format!("Connection health check failed: {}", e)))?;
        }

        // Update stats
        self.state.stats.lock().total_created += 1;
        self.state.stats.lock().cache_misses += 1;

        Ok(pooled_conn)
    }

    /// Check out a connection from the pool
    ///
    /// # Returns
    ///
    /// A guard that automatically returns the connection to the pool when dropped
    ///
    /// # Errors
    ///
    /// Returns error if timeout waiting for available connection or connection creation fails
    pub async fn get(&self) -> Result<ConnectionGuard> {
        // Acquire semaphore permit (limits concurrent checkouts)
        let permit = tokio::time::timeout(
            self.config.connection_timeout,
            self.semaphore.clone().acquire_owned(),
        )
        .await
        .map_err(|_| {
            Error::Storage(format!(
                "Connection pool timeout after {:?}",
                self.config.connection_timeout
            ))
        })?
        .map_err(|e| Error::Storage(format!("Failed to acquire permit: {}", e)))?;

        // Try to get an idle connection
        let mut pooled_conn = {
            let mut idle = self.state.idle_connections.lock();
            idle.pop()
        };

        let conn_id = if let Some(conn) = &pooled_conn {
            // Reusing existing connection - cache hit
            debug!("Reusing connection {}", conn.id());
            self.state.stats.lock().cache_hits += 1;
            conn.id()
        } else {
            // No idle connection available - create new
            let new_conn = self.create_connection().await?;
            let id = new_conn.id();
            pooled_conn = Some(new_conn);
            id
        };

        // Mark as active
        self.state.active_connection_ids.lock().insert(conn_id);
        self.state.stats.lock().total_checkouts += 1;
        self.state.stats.lock().active_connections += 1;
        self.state.stats.lock().idle_connections = self.state.idle_connections.lock().len();

        // SAFETY: pooled_conn is guaranteed to be Some at this point:
        // - Either we got it from idle.pop() and it was Some
        // - Or we created a new connection and stored it in pooled_conn
        let connection = pooled_conn.ok_or_else(|| {
            Error::Storage("Failed to get connection from pool: connection is None".to_string())
        })?;

        Ok(ConnectionGuard {
            // The guard owns a reference to the shared state, so it can return
            // the connection safely even if the pool value is dropped first.
            state: Arc::clone(&self.state),
            connection: Some(connection),
            _permit: Some(permit),
        })
    }

    /// Clean up idle connections that exceed max age or idle time
    pub fn cleanup_idle_connections(&self) -> usize {
        self.state.cleanup_idle_connections(&self.config)
    }

    /// Get current pool statistics
    pub fn stats(&self) -> CachingPoolStats {
        self.state.stats()
    }

    /// Get the cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        self.state.cache_hit_rate()
    }

    /// Get number of available connections
    pub fn available_connections(&self) -> usize {
        self.state.available_connections()
    }

    /// Get number of active (checked out) connections
    pub fn active_connections(&self) -> usize {
        self.state.active_connections()
    }
}

/// Guard for a checked-out connection
///
/// Automatically returns the connection to the pool when dropped. The guard
/// owns an `Arc` to the pool's shared state, so the return is safe even if the
/// `CachingPool` value itself has already been dropped.
pub struct ConnectionGuard {
    state: Arc<CachingPoolState>,
    connection: Option<PooledConnection>,
    _permit: Option<tokio::sync::OwnedSemaphorePermit>,
}

impl ConnectionGuard {
    /// Get the stable connection ID
    ///
    /// # Errors
    ///
    /// Returns an error if the connection has been taken (should not happen in normal usage).
    pub fn id(&self) -> do_memory_core::Result<u64> {
        self.connection.as_ref().map(|c| c.id()).ok_or_else(|| {
            do_memory_core::Error::Storage(
                "ConnectionGuard::id() called on guard without connection".to_string(),
            )
        })
    }

    /// Get a reference to the underlying connection
    ///
    /// # Errors
    ///
    /// Returns an error if the connection has been taken (should not happen in normal usage).
    pub fn connection(&self) -> do_memory_core::Result<&libsql::Connection> {
        self.connection
            .as_ref()
            .map(|c| c.connection())
            .ok_or_else(|| {
                do_memory_core::Error::Storage(
                    "ConnectionGuard::connection() called on guard without connection".to_string(),
                )
            })
    }

    /// Get the pooled connection wrapper
    pub fn pooled(&self) -> Option<&PooledConnection> {
        self.connection.as_ref()
    }
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        // Return connection to pool instead of destroying it. Booking the
        // return and releasing the permit happen exactly once here.
        if let (Some(_permit), Some(connection)) = (self._permit.take(), self.connection.take()) {
            self.state.return_connection(connection);
        }
    }
}

// `ConnectionGuard` is auto-`Send`: it owns an `Arc<CachingPoolState>` (whose
// mutex-protected contents are all `Send`) plus the pooled connection and an
// owned semaphore permit. No `unsafe` impl is needed.

#[cfg(test)]
#[path = "caching_pool_tests.rs"]
mod tests;
