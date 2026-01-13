//! Connection pool for Turso/libSQL database connections
//!
//! Provides efficient connection management, concurrency limits, and performance monitoring.
//!
//! Note: libSQL's Database is already a connection factory. This pool adds:
//! - Concurrency limits via semaphore
//! - Connection health validation
//! - Performance metrics and monitoring
//! - Graceful lifecycle management

mod config;

pub use config::{PoolConfig, PoolStatistics, PooledConnection};

use libsql::{Connection, Database};
use memory_core::{Error, Result};
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tracing::{debug, info, warn};

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
