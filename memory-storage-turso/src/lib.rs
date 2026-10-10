#![deny(unsafe_code)]
#![cfg_attr(
    any(test, feature = "compression"),
    expect(clippy::expect_used, reason = "expect in compression code and tests")
)]
// Intentional suppressions for memory-storage-turso
#![expect(clippy::unwrap_used, reason = "Intentional .unwrap() on mutex locks")]
#![expect(dead_code, reason = "Public API methods not used internally")]
// Additional suppressions for complex code patterns
#![expect(
    clippy::excessive_nesting,
    reason = "Complex control flow in cache logic"
)]
#![expect(unused_mut, reason = "Variables used conditionally")]
// Cast-related: necessary for SQL storage metrics and statistics
#![expect(
    clippy::cast_precision_loss,
    reason = "precision loss accepted in metric math"
)]
#![expect(
    clippy::cast_possible_truncation,
    reason = "integer narrowing bounded by checks"
)]
#![expect(
    clippy::cast_sign_loss,
    reason = "non-negative values cast to unsigned"
)]
#![expect(
    clippy::cast_possible_wrap,
    reason = "range validated before wrapping cast"
)]
#![expect(
    clippy::cast_lossless,
    reason = "explicit widening casts document intent"
)]
// Documentation/pedantic: would require extensive rework
#![expect(
    clippy::cognitive_complexity,
    reason = "long-standing complex functions"
)]
#![expect(
    clippy::missing_errors_doc,
    reason = "error variants documented separately"
)]
#![expect(clippy::doc_markdown, reason = "identifier backticks noisy in prose")]
#![expect(
    clippy::must_use_candidate,
    reason = "not every public value is must_use"
)]
#![expect(
    clippy::return_self_not_must_use,
    reason = "constructors not marked must_use"
)]
#![expect(clippy::map_unwrap_or, reason = "explicit pattern clearer than map_or")]
#![expect(
    clippy::redundant_closure_for_method_calls,
    reason = "explicit closures aid readability"
)]
#![expect(clippy::match_same_arms, reason = "explicit arms aid maintenance")]
#![expect(clippy::uninlined_format_args, reason = "format args kept explicit")]
// Format args: inlining not required for error message clarity
#![expect(
    clippy::needless_pass_by_value,
    reason = "owned params kept for ergonomics"
)]
#![expect(clippy::unused_self, reason = "method kept for API symmetry")]
#![expect(clippy::unused_async, reason = "async kept for API symmetry")]
#![cfg_attr(
    test,
    expect(
        clippy::unreadable_literal,
        reason = "literals kept verbatim for clarity"
    )
)]
#![expect(
    clippy::struct_excessive_bools,
    reason = "config struct expressed as flags"
)]
#![cfg_attr(
    test,
    expect(clippy::panic, reason = "panic used for invariant violations")
)]
#![expect(
    clippy::items_after_statements,
    reason = "helpers declared next to use"
)]
#![cfg_attr(
    not(test),
    expect(clippy::wildcard_imports, reason = "glob imports used for preludes")
)]
#![expect(
    clippy::used_underscore_binding,
    reason = "underscore-prefixed binding is read"
)]
#![expect(
    clippy::used_underscore_items,
    reason = "underscore-prefixed item is used"
)]
#![expect(
    clippy::format_push_string,
    reason = "push_str formatting kept explicit"
)]
#![expect(
    clippy::missing_fields_in_debug,
    reason = "Debug shows selected fields"
)]
#![expect(
    clippy::needless_raw_string_hashes,
    reason = "raw-string hashes kept for escaping"
)]
#![cfg_attr(
    test,
    expect(clippy::default_trait_access, reason = "explicit Default::default")
)]
#![expect(
    clippy::missing_panics_doc,
    reason = "panic paths documented separately"
)]
#![expect(tail_expr_drop_order, reason = "drop-order change tracked separately")]
#![expect(clippy::unnecessary_wraps, reason = "Result kept for API consistency")]
#![expect(
    clippy::unchecked_time_subtraction,
    reason = "time subtraction guarded by callers"
)]
#![expect(
    clippy::semicolon_if_nothing_returned,
    reason = "semicolon kept for consistency"
)]
#![expect(missing_docs, reason = "public docs still incomplete")]
#![expect(unknown_lints, reason = "unknown on older toolchains")]
#![expect(clippy::unknown_lints, reason = "unknown on older toolchains")]

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
//! use do_memory_storage_turso::TursoStorage;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let storage = TursoStorage::new("libsql://localhost:8080", "token").await?;
//! storage.initialize_schema().await?;
//! # Ok(())
//! # }
//! ```

use do_memory_core::{Error, Result};

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
pub use lib_impls::TursoStorage;
pub use lib_impls::{StorageMode, TursoConfig};

/// Extension trait for `SelfLearningMemory` to provide convenience constructors
/// using the Turso storage backend.
#[async_trait::async_trait]
pub trait SelfLearningMemoryExt {
    /// Create a memory system with local SQLite storage.
    async fn with_local_storage(
        path: impl AsRef<std::path::Path> + Send,
    ) -> Result<do_memory_core::memory::SelfLearningMemory>;
    /// Create a memory system with in-memory SQLite storage.
    async fn with_in_memory_storage() -> Result<do_memory_core::memory::SelfLearningMemory>;
}

#[async_trait::async_trait]
impl SelfLearningMemoryExt for do_memory_core::memory::SelfLearningMemory {
    async fn with_local_storage(
        path: impl AsRef<std::path::Path> + Send,
    ) -> Result<do_memory_core::memory::SelfLearningMemory> {
        let storage = TursoStorage::new_local(path).await?;
        storage.initialize_schema().await?;

        let cache = std::sync::Arc::new(
            do_memory_storage_redb::RedbStorage::new(std::path::Path::new(":memory:"))
                .await
                .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?,
        );
        Ok(do_memory_core::memory::SelfLearningMemory::with_storage(
            do_memory_core::MemoryConfig::default(),
            std::sync::Arc::new(storage),
            cache,
        ))
    }

    async fn with_in_memory_storage() -> Result<do_memory_core::memory::SelfLearningMemory> {
        let storage = TursoStorage::new_in_memory().await?;
        storage.initialize_schema().await?;

        let cache = std::sync::Arc::new(
            do_memory_storage_redb::RedbStorage::new(std::path::Path::new(":memory:"))
                .await
                .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?,
        );
        Ok(do_memory_core::memory::SelfLearningMemory::with_storage(
            do_memory_core::MemoryConfig::default(),
            std::sync::Arc::new(storage),
            cache,
        ))
    }
}

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
pub use pool::{
    ConnectionPool, PoolConfig, PoolMetrics, PoolMetricsSnapshot, PoolStatistics, PooledConnection,
};
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

#[cfg(test)]
mod ext_tests {
    use super::*;
    use do_memory_core::memory::SelfLearningMemory;

    #[tokio::test]
    async fn test_with_in_memory_storage() {
        let memory = SelfLearningMemory::with_in_memory_storage().await.unwrap();
        let (primary, cache) = memory.storage_backends();
        assert!(primary.is_some());
        assert!(cache.is_some());
    }

    #[tokio::test]
    #[ignore = "flaky in full suite: libsql concurrent connection race (ADR-027); passes in isolation"]
    async fn test_with_local_storage() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.db");
        let memory = SelfLearningMemory::with_local_storage(&path).await.unwrap();
        let (primary, cache) = memory.storage_backends();
        assert!(primary.is_some());
        assert!(cache.is_some());
    }
}
