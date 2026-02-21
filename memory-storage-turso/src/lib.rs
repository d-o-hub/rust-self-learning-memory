#![allow(clippy::expect_used)]
// Intentional allows for memory-storage-turso
#![allow(unsafe_code)] // Intentional unsafe for performance in connection pooling
#![allow(clippy::unwrap_used)] // Intentional .unwrap() on mutex locks
#![allow(invalid_value)] // Intentional zero-initialization in connection pool
#![allow(dead_code)] // Public API methods not used internally
// Additional allows for complex code patterns
#![allow(clippy::excessive_nesting)] // Complex control flow in cache logic
#![allow(unused_mut)] // Variables used conditionally
#![allow(unused_assignments)] // Variables assigned in loops
#![allow(clippy::derivable_impls)] // Prefer explicit impls for clarity
#![allow(clippy::should_implement_trait)] // Custom default methods
#![allow(clippy::unnecessary_map_or)] // Explicit is better than implicit
#![allow(clippy::useless_asref)] // Clarity in type conversions

//! # Memory Storage - Turso
//!
//! Turso/libSQL storage backend for durable persistence of episodes and patterns.
//!
//! This crate provides:
//! - Connection management for Turso databases
//! - SQL schema creation and migration
//! - CRUD operations for episodes, patterns, and heuristics
//! - Query capabilities for analytical retrieval
//! - Retry logic and circuit breaker pattern for resilience
//!
//! ## Example
//!
//! ```no_run
//! use memory_storage_turso::TursoStorage;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let storage = TursoStorage::new("libsql://localhost:8080", "token").await?;
//! storage.initialize_schema().await?;
//! # Ok(())
//! # }
//! ```

use memory_core::{Error, Result};

// Cache module for performance optimization
pub mod cache;
pub mod pool;
mod relationships;
mod resilient;
mod schema;
#[cfg(test)]
mod tests;

#[cfg(feature = "hybrid_search")]
mod fts5_schema;

// Storage module - split into submodules for file size compliance
pub mod storage;

// Trait implementations - moved to separate module for file size compliance
pub mod trait_impls;

// Schema initialization - moved to separate module for file size compliance
pub mod turso_config;

// Prepared statement caching for query optimization
pub mod prepared;

// Performance metrics and export module
pub mod metrics;

// Compression module for network bandwidth reduction (40% target)
#[cfg(feature = "compression")]
pub mod compression;

// Transport layer with compression support
#[cfg(feature = "compression")]
pub mod transport;

// Lib implementation submodules - split for file size compliance
mod lib_impls;

// Re-export public types from lib_impls
pub use lib_impls::TursoConfig;
pub use lib_impls::TursoStorage;

// Cache exports
pub use cache::query_cache::{AdvancedCacheStats, AdvancedQueryCache, QueryKey};
pub use cache::{
    AdaptiveTTLCache, CacheConfig, CacheEntry, CacheStats, CacheStatsSnapshot, CachedTursoStorage,
    TTLConfig, TTLConfigError,
};

// Performance metrics exports
pub use pool::{
    AdaptiveConnectionPool, AdaptivePoolConfig, AdaptivePoolMetrics, AdaptivePooledConnection,
};
pub use pool::{ConnectionPool, PoolConfig, PoolStatistics, PooledConnection};
#[cfg(feature = "keepalive-pool")]
pub use pool::{KeepAliveConfig, KeepAlivePool, KeepAliveStatistics};
pub use prepared::{PreparedCacheConfig, PreparedCacheStats, PreparedStatementCache};
pub use resilient::ResilientStorage;

// Metrics export re-exports
pub use metrics::{
    ExportConfig, ExportFormat, ExportStats, ExportTarget, ExportedMetric, MetricType, MetricValue,
    MetricsCollector, MetricsHttpServer, PrometheusExporter, TursoMetrics,
};
pub use storage::batch::episode_batch::BatchConfig;
pub use storage::capacity::CapacityStatistics;
pub use storage::episodes::EpisodeQuery;
pub use storage::patterns::{PatternMetadata, PatternQuery};
pub use trait_impls::StorageStatistics;

// Compression exports (when compression feature is enabled)
#[cfg(feature = "compression")]
pub use compression::{
    CompressedPayload, CompressionAlgorithm, CompressionStatistics, compress, compress_embedding,
    compress_json, decompress, decompress_embedding,
};

// Transport exports (when compression feature is enabled)
#[cfg(feature = "compression")]
pub use transport::{
    CompressedTransport, Transport, TransportCompressionConfig, TransportCompressionError,
    TransportCompressionStats, TransportMetadata, TransportResponse,
};

// Include constructor implementations from lib_impls modules
// These are automatically included via `mod lib_impls` declaration
// The impl blocks are in:
// - lib_impls::constructors_basic (new, from_database, with_config)
// - lib_impls::constructors_pool (new_with_pool_config, new_with_keepalive)
// - lib_impls::constructors_adaptive (new_with_adaptive_pool)
//
// Helper methods are in:
// - lib_impls::helpers (get_connection, get_count, etc.)
