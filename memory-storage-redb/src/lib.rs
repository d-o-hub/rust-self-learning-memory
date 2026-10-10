#![deny(unsafe_code)]
// Clippy suppressions for memory-storage-redb
// Cast-related: necessary for persistence statistics and metrics
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
#![cfg_attr(
    test,
    expect(
        clippy::cast_lossless,
        reason = "explicit widening casts document intent"
    )
)]
// Documentation: would require extensive rework
#![expect(
    clippy::missing_errors_doc,
    reason = "error variants documented separately"
)]
#![expect(missing_docs, reason = "public docs still incomplete")]
// Pedantic lints: provide minimal value
#![expect(
    clippy::must_use_candidate,
    reason = "not every public value is must_use"
)]
#![expect(clippy::map_unwrap_or, reason = "explicit pattern clearer than map_or")]
#![expect(
    clippy::redundant_closure_for_method_calls,
    reason = "explicit closures aid readability"
)]
#![expect(clippy::unused_self, reason = "method kept for API symmetry")]
#![cfg_attr(
    test,
    expect(clippy::unused_async, reason = "async kept for API symmetry")
)]
#![expect(
    clippy::semicolon_if_nothing_returned,
    reason = "semicolon kept for consistency"
)]
// Format args: inlining not required for error message clarity
#![expect(clippy::uninlined_format_args, reason = "format args kept explicit")]
#![expect(clippy::unnecessary_wraps, reason = "Result kept for API consistency")]
#![expect(
    clippy::return_self_not_must_use,
    reason = "constructors not marked must_use"
)]
#![expect(clippy::excessive_nesting, reason = "deeply nested control flow")]
#![expect(
    clippy::cognitive_complexity,
    reason = "long-standing complex functions"
)]
#![cfg_attr(
    test,
    expect(
        clippy::unreadable_literal,
        reason = "literals kept verbatim for clarity"
    )
)]
#![expect(clippy::doc_markdown, reason = "identifier backticks noisy in prose")]
#![expect(
    clippy::used_underscore_binding,
    reason = "underscore-prefixed binding is read"
)]

//! # Memory Storage - redb
//!
//! redb embedded database for fast cache layer.
//!
//! This crate provides:
//! - High-performance key-value storage using redb
//! - Zero-copy reads for fast retrieval
//! - Async wrappers for synchronous redb operations
//! - Episode and pattern caching
//! - Postcard serialization for efficient storage
//!
//! ## Example
//!
//! ```no_run
//! use do_memory_storage_redb::RedbStorage;
//! use std::path::Path;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let storage = RedbStorage::new(Path::new("./memory.redb")).await?;
//! # Ok(())
//! # }
//! ```

use do_memory_core::{Error, Result};
use redb::{Database, TableDefinition};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

use crate::cache::Cache as CacheTrait;

mod backend_impl;
mod cache;
mod embeddings;
mod embeddings_backend;
mod embeddings_impl;
mod episodes;
mod episodes_queries;
mod episodes_summaries;
mod heuristics;
mod patterns;
mod persistence;
mod procedural;
mod recommendation_index;
mod recommendations;
mod relationships;
mod statistics;
mod storage;
mod storage_ops;
mod tables;

// Re-export cache types for external use
pub use crate::cache::{
    AdaptiveCache, AdaptiveCacheAdapter, AdaptiveCacheConfig, AdaptiveCacheMetrics, Cache,
    CacheConfig, CacheMetrics, LRUCache,
};

pub use crate::statistics::StorageStatistics;
pub use persistence::{
    CachePersistence, CacheSnapshot, IncrementalUpdate, PersistedCacheEntry, PersistenceConfig,
    PersistenceManager, PersistenceMode, PersistenceStats, PersistenceStrategy,
};
pub use storage::RedbQuery;

// ============================================================================
// Deserialization Limits (Security)
// ============================================================================

/// Maximum size for episode deserialization (10MB).
///
/// Prevents OOM attacks from maliciously large serialized payloads.
pub const MAX_EPISODE_SIZE: u64 = 10_000_000;

/// Maximum size for pattern deserialization (1MB).
///
/// Limits pattern data size to prevent resource exhaustion.
pub const MAX_PATTERN_SIZE: u64 = 1_000_000;

/// Maximum size for heuristic deserialization (100KB).
///
/// Restricts heuristic data size for security.
pub const MAX_HEURISTIC_SIZE: u64 = 100_000;

/// Maximum size for embedding deserialization (1MB).
///
/// Limits embedding vector size to prevent resource exhaustion.
/// Typical embedding dimensions (384-1536) * 4 bytes/f32 = ~1.5KB-6KB.
pub const MAX_EMBEDDING_SIZE: u64 = 1_000_000;

// Table definitions
pub(crate) const EPISODES_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("episodes");
pub(crate) const PATTERNS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("patterns");
pub(crate) const HEURISTICS_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("heuristics");
pub(crate) const EMBEDDINGS_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("embeddings");
pub(crate) const METADATA_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("metadata");
pub(crate) const SUMMARIES_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("summaries");
pub(crate) const RELATIONSHIPS_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("relationships");
pub(crate) const EPISODE_PATTERN_RELATIONSHIPS_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("episode_pattern_relationships");
pub(crate) const RECOMMENDATION_SESSIONS_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("recommendation_sessions");
pub(crate) const RECOMMENDATION_FEEDBACK_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("recommendation_feedback");
pub(crate) const RECOMMENDATION_EPISODE_INDEX_TABLE: TableDefinition<&str, &str> =
    TableDefinition::new("recommendation_episode_index");
pub(crate) const PROCEDURAL_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("procedural");

/// Names of every user-data table with `(&str, &[u8])` rows.
///
/// The `recommendation_episode_index` table is typed `(&str, &str)` and is
/// therefore checked separately by the schema inspector.
pub(crate) const DATA_TABLE_NAMES: [&str; 11] = [
    "episodes",
    "patterns",
    "heuristics",
    "embeddings",
    "metadata",
    "summaries",
    "relationships",
    "episode_pattern_relationships",
    "recommendation_sessions",
    "recommendation_feedback",
    "procedural",
];

/// Every table this crate owns, including the schema-version metadata table.
pub(crate) const KNOWN_TABLE_NAMES: [&str; 13] = [
    "episodes",
    "patterns",
    "heuristics",
    "embeddings",
    "metadata",
    "summaries",
    "relationships",
    "episode_pattern_relationships",
    "recommendation_sessions",
    "recommendation_feedback",
    "recommendation_episode_index",
    "procedural",
    "schema_version",
];

// ============================================================================
// Schema Versioning (Automatic Cache Invalidation)
// ============================================================================

/// Schema version for the redb cache.
///
/// This version is stored in the database and checked on startup.
/// When the schema changes (e.g., Episode struct modified), increment this version
/// to automatically invalidate stale cached data.
///
/// ## When to increment:
/// - Adding/removing fields from Episode, Pattern, Heuristic, or other cached types
/// - Changing the serialization format (postcard schema)
/// - Any backward-incompatible change to cached data structures
///
/// ## Version history:
/// - v1: Initial version (pre-versioning)
/// - v2: Added checkpoints field to Episode (ADR-044 Feature 3)
/// - v3: Added weight to relationships, added EpisodePatternRelationship (WG-123)
/// - v4: Pattern externally-tagged for postcard (#831; was internally tagged)
pub(crate) const SCHEMA_VERSION: u64 = 4;

pub(crate) const SCHEMA_VERSION_TABLE: TableDefinition<&str, u64> =
    TableDefinition::new("schema_version");

// ============================================================================
// Timeout Helper Functions
// ============================================================================

/// Timeout duration for database operations (10 seconds)
const DB_OPERATION_TIMEOUT: Duration = Duration::from_secs(10);

/// Execute a spawn_blocking operation with timeout
pub(crate) async fn with_db_timeout<T, F>(operation: F) -> crate::Result<T>
where
    F: FnOnce() -> crate::Result<T> + Send + 'static,
    T: Send + 'static,
{
    // spawn_blocking returns Result<T, JoinError>
    // timeout wraps that, so we get Result<Result<T, JoinError>, Elapsed>
    match tokio::time::timeout(DB_OPERATION_TIMEOUT, tokio::task::spawn_blocking(operation)).await {
        Ok(Ok(result)) => result, // Inner Ok is JoinError, outer Ok is timeout success
        Ok(Err(join_err)) => Err(Error::Storage(format!("Task join error: {}", join_err))),
        Err(_) => Err(Error::Storage(format!(
            "Database operation timed out after {:?}",
            DB_OPERATION_TIMEOUT
        ))),
    }
}

/// redb storage backend for fast caching
pub struct RedbStorage {
    pub(crate) db: Arc<Database>,
    pub(crate) cache: Box<dyn CacheTrait>,
    /// Filesystem path of the underlying database file.
    ///
    /// Retained so schema/migration errors can report exactly which file an
    /// operator must inspect or back up (issue #1069).
    pub(crate) path: PathBuf,
}

impl RedbStorage {
    /// Create a new redb storage instance with default adaptive cache
    ///
    /// Uses `AdaptiveCacheAdapter` by default for intelligent TTL adjustment.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the redb database file
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use do_memory_storage_redb::RedbStorage;
    /// # use std::path::Path;
    /// # async fn example() -> anyhow::Result<()> {
    /// let storage = RedbStorage::new(Path::new("./memory.redb")).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(path: &Path) -> Result<Self> {
        Self::new_with_adaptive_config(path, AdaptiveCacheConfig::default()).await
    }

    /// Create a new redb storage instance with custom cache configuration
    ///
    /// Uses the legacy `LRUCache` implementation. For adaptive TTL features,
    /// use `new_with_adaptive_config` instead.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the redb database file
    /// * `cache_config` - Cache configuration settings
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use do_memory_storage_redb::{RedbStorage, CacheConfig};
    /// # use std::path::Path;
    /// # async fn example() -> anyhow::Result<()> {
    /// let config = CacheConfig {
    ///     max_size: 500,
    ///     default_ttl_secs: 1800,
    ///     cleanup_interval_secs: 600,
    ///     enable_background_cleanup: true,
    /// };
    /// let storage = RedbStorage::new_with_cache_config(Path::new("./memory.redb"), config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new_with_cache_config(path: &Path, cache_config: CacheConfig) -> Result<Self> {
        let storage = Self::open_with_cache(path, Box::new(LRUCache::new(cache_config))).await?;

        // Initialize tables
        storage.initialize_tables().await?;

        info!("Successfully opened redb database with LRU cache");
        Ok(storage)
    }

    /// Create a new redb storage instance with adaptive cache configuration
    ///
    /// Uses `AdaptiveCacheAdapter` for intelligent TTL adjustment based on
    /// access patterns. Frequently accessed items get longer TTL, rarely
    /// accessed items get shorter TTL.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the redb database file
    /// * `config` - Adaptive cache configuration settings
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use do_memory_storage_redb::{RedbStorage, AdaptiveCacheConfig};
    /// # use std::path::Path;
    /// # use std::time::Duration;
    /// # async fn example() -> anyhow::Result<()> {
    /// let config = AdaptiveCacheConfig {
    ///     max_size: 1000,
    ///     default_ttl: Duration::from_secs(1800),
    ///     min_ttl: Duration::from_secs(300),
    ///     max_ttl: Duration::from_secs(7200),
    ///     ..Default::default()
    /// };
    /// let storage = RedbStorage::new_with_adaptive_config(Path::new("./memory.redb"), config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new_with_adaptive_config(
        path: &Path,
        config: AdaptiveCacheConfig,
    ) -> Result<Self> {
        let storage =
            Self::open_with_cache(path, Box::new(AdaptiveCacheAdapter::new(config))).await?;

        // Initialize tables
        storage.initialize_tables().await?;

        info!("Successfully opened redb database with adaptive cache");
        Ok(storage)
    }

    /// Open a redb database **without** running the schema check and without
    /// creating any table.
    ///
    /// Escape hatch for operator tooling that must inspect, migrate or
    /// explicitly reset a database whose stored schema version does not match
    /// the current binary (issue #1069). Normal application code must use
    /// [`RedbStorage::new`], which fails closed instead of touching mismatched
    /// data. The returned handle has *not* validated the schema; call
    /// [`RedbStorage::migrate_schema`] or [`RedbStorage::reset_all_tables`]
    /// deliberately.
    pub async fn open_unchecked(path: &Path) -> Result<Self> {
        Self::open_with_cache(
            path,
            Box::new(AdaptiveCacheAdapter::new(AdaptiveCacheConfig::default())),
        )
        .await
    }

    /// Open the database file and build the storage handle, without touching
    /// any table or the schema version.
    async fn open_with_cache(path: &Path, cache: Box<dyn CacheTrait>) -> Result<Self> {
        info!("Opening redb database at {}", path.display());

        // Use spawn_blocking for synchronous redb initialization with timeout
        let db_path = path.to_path_buf();
        let path_buf = db_path.clone();
        let db = with_db_timeout(move || {
            Database::create(&path_buf)
                .map_err(|e| Error::Storage(format!("Failed to create redb database: {}", e)))
        })
        .await?;

        Ok(Self {
            db: Arc::new(db),
            cache,
            path: db_path,
        })
    }
}

#[cfg(test)]
mod tests;

impl RedbStorage {
    /// Initialize the database schema and tables.
    pub async fn initialize_schema(&self) -> Result<()> {
        self.initialize_tables().await
    }
}
