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

use async_trait::async_trait;
use libsql::{Builder, Connection, Database};
use memory_core::{Error, Result, StorageBackend};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};

pub mod pool;
mod resilient;
mod schema;
#[cfg(test)]
mod tests;

#[cfg(feature = "hybrid_search")]
mod fts5_schema;

// Storage module - split into submodules for file size compliance
pub mod storage;

pub use pool::{ConnectionPool, PoolConfig, PoolStatistics};
pub use resilient::ResilientStorage;
pub use storage::capacity::CapacityStatistics;
pub use storage::episodes::EpisodeQuery;
pub use storage::patterns::{PatternMetadata, PatternQuery};

/// Turso storage backend for durable persistence
pub struct TursoStorage {
    db: Arc<Database>,
    pool: Option<Arc<ConnectionPool>>,
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
}

impl Default for TursoConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_base_delay_ms: 100,
            retry_max_delay_ms: 5000,
            enable_pooling: true,
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

        info!("Successfully connected to Turso database");

        Ok(Self { db, pool, config })
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

        info!("Successfully connected to Turso database");

        Ok(Self {
            db,
            pool: Some(Arc::new(pool)),
            config,
        })
    }

    /// Initialize the database schema
    ///
    /// Creates tables and indexes if they don't exist.
    /// Safe to call multiple times.
    pub async fn initialize_schema(&self) -> Result<()> {
        info!("Initializing Turso database schema");
        let conn = self.get_connection().await?;

        // Enable WAL mode for better concurrent access (especially for file-based SQLite)
        // WAL mode allows concurrent reads while writing, reducing "database locked" errors
        // Use execute_raw that can handle PRAGMA statements returning rows
        let _ = self.execute_pragmas(&conn).await;

        // Create tables
        self.execute_with_retry(&conn, schema::CREATE_EPISODES_TABLE)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_PATTERNS_TABLE)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_HEURISTICS_TABLE)
            .await?;

        // Create legacy embeddings table only when multi-dimension feature is NOT enabled
        #[cfg(not(feature = "turso_multi_dimension"))]
        self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_TABLE)
            .await?;

        // Create monitoring tables
        self.execute_with_retry(&conn, schema::CREATE_EXECUTION_RECORDS_TABLE)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_AGENT_METRICS_TABLE)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_TASK_METRICS_TABLE)
            .await?;

        // Create indexes
        self.execute_with_retry(&conn, schema::CREATE_EPISODES_TASK_TYPE_INDEX)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_EPISODES_TIMESTAMP_INDEX)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_EPISODES_DOMAIN_INDEX)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_PATTERNS_CONTEXT_INDEX)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_HEURISTICS_CONFIDENCE_INDEX)
            .await?;

        // Create legacy embeddings indexes only when multi-dimension feature is NOT enabled
        #[cfg(not(feature = "turso_multi_dimension"))]
        {
            self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_ITEM_INDEX)
                .await?;
            self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_VECTOR_INDEX)
                .await?;
        }

        // Create monitoring indexes
        self.execute_with_retry(&conn, schema::CREATE_EXECUTION_RECORDS_TIME_INDEX)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_EXECUTION_RECORDS_AGENT_INDEX)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_AGENT_METRICS_TYPE_INDEX)
            .await?;

        // Create Phase 2 (GENESIS) tables and indexes
        self.execute_with_retry(&conn, schema::CREATE_EPISODE_SUMMARIES_TABLE)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_SUMMARIES_CREATED_AT_INDEX)
            .await?;
        self.execute_with_retry(&conn, schema::CREATE_METADATA_TABLE)
            .await?;

        // Create FTS5 tables for hybrid search (feature-gated)
        #[cfg(feature = "hybrid_search")]
        {
            info!("Initializing FTS5 schema for hybrid search");
            self.execute_with_retry(&conn, fts5_schema::CREATE_EPISODES_FTS_TABLE)
                .await?;
            self.execute_with_retry(&conn, fts5_schema::CREATE_PATTERNS_FTS_TABLE)
                .await?;
            self.execute_with_retry(&conn, fts5_schema::CREATE_EPISODES_FTS_TRIGGERS)
                .await?;
            self.execute_with_retry(&conn, fts5_schema::CREATE_PATTERNS_FTS_TRIGGERS)
                .await?;
            info!("FTS5 schema initialization complete");
        }

        // Create dimension-specific vector tables (Phase 0)
        #[cfg(feature = "turso_multi_dimension")]
        {
            info!("Initializing dimension-specific vector tables");

            // Create tables
            self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_384_TABLE)
                .await?;
            self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_1024_TABLE)
                .await?;
            self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_1536_TABLE)
                .await?;
            self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_3072_TABLE)
                .await?;
            self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_OTHER_TABLE)
                .await?;

            // Create vector indexes
            self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_384_VECTOR_INDEX)
                .await?;
            self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_1024_VECTOR_INDEX)
                .await?;
            self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_1536_VECTOR_INDEX)
                .await?;
            self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_3072_VECTOR_INDEX)
                .await?;

            // Create item indexes
            self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_384_ITEM_INDEX)
                .await?;
            self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_1024_ITEM_INDEX)
                .await?;
            self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_1536_ITEM_INDEX)
                .await?;
            self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_3072_ITEM_INDEX)
                .await?;
            self.execute_with_retry(&conn, schema::CREATE_EMBEDDINGS_OTHER_ITEM_INDEX)
                .await?;

            info!("Dimension-specific vector tables initialized");
        }

        info!("Schema initialization complete");
        Ok(())
    }

    /// Get a database connection
    ///
    /// If connection pooling is enabled, this will use a pooled connection.
    /// Otherwise, it creates a new connection each time.
    async fn get_connection(&self) -> Result<Connection> {
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
            None
        }
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

/// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStatistics {
    pub episode_count: usize,
    pub pattern_count: usize,
    pub heuristic_count: usize,
}

/// Implement the unified StorageBackend trait for TursoStorage
#[async_trait]
impl StorageBackend for TursoStorage {
    async fn store_episode(&self, episode: &memory_core::Episode) -> Result<()> {
        TursoStorage::store_episode(self, episode).await
    }

    async fn get_episode(&self, id: uuid::Uuid) -> Result<Option<memory_core::Episode>> {
        TursoStorage::get_episode(self, id).await
    }

    async fn store_pattern(&self, pattern: &memory_core::Pattern) -> Result<()> {
        TursoStorage::store_pattern(self, pattern).await
    }

    async fn get_pattern(
        &self,
        id: memory_core::episode::PatternId,
    ) -> Result<Option<memory_core::Pattern>> {
        TursoStorage::get_pattern(self, id).await
    }

    async fn store_heuristic(&self, heuristic: &memory_core::Heuristic) -> Result<()> {
        TursoStorage::store_heuristic(self, heuristic).await
    }

    async fn get_heuristic(&self, id: uuid::Uuid) -> Result<Option<memory_core::Heuristic>> {
        TursoStorage::get_heuristic(self, id).await
    }

    async fn query_episodes_since(
        &self,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<memory_core::Episode>> {
        TursoStorage::query_episodes_since(self, since).await
    }

    async fn query_episodes_by_metadata(
        &self,
        key: &str,
        value: &str,
    ) -> Result<Vec<memory_core::Episode>> {
        TursoStorage::query_episodes_by_metadata(self, key, value).await
    }

    async fn store_embedding(&self, id: &str, embedding: Vec<f32>) -> Result<()> {
        TursoStorage::store_embedding_backend(self, id, embedding).await
    }

    async fn get_embedding(&self, id: &str) -> Result<Option<Vec<f32>>> {
        TursoStorage::get_embedding_backend(self, id).await
    }

    async fn delete_embedding(&self, id: &str) -> Result<bool> {
        TursoStorage::delete_embedding_backend(self, id).await
    }

    async fn store_embeddings_batch(&self, embeddings: Vec<(String, Vec<f32>)>) -> Result<()> {
        TursoStorage::store_embeddings_batch_backend(self, embeddings).await
    }

    async fn get_embeddings_batch(&self, ids: &[String]) -> Result<Vec<Option<Vec<f32>>>> {
        TursoStorage::get_embeddings_batch_backend(self, ids).await
    }
}

/// Implement the MonitoringStorageBackend trait for TursoStorage
#[async_trait]
impl memory_core::monitoring::storage::MonitoringStorageBackend for TursoStorage {
    async fn store_execution_record(
        &self,
        record: &memory_core::monitoring::types::ExecutionRecord,
    ) -> Result<()> {
        TursoStorage::store_execution_record(self, record)
            .await
            .map_err(|e| memory_core::Error::Storage(format!("Storage error: {}", e)))
    }

    async fn store_agent_metrics(
        &self,
        metrics: &memory_core::monitoring::types::AgentMetrics,
    ) -> Result<()> {
        TursoStorage::store_agent_metrics(self, metrics)
            .await
            .map_err(|e| memory_core::Error::Storage(format!("Storage error: {}", e)))
    }

    async fn store_task_metrics(
        &self,
        metrics: &memory_core::monitoring::types::TaskMetrics,
    ) -> Result<()> {
        TursoStorage::store_task_metrics(self, metrics)
            .await
            .map_err(|e| memory_core::Error::Storage(format!("Storage error: {}", e)))
    }

    async fn load_agent_metrics(
        &self,
        agent_name: &str,
    ) -> Result<Option<memory_core::monitoring::types::AgentMetrics>> {
        TursoStorage::load_agent_metrics(self, agent_name)
            .await
            .map_err(|e| memory_core::Error::Storage(format!("Storage error: {}", e)))
    }

    async fn load_execution_records(
        &self,
        agent_name: Option<&str>,
        limit: usize,
    ) -> Result<Vec<memory_core::monitoring::types::ExecutionRecord>> {
        TursoStorage::load_execution_records(self, agent_name, limit)
            .await
            .map_err(|e| memory_core::Error::Storage(format!("Storage error: {}", e)))
    }

    async fn load_task_metrics(
        &self,
        task_type: &str,
    ) -> Result<Option<memory_core::monitoring::types::TaskMetrics>> {
        TursoStorage::load_task_metrics(self, task_type)
            .await
            .map_err(|e| memory_core::Error::Storage(format!("Storage error: {}", e)))
    }
}
