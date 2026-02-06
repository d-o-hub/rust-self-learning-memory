//! Turso Storage Struct Definition
//!
//! This module contains the TursoStorage struct definition.

use libsql::Database;
use std::sync::Arc;

use super::super::pool::CachingPool;
use super::super::pool::ConnectionPool;
use super::super::{AdaptiveConnectionPool, PreparedStatementCache, TursoConfig};

#[cfg(feature = "keepalive-pool")]
use super::super::pool::KeepAlivePool;

/// Turso storage backend for durable persistence
pub struct TursoStorage {
    pub(crate) db: Arc<Database>,
    pub(crate) pool: Option<Arc<ConnectionPool>>,
    #[cfg(feature = "keepalive-pool")]
    pub(crate) keepalive_pool: Option<Arc<KeepAlivePool>>,
    pub(crate) adaptive_pool: Option<Arc<AdaptiveConnectionPool>>,
    pub(crate) caching_pool: Option<Arc<CachingPool>>,
    pub(crate) prepared_cache: Arc<PreparedStatementCache>,
    pub(crate) config: TursoConfig,
}
