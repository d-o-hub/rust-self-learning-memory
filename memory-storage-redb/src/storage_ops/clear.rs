//! Table clearing operations for RedbStorage
//!
//! Clearing is split into an *explicit* operator action
//! ([`RedbStorage::reset_all_tables`]) and the single internal implementation
//! ([`RedbStorage::clear_all_tables`]) it delegates to. Nothing in the open or
//! schema-inspection path clears data any more (issue #1069).

use super::super::{
    EMBEDDINGS_TABLE, EPISODE_PATTERN_RELATIONSHIPS_TABLE, EPISODE_REVISIONS_TABLE, EPISODES_TABLE,
    HEURISTICS_TABLE, METADATA_TABLE, PATTERNS_TABLE, PROCEDURAL_TABLE,
    RECOMMENDATION_EPISODE_INDEX_TABLE, RECOMMENDATION_FEEDBACK_TABLE,
    RECOMMENDATION_SESSIONS_TABLE, RELATIONSHIPS_TABLE, SUMMARIES_TABLE, with_db_timeout,
};
use crate::RedbStorage;
use do_memory_core::{Error, Result};
use redb::ReadableTable;
use std::sync::Arc;
use tracing::{info, warn};

impl RedbStorage {
    /// **Destructive, explicit operator action.**
    ///
    /// Erases every row from every table (episodes, patterns, heuristics,
    /// embeddings, metadata, summaries, relationships, recommendation sessions,
    /// feedback and episode index, procedural data), clears the in-memory
    /// cache, then records the current schema version and rebuilds the derived
    /// indexes.
    ///
    /// All user data is permanently lost. This is the only supported path that
    /// clears every table; it is never invoked implicitly by opening a
    /// database. Call it deliberately, after taking a backup, when a schema
    /// mismatch must be resolved by discarding data.
    pub async fn reset_all_tables(&self) -> Result<()> {
        warn!(
            path = %self.path.display(),
            "reset_all_tables: permanently erasing every table (explicit operator reset)"
        );

        self.clear_all_tables().await?;
        self.store_schema_version().await?;
        self.rebuild_indexes().await?;

        info!("Explicit reset complete; database now records the current schema version");
        Ok(())
    }

    /// Clear all tables (internal implementation used only by explicit resets).
    pub(super) async fn clear_all_tables(&self) -> Result<()> {
        info!("Clearing all tables (explicit reset)");

        let db = Arc::clone(&self.db);

        with_db_timeout(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {}", e)))?;

            {
                // Clear each table by removing all entries
                Self::clear_table_entries(&write_txn, EPISODES_TABLE, "episodes")?;
                {
                    // `episode_revisions` is `(&str, i64)`, so it needs its own
                    // clearing pass rather than `clear_table_entries`.
                    let mut revisions =
                        write_txn.open_table(EPISODE_REVISIONS_TABLE).map_err(|e| {
                            Error::Storage(format!("Failed to open episode_revisions table: {}", e))
                        })?;
                    let keys: Vec<String> = revisions
                        .iter()
                        .map_err(|e| {
                            Error::Storage(format!("Failed to iterate episode_revisions: {}", e))
                        })?
                        .filter_map(|item| item.ok())
                        .map(|(k, _v)| k.value().to_string())
                        .collect();
                    for key in keys {
                        revisions.remove(key.as_str()).map_err(|e| {
                            Error::Storage(format!("Failed to remove episode_revisions key: {}", e))
                        })?;
                    }
                }
                Self::clear_table_entries(&write_txn, PATTERNS_TABLE, "patterns")?;
                Self::clear_table_entries(&write_txn, HEURISTICS_TABLE, "heuristics")?;
                Self::clear_table_entries(&write_txn, EMBEDDINGS_TABLE, "embeddings")?;
                Self::clear_table_entries(&write_txn, METADATA_TABLE, "metadata")?;
                Self::clear_table_entries(&write_txn, SUMMARIES_TABLE, "summaries")?;
                Self::clear_table_entries(&write_txn, RELATIONSHIPS_TABLE, "relationships")?;
                Self::clear_table_entries(
                    &write_txn,
                    EPISODE_PATTERN_RELATIONSHIPS_TABLE,
                    "episode_pattern_relationships",
                )?;
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
                Self::clear_table_entries(&write_txn, PROCEDURAL_TABLE, "procedural")?;
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

    /// **Destructive.** Clear all cached data (use with caution!).
    ///
    /// Retained for backward compatibility; it delegates to the same single
    /// clearing implementation as [`RedbStorage::reset_all_tables`], so it
    /// erases every table, not just cached rows. Prefer `reset_all_tables`,
    /// which also records the current schema version.
    pub async fn clear_all(&self) -> Result<()> {
        info!("Clearing all cached data from redb");
        self.clear_all_tables().await?;
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
