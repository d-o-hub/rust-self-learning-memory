//! TursoStorage Pool Constructors
//!
//! This module contains pool-related constructor methods for TursoStorage:
//! - new_with_pool_config()
//! - new_with_keepalive()

#[cfg(feature = "keepalive-pool")]
use libsql::Builder;
#[cfg(feature = "keepalive-pool")]
use memory_core::Result;
#[cfg(feature = "keepalive-pool")]
use std::sync::Arc;
#[cfg(feature = "keepalive-pool")]
use std::time::Duration;
#[cfg(feature = "keepalive-pool")]
use tracing::info;

#[cfg(feature = "keepalive-pool")]
use super::super::{
    ConnectionPool, KeepAliveConfig, KeepAlivePool, PoolConfig, PreparedCacheConfig,
    PreparedStatementCache, TursoConfig,
};
#[cfg(feature = "keepalive-pool")]
use super::storage::TursoStorage;

#[cfg(feature = "keepalive-pool")]
impl TursoStorage {
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
            prepared_cache: Arc::new(PreparedStatementCache::with_config(
                PreparedCacheConfig::default(),
            )),
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
            prepared_cache: Arc::new(PreparedStatementCache::with_config(
                PreparedCacheConfig::default(),
            )),
            config,
        })
    }
}
