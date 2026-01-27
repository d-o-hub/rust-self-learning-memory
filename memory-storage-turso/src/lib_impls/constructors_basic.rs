//! TursoStorage Basic Constructors
//!
//! This module contains basic constructor methods for TursoStorage:
//! - new()
//! - from_database()
//! - with_config()

use libsql::{Builder, Database};
use memory_core::Result;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

use super::super::{
    ConnectionPool, PoolConfig, PreparedCacheConfig, PreparedStatementCache, TursoConfig,
};
use super::storage::TursoStorage;

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
    pub fn from_database(db: Database) -> Result<Self> {
        Ok(Self {
            db: Arc::new(db),
            pool: None,
            #[cfg(feature = "keepalive-pool")]
            keepalive_pool: None,
            adaptive_pool: None,
            prepared_cache: Arc::new(PreparedStatementCache::with_config(
                PreparedCacheConfig::default(),
            )),
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
            return Err(memory_core::Error::Security(format!(
                "Insecure database URL: {}. Only libsql://, file:, or :memory: protocols are allowed",
                url
            )));
        }

        // SECURITY: Validate token is provided for remote connections
        if url.starts_with("libsql://") && token.trim().is_empty() {
            return Err(memory_core::Error::Security(
                "Authentication token required for remote Turso connections".to_string(),
            ));
        }

        let db = if url.starts_with("libsql://") {
            Builder::new_remote(url.to_string(), token.to_string())
                .build()
                .await
                .map_err(|e| {
                    memory_core::Error::Storage(format!("Failed to connect to Turso: {}", e))
                })?
        } else {
            let path = if let Some(stripped) = url.strip_prefix("file:") {
                stripped
            } else {
                url
            };
            Builder::new_local(path).build().await.map_err(|e| {
                memory_core::Error::Storage(format!("Failed to connect to Turso: {}", e))
            })?
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
                tracing::warn!("Keep-alive requested but pooling disabled, skipping");
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
            prepared_cache: Arc::new(PreparedStatementCache::with_config(
                PreparedCacheConfig::default(),
            )),
            config,
        };

        // Return the storage - caller can wrap with CachedTursoStorage if needed
        Ok(storage)
    }
}
