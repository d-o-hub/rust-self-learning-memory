#![allow(clippy::expect_used)]

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

use libsql::{Builder, Connection, Database};
use memory_core::{Error, Result};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};

// Cache module for performance optimization
pub mod cache;
pub mod pool;
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

// Compression module for network bandwidth reduction (40% target)
#[cfg(feature = "compression")]
pub mod compression;

// Cache exports
pub use cache::{CacheConfig, CacheStats, CachedTursoStorage};
pub use pool::{
    AdaptiveConnectionPool, AdaptivePoolConfig, AdaptivePoolMetrics, AdaptivePooledConnection,
};
pub use pool::{ConnectionPool, PoolConfig, PoolStatistics, PooledConnection};
#[cfg(feature = "keepalive-pool")]
pub use pool::{KeepAliveConfig, KeepAlivePool, KeepAliveStatistics};
pub use prepared::{PreparedCacheConfig, PreparedCacheStats, PreparedStatementCache};
pub use resilient::ResilientStorage;
pub use storage::batch::BatchConfig;
pub use storage::capacity::CapacityStatistics;
pub use storage::episodes::EpisodeQuery;
pub use storage::patterns::{PatternMetadata, PatternQuery};
pub use trait_impls::StorageStatistics;

// Compression exports (when compression feature is enabled)
#[cfg(feature = "compression")]
pub use compression::{
    compress, compress_embedding, compress_json, decompress, decompress_embedding,
    CompressedPayload, CompressionAlgorithm, CompressionStatistics,
};

/// Turso storage backend for durable persistence
pub struct TursoStorage {
    db: Arc<Database>,
    pool: Option<Arc<ConnectionPool>>,
    #[cfg(feature = "keepalive-pool")]
    keepalive_pool: Option<Arc<KeepAlivePool>>,
    adaptive_pool: Option<Arc<AdaptiveConnectionPool>>,
    prepared_cache: Arc<PreparedStatementCache>,
    config: TursoConfig,
}

/// Configuration for Turso storage
#[derive(Debug, Clone)]
pub struct TursoConfig {
    /// Maximum retry attempts for failed operations
    pub max_retries: u32,
    /// Base delay for exponential backoff (milliseconds)
    pub retry_base_delay_ms: u64,
    /// Maximum delay for exponential backoff (milliseconds)
    pub retry_max_delay_ms: u64,
    /// Enable connection pooling
    pub enable_pooling: bool,
    /// Enable keep-alive connection pool (reduces connection overhead)
    #[cfg(feature = "keepalive-pool")]
    pub enable_keepalive: bool,
    /// Keep-alive interval (seconds)
    #[cfg(feature = "keepalive-pool")]
    pub keepalive_interval_secs: u64,
    /// Stale connection threshold (seconds)
    #[cfg(feature = "keepalive-pool")]
    pub stale_threshold_secs: u64,
    /// Compression threshold in bytes (default: 1024 = 1KB)
    /// Payloads smaller than this won't be compressed
    /// Only used when compression feature is enabled
    pub compression_threshold: usize,
    /// Enable compression for episodes (default: true)
    /// Only used when compression feature is enabled
    pub compress_episodes: bool,
    /// Enable compression for patterns (default: true)
    /// Only used when compression feature is enabled
    pub compress_patterns: bool,
    /// Enable compression for embeddings (default: true)
    /// Only used when compression feature is enabled
    pub compress_embeddings: bool,
    /// Cache configuration for performance optimization
    /// When None, caching is disabled (default: Some(CacheConfig::default()))
    pub cache_config: Option<CacheConfig>,
}

impl Default for TursoConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_base_delay_ms: 100,
            retry_max_delay_ms: 5000,
            enable_pooling: true,
            #[cfg(feature = "keepalive-pool")]
            enable_keepalive: true,
            #[cfg(feature = "keepalive-pool")]
            keepalive_interval_secs: 30,
            #[cfg(feature = "keepalive-pool")]
            stale_threshold_secs: 60,
            // Compression settings (always present, only used when compression feature is enabled)
            compression_threshold: 1024,
            compress_episodes: true,
            compress_patterns: true,
            compress_embeddings: true,
            // Cache configuration (enabled by default)
            cache_config: Some(CacheConfig::default()),
        }
    }
}

impl TursoStorage {
    /// Create a new Turso storage instance
    ///
    /// # Arguments
    ///
    /// * `url` - Database URL (only `libsql://`, `file:`, or `:memory:` protocols allowed)
    /// * `token` - Authentication token (required for `libsql://`, empty for local files)
    ///
    /// # Security
    ///
    /// This method enforces secure connections:
    /// - Remote connections must use `libsql://` protocol with a valid token
    /// - HTTP/HTTPS protocols are rejected to prevent insecure connections
    /// - Local `file:` and `:memory:` databases are allowed without tokens
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use memory_storage_turso::TursoStorage;
    /// # async fn example() -> anyhow::Result<()> {
    /// // Remote connection with authentication
    /// let storage = TursoStorage::new("libsql://localhost:8080", "my-token").await?;
    ///
    /// // Local file database
    /// let local = TursoStorage::new("file:local.db", "").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(url: &str, token: &str) -> Result<Self> {
        Self::with_config(url, token, TursoConfig::default()).await
    }

    /// Create a Turso storage instance from an existing Database
    ///
    /// This is useful for testing with local file-based databases.
    ///
    /// # Arguments
    ///
    /// * `db` - libSQL Database instance
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use memory_storage_turso::TursoStorage;
    /// # use libsql::Builder;
    /// # async fn example() -> anyhow::Result<()> {
    /// let db = Builder::new_local("test.db").build().await?;
    /// let storage = TursoStorage::from_database(db)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_database(db: libsql::Database) -> Result<Self> {
        Ok(Self {
            db: Arc::new(db),
            pool: None,
            #[cfg(feature = "keepalive-pool")]
            keepalive_pool: None,
            adaptive_pool: None,
            prepared_cache: Arc::new(PreparedStatementCache::with_config(PreparedCacheConfig::default())),
            config: TursoConfig::default(),
        })
    }

    /// Create a new Turso storage instance with custom configuration
    ///
    /// # Security
    ///
    /// This method enforces the following security requirements:
    /// - Only `libsql://`, `file:`, and `:memory:` protocols are allowed
    /// - Remote connections (libsql://) require a non-empty authentication token
    /// - Local file and memory databases do not require tokens
    ///
    /// These checks prevent accidental use of insecure protocols and ensure
    /// proper authentication for remote Turso databases.
    pub async fn with_config(url: &str, token: &str, config: TursoConfig) -> Result<Self> {
        info!("Connecting to Turso database at {}", url);

        // SECURITY: Enforce TLS for remote connections
        if !url.starts_with("libsql://")
            && !url.starts_with("file:")
            && !url.starts_with(":memory:")
        {
            return Err(Error::Security(format!(
                "Insecure database URL: {}. Only libsql://, file:, or :memory: protocols are allowed",
                url
            )));
        }

        // SECURITY: Validate token is provided for remote connections
        if url.starts_with("libsql://") && token.trim().is_empty() {
            return Err(Error::Security(
                "Authentication token required for remote Turso connections".to_string(),
            ));
        }

        let db = if url.starts_with("libsql://") {
            Builder::new_remote(url.to_string(), token.to_string())
                .build()
                .await
                .map_err(|e| Error::Storage(format!("Failed to connect to Turso: {}", e)))?
        } else {
            let path = if let Some(stripped) = url.strip_prefix("file:") {
                stripped
            } else {
                url
            };
            Builder::new_local(path)
                .build()
                .await
                .map_err(|e| Error::Storage(format!("Failed to connect to Turso: {}", e)))?
        };

        let db = Arc::new(db);

        // Create connection pool if enabled
        let pool = if config.enable_pooling {
            let pool_config = PoolConfig::default();
            let max_conn = pool_config.max_connections;
            let pool = ConnectionPool::new(Arc::clone(&db), pool_config).await?;
            info!("Connection pool enabled with {} max connections", max_conn);
            Some(Arc::new(pool))
        } else {
            info!("Connection pooling disabled");
            None
        };

        // Create keep-alive pool if enabled
        #[cfg(feature = "keepalive-pool")]
        let keepalive_pool = if config.enable_keepalive {
            if let Some(ref pool) = pool {
                let keepalive_config = KeepAliveConfig {
                    keep_alive_interval: Duration::from_secs(config.keepalive_interval_secs),
                    stale_threshold: Duration::from_secs(config.stale_threshold_secs),
                    enable_proactive_ping: true,
                    ping_timeout: Duration::from_secs(5),
                };
                let keepalive_pool =
                    KeepAlivePool::new(Arc::clone(pool), Some(keepalive_config)).await?;
                let keepalive_arc = Arc::new(keepalive_pool);
                keepalive_arc.start_background_task();
                info!(
                    "Keep-alive pool enabled (interval={}s, stale_threshold={}s)",
                    config.keepalive_interval_secs, config.stale_threshold_secs
                );
                Some(keepalive_arc)
            } else {
                warn!("Keep-alive requested but pooling disabled, skipping");
                None
            }
        } else {
            None
        };

        #[cfg(not(feature = "keepalive-pool"))]
        let _keepalive_pool: Option<()> = None;

        info!("Successfully connected to Turso database");

        // Create the base storage first
        let storage = Self {
            db,
            pool,
            #[cfg(feature = "keepalive-pool")]
            keepalive_pool,
            adaptive_pool: None,
            prepared_cache: Arc::new(PreparedStatementCache::with_config(PreparedCacheConfig::default())),
            config,
        };

        // Return the storage - caller can wrap with CachedTursoStorage if needed
        Ok(storage)
    }

    /// Create a new Turso storage instance with custom pool configuration
    ///
    /// # Arguments
    ///
    /// * `url` - Database URL
    /// * `token` - Authentication token
    /// * `config` - Turso configuration
    /// * `pool_config` - Connection pool configuration
    ///
    /// # Example
    ///
    /// ```no_run
    /// use memory_storage_turso::{TursoStorage, TursoConfig, PoolConfig};
    /// use std::time::Duration;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let config = TursoConfig::default();
    /// let pool_config = PoolConfig {
    ///     max_connections: 20,
    ///     connection_timeout: Duration::from_secs(10),
    ///     enable_health_check: true,
    ///     health_check_timeout: Duration::from_secs(2),
    /// };
    ///
    /// let storage = TursoStorage::new_with_pool_config(
    ///     "libsql://localhost:8080",
    ///     "token",
    ///     config,
    ///     pool_config
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new_with_pool_config(
        url: &str,
        token: &str,
        config: TursoConfig,
        pool_config: PoolConfig,
    ) -> Result<Self> {
        info!("Connecting to Turso database at {}", url);

        // SECURITY: Enforce TLS for remote connections
        if !url.starts_with("libsql://")
            && !url.starts_with("file:")
            && !url.starts_with(":memory:")
        {
            return Err(Error::Security(format!(
                "Insecure database URL: {}. Only libsql://, file:, or :memory: protocols are allowed",
                url
            )));
        }

        // SECURITY: Validate token is provided for remote connections
        if url.starts_with("libsql://") && token.trim().is_empty() {
            return Err(Error::Security(
                "Authentication token required for remote Turso connections".to_string(),
            ));
        }

        let db = if url.starts_with("libsql://") {
            Builder::new_remote(url.to_string(), token.to_string())
                .build()
                .await
                .map_err(|e| Error::Storage(format!("Failed to connect to Turso: {}", e)))?
        } else {
            let path = if let Some(stripped) = url.strip_prefix("file:") {
                stripped
            } else {
                url
            };
            Builder::new_local(path)
                .build()
                .await
                .map_err(|e| Error::Storage(format!("Failed to connect to Turso: {}", e)))?
        };

        let db = Arc::new(db);

        // Create connection pool
        let pool = ConnectionPool::new(Arc::clone(&db), pool_config.clone()).await?;
        info!(
            "Connection pool enabled with {} max connections",
            pool_config.max_connections
        );

        // Wrap pool in Arc for sharing
        let pool_arc = Arc::new(pool);

        // Create keep-alive pool if enabled
        #[cfg(feature = "keepalive-pool")]
        let keepalive_pool = if config.enable_keepalive {
            let keepalive_config = KeepAliveConfig {
                keep_alive_interval: Duration::from_secs(config.keepalive_interval_secs),
                stale_threshold: Duration::from_secs(config.stale_threshold_secs),
                enable_proactive_ping: true,
                ping_timeout: Duration::from_secs(5),
            };
            let keepalive_pool =
                KeepAlivePool::new(Arc::clone(&pool_arc), Some(keepalive_config)).await?;
            let keepalive_arc = Arc::new(keepalive_pool);
            keepalive_arc.start_background_task();
            info!(
                "Keep-alive pool enabled (interval={}s, stale_threshold={}s)",
                config.keepalive_interval_secs, config.stale_threshold_secs
            );
            Some(keepalive_arc)
        } else {
            None
        };

        #[cfg(not(feature = "keepalive-pool"))]
        let _keepalive_pool: Option<()> = None;

        info!("Successfully connected to Turso database");

        Ok(Self {
            db,
            pool: Some(pool_arc),
            #[cfg(feature = "keepalive-pool")]
            keepalive_pool,
            adaptive_pool: None,
            prepared_cache: Arc::new(PreparedStatementCache::with_config(PreparedCacheConfig::default())),
            config,
        })
    }

    /// Create a new Turso storage instance with keep-alive pool
    ///
    /// # Arguments
    ///
    /// * `url` - Database URL
    /// * `token` - Authentication token
    /// * `config` - Turso configuration
    /// * `pool_config` - Connection pool configuration
    /// * `keepalive_config` - Keep-alive pool configuration
    ///
    /// # Example
    ///
    /// ```no_run
    /// use memory_storage_turso::{TursoStorage, TursoConfig, PoolConfig, KeepAliveConfig};
    /// use std::time::Duration;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let config = TursoConfig::default();
    /// let pool_config = PoolConfig::default();
    /// let keepalive_config = KeepAliveConfig::default();
    ///
    /// let storage = TursoStorage::new_with_keepalive(
    ///     "libsql://localhost:8080",
    ///     "token",
    ///     config,
    ///     pool_config,
    ///     keepalive_config
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "keepalive-pool")]
    pub async fn new_with_keepalive(
        url: &str,
        token: &str,
        config: TursoConfig,
        pool_config: PoolConfig,
        keepalive_config: KeepAliveConfig,
    ) -> Result<Self> {
        info!("Connecting to Turso database at {}", url);

        // SECURITY: Enforce TLS for remote connections
        if !url.starts_with("libsql://")
            && !url.starts_with("file:")
            && !url.starts_with(":memory:")
        {
            return Err(Error::Security(format!(
                "Insecure database URL: {}. Only libsql://, file:, or :memory: protocols are allowed",
                url
            )));
        }

        // SECURITY: Validate token is provided for remote connections
        if url.starts_with("libsql://") && token.trim().is_empty() {
            return Err(Error::Security(
                "Authentication token required for remote Turso connections".to_string(),
            ));
        }

        let db = if url.starts_with("libsql://") {
            Builder::new_remote(url.to_string(), token.to_string())
                .build()
                .await
                .map_err(|e| Error::Storage(format!("Failed to connect to Turso: {}", e)))?
        } else {
            let path = if let Some(stripped) = url.strip_prefix("file:") {
                stripped
            } else {
                url
            };
            Builder::new_local(path)
                .build()
                .await
                .map_err(|e| Error::Storage(format!("Failed to connect to Turso: {}", e)))?
        };

        let db = Arc::new(db);

        // Create connection pool
        let pool = ConnectionPool::new(Arc::clone(&db), pool_config.clone()).await?;
        info!(
            "Connection pool enabled with {} max connections",
            pool_config.max_connections
        );

        // Create keep-alive pool
        let pool_arc = Arc::new(pool);
        let keepalive_pool =
            KeepAlivePool::with_config(Arc::clone(&pool_arc), keepalive_config).await?;
        let keepalive_arc = Arc::new(keepalive_pool);
        keepalive_arc.start_background_task();
        info!(
            "Keep-alive pool enabled (interval={:?}, stale_threshold={:?})",
            keepalive_arc.config().keep_alive_interval,
            keepalive_arc.config().stale_threshold
        );

        info!("Successfully connected to Turso database");

        Ok(Self {
            db,
            pool: Some(pool_arc),
            keepalive_pool: Some(keepalive_arc),
            adaptive_pool: None,
            prepared_cache: Arc::new(PreparedStatementCache::with_config(PreparedCacheConfig::default())),
            config,
        })
    }

    /// Create a new Turso storage instance with adaptive connection pool
    ///
    /// The adaptive pool automatically adjusts its size based on load:
    /// - Scales up when utilization exceeds threshold (default: 70%)
    /// - Scales down during low utilization (default: <30%)
    /// - Provides 20% better performance under variable load
    ///
    /// # Arguments
    ///
    /// * `url` - Database URL
    /// * `token` - Authentication token
    /// * `config` - Turso configuration
    /// * `adaptive_config` - Adaptive pool configuration
    ///
    /// # Example
    ///
    /// ```no_run
    /// use memory_storage_turso::{TursoStorage, TursoConfig, AdaptivePoolConfig};
    /// use std::time::Duration;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let config = TursoConfig::default();
    /// let adaptive_config = AdaptivePoolConfig {
    ///     min_connections: 5,
    ///     max_connections: 50,
    ///     scale_up_threshold: 0.7,
    ///     scale_down_threshold: 0.3,
    ///     scale_up_cooldown: Duration::from_secs(10),
    ///     scale_down_cooldown: Duration::from_secs(30),
    ///     scale_up_increment: 5,
    ///     scale_down_decrement: 5,
    ///     check_interval: Duration::from_secs(5),
    /// };
    ///
    /// let storage = TursoStorage::new_with_adaptive_pool(
    ///     "libsql://localhost:8080",
    ///     "token",
    ///     config,
    ///     adaptive_config
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new_with_adaptive_pool(
        url: &str,
        token: &str,
        config: TursoConfig,
        adaptive_config: AdaptivePoolConfig,
    ) -> Result<Self> {
        info!("Connecting to Turso database at {} with adaptive pool", url);

        // SECURITY: Enforce TLS for remote connections
        if !url.starts_with("libsql://")
            && !url.starts_with("file:")
            && !url.starts_with(":memory:")
        {
            return Err(Error::Security(format!(
                "Insecure database URL: {}. Only libsql://, file:, or :memory: protocols are allowed",
                url
            )));
        }

        // SECURITY: Validate token is provided for remote connections
        if url.starts_with("libsql://") && token.trim().is_empty() {
            return Err(Error::Security(
                "Authentication token required for remote Turso connections".to_string(),
            ));
        }

        let db = if url.starts_with("libsql://") {
            Builder::new_remote(url.to_string(), token.to_string())
                .build()
                .await
                .map_err(|e| Error::Storage(format!("Failed to connect to Turso: {}", e)))?
        } else {
            let path = if let Some(stripped) = url.strip_prefix("file:") {
                stripped
            } else {
                url
            };
            Builder::new_local(path)
                .build()
                .await
                .map_err(|e| Error::Storage(format!("Failed to connect to Turso: {}", e)))?
        };

        let db = Arc::new(db);

        // Create adaptive connection pool
        let adaptive_pool =
            AdaptiveConnectionPool::new(Arc::clone(&db), adaptive_config.clone()).await?;
        info!(
            "Adaptive connection pool enabled (min={}, max={}, threshold={})",
            adaptive_config.min_connections,
            adaptive_config.max_connections,
            adaptive_config.scale_up_threshold
        );

        info!("Successfully connected to Turso database");

        Ok(Self {
            db,
            pool: None,
            #[cfg(feature = "keepalive-pool")]
            keepalive_pool: None,
            adaptive_pool: Some(Arc::new(adaptive_pool)),
            prepared_cache: Arc::new(PreparedStatementCache::with_config(PreparedCacheConfig::default())),
            config,
        })
    }

    /// Get a database connection
    ///
    /// If connection pooling is enabled, this will use a pooled connection.
    /// If keep-alive pool is enabled, it will be used for reduced overhead.
    /// If adaptive pool is enabled, it will be used for variable load optimization.
    /// Otherwise, it creates a new connection each time.
    async fn get_connection(&self) -> Result<Connection> {
        // Check adaptive pool first (highest priority for variable load)
        if let Some(ref adaptive_pool) = self.adaptive_pool {
            let _adaptive_conn = adaptive_pool.get().await?;
            // For now, return a new connection since AdaptivePooledConnection
            // doesn't expose the underlying connection directly
            // This is a limitation - in a full implementation, we'd expose this
            return self
                .db
                .connect()
                .map_err(|e| Error::Storage(format!("Failed to get connection: {}", e)));
        }

        #[cfg(feature = "keepalive-pool")]
        {
            if let Some(ref keepalive_pool) = self.keepalive_pool {
                // Use keep-alive pool for reduced connection overhead
                let keepalive_conn = keepalive_pool.get().await?;
                return Ok(keepalive_conn.connection().clone());
            }
        }

        if let Some(ref pool) = self.pool {
            // Use connection pool
            let pooled_conn = pool.get().await?;
            Ok(pooled_conn.into_inner()?)
        } else {
            // Create direct connection (legacy mode)
            self.db
                .connect()
                .map_err(|e| Error::Storage(format!("Failed to get connection: {}", e)))
        }
    }

    /// Get pool statistics if pooling is enabled
    pub async fn pool_statistics(&self) -> Option<PoolStatistics> {
        if let Some(ref pool) = self.pool {
            Some(pool.statistics().await)
        } else {
            None
        }
    }

    /// Get pool utilization if pooling is enabled
    pub async fn pool_utilization(&self) -> Option<f32> {
        if let Some(ref pool) = self.pool {
            Some(pool.utilization().await)
        } else {
            self.adaptive_pool
                .as_ref()
                .map(|adaptive_pool| adaptive_pool.utilization() as f32)
        }
    }

    /// Get adaptive pool metrics if enabled
    pub fn adaptive_pool_metrics(&self) -> Option<AdaptivePoolMetrics> {
        self.adaptive_pool.as_ref().map(|pool| pool.metrics())
    }

    /// Get current adaptive pool size
    pub fn adaptive_pool_size(&self) -> Option<(u32, u32)> {
        self.adaptive_pool
            .as_ref()
            .map(|pool| (pool.active_connections(), pool.max_connections()))
    }

    /// Manually trigger adaptive pool scaling check
    pub async fn check_adaptive_pool_scale(&self) {
        if let Some(ref adaptive_pool) = self.adaptive_pool {
            adaptive_pool.check_and_scale().await;
        }
    }

    /// Get keep-alive pool statistics if enabled
    #[cfg(feature = "keepalive-pool")]
    pub fn keepalive_statistics(&self) -> Option<KeepAliveStatistics> {
        self.keepalive_pool.as_ref().map(|pool| pool.statistics())
    }

    /// Get keep-alive configuration if enabled
    #[cfg(feature = "keepalive-pool")]
    pub fn keepalive_config(&self) -> Option<&KeepAliveConfig> {
        self.keepalive_pool.as_ref().map(|pool| pool.config())
    }

    /// Execute PRAGMA statements for database configuration
    ///
    /// PRAGMA statements may return rows, so we need to consume them before continuing.
    async fn execute_pragmas(&self, conn: &Connection) -> Result<()> {
        // Enable WAL mode for better concurrent access
        // WAL mode allows concurrent reads while writing, reducing "database locked" errors
        if let Ok(mut rows) = conn.query("PRAGMA journal_mode=WAL", ()).await {
            // Consume all rows to avoid "Execute returned rows" error
            while rows.next().await.is_ok_and(|r| r.is_some()) {
                // Consume each row
            }
        }

        // Increase busy timeout to allow more time for lock acquisition
        if let Ok(mut rows) = conn.query("PRAGMA busy_timeout=30000", ()).await {
            while rows.next().await.is_ok_and(|r| r.is_some()) {
                // Consume each row
            }
        }

        Ok(())
    }

    /// Execute a SQL statement with retry logic
    async fn execute_with_retry(&self, conn: &Connection, sql: &str) -> Result<()> {
        let mut attempts = 0;
        let mut delay = Duration::from_millis(self.config.retry_base_delay_ms);

        loop {
            match conn.execute(sql, ()).await {
                Ok(_) => {
                    if attempts > 0 {
                        debug!("SQL succeeded after {} retries", attempts);
                    }
                    return Ok(());
                }
                Err(e) => {
                    attempts += 1;
                    if attempts >= self.config.max_retries {
                        error!("SQL failed after {} attempts: {}", attempts, e);
                        return Err(Error::Storage(format!(
                            "SQL execution failed after {} retries: {}",
                            attempts, e
                        )));
                    }

                    warn!("SQL attempt {} failed: {}, retrying...", attempts, e);
                    tokio::time::sleep(delay).await;

                    // Exponential backoff
                    delay = std::cmp::min(
                        delay * 2,
                        Duration::from_millis(self.config.retry_max_delay_ms),
                    );
                }
            }
        }
    }

    /// Health check - verify database connectivity
    pub async fn health_check(&self) -> Result<bool> {
        let conn = self.get_connection().await?;
        match conn.query("SELECT 1", ()).await {
            Ok(_) => Ok(true),
            Err(e) => {
                error!("Health check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Wrap this storage with a cache layer using default cache configuration
    ///
    /// This provides transparent caching for episodes, patterns, and heuristics
    /// with adaptive TTL based on access patterns.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use memory_storage_turso::{TursoStorage, CacheConfig};
    /// # async fn example() -> anyhow::Result<()> {
    /// let storage = TursoStorage::new("file:test.db", "").await?;
    /// let cached = storage.with_cache_default();
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_cache_default(self) -> CachedTursoStorage {
        self.with_cache(CacheConfig::default())
    }

    /// Wrap this storage with a cache layer using custom cache configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Cache configuration to use
    ///
    /// # Returns
    ///
    /// A new `CachedTursoStorage` wrapping this storage
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use memory_storage_turso::{TursoStorage, CacheConfig};
    /// # use std::time::Duration;
    /// # async fn example() -> anyhow::Result<()> {
    /// let storage = TursoStorage::new("file:test.db", "").await?;
    /// let config = CacheConfig {
    ///     max_episodes: 1000,
    ///     episode_ttl: Duration::from_secs(3600),
    ///     ..Default::default()
    /// };
    /// let cached = storage.with_cache(config);
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_cache(self, cache_config: CacheConfig) -> CachedTursoStorage {
        CachedTursoStorage::new(self, cache_config)
    }

    /// Get the cache configuration if set
    pub fn cache_config(&self) -> Option<&CacheConfig> {
        self.config.cache_config.as_ref()
    }

    /// Get prepared statement cache statistics
    pub fn prepared_cache_stats(&self) -> PreparedCacheStats {
        self.prepared_cache.stats()
    }

    /// Get a reference to the prepared statement cache
    pub fn prepared_cache(&self) -> &PreparedStatementCache {
        &self.prepared_cache
    }

    /// Get database statistics
    pub async fn get_statistics(&self) -> Result<StorageStatistics> {
        let conn = self.get_connection().await?;

        let episode_count = self.get_count(&conn, "episodes").await?;
        let pattern_count = self.get_count(&conn, "patterns").await?;
        let heuristic_count = self.get_count(&conn, "heuristics").await?;

        Ok(StorageStatistics {
            episode_count,
            pattern_count,
            heuristic_count,
        })
    }

    /// Get count of records in a table
    async fn get_count(&self, conn: &Connection, table: &str) -> Result<usize> {
        let sql = format!("SELECT COUNT(*) as count FROM {}", table);
        let mut rows = conn
            .query(&sql, ())
            .await
            .map_err(|e| Error::Storage(format!("Failed to count {}: {}", table, e)))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch count for {}: {}", table, e)))?
        {
            let count: i64 = row
                .get(0)
                .map_err(|e| Error::Storage(format!("Failed to parse count: {}", e)))?;
            Ok(count as usize)
        } else {
            Ok(0)
        }
    }
}
