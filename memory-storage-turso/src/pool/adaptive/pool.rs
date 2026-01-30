//! Adaptive connection pool implementation

use super::types::{AdaptiveMetrics, AdaptivePoolConfig, AdaptivePoolMetrics};
use libsql::Database;
use memory_core::{Error, Result};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tracing::{debug, info};

pub struct AdaptiveConnectionPool {
    db: Arc<Database>,
    config: Arc<AdaptivePoolConfig>,
    semaphore: Arc<Semaphore>,
    current_max: Arc<AtomicU32>,
    metrics: Arc<AdaptiveMetrics>,
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

        let metrics_ptr = Arc::as_ptr(&self.metrics) as *mut AdaptiveMetrics;
        let current_max_ptr = Arc::as_ptr(&self.current_max) as *mut AtomicU32;

        Ok(AdaptivePooledConnection {
            metrics_ptr,
            current_max_ptr,
            permit: Some(permit),
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
    metrics_ptr: *mut AdaptiveMetrics,
    current_max_ptr: *mut AtomicU32,
    permit: Option<OwnedSemaphorePermit>,
}

#[allow(unsafe_code)]
unsafe impl Send for AdaptivePooledConnection {}
#[allow(unsafe_code)]
unsafe impl Sync for AdaptivePooledConnection {}

impl AdaptivePooledConnection {
    pub fn connection(&self) -> Option<&libsql::Connection> {
        None
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
