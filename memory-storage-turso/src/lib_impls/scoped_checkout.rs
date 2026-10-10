//! Scoped connection checkout for TursoStorage.
//!
//! Historically `TursoStorage::get_connection` returned a raw `libsql::Connection`
//! and, when a pool was configured, consumed the guard that owned the pool permit:
//! the permit was released *before* the caller ran any SQL, so the pool reported
//! free capacity while the operation was still in flight. `get_connection` is now
//! direct-connection mode only.
//!
//! This module gives the storage a scoped checkout: the guard
//! (standard, keep-alive, or adaptive) is owned for the whole operation and is
//! only released once the closure — including row iteration and any dependent
//! work — has returned. Direct-connection mode has no permit and joins the same
//! contract for uniformity.

use do_memory_core::{Error, Result};
use libsql::Connection;

use crate::pool::{AdaptivePooledConnection, PooledConnection};
use crate::prepared::ConnectionId;

#[cfg(feature = "keepalive-pool")]
use crate::pool::KeepAliveConnection;

use super::storage::TursoStorage;

/// A connection checked out under a pool guard that is held until it is dropped.
///
/// This is deliberately private to the crate: it is the one place that unifies
/// the three pool guard types behind the scoped contract. Callers must go
/// through [`TursoStorage::with_connection`] or
/// [`TursoStorage::with_connection_with_id`] so a raw `Connection` can never
/// escape without its permit.
pub(crate) enum ScopedCheckout {
    /// Checked out from the standard [`crate::pool::ConnectionPool`].
    Standard(PooledConnection),
    /// Checked out from the [`crate::pool::AdaptiveConnectionPool`].
    Adaptive(AdaptivePooledConnection),
    /// Checked out from the [`crate::pool::KeepAlivePool`].
    #[cfg(feature = "keepalive-pool")]
    KeepAlive(KeepAliveConnection),
    /// Direct connection (no pool configured); there is no permit to retain.
    Direct(Connection),
}

impl ScopedCheckout {
    /// Borrow the underlying connection for the duration of the checkout.
    fn connection(&self) -> Result<&Connection> {
        match self {
            Self::Standard(guard) => guard
                .connection()
                .ok_or_else(|| Error::Storage("Pooled connection was already taken".to_string())),
            Self::Adaptive(guard) => guard.connection().ok_or_else(|| {
                Error::Storage("Adaptive pooled connection was already taken".to_string())
            }),
            #[cfg(feature = "keepalive-pool")]
            Self::KeepAlive(guard) => guard.connection(),
            Self::Direct(connection) => Ok(connection),
        }
    }
}

/// Releases a connection's prepared-statement cache entry when dropped.
///
/// [`TursoStorage::with_connection_with_id`] must clear the cache of its
/// checkout after the operation returns, on *every* exit path: an early `?` from
/// `connection()` or a panic inside the closure would otherwise leave the
/// statements of a finished checkout cached. The guard also runs before the pool
/// permit is released, preserving the documented ordering.
struct PreparedCacheGuard<'a> {
    storage: &'a TursoStorage,
    conn_id: ConnectionId,
}

impl Drop for PreparedCacheGuard<'_> {
    fn drop(&mut self) {
        self.storage.clear_prepared_cache(self.conn_id);
    }
}

impl TursoStorage {
    /// Check out a connection from the configured pool, or create a direct one.
    ///
    /// Pool priority matches the historical `get_connection` ordering: adaptive,
    /// then keep-alive, then the standard pool, then a fresh direct connection.
    async fn checkout(&self) -> Result<ScopedCheckout> {
        if let Some(adaptive_pool) = &self.adaptive_pool {
            return Ok(ScopedCheckout::Adaptive(adaptive_pool.get().await?));
        }

        #[cfg(feature = "keepalive-pool")]
        {
            if let Some(keepalive_pool) = &self.keepalive_pool {
                return Ok(ScopedCheckout::KeepAlive(keepalive_pool.get().await?));
            }
        }

        if let Some(pool) = &self.pool {
            return Ok(ScopedCheckout::Standard(pool.get().await?));
        }

        let connection = self
            .db
            .connect()
            .map_err(|e| Error::Storage(format!("Failed to get connection: {}", e)))?;
        Ok(ScopedCheckout::Direct(connection))
    }

    /// Run an operation against a pooled connection, holding its permit for the
    /// entire closure.
    ///
    /// The pool guard (and therefore the semaphore permit, the active-connection
    /// accounting, and any cleanup callback) is released only after `f` — queries,
    /// row iteration, and dependent work included — has returned. A pool
    /// saturated by one slow operation therefore rejects or waits for the next
    /// checkout instead of handing out a permit it no longer tracks.
    ///
    /// In direct-connection mode a fresh connection is created for the closure;
    /// there is no permit to retain.
    ///
    /// # Errors
    ///
    /// Propagates the closure's error, or a pool checkout error (for example a
    /// timeout when the pool is saturated).
    pub async fn with_connection<F, T>(&self, f: F) -> Result<T>
    where
        F: AsyncFnOnce(&Connection) -> Result<T>,
    {
        let checkout = self.checkout().await?;
        let result = f(checkout.connection()?).await;
        // Release the permit / active-count / cleanup callback only now that the
        // operation (and any row iteration it performed) has finished.
        drop(checkout);
        result
    }

    /// Run an operation against a pooled connection with its prepared-statement
    /// cache id, holding the permit for the entire closure.
    ///
    /// Like [`TursoStorage::with_connection`], but the closure also receives the
    /// [`ConnectionId`] to use with
    /// [`TursoStorage::prepare_cached`](super::TursoStorage::prepare_cached). The
    /// connection's cached statements are cleared only after the closure returns.
    ///
    /// # Errors
    ///
    /// Propagates the closure's error, or a pool checkout error.
    pub async fn with_connection_with_id<F, T>(&self, f: F) -> Result<T>
    where
        F: AsyncFnOnce(&Connection, ConnectionId) -> Result<T>,
    {
        let checkout = self.checkout().await?;
        let conn_id = self.prepared_cache.get_connection_id();
        // A guard, not a trailing call: the cache is cleared on the normal
        // return, on an early `?`, and during unwinding — always before the pool
        // permit is released, because the guard is declared after `checkout`.
        let cache_guard = PreparedCacheGuard {
            storage: self,
            conn_id,
        };
        let result = f(checkout.connection()?, conn_id).await;
        drop(cache_guard);
        drop(checkout);
        result
    }
}
