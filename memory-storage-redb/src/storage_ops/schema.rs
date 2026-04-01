//! Schema version management for RedbStorage
//!
//! Handles schema versioning and automatic cache invalidation when
//! data structures change.

use super::super::{
    EMBEDDINGS_TABLE, EPISODES_TABLE, HEURISTICS_TABLE, METADATA_TABLE, PATTERNS_TABLE,
    RECOMMENDATION_EPISODE_INDEX_TABLE, RECOMMENDATION_FEEDBACK_TABLE,
    RECOMMENDATION_SESSIONS_TABLE, RELATIONSHIPS_TABLE, SCHEMA_VERSION, SCHEMA_VERSION_TABLE,
    SUMMARIES_TABLE, with_db_timeout,
};
use crate::RedbStorage;
use do_memory_core::{Error, Result};
use redb::ReadableDatabase;
use std::sync::Arc;
use tracing::info;

impl RedbStorage {
    /// Initialize database tables with schema version check
    ///
    /// This method:
    /// 1. Opens all tables to ensure they exist
    /// 2. Checks the stored schema version against the current version
    /// 3. If versions differ, clears all cached data to prevent deserialization errors
    /// 4. Stores the new schema version
    pub(crate) async fn initialize_tables(&self) -> Result<()> {
        let db = Arc::clone(&self.db);

        with_db_timeout(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {}", e)))?;

            // Open tables to ensure they exist
            {
                let _episodes = write_txn
                    .open_table(EPISODES_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open episodes table: {}", e)))?;
                let _patterns = write_txn
                    .open_table(PATTERNS_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open patterns table: {}", e)))?;
                let _heuristics = write_txn.open_table(HEURISTICS_TABLE).map_err(|e| {
                    Error::Storage(format!("Failed to open heuristics table: {}", e))
                })?;
                let _embeddings = write_txn.open_table(EMBEDDINGS_TABLE).map_err(|e| {
                    Error::Storage(format!("Failed to open embeddings table: {}", e))
                })?;
                let _metadata = write_txn
                    .open_table(METADATA_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open metadata table: {}", e)))?;
                let _summaries = write_txn.open_table(SUMMARIES_TABLE).map_err(|e| {
                    Error::Storage(format!("Failed to open summaries table: {}", e))
                })?;
                let _relationships = write_txn.open_table(RELATIONSHIPS_TABLE).map_err(|e| {
                    Error::Storage(format!("Failed to open relationships table: {}", e))
                })?;
                let _rec_sessions = write_txn
                    .open_table(RECOMMENDATION_SESSIONS_TABLE)
                    .map_err(|e| {
                        Error::Storage(format!(
                            "Failed to open recommendation sessions table: {}",
                            e
                        ))
                    })?;
                let _rec_feedback = write_txn
                    .open_table(RECOMMENDATION_FEEDBACK_TABLE)
                    .map_err(|e| {
                        Error::Storage(format!(
                            "Failed to open recommendation feedback table: {}",
                            e
                        ))
                    })?;
                let _rec_episode = write_txn
                    .open_table(RECOMMENDATION_EPISODE_INDEX_TABLE)
                    .map_err(|e| {
                        Error::Storage(format!(
                            "Failed to open recommendation episode index: {}",
                            e
                        ))
                    })?;
                let _schema_version = write_txn.open_table(SCHEMA_VERSION_TABLE).map_err(|e| {
                    Error::Storage(format!("Failed to open schema version table: {}", e))
                })?;
            }

            write_txn
                .commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;

            Ok::<(), Error>(())
        })
        .await?;

        // Check schema version and invalidate cache if needed
        self.check_and_update_schema_version().await?;

        info!("Initialized redb tables");
        Ok(())
    }

    /// Check schema version and clear cache if version mismatch
    ///
    /// This prevents deserialization errors when the Episode or other cached
    /// structs have been modified and the cached data is stale.
    pub(super) async fn check_and_update_schema_version(&self) -> Result<()> {
        let db = Arc::clone(&self.db);
        let current_version = SCHEMA_VERSION;

        let needs_clear = with_db_timeout(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;

            let version_table = read_txn.open_table(SCHEMA_VERSION_TABLE).map_err(|e| {
                Error::Storage(format!("Failed to open schema version table: {}", e))
            })?;

            let stored_version = version_table
                .get("version")
                .map_err(|e| Error::Storage(format!("Failed to read schema version: {}", e)))?
                .map(|guard| guard.value());

            match stored_version {
                Some(v) if v == current_version => {
                    info!("Schema version {} matches, cache is valid", current_version);
                    Ok(false)
                }
                Some(old_version) => {
                    info!(
                        "Schema version mismatch: stored={}, current={}. Clearing cache.",
                        old_version, current_version
                    );
                    Ok(true)
                }
                None => {
                    info!(
                        "No schema version found, storing version {}",
                        current_version
                    );
                    Ok(true)
                }
            }
        })
        .await?;

        if needs_clear {
            self.clear_all_tables().await?;
            self.store_schema_version().await?;
        }

        Ok(())
    }

    /// Store the current schema version
    pub(super) async fn store_schema_version(&self) -> Result<()> {
        let db = Arc::clone(&self.db);
        let version = SCHEMA_VERSION;

        with_db_timeout(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {}", e)))?;

            {
                let mut version_table =
                    write_txn.open_table(SCHEMA_VERSION_TABLE).map_err(|e| {
                        Error::Storage(format!("Failed to open schema version table: {}", e))
                    })?;
                version_table.insert("version", version).map_err(|e| {
                    Error::Storage(format!("Failed to store schema version: {}", e))
                })?;
            }

            write_txn
                .commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;

            info!("Stored schema version {}", version);
            Ok::<(), Error>(())
        })
        .await
    }
}
