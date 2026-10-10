//! Scoped checkout helpers shared by every connection pool.
//!
//! Each pool exposes the same closure-based contract: the guard returned by the
//! pool's `get` is owned for the whole operation, so the semaphore permit, the
//! active-connection accounting, and any cleanup callback run only after the
//! closure — queries and row iteration included — has returned. A pool saturated
//! by one slow operation therefore rejects or waits for the next checkout
//! instead of handing out a connection whose permit was already released.

use do_memory_core::{Error, Result};
use libsql::Connection;

use super::{AdaptiveConnectionPool, ConnectionPool, KeepAlivePool};

impl ConnectionPool {
    /// Run an operation against a pooled connection, holding its permit for the
    /// entire closure.
    ///
    /// The guard returned by [`ConnectionPool::get`] is kept alive across `f`, so
    /// the permit and the active-connection accounting are released only after the
    /// operation has finished.
    ///
    /// # Errors
    ///
    /// Propagates the closure's error, or a checkout error such as a timeout when
    /// the pool is saturated.
    pub async fn with_connection<F, T>(&self, f: F) -> Result<T>
    where
        F: AsyncFnOnce(&Connection) -> Result<T>,
    {
        let guard = self.get().await?;
        let connection = guard
            .connection()
            .ok_or_else(|| Error::Storage("Pooled connection was already taken".to_string()))?;
        let result = f(connection).await;
        drop(guard);
        result
    }
}

impl AdaptiveConnectionPool {
    /// Run an operation against a pooled connection, holding its permit for the
    /// entire closure.
    ///
    /// The guard returned by [`AdaptiveConnectionPool::get`] is kept alive across
    /// `f`, so the permit, the active-connection accounting, and the registered
    /// cleanup callback run only after the operation has finished.
    ///
    /// # Errors
    ///
    /// Propagates the closure's error, or a checkout error such as a timeout when
    /// the pool is saturated.
    pub async fn with_connection<F, T>(&self, f: F) -> Result<T>
    where
        F: AsyncFnOnce(&Connection) -> Result<T>,
    {
        let guard = self.get().await?;
        let connection = guard.connection().ok_or_else(|| {
            Error::Storage("Adaptive pooled connection was already taken".to_string())
        })?;
        let result = f(connection).await;
        drop(guard);
        result
    }
}

impl KeepAlivePool {
    /// Run an operation against a pooled connection, holding its permit for the
    /// entire closure.
    ///
    /// The guard returned by [`KeepAlivePool::get`] (and the underlying pool
    /// permit it owns) is kept alive across `f`, so the active-connection
    /// accounting and the keep-alive statistics update only after the operation
    /// has finished.
    ///
    /// # Errors
    ///
    /// Propagates the closure's error, or a checkout error such as a timeout when
    /// the pool is saturated.
    pub async fn with_connection<F, T>(&self, f: F) -> Result<T>
    where
        F: AsyncFnOnce(&Connection) -> Result<T>,
    {
        let guard = self.get().await?;
        let connection = guard.connection()?;
        let result = f(connection).await;
        drop(guard);
        result
    }
}
