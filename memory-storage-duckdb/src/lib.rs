//! # Memory Storage - `DuckDB`
//!
//! `DuckDB` storage backend for episodic memory.

use do_memory_core::{Error, Result};
use duckdb::Connection;
use parking_lot::Mutex;
use std::path::Path;
use std::sync::Arc;
use tracing::info;

/// Schema definitions for `DuckDB`.
pub mod schema;
/// Storage implementation for `DuckDB`.
pub mod storage;
/// Trait implementations for `DuckDB`.
pub mod trait_impls;

/// `DuckDB` storage backend.
pub struct DuckDbStorage {
    /// Shared connection pool (simplified to `Arc<Mutex<Connection>>` as `DuckDB` is embedded).
    pub(crate) conn: Arc<Mutex<Connection>>,
    /// Pluggable event emitter for standardized lifecycle notifications
    pub(crate) event_emitter:
        Arc<parking_lot::RwLock<Option<Arc<dyn do_memory_core::types::event::EventEmitter>>>>,
}

impl DuckDbStorage {
    /// Create a new `DuckDB` storage instance.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `DuckDB` database file.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be opened or initialized.
    pub async fn new(path: &Path) -> Result<Self> {
        info!("Opening DuckDB database at {}", path.display());

        let path_buf = path.to_path_buf();
        let conn = tokio::task::spawn_blocking(move || {
            Connection::open(&path_buf)
                .map_err(|e| Error::Storage(format!("Failed to open DuckDB: {e}")))
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;

        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
            event_emitter: Arc::new(parking_lot::RwLock::new(None)),
        };

        // Initialize schema
        storage.initialize_schema().await?;

        // Load extensions
        storage.load_vss_extension().await?;

        info!("Successfully opened DuckDB database and initialized schema");
        Ok(storage)
    }

    /// Initialize the database schema.
    ///
    /// # Errors
    ///
    /// Returns an error if any schema creation statement fails.
    pub async fn initialize_schema(&self) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();

            // Execute all schema creation statements in a batch for reliability
            let schema_sql = format!(
                "{}{}{}{}{}{}{}{}{}{}{}{}{}",
                schema::CREATE_EPISODES_TABLE,
                schema::CREATE_PATTERNS_TABLE,
                schema::CREATE_HEURISTICS_TABLE,
                schema::CREATE_RECOMMENDATION_SESSIONS_TABLE,
                schema::CREATE_RECOMMENDATION_FEEDBACK_TABLE,
                schema::CREATE_EMBEDDINGS_TABLE,
                schema::CREATE_EPISODE_RELATIONSHIPS_TABLE,
                schema::CREATE_EXECUTION_RECORDS_SEQUENCE,
                schema::CREATE_EXECUTION_RECORDS_TABLE,
                schema::CREATE_AGENT_METRICS_TABLE,
                schema::CREATE_TASK_METRICS_TABLE,
                schema::CREATE_EPISODE_SUMMARIES_TABLE,
                schema::CREATE_EPISODE_TAGS_TABLE,
            );

            conn.execute_batch(&schema_sql)
                .map_err(|e| Error::Storage(format!("Failed to initialize schema: {e}")))?;

            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(())
    }

    /// Emit a standardized event if an emitter is configured.
    pub(crate) async fn emit_event(&self, event: do_memory_core::types::event::MemoryEvent) {
        let emitter = {
            let lock = self.event_emitter.read();
            lock.as_ref().map(Arc::clone)
        };

        if let Some(emitter) = emitter {
            let _ = emitter.emit(event).await;
        }
    }
}
