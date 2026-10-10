//! Non-destructive schema migration and index rebuild for RedbStorage
//!
//! Nothing in this module clears primary data. [`RedbStorage::migrate_schema`]
//! validates that existing rows still decode with the current schema before
//! recording the current version; on failure it preserves the file and returns
//! a typed error (issue #1069).

use super::super::{
    EPISODE_REVISIONS_TABLE, EPISODES_TABLE, RECOMMENDATION_EPISODE_INDEX_TABLE,
    RECOMMENDATION_SESSIONS_TABLE, SCHEMA_VERSION, with_db_timeout,
};
use super::schema::SchemaInspection;
use crate::RedbStorage;
use do_memory_core::memory::attribution::RecommendationSession;
use do_memory_core::{Episode, Error, Heuristic, Pattern, Result};
use redb::{ReadableDatabase, ReadableTable, TableDefinition, TableError};
use serde::de::DeserializeOwned;
use std::sync::Arc;
use tracing::info;

impl RedbStorage {
    /// Validate and adopt existing data under the current schema version.
    ///
    /// Non-destructive migration step: every existing row of the primary data
    /// tables is decoded with the current types inside a read transaction
    /// *before* any write happens. If every row decodes, the current version is
    /// recorded and derived indexes are rebuilt.
    ///
    /// If a row cannot be decoded, [`Error::SchemaMigrationRequired`] is
    /// returned and the database file is left untouched so an external backup
    /// or migration tool can act on it. This never clears data.
    pub async fn migrate_schema(&self) -> Result<()> {
        let inspection = self.inspect_schema().await?;

        if inspection.stored_version == Some(SCHEMA_VERSION) {
            // Already current: only refresh derived indexes.
            return self.rebuild_indexes().await;
        }

        if !inspection.has_data {
            self.store_schema_version().await?;
            return self.rebuild_indexes().await;
        }

        self.validate_existing_rows(&inspection).await?;

        self.store_schema_version().await?;
        self.rebuild_indexes().await
    }

    /// Decode every existing row of the primary data tables with the current
    /// types, returning a typed error (and preserving the file) on failure.
    async fn validate_existing_rows(&self, inspection: &SchemaInspection) -> Result<()> {
        let db = Arc::clone(&self.db);
        let path = inspection.path.display().to_string();
        let stored_version = inspection.stored_version;
        let current_version = inspection.current_version;

        with_db_timeout(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;

            validate_decodable::<Episode>(
                &read_txn,
                "episodes",
                &path,
                stored_version,
                current_version,
            )?;
            validate_decodable::<Pattern>(
                &read_txn,
                "patterns",
                &path,
                stored_version,
                current_version,
            )?;
            validate_decodable::<Heuristic>(
                &read_txn,
                "heuristics",
                &path,
                stored_version,
                current_version,
            )?;

            Ok(())
        })
        .await
    }

    /// Rebuild derived index tables from their source tables.
    ///
    /// Currently rebuilds `recommendation_episode_index` from the persisted
    /// recommendation sessions. This is the S07 index rebuild step and is
    /// deliberately independent of any table-clearing reset: it never removes
    /// primary data.
    pub async fn rebuild_indexes(&self) -> Result<()> {
        let db = Arc::clone(&self.db);

        with_db_timeout(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {}", e)))?;

            let mut pairs: Vec<(String, String)> = Vec::new();

            {
                let sessions = write_txn
                    .open_table(RECOMMENDATION_SESSIONS_TABLE)
                    .map_err(|e| {
                        Error::Storage(format!(
                            "Failed to open recommendation sessions table: {}",
                            e
                        ))
                    })?;

                for entry in sessions.iter().map_err(|e| {
                    Error::Storage(format!("Failed to iterate recommendation sessions: {}", e))
                })? {
                    let (_key, value) = entry.map_err(|e| {
                        Error::Storage(format!("Failed to read recommendation session: {}", e))
                    })?;
                    let session: RecommendationSession =
                        postcard::from_bytes(value.value()).map_err(|e| {
                            Error::Storage(format!(
                                "Failed to decode recommendation session while rebuilding index: {}",
                                e
                            ))
                        })?;
                    pairs.push((
                        session.episode_id.to_string(),
                        session.session_id.to_string(),
                    ));
                }
            }

            {
                let mut index = write_txn
                    .open_table(RECOMMENDATION_EPISODE_INDEX_TABLE)
                    .map_err(|e| {
                        Error::Storage(format!(
                            "Failed to open recommendation episode index: {}",
                            e
                        ))
                    })?;

                let existing: Vec<String> = index
                    .iter()
                    .map_err(|e| {
                        Error::Storage(format!(
                            "Failed to iterate recommendation episode index: {}",
                            e
                        ))
                    })?
                    .filter_map(|item| item.ok())
                    .map(|(key, _value)| key.value().to_string())
                    .collect();

                for key in existing {
                    index.remove(key.as_str()).map_err(|e| {
                        Error::Storage(format!(
                            "Failed to clear recommendation episode index entry: {}",
                            e
                        ))
                    })?;
                }

                for (episode_id, session_id) in &pairs {
                    index
                        .insert(episode_id.as_str(), session_id.as_str())
                        .map_err(|e| {
                            Error::Storage(format!(
                                "Failed to rebuild recommendation episode index: {}",
                                e
                            ))
                        })?;
                }
            }

            // Backfill modification watermarks for cache rows written before
            // `episode_revisions` existed. Existing rows use their start_time as
            // the best available watermark; later writes refresh it.
            {
                let mut revisions = write_txn
                    .open_table(EPISODE_REVISIONS_TABLE)
                    .map_err(|e| {
                        Error::Storage(format!("Failed to open episode revisions table: {}", e))
                    })?;

                let mut missing: Vec<(String, i64)> = Vec::new();
                {
                    let episodes = write_txn.open_table(EPISODES_TABLE).map_err(|e| {
                        Error::Storage(format!("Failed to open episodes table: {}", e))
                    })?;

                    for entry in episodes.iter().map_err(|e| {
                        Error::Storage(format!("Failed to iterate episodes: {}", e))
                    })? {
                        let (key, value) = entry.map_err(|e| {
                            Error::Storage(format!("Failed to read episode entry: {}", e))
                        })?;
                        let episode_id = key.value().to_string();

                        let already_tracked = revisions
                            .get(episode_id.as_str())
                            .map_err(|e| {
                                Error::Storage(format!(
                                    "Failed to read episode revision: {}",
                                    e
                                ))
                            })?
                            .is_some();
                        if already_tracked {
                            continue;
                        }

                        let episode: Episode =
                            postcard::from_bytes(value.value()).map_err(|e| {
                                Error::Storage(format!(
                                    "Failed to decode episode while backfilling revisions: {}",
                                    e
                                ))
                            })?;
                        missing.push((episode_id, episode.start_time.timestamp_millis()));
                    }
                }

                for (episode_id, modified_at_ms) in missing {
                    revisions
                        .insert(episode_id.as_str(), modified_at_ms)
                        .map_err(|e| {
                            Error::Storage(format!("Failed to backfill episode revision: {}", e))
                        })?;
                }
            }

            write_txn
                .commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;

            Ok::<(), Error>(())
        })
        .await?;

        info!("Rebuilt derived indexes");
        Ok(())
    }
}

/// Decode every row of `table_name` as `T`, returning a typed, file-preserving
/// error when a row cannot be decoded with the current schema.
fn validate_decodable<T>(
    read_txn: &redb::ReadTransaction,
    table_name: &'static str,
    path: &str,
    stored_version: Option<u64>,
    current_version: u64,
) -> Result<()>
where
    T: DeserializeOwned,
{
    let def = TableDefinition::<&str, &[u8]>::new(table_name);
    let table = match read_txn.open_table(def) {
        Ok(table) => table,
        Err(TableError::TableDoesNotExist(_)) => return Ok(()),
        Err(e) => {
            return Err(Error::Storage(format!(
                "Failed to open {} table: {}",
                table_name, e
            )));
        }
    };

    for entry in table
        .iter()
        .map_err(|e| Error::Storage(format!("Failed to iterate {} table: {}", table_name, e)))?
    {
        let (key, value) = entry.map_err(|e| {
            Error::Storage(format!("Failed to read {} table row: {}", table_name, e))
        })?;
        postcard::from_bytes::<T>(value.value()).map_err(|e| Error::SchemaMigrationRequired {
            path: path.to_string(),
            stored_version,
            current_version,
            detail: format!(
                "existing {} row '{}' cannot be decoded with the current schema: {}; database preserved",
                table_name,
                key.value(),
                e
            ),
        })?;
    }

    Ok(())
}
