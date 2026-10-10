//! Schema version management for RedbStorage
//!
//! Schema *inspection* is deliberately separated from *migration* (see
//! [`super::migration`]) and from *reset* (see [`super::clear`]). Opening an
//! existing database never clears data: when the stored schema version is
//! missing or stale the open fails closed with a typed
//! [`do_memory_core::Error::SchemaMigrationRequired`] error and leaves every
//! table untouched (issue #1069).

use super::super::{
    DATA_TABLE_NAMES, EMBEDDINGS_TABLE, EPISODE_PATTERN_RELATIONSHIPS_TABLE,
    EPISODE_REVISIONS_TABLE, EPISODES_TABLE, HEURISTICS_TABLE, KNOWN_TABLE_NAMES, METADATA_TABLE,
    PATTERNS_TABLE, PROCEDURAL_TABLE, RECOMMENDATION_EPISODE_INDEX_TABLE,
    RECOMMENDATION_FEEDBACK_TABLE, RECOMMENDATION_SESSIONS_TABLE, RELATIONSHIPS_TABLE,
    SCHEMA_VERSION, SCHEMA_VERSION_TABLE, SUMMARIES_TABLE, with_db_timeout,
};
use crate::RedbStorage;
use do_memory_core::{Error, Result};
use redb::{ReadableDatabase, ReadableTableMetadata, TableDefinition, TableError, TableHandle};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{info, warn};

/// Read-only snapshot of the schema state of a redb database.
#[derive(Debug, Clone)]
pub(super) struct SchemaInspection {
    /// Path of the database file the inspection was taken from.
    pub(super) path: PathBuf,
    /// Version recorded in the database, or `None` when never recorded.
    pub(super) stored_version: Option<u64>,
    /// Version this binary expects.
    pub(super) current_version: u64,
    /// Whether any user-data table (or unknown table) holds rows.
    pub(super) has_data: bool,
}

impl SchemaInspection {
    /// A database requires migration when its stored version differs from the
    /// current one **and** it actually holds data.
    ///
    /// An empty database (brand-new, or one whose tables are all empty) is
    /// always safe to initialize.
    pub(super) fn requires_migration(&self) -> bool {
        self.stored_version != Some(self.current_version) && self.has_data
    }

    /// Build the typed error describing why an operator must intervene.
    fn migration_error(&self, detail: String) -> Error {
        Error::SchemaMigrationRequired {
            path: self.path.display().to_string(),
            stored_version: self.stored_version,
            current_version: self.current_version,
            detail,
        }
    }
}

impl RedbStorage {
    /// Initialize database tables with schema version check
    ///
    /// This method:
    /// 1. Inspects the stored schema version without writing anything
    /// 2. If the database holds data written by a different schema version,
    ///    fails closed with [`Error::SchemaMigrationRequired`] and leaves the
    ///    database untouched (no clearing, no version rewrite)
    /// 3. Otherwise opens/creates the tables and records the current version
    pub(crate) async fn initialize_tables(&self) -> Result<()> {
        let inspection = self.inspect_schema().await?;

        if inspection.requires_migration() {
            warn!(
                path = %inspection.path.display(),
                stored_version = ?inspection.stored_version,
                current_version = inspection.current_version,
                "schema version mismatch on a non-empty database; refusing to clear data"
            );
            return Err(inspection.migration_error(
                "database holds data written by a different schema version; \
                 run migrate_schema() to validate, or reset_all_tables() to erase explicitly"
                    .to_string(),
            ));
        }

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
                let _episode_revisions =
                    write_txn.open_table(EPISODE_REVISIONS_TABLE).map_err(|e| {
                        Error::Storage(format!("Failed to open episode revisions table: {}", e))
                    })?;
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
                let _ep_pt_relationships = write_txn
                    .open_table(EPISODE_PATTERN_RELATIONSHIPS_TABLE)
                    .map_err(|e| {
                        Error::Storage(format!(
                            "Failed to open episode pattern relationships table: {}",
                            e
                        ))
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
                let _procedural = write_txn.open_table(PROCEDURAL_TABLE).map_err(|e| {
                    Error::Storage(format!("Failed to open procedural table: {}", e))
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

        // A brand-new (or previously unversioned but empty) database gets the
        // current version stamped. Version bumps on a *non-empty* database were
        // rejected above and never reach this point.
        if inspection.stored_version != Some(SCHEMA_VERSION) {
            self.store_schema_version().await?;
        }

        // The episode index stores no ordering key, so rows left by the pre-#1066
        // last-write-wins path are reconciled against the session rows on every open.
        let repaired = self.repair_recommendation_index().await?;
        if repaired > 0 {
            info!("Reconciled {repaired} recommendation episode index entries");
        }

        info!("Initialized redb tables");
        Ok(())
    }

    /// Inspect the stored schema state without mutating the database.
    ///
    /// Runs a single read transaction: no table is created, no row written and
    /// no version rewritten.
    pub(super) async fn inspect_schema(&self) -> Result<SchemaInspection> {
        let db = Arc::clone(&self.db);
        let path = self.path.clone();
        let current_version = SCHEMA_VERSION;

        with_db_timeout(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;

            let stored_version = match read_txn.open_table(SCHEMA_VERSION_TABLE) {
                Ok(version_table) => version_table
                    .get("version")
                    .map_err(|e| Error::Storage(format!("Failed to read schema version: {}", e)))?
                    .map(|guard| guard.value()),
                Err(TableError::TableDoesNotExist(_)) => None,
                Err(e) => {
                    return Err(Error::Storage(format!(
                        "Failed to open schema version table: {}",
                        e
                    )));
                }
            };

            let mut has_data = false;

            // Any row in a known user-data table counts as data.
            for name in DATA_TABLE_NAMES {
                let def = TableDefinition::<&str, &[u8]>::new(name);
                match read_txn.open_table(def) {
                    Ok(table) => {
                        let len = table.len().map_err(|e| {
                            Error::Storage(format!("Failed to count {} rows: {}", name, e))
                        })?;
                        if len > 0 {
                            has_data = true;
                            break;
                        }
                    }
                    Err(TableError::TableDoesNotExist(_)) => {}
                    Err(e) => {
                        return Err(Error::Storage(format!(
                            "Failed to open {} table: {}",
                            name, e
                        )));
                    }
                }
            }

            // The recommendation episode index is the only `(&str, &str)`
            // table, so it is probed with its own type.
            if !has_data {
                match read_txn.open_table(RECOMMENDATION_EPISODE_INDEX_TABLE) {
                    Ok(table) => {
                        let len = table.len().map_err(|e| {
                            Error::Storage(format!(
                                "Failed to count recommendation_episode_index rows: {}",
                                e
                            ))
                        })?;
                        has_data = len > 0;
                    }
                    Err(TableError::TableDoesNotExist(_)) => {}
                    Err(e) => {
                        return Err(Error::Storage(format!(
                            "Failed to open recommendation_episode_index table: {}",
                            e
                        )));
                    }
                }
            }

            // Tables with non-byte values are probed with their own type.
            if !has_data {
                match read_txn.open_table(EPISODE_REVISIONS_TABLE) {
                    Ok(table) => {
                        let len = table.len().map_err(|e| {
                            Error::Storage(format!("Failed to count episode_revisions rows: {}", e))
                        })?;
                        has_data = len > 0;
                    }
                    Err(TableError::TableDoesNotExist(_)) => {}
                    Err(e) => {
                        return Err(Error::Storage(format!(
                            "Failed to open episode_revisions table: {}",
                            e
                        )));
                    }
                }
            }

            // Tables this crate does not know about are treated conservatively
            // as data so an unrecognised database is never re-initialized.
            if !has_data {
                let tables = read_txn
                    .list_tables()
                    .map_err(|e| Error::Storage(format!("Failed to list tables: {}", e)))?;
                for handle in tables {
                    if !KNOWN_TABLE_NAMES.contains(&handle.name()) {
                        has_data = true;
                        break;
                    }
                }
            }

            Ok(SchemaInspection {
                path,
                stored_version,
                current_version,
                has_data,
            })
        })
        .await
    }

    /// Schema version recorded in the database, if any.
    ///
    /// Read-only: never creates tables or rewrites the version.
    pub async fn stored_schema_version(&self) -> Result<Option<u64>> {
        Ok(self.inspect_schema().await?.stored_version)
    }

    /// Whether opening this database must fail closed until it is migrated or
    /// explicitly reset.
    ///
    /// `true` only when the stored version differs from the current version
    /// **and** the database holds data.
    pub async fn requires_schema_migration(&self) -> Result<bool> {
        Ok(self.inspect_schema().await?.requires_migration())
    }

    /// Path of the underlying database file.
    #[must_use]
    pub fn path(&self) -> &std::path::Path {
        &self.path
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
