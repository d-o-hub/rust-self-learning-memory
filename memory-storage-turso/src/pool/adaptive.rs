//! Adaptive connection pool that dynamically adjusts pool size based on load.

use super::adaptive_scale::{AdaptiveCore, AdaptiveMetrics, MonitorHandle, spawn_monitor};
use do_memory_core::{Error, Result};
use libsql::Database;
use parking_lot::{Mutex, RwLock};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::OwnedSemaphorePermit;
use tracing::{debug, info, warn};

/// Unique identifier for a connection
pub type ConnectionId = u64;

/// Callback type for connection lifecycle events
///
/// This is called when a connection is dropped, allowing external components
/// (like the prepared statement cache) to clean up resources associated with
/// the connection.
pub type ConnectionCleanupCallback = Arc<dyn Fn(ConnectionId) + Send + Sync>;

#[derive(Debug, Clone)]
pub struct AdaptivePoolConfig {
    pub min_connections: u32,
    pub max_connections: u32,
    pub scale_up_threshold: f64,
    pub scale_down_threshold: f64,
    pub scale_up_cooldown: Duration,
    pub scale_down_cooldown: Duration,
    pub scale_up_increment: u32,
    pub scale_down_decrement: u32,
    pub check_interval: Duration,
}

impl Default for AdaptivePoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 5,
            max_connections: 50,
            scale_up_threshold: 0.7,
            scale_down_threshold: 0.3,
            scale_up_cooldown: Duration::from_secs(10),
            scale_down_cooldown: Duration::from_secs(30),
            scale_up_increment: 5,
            scale_down_decrement: 5,
            check_interval: Duration::from_secs(5),
        }
    }
}

#[derive(Debug, Default)]
pub struct AdaptivePoolMetrics {
    pub utilization_percent: f64,
    pub active_connections: u32,
    pub max_connections: u32,
    pub scale_up_count: u32,
    pub scale_down_count: u32,
    pub avg_wait_time_us: u64,
    pub total_acquired: u64,
    pub total_released: u64,
}

pub struct AdaptiveConnectionPool {
    db: Arc<Database>,
    core: Arc<AdaptiveCore>,
    next_conn_id: Arc<AtomicU64>,
    cleanup_callback: RwLock<Option<ConnectionCleanupCallback>>,
    monitor: Mutex<Option<MonitorHandle>>,
}

impl AdaptiveConnectionPool {
    pub async fn new(db: Arc<Database>, config: AdaptivePoolConfig) -> Result<Self> {
        let config = Arc::new(config);
        let initial_max = config.min_connections;

        info!(
            "Creating adaptive connection pool with min={}, max={}",
            config.min_connections, config.max_connections
        );

        let core = Arc::new(AdaptiveCore::new(config));
        let monitor = spawn_monitor(&core);

        let pool = Self {
            db,
            core,
            next_conn_id: Arc::new(AtomicU64::new(1)),
            cleanup_callback: RwLock::new(None),
            monitor: Mutex::new(Some(monitor)),
        };

        let conn = pool
            .db
            .connect()
            .map_err(|e| Error::Storage(format!("Failed to connect: {}", e)))?;
        conn.query("SELECT 1", ())
            .await
            .map_err(|e| Error::Storage(format!("Database validation failed: {}", e)))?;

        info!(
            "Adaptive connection pool created successfully (initial capacity {})",
            initial_max
        );

        Ok(pool)
    }

    #[expect(clippy::unused_async, clippy::unused_async_trait_impl)]
    pub async fn new_sync(db: Arc<Database>, config: AdaptivePoolConfig) -> Result<Self> {
        let config = Arc::new(config);

        info!(
            "Creating adaptive connection pool (sync mode) with min={}, max={}",
            config.min_connections, config.max_connections
        );

        let core = Arc::new(AdaptiveCore::new(config));
        let monitor = spawn_monitor(&core);

        Ok(Self {
            db,
            core,
            next_conn_id: Arc::new(AtomicU64::new(1)),
            cleanup_callback: RwLock::new(None),
            monitor: Mutex::new(Some(monitor)),
        })
    }

    async fn try_acquire(&self, timeout: Duration) -> Result<OwnedSemaphorePermit> {
        let start = Instant::now();

        match tokio::time::timeout(timeout, self.core.semaphore().clone().acquire_owned()).await {
            Ok(Ok(permit)) => {
                let wait_us = start.elapsed().as_micros() as u64;

                self.core
                    .metrics
                    .wait_time_total_us
                    .fetch_add(wait_us, Ordering::Relaxed);
                self.core.metrics.wait_count.fetch_add(1, Ordering::Relaxed);

                let total_time = self.core.metrics.wait_time_total_us.load(Ordering::Relaxed);
                let count = self.core.metrics.wait_count.load(Ordering::Relaxed);
                if let Some(avg) = total_time.checked_div(count) {
                    self.core
                        .metrics
                        .avg_wait_time_us
                        .store(avg, Ordering::Relaxed);
                }

                let active = self
                    .core
                    .metrics
                    .active_connections
                    .fetch_add(1, Ordering::Relaxed)
                    + 1;
                self.core.refresh_utilization(active);
                self.core
                    .metrics
                    .total_acquired
                    .fetch_add(1, Ordering::Relaxed);

                Ok(permit)
            }
            Ok(Err(e)) => Err(Error::Storage(format!(
                "Failed to acquire connection permit: {}",
                e
            ))),
            Err(_) => Err(Error::Storage(format!(
                "Connection acquisition timed out after {:?} (effective capacity {})",
                timeout,
                self.core.effective_max()
            ))),
        }
    }

    /// Run one scaling decision immediately.
    ///
    /// The background monitor calls this every `check_interval`; callers may
    /// also trigger it explicitly. Returns `true` when the pool resized.
    ///
    /// The `async` signature is kept for API compatibility even though the
    /// decision itself is synchronous.
    #[expect(clippy::unused_async, clippy::unused_async_trait_impl)]
    pub async fn check_and_scale(&self) -> bool {
        self.core.check_and_scale()
    }

    pub async fn get(&self) -> Result<AdaptivePooledConnection> {
        let permit = self.try_acquire(self.core.check_interval()).await?;

        // Generate unique connection ID
        let conn_id = self.next_conn_id.fetch_add(1, Ordering::Relaxed);

        // Create a new database connection from the database
        let connection = self
            .db
            .connect()
            .map_err(|e| Error::Storage(format!("Failed to create connection: {}", e)))?;

        // Get cleanup callback if registered
        let cleanup_callback = self.cleanup_callback.read().clone();

        debug!("Created connection with ID: {}", conn_id);

        Ok(AdaptivePooledConnection {
            conn_id,
            // Own the shared state so the guard stays valid even if the pool is
            // dropped while this connection is checked out.
            metrics: Arc::clone(&self.core.metrics),
            current_max: Arc::clone(self.core.current_max()),
            permit: Some(permit),
            connection: Some(connection),
            cleanup_callback,
        })
    }

    pub fn available_connections(&self) -> usize {
        self.core.semaphore().available_permits()
    }

    /// Current utilization, computed against the effective capacity.
    pub fn utilization(&self) -> f64 {
        let active = self.core.metrics.active_connections.load(Ordering::Relaxed);
        f64::from(active) / f64::from(self.core.effective_max().max(1))
    }

    pub fn active_connections(&self) -> u32 {
        self.core.metrics.active_connections.load(Ordering::Relaxed)
    }

    /// Effective capacity (permits issued by the pool).
    pub fn max_connections(&self) -> u32 {
        self.core.effective_max()
    }

    pub fn metrics(&self) -> AdaptivePoolMetrics {
        AdaptivePoolMetrics {
            utilization_percent: self
                .core
                .metrics
                .utilization_percent
                .load(Ordering::Relaxed) as f64,
            active_connections: self.core.metrics.active_connections.load(Ordering::Relaxed),
            max_connections: self.core.metrics.max_connections.load(Ordering::Relaxed),
            scale_up_count: self.core.metrics.scale_up_count.load(Ordering::Relaxed),
            scale_down_count: self.core.metrics.scale_down_count.load(Ordering::Relaxed),
            avg_wait_time_us: self.core.metrics.avg_wait_time_us.load(Ordering::Relaxed),
            total_acquired: self.core.metrics.total_acquired.load(Ordering::Relaxed),
            total_released: self.core.metrics.total_released.load(Ordering::Relaxed),
        }
    }

    /// Register a cleanup callback to be called when connections are dropped
    ///
    /// This allows external components (like the prepared statement cache) to
    /// clean up resources when a connection is returned to the pool.
    ///
    /// # Arguments
    ///
    /// * `callback` - Function to call with the connection ID when a connection is dropped
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use do_memory_storage_turso::pool::{AdaptiveConnectionPool, ConnectionId};
    /// use do_memory_storage_turso::PreparedStatementCache;
    ///
    /// # async fn example(pool: AdaptiveConnectionPool) {
    /// let cache = Arc::new(PreparedStatementCache::new(100));
    /// let cache_clone = Arc::clone(&cache);
    ///
    /// pool.set_cleanup_callback(Arc::new(move |conn_id: ConnectionId| {
    ///     cache_clone.clear_connection(conn_id);
    /// }));
    /// # }
    /// ```
    pub fn set_cleanup_callback(&self, callback: ConnectionCleanupCallback) {
        *self.cleanup_callback.write() = Some(callback);
        info!("Connection cleanup callback registered");
    }

    /// Remove the cleanup callback
    ///
    /// This disables automatic cleanup notifications.
    pub fn remove_cleanup_callback(&self) {
        *self.cleanup_callback.write() = None;
        info!("Connection cleanup callback removed");
    }

    /// Stop the background monitor and wait for it to finish.
    pub async fn shutdown(&self) {
        info!("Shutting down adaptive connection pool");

        // Take the handle out before awaiting so no lock is held across await.
        let handle = self.monitor.lock().take();
        if let Some(handle) = handle {
            handle.shutdown.notify_one();
            if let Err(error) = handle.task.await {
                warn!("Adaptive connection pool monitor stopped abnormally: {error}");
            }
        }

        info!("Adaptive connection pool shutdown complete");
    }
}

impl Drop for AdaptiveConnectionPool {
    fn drop(&mut self) {
        // Abort a monitor that was never explicitly shut down so it cannot
        // outlive the pool.
        if let Some(handle) = self.monitor.get_mut().take() {
            handle.shutdown.notify_one();
            handle.task.abort();
        }
    }
}

pub struct AdaptivePooledConnection {
    conn_id: ConnectionId,
    /// Shared adaptive metrics, owned so the guard never outlives its state.
    metrics: Arc<AdaptiveMetrics>,
    /// Shared current-max counter, owned for the same reason.
    current_max: Arc<AtomicU32>,
    permit: Option<OwnedSemaphorePermit>,
    connection: Option<libsql::Connection>,
    cleanup_callback: Option<ConnectionCleanupCallback>,
}

impl std::fmt::Debug for AdaptivePooledConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdaptivePooledConnection")
            .field("conn_id", &self.conn_id)
            .field("has_cleanup_callback", &self.cleanup_callback.is_some())
            .finish()
    }
}

// `AdaptivePooledConnection` is auto-`Send`/`Sync`: every field is composed of
// `Send + Sync` types (`Arc` over atomics, an owned semaphore permit, and
// `libsql::Connection`, which is itself `Send + Sync`). No `unsafe` impls are
// required now that the guard owns its shared state instead of raw pointers.

impl AdaptivePooledConnection {
    /// Get the unique connection identifier
    ///
    /// This ID is stable for the lifetime of the connection and can be used
    /// to associate cached data (like prepared statements) with the connection.
    pub fn connection_id(&self) -> ConnectionId {
        self.conn_id
    }

    /// Get a reference to the underlying database connection
    pub fn connection(&self) -> Option<&libsql::Connection> {
        self.connection.as_ref()
    }

    /// Take ownership of the underlying connection
    pub fn into_inner(mut self) -> Option<libsql::Connection> {
        self.connection.take()
    }
}

impl Drop for AdaptivePooledConnection {
    fn drop(&mut self) {
        if let Some(permit) = self.permit.take() {
            drop(permit);

            let active = self
                .metrics
                .active_connections
                .fetch_sub(1, Ordering::Relaxed);

            let max = self.current_max.load(Ordering::Acquire).max(1);

            let new_utilization = (active.saturating_sub(1) as f64 / max as f64) * 100.0;
            self.metrics
                .utilization_percent
                .store(new_utilization as u64, Ordering::Relaxed);

            self.metrics.total_released.fetch_add(1, Ordering::Relaxed);

            // Call cleanup callback if registered
            if let Some(callback) = &self.cleanup_callback {
                callback(self.conn_id);
            }
        }
    }
}

#[cfg(test)]
#[path = "adaptive_tests.rs"]
mod tests;
