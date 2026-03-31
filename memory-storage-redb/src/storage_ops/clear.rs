//! Table clearing operations for RedbStorage
//!
//! Provides functionality to clear tables for schema version changes
//! and manual cache clearing.

use super::super::{
    EMBEDDINGS_TABLE, EPISODES_TABLE, HEURISTICS_TABLE, METADATA_TABLE, PATTERNS_TABLE,
    RECOMMENDATION_EPISODE_INDEX_TABLE, RECOMMENDATION_FEEDBACK_TABLE,
    RECOMMENDATION_SESSIONS_TABLE, RELATIONSHIPS_TABLE, SUMMARIES_TABLE, with_db_timeout,
};
use crate::RedbStorage;
use memory_core::{Error, Result};
use redb::ReadableTable;
use std::sync::Arc;
use tracing::info;

impl RedbStorage {
    /// Clear all tables (internal helper for schema version changes)
    pub(super) async fn clear_all_tables(&self) -> Result<()> {
        info!("Clearing all tables due to schema version change");

        let db = Arc::clone(&self.db);

        with_db_timeout(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {}", e)))?;

            {
                // Clear each table by removing all entries
                Self::clear_table_entries(&write_txn, EPISODES_TABLE, "episodes")?;
                Self::clear_table_entries(&write_txn, PATTERNS_TABLE, "patterns")?;
                Self::clear_table_entries(&write_txn, HEURISTICS_TABLE, "heuristics")?;
                Self::clear_table_entries(&write_txn, EMBEDDINGS_TABLE, "embeddings")?;
                Self::clear_table_entries(&write_txn, METADATA_TABLE, "metadata")?;
                Self::clear_table_entries(&write_txn, SUMMARIES_TABLE, "summaries")?;
                Self::clear_table_entries(&write_txn, RELATIONSHIPS_TABLE, "relationships")?;
                Self::clear_table_entries(
                    &write_txn,
                    RECOMMENDATION_SESSIONS_TABLE,
                    "recommendation_sessions",
                )?;
                Self::clear_table_entries(
                    &write_txn,
                    RECOMMENDATION_FEEDBACK_TABLE,
                    "recommendation_feedback",
                )?;
                Self::clear_table_entries_str(
                    &write_txn,
                    RECOMMENDATION_EPISODE_INDEX_TABLE,
                    "recommendation_episode_index",
                )?;
            }

            write_txn
                .commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;

            Ok::<(), Error>(())
        })
        .await?;

        // Also clear the in-memory cache
        self.cache.clear().await;

        info!("Successfully cleared all tables");
        Ok(())
    }

    /// Clear all cached data (use with caution!)
    pub async fn clear_all(&self) -> Result<()> {
        info!("Clearing all cached data from redb");

        // Clear the LRU cache metadata
        self.cache.clear().await;

        let db = Arc::clone(&self.db);

        with_db_timeout(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {}", e)))?;

            {
                Self::clear_table_entries(&write_txn, EPISODES_TABLE, "episodes")?;
                Self::clear_table_entries(&write_txn, PATTERNS_TABLE, "patterns")?;
                Self::clear_table_entries(&write_txn, HEURISTICS_TABLE, "heuristics")?;
                Self::clear_table_entries(&write_txn, EMBEDDINGS_TABLE, "embeddings")?;
                Self::clear_table_entries(&write_txn, METADATA_TABLE, "metadata")?;
                Self::clear_table_entries(&write_txn, SUMMARIES_TABLE, "summaries")?;
                Self::clear_table_entries(&write_txn, RELATIONSHIPS_TABLE, "relationships")?;
            }

            write_txn
                .commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;

            Ok::<(), Error>(())
        })
        .await?;

        info!("Successfully cleared all cached data");
        Ok(())
    }

    /// Helper to clear all entries from a table with string key and byte value
    fn clear_table_entries(
        write_txn: &redb::WriteTransaction,
        table_def: redb::TableDefinition<&str, &[u8]>,
        table_name: &str,
    ) -> Result<()> {
        let mut table = write_txn
            .open_table(table_def)
            .map_err(|e| Error::Storage(format!("Failed to open {} table: {}", table_name, e)))?;
        let keys: Vec<String> = table
            .iter()
            .map_err(|e| Error::Storage(format!("Failed to iterate {}: {}", table_name, e)))?
            .filter_map(|item| item.ok())
            .map(|(k, _v)| k.value().to_string())
            .collect();
        for key in keys {
            table.remove(key.as_str()).map_err(|e| {
                Error::Storage(format!("Failed to remove {} key: {}", table_name, e))
            })?;
        }
        Ok(())
    }

    /// Helper to clear all entries from a table with string key and string value
    fn clear_table_entries_str(
        write_txn: &redb::WriteTransaction,
        table_def: redb::TableDefinition<&str, &str>,
        table_name: &str,
    ) -> Result<()> {
        let mut table = write_txn
            .open_table(table_def)
            .map_err(|e| Error::Storage(format!("Failed to open {} table: {}", table_name, e)))?;
        let keys: Vec<String> = table
            .iter()
            .map_err(|e| Error::Storage(format!("Failed to iterate {}: {}", table_name, e)))?
            .filter_map(|item| item.ok())
            .map(|(k, _v)| k.value().to_string())
            .collect();
        for key in keys {
            table.remove(key.as_str()).map_err(|e| {
                Error::Storage(format!("Failed to remove {} key: {}", table_name, e))
            })?;
        }
        Ok(())
    }
}
