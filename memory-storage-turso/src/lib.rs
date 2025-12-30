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
        self.store_episode(episode).await
    }

    async fn get_episode(&self, id: uuid::Uuid) -> Result<Option<memory_core::Episode>> {
        self.get_episode(id).await
    }

    async fn store_pattern(&self, pattern: &memory_core::Pattern) -> Result<()> {
        self.store_pattern(pattern).await
    }

    async fn get_pattern(
        &self,
        id: memory_core::episode::PatternId,
    ) -> Result<Option<memory_core::Pattern>> {
        self.get_pattern(id).await
    }

    async fn store_heuristic(&self, heuristic: &memory_core::Heuristic) -> Result<()> {
        self.store_heuristic(heuristic).await
    }

    async fn get_heuristic(&self, id: uuid::Uuid) -> Result<Option<memory_core::Heuristic>> {
        self.get_heuristic(id).await
    }

    async fn query_episodes_since(
        &self,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<memory_core::Episode>> {
        self.query_episodes_since(since).await
    }

    async fn query_episodes_by_metadata(
        &self,
        key: &str,
        value: &str,
    ) -> Result<Vec<memory_core::Episode>> {
        self.query_episodes_by_metadata(key, value).await
    }

    async fn store_embedding(&self, id: &str, embedding: Vec<f32>) -> Result<()> {
        self.store_embedding(id, embedding).await
    }

    async fn get_embedding(&self, id: &str) -> Result<Option<Vec<f32>>> {
        self.get_embedding(id).await
    }

    async fn delete_embedding(&self, id: &str) -> Result<bool> {
        self.delete_embedding(id).await
    }

    async fn store_embeddings_batch(&self, embeddings: Vec<(String, Vec<f32>)>) -> Result<()> {
        self.store_embeddings_batch(embeddings).await
    }

    async fn get_embeddings_batch(&self, ids: &[String]) -> Result<Vec<Option<Vec<f32>>>> {
        self.get_embeddings_batch(ids).await
    }
}

/// Implement the MonitoringStorageBackend trait for TursoStorage
#[async_trait]
impl memory_core::monitoring::storage::MonitoringStorageBackend for TursoStorage {
    async fn store_execution_record(
        &self,
        record: &memory_core::monitoring::types::ExecutionRecord,
    ) -> Result<()> {
        self.store_execution_record(record)
            .await
            .map_err(|e| memory_core::Error::Storage(format!("Storage error: {}", e)))
    }

    async fn store_agent_metrics(
        &self,
        metrics: &memory_core::monitoring::types::AgentMetrics,
    ) -> Result<()> {
        self.store_agent_metrics(metrics)
            .await
            .map_err(|e| memory_core::Error::Storage(format!("Storage error: {}", e)))
    }

    async fn store_task_metrics(
        &self,
        metrics: &memory_core::monitoring::types::TaskMetrics,
    ) -> Result<()> {
        self.store_task_metrics(metrics)
            .await
            .map_err(|e| memory_core::Error::Storage(format!("Storage error: {}", e)))
    }

    async fn load_agent_metrics(
        &self,
        agent_name: &str,
    ) -> Result<Option<memory_core::monitoring::types::AgentMetrics>> {
        self.load_agent_metrics(agent_name)
            .await
            .map_err(|e| memory_core::Error::Storage(format!("Storage error: {}", e)))
    }

    async fn load_execution_records(
        &self,
        agent_name: Option<&str>,
        limit: usize,
    ) -> Result<Vec<memory_core::monitoring::types::ExecutionRecord>> {
        self.load_execution_records(agent_name, limit)
            .await
            .map_err(|e| memory_core::Error::Storage(format!("Storage error: {}", e)))
    }

    async fn load_task_metrics(
        &self,
        task_type: &str,
    ) -> Result<Option<memory_core::monitoring::types::TaskMetrics>> {
        self.load_task_metrics(task_type)
            .await
            .map_err(|e| memory_core::Error::Storage(format!("Storage error: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_storage() -> Result<(TursoStorage, TempDir)> {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");

        // Use Builder::new_local for file-based databases
        let db = libsql::Builder::new_local(&db_path)
            .build()
            .await
            .map_err(|e| Error::Storage(format!("Failed to create test database: {}", e)))?;

        let storage = TursoStorage {
            db: Arc::new(db),
            pool: None,
            config: TursoConfig::default(),
        };

        storage.initialize_schema().await?;
        Ok((storage, dir))
    }

    #[tokio::test]
    async fn test_storage_creation() {
        let result = create_test_storage().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_health_check() {
        let (storage, _dir) = create_test_storage().await.unwrap();
        let healthy = storage.health_check().await.unwrap();
        assert!(healthy);
    }

    #[tokio::test]
    async fn test_statistics() {
        let (storage, _dir) = create_test_storage().await.unwrap();
        let stats = storage.get_statistics().await.unwrap();
        assert_eq!(stats.episode_count, 0);
        assert_eq!(stats.pattern_count, 0);
        assert_eq!(stats.heuristic_count, 0);
    }

    // ========== Embedding Storage Tests ==========

    #[tokio::test]
    async fn test_store_and_get_embedding() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let id = "test_embedding_1";
        let embedding = vec![0.1_f32, 0.2, 0.3, 0.4];

        // Store embedding
        storage
            .store_embedding(id, embedding.clone())
            .await
            .unwrap();

        // Retrieve embedding
        let retrieved = storage.get_embedding(id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), embedding);
    }

    #[tokio::test]
    async fn test_get_nonexistent_embedding() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let retrieved = storage.get_embedding("nonexistent").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_delete_embedding() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let id = "test_embedding_delete";
        let embedding = vec![0.1_f32, 0.2, 0.3];

        // Store embedding
        storage
            .store_embedding(id, embedding.clone())
            .await
            .unwrap();

        // Verify it exists
        let retrieved = storage.get_embedding(id).await.unwrap();
        assert!(retrieved.is_some());

        // Delete embedding
        let deleted = storage.delete_embedding(id).await.unwrap();
        assert!(deleted);

        // Verify it's gone
        let retrieved = storage.get_embedding(id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_embedding() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let deleted = storage.delete_embedding("nonexistent").await.unwrap();
        assert!(!deleted);
    }

    #[tokio::test]
    async fn test_store_embeddings_batch() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let embeddings = vec![
            ("batch_1".to_string(), vec![0.1_f32, 0.2, 0.3]),
            ("batch_2".to_string(), vec![0.4_f32, 0.5, 0.6]),
            ("batch_3".to_string(), vec![0.7_f32, 0.8, 0.9]),
        ];

        // Store embeddings in batch
        storage
            .store_embeddings_batch(embeddings.clone())
            .await
            .unwrap();

        // Verify all embeddings were stored
        for (id, expected_embedding) in &embeddings {
            let retrieved = storage.get_embedding(id).await.unwrap();
            assert!(retrieved.is_some());
            assert_eq!(retrieved.unwrap(), *expected_embedding);
        }
    }

    #[tokio::test]
    async fn test_get_embeddings_batch() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let embeddings = vec![
            ("get_batch_1".to_string(), vec![0.1_f32, 0.2]),
            ("get_batch_2".to_string(), vec![0.3_f32, 0.4]),
            ("get_batch_3".to_string(), vec![0.5_f32, 0.6]),
        ];

        // Store embeddings
        storage
            .store_embeddings_batch(embeddings.clone())
            .await
            .unwrap();

        // Get embeddings in batch
        let ids = vec![
            "get_batch_1".to_string(),
            "get_batch_2".to_string(),
            "get_batch_3".to_string(),
            "nonexistent".to_string(),
        ];

        let results = storage.get_embeddings_batch(&ids).await.unwrap();

        // Verify results
        assert_eq!(results.len(), 4);

        assert!(results[0].is_some());
        assert_eq!(results[0].as_ref().unwrap(), &embeddings[0].1);

        assert!(results[1].is_some());
        assert_eq!(results[1].as_ref().unwrap(), &embeddings[1].1);

        assert!(results[2].is_some());
        assert_eq!(results[2].as_ref().unwrap(), &embeddings[2].1);

        assert!(results[3].is_none()); // Nonexistent embedding
    }

    #[tokio::test]
    async fn test_different_embedding_dimensions() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        // Test different dimensions (384, 1024, 1536)
        let dim_384: Vec<f32> = (0..384).map(|i| i as f32 / 384.0).collect();
        let dim_1024: Vec<f32> = (0..1024).map(|i| i as f32 / 1024.0).collect();
        let dim_1536: Vec<f32> = (0..1536).map(|i| i as f32 / 1536.0).collect();

        // Store different dimensions
        storage
            .store_embedding("dim_384", dim_384.clone())
            .await
            .unwrap();

        storage
            .store_embedding("dim_1024", dim_1024.clone())
            .await
            .unwrap();

        storage
            .store_embedding("dim_1536", dim_1536.clone())
            .await
            .unwrap();

        // Retrieve and verify dimensions
        let retrieved_384 = storage.get_embedding("dim_384").await.unwrap();
        assert!(retrieved_384.is_some());
        assert_eq!(retrieved_384.unwrap().len(), 384);

        let retrieved_1024 = storage.get_embedding("dim_1024").await.unwrap();
        assert!(retrieved_1024.is_some());
        assert_eq!(retrieved_1024.unwrap().len(), 1024);

        let retrieved_1536 = storage.get_embedding("dim_1536").await.unwrap();
        assert!(retrieved_1536.is_some());
        assert_eq!(retrieved_1536.unwrap().len(), 1536);
    }

    #[tokio::test]
    async fn test_update_existing_embedding() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let id = "update_test";
        let embedding_v1 = vec![0.1_f32, 0.2, 0.3];
        let embedding_v2 = vec![0.9_f32, 0.8, 0.7];

        // Store initial embedding
        storage
            .store_embedding(id, embedding_v1.clone())
            .await
            .unwrap();

        // Verify initial embedding
        let retrieved = storage.get_embedding(id).await.unwrap();
        assert_eq!(retrieved.unwrap(), embedding_v1);

        // Update embedding
        storage
            .store_embedding(id, embedding_v2.clone())
            .await
            .unwrap();

        // Verify updated embedding
        let retrieved = storage.get_embedding(id).await.unwrap();
        assert_eq!(retrieved.unwrap(), embedding_v2);
    }

    #[tokio::test]
    async fn test_empty_embeddings_batch() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        // Store empty batch
        storage.store_embeddings_batch(vec![]).await.unwrap();

        // Get empty batch
        let results = storage.get_embeddings_batch(&[]).await.unwrap();
        assert!(results.is_empty());
    }
}
