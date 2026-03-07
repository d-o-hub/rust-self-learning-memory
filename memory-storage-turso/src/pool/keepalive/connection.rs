//! Keep-Alive connection wrapper with tracking

use libsql::Connection;
use parking_lot::RwLock;
use std::mem::ManuallyDrop;
use std::sync::Arc;
use std::time::Instant;

use super::PooledConnection;
use super::config::KeepAliveStatistics;

/// A connection wrapper that tracks last used time
#[derive(Debug)]
pub struct KeepAliveConnection {
    /// The underlying pooled connection
    pooled: ManuallyDrop<PooledConnection>,
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
            pooled: ManuallyDrop::new(pooled),
            connection_id,
            last_used: RwLock::new(last_used),
            stats,
        }
    }

    /// Get a reference to the underlying connection
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying connection is not available.
    pub fn connection(&self) -> memory_core::Result<&Connection> {
        self.pooled.connection().ok_or_else(|| {
            memory_core::Error::Storage(
                "KeepAliveConnection: underlying connection is None".to_string(),
            )
        })
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

    /// Extract the owned connection from the underlying pooled connection
    ///
    /// This consumes the `KeepAliveConnection` and returns the owned `Connection`.
    /// Use this when you need to take ownership of the connection.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying connection is not available.
    pub fn into_connection(mut self) -> memory_core::Result<Connection> {
        // Safety: We're taking the pooled connection out and preventing the Drop
        // from running on self.pooled by using ManuallyDrop.
        let pooled = unsafe { ManuallyDrop::take(&mut self.pooled) };
        pooled.into_inner()
    }
}

impl Drop for KeepAliveConnection {
    fn drop(&mut self) {
        // Safety: We only drop the pooled connection if it wasn't already taken
        // by into_connection(). If into_connection was called, the ManuallyDrop
        // was already taken.
        unsafe {
            ManuallyDrop::drop(&mut self.pooled);
        }
        // Update stats through the Arc reference
        let mut stats = self.stats.write();
        if stats.active_connections > 0 {
            stats.active_connections -= 1;
        }
    }
}
