//! Connection pool for Turso/libSQL database connections
//!
//! Provides efficient connection management, concurrency limits, and performance monitoring.
//!
//! Note: libSQL's Database is already a connection factory. This pool adds:
//! - Concurrency limits via semaphore
//! - Connection health validation
//! - Performance metrics and monitoring
//! - Graceful lifecycle management

use libsql::{Connection, Database};
use memory_core::{Error, Result};
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tracing::{debug, info, warn};

/// Configuration for connection pool
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Maximum time to wait for a connection (seconds)
    pub connection_timeout: Duration,
    /// Enable connection health checks
    pub enable_health_check: bool,
    /// Health check timeout
    pub health_check_timeout: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            connection_timeout: Duration::from_secs(5),
            enable_health_check: true,
            health_check_timeout: Duration::from_secs(2),
        }
    }
}

/// Pool statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct PoolStatistics {
    /// Total connections created
    pub total_created: usize,
    /// Total connections that passed health check
    pub total_health_checks_passed: usize,
    /// Total connections failed health check
    pub total_health_checks_failed: usize,
    /// Current number of active (checked out) connections
    pub active_connections: usize,
    /// Total checkout wait time (milliseconds)
    pub total_wait_time_ms: u64,
    /// Number of checkouts
    pub total_checkouts: usize,
    /// Average wait time per checkout (milliseconds)
    pub avg_wait_time_ms: u64,
}

impl PoolStatistics {
    fn update_averages(&mut self) {
        if self.total_checkouts > 0 {
            self.avg_wait_time_ms = self.total_wait_time_ms / self.total_checkouts as u64;
        }
    }
}

/// A guard that returns a permit to the pool when dropped
#[derive(Debug)]
pub struct PooledConnection {
    connection: Option<Connection>,
    _permit: OwnedSemaphorePermit,
    stats: Arc<RwLock<PoolStatistics>>,
}

impl PooledConnection {
    /// Get a reference to the underlying connection
    pub fn connection(&self) -> Option<&Connection> {
        self.connection.as_ref()
    }

    /// Take ownership of the connection
    pub fn into_inner(mut self) -> Result<Connection> {
        self.connection
            .take()
            .ok_or_else(|| Error::Storage("Connection already taken".to_string()))
    }
}

impl Drop for PooledConnection {
    fn drop(&mut self) {
        // Decrement active connections when the guard is dropped
        // Using parking_lot's RwLock which supports blocking operations in Drop
        let mut stats = self.stats.write();
        if stats.active_connections > 0 {
            stats.active_connections -= 1;
        }
    }
}

/// Connection pool for managing database connections
///
/// This pool provides:
/// - Concurrency limits via semaphore (max_connections)
/// - Connection health validation
/// - Performance metrics
/// - Graceful shutdown
pub struct ConnectionPool {
    db: Arc<Database>,
    config: PoolConfig,
    semaphore: Arc<Semaphore>,
    stats: Arc<RwLock<PoolStatistics>>,
}

impl ConnectionPool {
    /// Create a new connection pool
    ///
    /// # Arguments
    ///
    /// * `db` - Database instance to create connections from
    /// * `config` - Pool configuration
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use libsql::Builder;
    /// use memory_storage_turso::pool::{ConnectionPool, PoolConfig};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let db = Builder::new_local("test.db").build().await?;
    /// let config = PoolConfig::default();
    /// let pool = ConnectionPool::new(Arc::new(db), config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(db: Arc<Database>, config: PoolConfig) -> Result<Self> {
        info!(
            "Creating connection pool with max_connections={}",
            config.max_connections
        );

        // Create a semaphore wrapped in Arc for shared ownership
        let semaphore = Arc::new(Semaphore::new(config.max_connections));
        let stats = Arc::new(RwLock::new(PoolStatistics::default()));

        let pool = Self {
            db,
            config,
            semaphore,
            stats,
        };

        // Validate database connectivity
        pool.validate_database().await?;

        info!("Connection pool created successfully");
        Ok(pool)
    }

    /// Validate database connectivity
    async fn validate_database(&self) -> Result<()> {
        let conn = self
            .db
            .connect()
            .map_err(|e| Error::Storage(format!("Failed to connect to database: {}", e)))?;

        conn.query("SELECT 1", ())
            .await
            .map_err(|e| Error::Storage(format!("Database validation failed: {}", e)))?;

        Ok(())
    }

    /// Create a new database connection
    async fn create_connection(&self) -> Result<Connection> {
        let conn = self
            .db
            .connect()
            .map_err(|e| Error::Storage(format!("Failed to create connection: {}", e)))?;

        // Update statistics
        {
            let mut stats = self.stats.write();
            stats.total_created += 1;
        }

        Ok(conn)
    }

    /// Get a connection from the pool
    ///
    /// This will:
    /// 1. Wait for a semaphore permit (respects max_connections limit)
    /// 2. Create a new connection from the database
    /// 3. Optionally validate the connection health
    /// 4. Return a PooledConnection guard that releases the permit on drop
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Timeout waiting for available connection slot
    /// - Failed to create connection
    /// - Connection health check fails
    pub async fn get(&self) -> Result<PooledConnection> {
        let start = Instant::now();

        // Acquire an owned semaphore permit (limits concurrent connections)
        let owned_permit_fut = self.semaphore.clone().acquire_owned();
        let permit = tokio::time::timeout(self.config.connection_timeout, owned_permit_fut)
            .await
            .map_err(|_| {
                Error::Storage(format!(
                    "Connection pool timeout after {:?}: max {} connections in use",
                    self.config.connection_timeout, self.config.max_connections
                ))
            })?
            .map_err(|e| Error::Storage(format!("Failed to acquire connection permit: {}", e)))?;

        let wait_time = start.elapsed();

        // Create a new connection
        let conn = self.create_connection().await?;

        // Validate connection health if enabled
        if self.config.enable_health_check {
            if let Err(e) = self.validate_connection_health(&conn).await {
                let mut stats = self.stats.write();
                stats.total_health_checks_failed += 1;
                return Err(e);
            }

            let mut stats = self.stats.write();
            stats.total_health_checks_passed += 1;
        }

        // Update statistics
        {
            let mut stats = self.stats.write();
            stats.total_checkouts += 1;
            stats.total_wait_time_ms += wait_time.as_millis() as u64;
            stats.active_connections += 1;
            stats.update_averages();
        }

        debug!(
            "Connection acquired (wait: {:?}, active: {})",
            wait_time,
            self.stats.read().active_connections
        );

        Ok(PooledConnection {
            connection: Some(conn),
            _permit: permit,
            stats: Arc::clone(&self.stats),
        })
    }

    /// Validate a connection is still healthy
    async fn validate_connection_health(&self, conn: &Connection) -> Result<()> {
        tokio::time::timeout(self.config.health_check_timeout, conn.query("SELECT 1", ()))
            .await
            .map_err(|_| Error::Storage("Connection health check timeout".to_string()))?
            .map_err(|e| Error::Storage(format!("Connection health check failed: {}", e)))?;

        Ok(())
    }

    /// Get current pool statistics
    pub async fn statistics(&self) -> PoolStatistics {
        self.stats.read().clone()
    }

    /// Get current pool utilization (0.0 to 1.0)
    pub async fn utilization(&self) -> f32 {
        let stats = self.stats.read();
        if self.config.max_connections == 0 {
            return 0.0;
        }
        stats.active_connections as f32 / self.config.max_connections as f32
    }

    /// Get number of available connection slots
    pub async fn available_connections(&self) -> usize {
        let stats = self.stats.read();
        self.config
            .max_connections
            .saturating_sub(stats.active_connections)
    }

    /// Check if pool has available capacity
    pub async fn has_capacity(&self) -> bool {
        self.available_connections().await > 0
    }

    /// Gracefully shutdown the pool
    ///
    /// Waits for active connections to be returned (up to 30 seconds).
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down connection pool");

        let shutdown_timeout = Duration::from_secs(30);
        let start = Instant::now();

        while start.elapsed() < shutdown_timeout {
            let active = self.stats.read().active_connections;
            if active == 0 {
                break;
            }

            debug!("Waiting for {} active connections to complete", active);
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        let final_active = self.stats.read().active_connections;
        if final_active > 0 {
            warn!(
                "Shutdown completed with {} active connections still in use",
                final_active
            );
        } else {
            info!("Connection pool shutdown complete");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_pool() -> (ConnectionPool, TempDir) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");

        let db = libsql::Builder::new_local(&db_path).build().await.unwrap();

        let config = PoolConfig {
            max_connections: 5,
            connection_timeout: Duration::from_secs(5),
            enable_health_check: true,
            health_check_timeout: Duration::from_secs(2),
        };

        let pool = ConnectionPool::new(Arc::new(db), config).await.unwrap();
        (pool, dir)
    }

    #[tokio::test]
    async fn test_pool_creation() {
        let (pool, _dir) = create_test_pool().await;
        let stats = pool.statistics().await;

        // Pool should be created but no connections yet
        assert_eq!(stats.total_checkouts, 0);
        assert_eq!(stats.active_connections, 0);
    }

    #[tokio::test]
    async fn test_connection_checkout() {
        let (pool, _dir) = create_test_pool().await;

        let conn = pool.get().await;
        assert!(conn.is_ok());

        let stats = pool.statistics().await;
        assert_eq!(stats.total_checkouts, 1);
        assert_eq!(stats.active_connections, 1);
        assert!(stats.total_created >= 1);
    }

    #[tokio::test]
    async fn test_connection_auto_return() {
        let (pool, _dir) = create_test_pool().await;

        {
            let _conn = pool.get().await.unwrap();
            let stats = pool.statistics().await;
            assert_eq!(stats.active_connections, 1);
        }

        // Wait for drop to complete
        tokio::time::sleep(Duration::from_millis(50)).await;

        let stats = pool.statistics().await;
        assert_eq!(stats.active_connections, 0);
    }

    #[tokio::test]
    async fn test_concurrent_checkouts() {
        let (pool, _dir) = create_test_pool().await;
        let pool = Arc::new(pool);

        let mut handles = vec![];

        for i in 0..3 {
            let pool_clone = Arc::clone(&pool);
            let handle = tokio::spawn(async move {
                let conn = pool_clone.get().await;
                assert!(conn.is_ok(), "Checkout {} failed", i);

                // Simulate work
                tokio::time::sleep(Duration::from_millis(10)).await;

                conn
            });
            handles.push(handle);
        }

        // Wait for all checkouts to complete
        for handle in handles {
            let _ = handle.await.unwrap();
        }

        let stats = pool.statistics().await;
        assert_eq!(stats.total_checkouts, 3);
        assert!(stats.total_created >= 3);
    }

    #[tokio::test]
    async fn test_pool_statistics() {
        let (pool, _dir) = create_test_pool().await;

        let _conn = pool.get().await.unwrap();

        let stats = pool.statistics().await;
        assert_eq!(stats.total_checkouts, 1);
        assert_eq!(stats.active_connections, 1);
        assert!(stats.total_created >= 1);
        // total_wait_time_ms is u64, always >= 0
    }

    #[tokio::test]
    async fn test_average_wait_time() {
        let (pool, _dir) = create_test_pool().await;

        let _conn1 = pool.get().await.unwrap();
        let _conn2 = pool.get().await.unwrap();

        let stats = pool.statistics().await;
        assert_eq!(stats.total_checkouts, 2);
        // avg_wait_time_ms is u64, always >= 0
    }

    #[tokio::test]
    async fn test_pool_utilization() {
        let (pool, _dir) = create_test_pool().await;

        let utilization = pool.utilization().await;
        assert_eq!(utilization, 0.0);

        let _conn = pool.get().await.unwrap();

        let utilization = pool.utilization().await;
        assert!(utilization > 0.0 && utilization <= 1.0);
    }

    #[tokio::test]
    async fn test_available_connections() {
        let (pool, _dir) = create_test_pool().await;

        let available = pool.available_connections().await;
        assert_eq!(available, 5);

        let _conn1 = pool.get().await.unwrap();
        let available = pool.available_connections().await;
        assert_eq!(available, 4);

        let _conn2 = pool.get().await.unwrap();
        let available = pool.available_connections().await;
        assert_eq!(available, 3);
    }

    #[tokio::test]
    async fn test_has_capacity() {
        let (pool, _dir) = create_test_pool().await;

        assert!(pool.has_capacity().await);

        let _conns: Vec<_> = futures::future::join_all((0..5).map(|_| pool.get())).await;

        assert!(!pool.has_capacity().await);
    }

    #[tokio::test]
    async fn test_graceful_shutdown() {
        let (pool, _dir) = create_test_pool().await;

        let _conn = pool.get().await.unwrap();
        drop(_conn);

        // Wait for drop to complete
        tokio::time::sleep(Duration::from_millis(50)).await;

        let result = pool.shutdown().await;
        assert!(result.is_ok());

        let stats = pool.statistics().await;
        assert_eq!(stats.active_connections, 0);
    }

    #[tokio::test]
    async fn test_connection_timeout() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");

        let db = libsql::Builder::new_local(&db_path).build().await.unwrap();

        let config = PoolConfig {
            max_connections: 1,
            connection_timeout: Duration::from_millis(100),
            enable_health_check: false,
            health_check_timeout: Duration::from_secs(2),
        };

        let pool = Arc::new(ConnectionPool::new(Arc::new(db), config).await.unwrap());

        // Get the only available connection
        let _conn1 = pool.get().await.unwrap();

        // Try to get another connection - should timeout
        let result = pool.get().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timeout"));
    }

    #[tokio::test]
    async fn test_health_check() {
        let (pool, _dir) = create_test_pool().await;

        let conn = pool.get().await.unwrap();

        let stats = pool.statistics().await;
        assert_eq!(stats.total_health_checks_passed, 1);
        assert_eq!(stats.total_health_checks_failed, 0);

        drop(conn);
    }

    #[tokio::test]
    async fn test_connection_usage() {
        let (pool, _dir) = create_test_pool().await;

        let conn = pool.get().await.unwrap();

        // Use the connection
        let result = conn.connection().unwrap().query("SELECT 1", ()).await;
        assert!(result.is_ok());

        drop(conn);
    }

    #[tokio::test]
    async fn test_high_concurrency() {
        let (pool, _dir) = create_test_pool().await;
        let pool = Arc::new(pool);

        let mut handles = vec![];

        // Spawn 20 concurrent tasks (more than pool size of 5)
        for i in 0..20 {
            let pool_clone = Arc::clone(&pool);
            let handle = tokio::spawn(async move {
                let conn = pool_clone.get().await;
                assert!(conn.is_ok(), "Checkout {} failed", i);

                // Simulate work
                tokio::time::sleep(Duration::from_millis(5)).await;

                conn
            });
            handles.push(handle);
        }

        // Wait for all to complete
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok());
        }

        let stats = pool.statistics().await;
        assert_eq!(stats.total_checkouts, 20);
        assert!(stats.total_created >= 5);
    }
}
