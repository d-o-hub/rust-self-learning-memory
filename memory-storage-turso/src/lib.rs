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

mod schema;
mod storage;

pub use storage::{EpisodeQuery, PatternQuery};

/// Turso storage backend for durable persistence
pub struct TursoStorage {
    db: Arc<Database>,
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

        let builder = Builder::new_remote(url.to_string(), token.to_string());

        let db = builder
            .build()
            .await
            .map_err(|e| Error::Storage(format!("Failed to connect to Turso: {}", e)))?;

        info!("Successfully connected to Turso database");

        Ok(Self {
            db: Arc::new(db),
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

        info!("Schema initialization complete");
        Ok(())
    }

    /// Get a database connection
    async fn get_connection(&self) -> Result<Connection> {
        self.db
            .connect()
            .map_err(|e| Error::Storage(format!("Failed to get connection: {}", e)))
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
}
