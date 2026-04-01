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

#[cfg(feature = "compression")]
use super::super::CompressionStatistics;

#[cfg(feature = "adaptive-ttl")]
use super::super::cache::AdaptiveTTLCache;

#[cfg(feature = "adaptive-ttl")]
use do_memory_core::Episode;

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
    /// Compression statistics tracking (when compression feature is enabled)
    #[cfg(feature = "compression")]
    pub(crate) compression_stats: Arc<std::sync::Mutex<CompressionStatistics>>,
    /// Adaptive TTL cache for episode query results (when adaptive-ttl feature is enabled)
    #[cfg(feature = "adaptive-ttl")]
    pub(crate) episode_cache: Option<AdaptiveTTLCache<String, Episode>>,
}
