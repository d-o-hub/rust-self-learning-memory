//! Connection pool configuration and statistics
//!
//! Provides configuration structs, statistics tracking, and connection guards.

use do_memory_core::{Error, Result};
use libsql::Connection;
use parking_lot::RwLock;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize};
use std::time::Duration;
use tokio::sync::OwnedSemaphorePermit;

/// Configuration for connection pool
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Minimum number of connections to maintain in the pool
    pub min_connections: usize,
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Maximum time to wait for a connection (seconds)
    pub connection_timeout: Duration,
    /// Enable connection health checks
    pub enable_health_check: bool,
    /// Health check timeout
    pub health_check_timeout: Duration,
    /// Timeout for acquiring a connection from the pool (milliseconds).
    /// Alias for `connection_timeout` — preferred for new code.
    pub acquire_timeout_ms: u64,
    /// Idle timeout for connections (milliseconds).
    /// Connections idle longer than this may be dropped. 0 = no idle timeout.
    pub idle_timeout_ms: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 1,
            max_connections: 10,
            connection_timeout: Duration::from_secs(5),
            enable_health_check: true,
            health_check_timeout: Duration::from_secs(2),
            acquire_timeout_ms: 5000,
            idle_timeout_ms: 0,
        }
    }
}

/// Atomic pool metrics for lock-free monitoring
///
/// Uses atomic types for safe concurrent access without locking.
/// Suitable for real-time dashboards and health-check endpoints.
#[derive(Debug)]
pub struct PoolMetrics {
    /// Current number of active (checked out) connections
    pub active_connections: AtomicUsize,
    /// Total connections acquired (cumulative)
    pub total_acquired: AtomicU64,
    /// Total wait time for acquiring connections (milliseconds)
    pub total_wait_ms: AtomicU64,
    /// Number of reconnection attempts
    pub reconnect_count: AtomicU64,
}

impl Default for PoolMetrics {
    fn default() -> Self {
        Self {
            active_connections: AtomicUsize::new(0),
            total_acquired: AtomicU64::new(0),
            total_wait_ms: AtomicU64::new(0),
            reconnect_count: AtomicU64::new(0),
        }
    }
}

impl PoolMetrics {
    /// Record a connection acquisition
    pub fn record_acquire(&self, wait_ms: u64) {
        self.active_connections
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.total_acquired
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.total_wait_ms
            .fetch_add(wait_ms, std::sync::atomic::Ordering::Relaxed);
    }

    /// Record a connection release
    pub fn record_release(&self) {
        self.active_connections
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Record a reconnection attempt
    pub fn record_reconnect(&self) {
        self.reconnect_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Snapshot current metrics as plain values
    pub fn snapshot(&self) -> PoolMetricsSnapshot {
        PoolMetricsSnapshot {
            active_connections: self
                .active_connections
                .load(std::sync::atomic::Ordering::Relaxed),
            total_acquired: self
                .total_acquired
                .load(std::sync::atomic::Ordering::Relaxed),
            total_wait_ms: self
                .total_wait_ms
                .load(std::sync::atomic::Ordering::Relaxed),
            reconnect_count: self
                .reconnect_count
                .load(std::sync::atomic::Ordering::Relaxed),
        }
    }
}

/// Snapshot of pool metrics (plain values, safe to send across threads)
#[derive(Debug, Clone, Default)]
pub struct PoolMetricsSnapshot {
    /// Current active connections
    pub active_connections: usize,
    /// Total connections acquired
    pub total_acquired: u64,
    /// Total wait time in milliseconds
    pub total_wait_ms: u64,
    /// Number of reconnection attempts
    pub reconnect_count: u64,
}

/// Pool statistics for monitoring (legacy, non-atomic)
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
    pub fn update_averages(&mut self) {
        if self.total_checkouts > 0 {
            self.avg_wait_time_ms = self.total_wait_time_ms / self.total_checkouts as u64;
        }
    }
}

/// A guard that returns a permit to the pool when dropped
#[derive(Debug)]
pub struct PooledConnection {
    pub(super) connection: Option<Connection>,
    pub(super) _permit: OwnedSemaphorePermit,
    pub(super) stats: Arc<RwLock<PoolStatistics>>,
    pub(super) metrics: Option<Arc<PoolMetrics>>,
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
        // Also update atomic metrics if available
        if let Some(ref metrics) = self.metrics {
            metrics.record_release();
        }
    }
}
