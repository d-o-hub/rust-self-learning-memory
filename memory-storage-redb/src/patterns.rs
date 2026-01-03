//! Pattern storage operations for redb cache

use crate::{Error, RedbStorage, PATTERNS_TABLE};
use memory_core::{episode::PatternId, Pattern, Result};
use redb::ReadableTable;
use std::sync::Arc;
use tracing::debug;
use tracing::info;

use crate::episodes::RedbQuery;

impl RedbStorage {
    /// Store a pattern in cache
    pub async fn store_pattern(&self, pattern: &Pattern) -> Result<()> {
        debug!("Storing pattern in cache: {}", pattern.id());
        let db = Arc::clone(&self.db);
        let pattern_id = pattern.id().to_string();
        let pattern_bytes = postcard::to_allocvec(pattern)
            .map_err(|e| Error::Storage(format!("Failed to serialize pattern: {}", e)))?;

        tokio::task::spawn_blocking(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {}", e)))?;

            {
                let mut table = write_txn
                    .open_table(PATTERNS_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open patterns table: {}", e)))?;

                table
                    .insert(pattern_id.as_str(), pattern_bytes.as_slice())
                    .map_err(|e| Error::Storage(format!("Failed to insert pattern: {}", e)))?;
            }

            write_txn
                .commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;

            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;

        info!("Successfully cached pattern: {}", pattern.id());
        Ok(())
    }

    /// Retrieve a pattern from cache
    pub async fn get_pattern(&self, pattern_id: PatternId) -> Result<Option<Pattern>> {
        debug!("Retrieving pattern from cache: {}", pattern_id);
        let db = Arc::clone(&self.db);
        let pattern_id_str = pattern_id.to_string();

        tokio::task::spawn_blocking(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;

            let table = read_txn
                .open_table(PATTERNS_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open patterns table: {}", e)))?;

            match table
                .get(pattern_id_str.as_str())
                .map_err(|e| Error::Storage(format!("Failed to get pattern: {}", e)))?
            {
                Some(bytes_guard) => {
                    let _bytes = bytes_guard.value();
                    let pattern: Pattern =
                        postcard::from_bytes(bytes_guard.value()).map_err(|e| {
                            Error::Storage(format!("Failed to deserialize pattern: {}", e))
                        })?;
                    Ok(Some(pattern))
                }
                None => Ok(None),
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }

    /// Get all patterns from cache (with optional limit)
    pub async fn get_all_patterns(&self, query: &RedbQuery) -> Result<Vec<Pattern>> {
        debug!("Retrieving all patterns from cache");
        let db = Arc::clone(&self.db);
        let limit = query.limit;

        tokio::task::spawn_blocking(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;

            let table = read_txn
                .open_table(PATTERNS_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open patterns table: {}", e)))?;

            let mut patterns = Vec::new();
            let iter = table
                .iter()
                .map_err(|e| Error::Storage(format!("Failed to iterate patterns: {}", e)))?;

            for (count, result) in iter.enumerate() {
                if let Some(max) = limit {
                    if count >= max {
                        break;
                    }
                }

                let (_, bytes_guard) = result
                    .map_err(|e| Error::Storage(format!("Failed to read pattern entry: {}", e)))?;

                let pattern: Pattern = postcard::from_bytes(bytes_guard.value())
                    .map_err(|e| Error::Storage(format!("Failed to deserialize pattern: {}", e)))?;

                patterns.push(pattern);
            }

            Ok(patterns)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }
}
