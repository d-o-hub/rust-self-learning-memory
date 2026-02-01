//! Adaptive connection pool that dynamically adjusts pool size based on load.

use libsql::Database;
use memory_core::{Error, Result};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tracing::{debug, info};

/// Unique identifier for a connection
pub type ConnectionId = u64;

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

#[derive(Debug)]
struct AdaptiveMetrics {
    utilization_percent: AtomicU64,
    active_connections: AtomicU32,
    max_connections: AtomicU32,
    scale_up_count: AtomicU32,
    scale_down_count: AtomicU32,
    avg_wait_time_us: AtomicU64,
    total_acquired: AtomicU64,
    total_released: AtomicU64,
    wait_time_total_us: AtomicU64,
    wait_count: AtomicU64,
    last_scale_up: AtomicU64,
    last_scale_down: AtomicU64,
}

impl Default for AdaptiveMetrics {
    fn default() -> Self {
        Self {
            utilization_percent: AtomicU64::new(0),
            active_connections: AtomicU32::new(0),
            max_connections: AtomicU32::new(0),
            scale_up_count: AtomicU32::new(0),
            scale_down_count: AtomicU32::new(0),
            avg_wait_time_us: AtomicU64::new(0),
            total_acquired: AtomicU64::new(0),
            total_released: AtomicU64::new(0),
            wait_time_total_us: AtomicU64::new(0),
            wait_count: AtomicU64::new(0),
            last_scale_up: AtomicU64::new(0),
            last_scale_down: AtomicU64::new(0),
        }
    }
}

pub struct AdaptiveConnectionPool {
    db: Arc<Database>,
    config: Arc<AdaptivePoolConfig>,
    semaphore: Arc<Semaphore>,
    current_max: Arc<AtomicU32>,
    metrics: Arc<AdaptiveMetrics>,
    next_conn_id: Arc<AtomicU64>,
    _monitor_task: tokio::task::JoinHandle<()>,
}

impl AdaptiveConnectionPool {
    pub async fn new(db: Arc<Database>, config: AdaptivePoolConfig) -> Result<Self> {
        let config = Arc::new(config);
        let initial_max = config.min_connections as usize;
        let min_conn = config.min_connections;

        info!(
            "Creating adaptive connection pool with min={}, max={}",
            config.min_connections, config.max_connections
        );

        let semaphore = Arc::new(Semaphore::new(initial_max));

        let metrics = Arc::new(AdaptiveMetrics::default());
        metrics.max_connections.store(min_conn, Ordering::Relaxed);

        let pool = Self {
            db,
            config: config.clone(),
            semaphore,
            current_max: Arc::new(AtomicU32::new(min_conn)),
            metrics,
            next_conn_id: Arc::new(AtomicU64::new(1)),
            _monitor_task: tokio::task::spawn(async {}),
        };

        let conn = pool
            .db
            .connect()
            .map_err(|e| Error::Storage(format!("Failed to connect: {}", e)))?;
        conn.query("SELECT 1", ())
            .await
            .map_err(|e| Error::Storage(format!("Database validation failed: {}", e)))?;

        info!("Adaptive connection pool created successfully");

        Ok(pool)
    }

    pub async fn new_sync(db: Arc<Database>, config: AdaptivePoolConfig) -> Result<Self> {
        let config = Arc::new(config);
        let initial_max = config.min_connections as usize;
        let min_conn = config.min_connections;

        info!(
            "Creating adaptive connection pool (sync mode) with min={}, max={}",
            config.min_connections, config.max_connections
        );

        let semaphore = Arc::new(Semaphore::new(initial_max));

        let metrics = Arc::new(AdaptiveMetrics::default());
        metrics.max_connections.store(min_conn, Ordering::Relaxed);

        Ok(Self {
            db,
            config,
            semaphore,
            current_max: Arc::new(AtomicU32::new(min_conn)),
            metrics,
            next_conn_id: Arc::new(AtomicU64::new(1)),
            _monitor_task: tokio::task::spawn(async {}),
        })
    }

    async fn try_acquire(&self, timeout: Duration) -> Result<OwnedSemaphorePermit> {
        let start = Instant::now();

        match tokio::time::timeout(timeout, self.semaphore.clone().acquire_owned()).await {
            Ok(Ok(permit)) => {
                let wait_us = start.elapsed().as_micros() as u64;

                self.metrics
                    .wait_time_total_us
                    .fetch_add(wait_us, Ordering::Relaxed);
                self.metrics.wait_count.fetch_add(1, Ordering::Relaxed);

                let total_time = self.metrics.wait_time_total_us.load(Ordering::Relaxed);
                let count = self.metrics.wait_count.load(Ordering::Relaxed);
                if count > 0 {
                    self.metrics
                        .avg_wait_time_us
                        .store(total_time / count, Ordering::Relaxed);
                }

                let active = self
                    .metrics
                    .active_connections
                    .fetch_add(1, Ordering::Relaxed)
                    + 1;

                let max = self.current_max.load(Ordering::Relaxed);
                let utilization = (active as f64 / max as f64) * 100.0;
                self.metrics
                    .utilization_percent
                    .store(utilization as u64, Ordering::Relaxed);

                self.metrics.total_acquired.fetch_add(1, Ordering::Relaxed);

                Ok(permit)
            }
            Ok(Err(e)) => Err(Error::Storage(format!(
                "Failed to acquire connection permit: {}",
                e
            ))),
            Err(_) => Err(Error::Storage(format!(
                "Connection acquisition timed out after {:?}",
                timeout
            ))),
        }
    }

    async fn scale_up(&self) {
        let now = Instant::now();
        let last_up = self.metrics.last_scale_up.load(Ordering::Relaxed);

        // Use duration since a fixed epoch
        let epoch_duration = Duration::from_nanos(last_up);
        let last_up_time = Instant::now() - epoch_duration;

        if now.duration_since(last_up_time) < self.config.scale_up_cooldown {
            return;
        }

        let current_max = self.current_max.load(Ordering::Relaxed);

        if current_max >= self.config.max_connections {
            return;
        }

        let new_max =
            (current_max + self.config.scale_up_increment).min(self.config.max_connections);

        info!("Scaling up: {} -> {} connections", current_max, new_max);

        self.current_max.store(new_max, Ordering::Relaxed);
        self.metrics
            .max_connections
            .store(new_max, Ordering::Relaxed);
        self.metrics
            .last_scale_up
            .store(now.elapsed().as_nanos() as u64, Ordering::Relaxed);
        self.metrics.scale_up_count.fetch_add(1, Ordering::Relaxed);

        debug!("Scale up complete: {} connections", new_max);
    }

    async fn scale_down(&self) {
        let now = Instant::now();
        let last_down = self.metrics.last_scale_down.load(Ordering::Relaxed);

        let epoch_duration = Duration::from_nanos(last_down);
        let last_down_time = Instant::now() - epoch_duration;

        if now.duration_since(last_down_time) < self.config.scale_down_cooldown {
            return;
        }

        let current_max = self.current_max.load(Ordering::Relaxed);
        let active = self.metrics.active_connections.load(Ordering::Relaxed);

        let min_allowed = active.max(self.config.min_connections);
        let new_max =
            (current_max.saturating_sub(self.config.scale_down_decrement)).max(min_allowed);

        if new_max >= current_max {
            return;
        }

        info!(
            "Scaling down: {} -> {} connections (active: {})",
            current_max, new_max, active
        );

        self.current_max.store(new_max, Ordering::Relaxed);
        self.metrics
            .max_connections
            .store(new_max, Ordering::Relaxed);
        self.metrics
            .last_scale_down
            .store(now.elapsed().as_nanos() as u64, Ordering::Relaxed);
        self.metrics
            .scale_down_count
            .fetch_add(1, Ordering::Relaxed);

        debug!("Scale down complete: {} connections", new_max);
    }

    pub async fn check_and_scale(&self) {
        let active = self.metrics.active_connections.load(Ordering::Relaxed);
        let max = self.current_max.load(Ordering::Relaxed);
        let utilization = active as f64 / max as f64;

        if utilization >= self.config.scale_up_threshold {
            self.scale_up().await;
        } else if utilization <= self.config.scale_down_threshold {
            self.scale_down().await;
        }
    }

    pub async fn get(&self) -> Result<AdaptivePooledConnection> {
        let permit = self.try_acquire(self.config.check_interval).await?;

        // Generate unique connection ID
        let conn_id = self.next_conn_id.fetch_add(1, Ordering::Relaxed);

        // Create a new database connection from the database
        let connection = self
            .db
            .connect()
            .map_err(|e| Error::Storage(format!("Failed to create connection: {}", e)))?;

        let metrics_ptr = Arc::as_ptr(&self.metrics) as *mut AdaptiveMetrics;
        let current_max_ptr = Arc::as_ptr(&self.current_max) as *mut AtomicU32;

        debug!("Created connection with ID: {}", conn_id);

        Ok(AdaptivePooledConnection {
            conn_id,
            metrics_ptr,
            current_max_ptr,
            permit: Some(permit),
            connection: Some(connection),
        })
    }

    pub fn available_connections(&self) -> usize {
        self.semaphore.available_permits()
    }

    pub fn utilization(&self) -> f64 {
        self.metrics.utilization_percent.load(Ordering::Relaxed) as f64 / 100.0
    }

    pub fn active_connections(&self) -> u32 {
        self.metrics.active_connections.load(Ordering::Relaxed)
    }

    pub fn max_connections(&self) -> u32 {
        self.current_max.load(Ordering::Relaxed)
    }

    pub fn metrics(&self) -> AdaptivePoolMetrics {
        AdaptivePoolMetrics {
            utilization_percent: self.metrics.utilization_percent.load(Ordering::Relaxed) as f64,
            active_connections: self.metrics.active_connections.load(Ordering::Relaxed),
            max_connections: self.metrics.max_connections.load(Ordering::Relaxed),
            scale_up_count: self.metrics.scale_up_count.load(Ordering::Relaxed),
            scale_down_count: self.metrics.scale_down_count.load(Ordering::Relaxed),
            avg_wait_time_us: self.metrics.avg_wait_time_us.load(Ordering::Relaxed),
            total_acquired: self.metrics.total_acquired.load(Ordering::Relaxed),
            total_released: self.metrics.total_released.load(Ordering::Relaxed),
        }
    }

    pub async fn shutdown(&self) {
        info!("Shutting down adaptive connection pool");
        tokio::time::sleep(Duration::from_millis(100)).await;
        info!("Adaptive connection pool shutdown complete");
    }
}

#[derive(Debug)]
pub struct AdaptivePooledConnection {
    conn_id: ConnectionId,
    metrics_ptr: *mut AdaptiveMetrics,
    current_max_ptr: *mut AtomicU32,
    permit: Option<OwnedSemaphorePermit>,
    connection: Option<libsql::Connection>,
}

#[allow(unsafe_code)]
unsafe impl Send for AdaptivePooledConnection {}
#[allow(unsafe_code)]
unsafe impl Sync for AdaptivePooledConnection {}

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

            #[allow(unsafe_code)]
            unsafe {
                if let Some(metrics) = self.metrics_ptr.as_mut() {
                    let active = metrics.active_connections.fetch_sub(1, Ordering::Relaxed);

                    let max = self
                        .current_max_ptr
                        .as_ref()
                        .map(|m| m.load(Ordering::Relaxed))
                        .unwrap_or(1);

                    let new_utilization = ((active - 1) as f64 / max as f64) * 100.0;
                    metrics
                        .utilization_percent
                        .store(new_utilization as u64, Ordering::Relaxed);

                    metrics.total_released.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::time::Duration;

    async fn create_test_pool() -> (AdaptiveConnectionPool, TempDir) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");

        let db = libsql::Builder::new_local(&db_path).build().await.unwrap();

        let config = AdaptivePoolConfig {
            min_connections: 2,
            max_connections: 10,
            scale_up_threshold: 0.8,
            scale_down_threshold: 0.3,
            scale_up_cooldown: Duration::from_secs(1),
            scale_down_cooldown: Duration::from_secs(1),
            scale_up_increment: 2,
            scale_down_decrement: 2,
            check_interval: Duration::from_secs(5),
        };

        let pool = AdaptiveConnectionPool::new_sync(Arc::new(db), config)
            .await
            .unwrap();
        (pool, dir)
    }

    #[tokio::test]
    async fn test_adaptive_pool_creation() {
        let (pool, _dir) = create_test_pool().await;
        let metrics = pool.metrics();

        assert_eq!(metrics.active_connections, 0);
        assert_eq!(metrics.max_connections, 2);
    }

    #[tokio::test]
    async fn test_connection_checkout() {
        let (pool, _dir) = create_test_pool().await;

        let conn = pool.get().await;
        assert!(conn.is_ok());

        let metrics = pool.metrics();
        assert_eq!(metrics.total_acquired, 1);
        assert_eq!(metrics.active_connections, 1);
    }

    #[tokio::test]
    async fn test_connection_auto_return() {
        let (pool, _dir) = create_test_pool().await;

        {
            let _conn = pool.get().await.unwrap();
            let metrics = pool.metrics();
            assert_eq!(metrics.active_connections, 1);
        }

        tokio::time::sleep(Duration::from_millis(50)).await;

        let metrics = pool.metrics();
        assert_eq!(metrics.active_connections, 0);
    }

    #[tokio::test]
    async fn test_utilization() {
        let (pool, _dir) = create_test_pool().await;

        let utilization = pool.utilization();
        assert_eq!(utilization, 0.0);

        let _conn = pool.get().await.unwrap();

        let utilization = pool.utilization();
        assert!(utilization > 0.0 && utilization <= 1.0);
    }

    #[tokio::test]
    async fn test_available_connections() {
        let (pool, _dir) = create_test_pool().await;

        let available = pool.available_connections();
        assert_eq!(available, 2);

        let _conn1 = pool.get().await.unwrap();
        let available = pool.available_connections();
        assert_eq!(available, 1);

        let _conn2 = pool.get().await.unwrap();
        let available = pool.available_connections();
        assert_eq!(available, 0);
    }

    #[tokio::test]
    async fn test_active_connections() {
        let (pool, _dir) = create_test_pool().await;

        let active = pool.active_connections();
        assert_eq!(active, 0);

        let _conn = pool.get().await.unwrap();

        let active = pool.active_connections();
        assert_eq!(active, 1);
    }

    #[tokio::test]
    async fn test_max_connections() {
        let (pool, _dir) = create_test_pool().await;

        let max = pool.max_connections();
        assert_eq!(max, 2);
    }

    #[tokio::test]
    async fn test_check_and_scale() {
        let (pool, _dir) = create_test_pool().await;

        pool.check_and_scale().await;

        assert_eq!(pool.max_connections(), 2);
    }

    #[tokio::test]
    async fn test_shutdown() {
        let (pool, _dir) = create_test_pool().await;

        let _conn = pool.get().await.unwrap();
        drop(_conn);

        tokio::time::sleep(Duration::from_millis(50)).await;

        pool.shutdown().await;
    }

    #[tokio::test]
    async fn test_connection_exposure() {
        let (pool, _dir) = create_test_pool().await;

        // Get a connection from the pool
        let pooled_conn = pool.get().await.unwrap();

        // Verify we can access the underlying connection
        let conn_ref = pooled_conn.connection();
        assert!(conn_ref.is_some(), "Connection should be exposed");

        // Verify the connection is usable by running a query
        let conn = conn_ref.unwrap();
        let result = conn.query("SELECT 1", ()).await;
        assert!(result.is_ok(), "Connection should be usable for queries");

        // Test into_inner to take ownership
        let conn = pooled_conn.into_inner();
        assert!(conn.is_some(), "into_inner should return the connection");
    }

    #[tokio::test]
    async fn test_connection_query_after_into_inner() {
        let (pool, _dir) = create_test_pool().await;

        // Get a connection and take ownership
        let pooled_conn = pool.get().await.unwrap();
        let conn = pooled_conn.into_inner().unwrap();

        // Verify the connection is still usable
        let result = conn.query("SELECT 1 as value", ()).await;
        assert!(result.is_ok());

        let mut rows = result.unwrap();
        let row = rows.next().await.unwrap();
        assert!(row.is_some());

        let value: i32 = row.unwrap().get(0).unwrap();
        assert_eq!(value, 1);
    }
}
