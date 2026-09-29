//! Keep-Alive connection wrapper with tracking

use libsql::Connection;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Instant;

use super::PooledConnection;
use super::config::KeepAliveStatistics;

/// A connection wrapper that tracks last used time
#[derive(Debug)]
pub struct KeepAliveConnection {
    /// The underlying pooled connection.
    ///
    /// `None` once [`KeepAliveConnection::into_connection`] has extracted it,
    /// which makes the custom `Drop` a no-op for the extracted case.
    pooled: Option<PooledConnection>,
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
            pooled: Some(pooled),
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
    pub fn connection(&self) -> do_memory_core::Result<&Connection> {
        self.pooled
            .as_ref()
            .and_then(PooledConnection::connection)
            .ok_or_else(|| {
                do_memory_core::Error::Storage(
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
    pub fn into_connection(mut self) -> do_memory_core::Result<Connection> {
        // Take the pooled connection out. The option is left as `None`, so the
        // `Drop` below will not run it a second time (no double drop), and the
        // extracted `Connection` stays usable until its caller drops it.
        let pooled = self.pooled.take().ok_or_else(|| {
            do_memory_core::Error::Storage(
                "KeepAliveConnection: underlying connection is None".to_string(),
            )
        })?;
        pooled.into_inner()
    }
}

impl Drop for KeepAliveConnection {
    fn drop(&mut self) {
        // Only drop a pooled connection that was not already extracted by
        // `into_connection`. When it is present, its own `Drop` releases the
        // underlying pool permit exactly once.
        drop(self.pooled.take());

        // Update stats through the Arc reference, exactly once per wrapper.
        let mut stats = self.stats.write();
        if stats.active_connections > 0 {
            stats.active_connections -= 1;
        }
    }
}
