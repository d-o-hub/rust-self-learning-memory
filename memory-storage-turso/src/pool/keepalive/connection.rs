//! Keep-Alive connection wrapper with tracking

use libsql::Connection;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Instant;

use super::config::KeepAliveStatistics;
use super::PooledConnection;

/// A connection wrapper that tracks last used time
#[derive(Debug)]
pub struct KeepAliveConnection {
    /// The underlying pooled connection
    pooled: PooledConnection,
    /// The connection ID for tracking
    connection_id: usize,
    /// Timestamp when this connection was last used
    last_used: RwLock<Instant>,
    /// Shared reference to stats for updating on drop
    stats: Arc<RwLock<KeepAliveStatistics>>,
}

impl KeepAliveConnection {
    /// Create a new keep-alive connection wrapper
    pub fn new(
        pooled: PooledConnection,
        connection_id: usize,
        last_used: Instant,
        stats: Arc<RwLock<KeepAliveStatistics>>,
    ) -> Self {
        Self {
            pooled,
            connection_id,
            last_used: RwLock::new(last_used),
            stats,
        }
    }

    /// Get a reference to the underlying connection
    pub fn connection(&self) -> &Connection {
        self.pooled.connection().expect("Connection should exist")
    }

    /// Get the connection ID
    pub fn connection_id(&self) -> usize {
        self.connection_id
    }

    /// Get the last used timestamp
    pub fn last_used(&self) -> Instant {
        *self.last_used.read()
    }

    /// Update the last used timestamp
    pub fn update_last_used(&self) {
        let mut last_used = self.last_used.write();
        *last_used = Instant::now();
    }
}

impl Drop for KeepAliveConnection {
    fn drop(&mut self) {
        // Update stats through the Arc reference
        let mut stats = self.stats.write();
        if stats.active_connections > 0 {
            stats.active_connections -= 1;
        }
    }
}
