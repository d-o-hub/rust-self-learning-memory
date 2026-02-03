//! Connection wrapper with stable ID for cache-aware pooling
//!
//! This module provides a wrapper around libsql::Connection that includes
//! a stable connection ID for use with the prepared statement cache.

use libsql::Connection;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::debug;

/// Global counter for generating unique connection IDs
static NEXT_CONNECTION_ID: AtomicU64 = AtomicU64::new(1);

/// Generate a new unique connection ID
pub fn generate_connection_id() -> u64 {
    NEXT_CONNECTION_ID.fetch_add(1, Ordering::Relaxed)
}

/// A database connection with a stable ID for prepared statement caching
///
/// This wrapper ensures that a connection has a stable ID throughout its lifetime,
/// enabling proper prepared statement cache management.
///
/// # Architecture
///
/// ```text
/// PooledConnection {
///     id: ConnectionId (stable for the lifetime of the connection)
///     connection: libsql::Connection (actual database connection)
///     created_at: Instant (for lifecycle management)
///     last_used: Instant (for idle detection)
/// }
/// ```
///
/// # Thread Safety
///
/// This type is `!Send` and `!Sync` because libsql::Connection is not thread-safe.
/// It should only be used within a single async task.
#[derive(Debug)]
pub struct PooledConnection {
    /// Stable connection ID for prepared statement cache
    id: u64,
    /// The actual database connection
    connection: Connection,
    /// When this connection was created
    created_at: std::time::Instant,
    /// When this connection was last used
    last_used: std::time::Instant,
}

impl PooledConnection {
    /// Create a new pooled connection with a unique ID
    ///
    /// # Arguments
    ///
    /// * `connection` - The libsql connection to wrap
    ///
    /// # Returns
    ///
    /// A new PooledConnection with a unique, stable ID
    pub fn new(connection: Connection) -> Self {
        let id = generate_connection_id();
        let now = std::time::Instant::now();

        debug!("Creating pooled connection with ID {}", id);

        Self {
            id,
            connection,
            created_at: now,
            last_used: now,
        }
    }

    /// Get the stable connection ID
    ///
    /// This ID remains constant for the lifetime of the connection and should
    /// be used for all prepared statement cache operations.
    ///
    /// # Returns
    ///
    /// The connection's unique identifier
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Get a reference to the underlying libsql connection
    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    /// Get a mutable reference to the underlying libsql connection
    pub fn connection_mut(&mut self) -> &mut Connection {
        &mut self.connection
    }

    /// Update the last-used timestamp
    pub fn touch(&mut self) {
        self.last_used = std::time::Instant::now();
    }

    /// Get the connection age (time since creation)
    pub fn age(&self) -> std::time::Duration {
        self.created_at.elapsed()
    }

    /// Get the idle time (time since last use)
    pub fn idle_time(&self) -> std::time::Duration {
        self.last_used.elapsed()
    }

    /// Take ownership of the underlying connection
    ///
    /// This consumes the wrapper and returns the raw libsql connection.
    pub fn into_inner(self) -> Connection {
        self.connection
    }

    /// Validate the connection is still healthy
    ///
    /// # Errors
    ///
    /// Returns an error if the connection is not healthy
    pub async fn validate(&self) -> anyhow::Result<()> {
        self.connection
            .query("SELECT 1", ())
            .await
            .map_err(|e| anyhow::anyhow!("Connection validation failed: {}", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_id_generation() {
        let id1 = generate_connection_id();
        let id2 = generate_connection_id();

        assert!(
            id2 > id1,
            "Connection IDs should be monotonically increasing"
        );
    }

    #[test]
    fn test_pooled_connection_creation() {
        // This test just verifies the struct can be created
        // Actual connection testing requires async context
        let _conn = PooledConnection {
            id: 1,
            connection: unsafe { std::mem::zeroed() }, // Dummy for test
            created_at: std::time::Instant::now(),
            last_used: std::time::Instant::now(),
        };
    }
}
