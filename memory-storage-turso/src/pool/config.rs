//! Connection pool configuration and statistics
//!
//! Provides configuration structs, statistics tracking, and connection guards.

use libsql::Connection;
use memory_core::{Error, Result};
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::OwnedSemaphorePermit;

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
