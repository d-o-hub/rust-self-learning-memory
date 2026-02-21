//! Heuristic storage operations for redb cache

use crate::{Error, HEURISTICS_TABLE, RedbStorage};
use memory_core::{Heuristic, Result};
use redb::ReadableTable;
use std::sync::Arc;
use tracing::debug;
use tracing::info;
use uuid::Uuid;

use crate::episodes::RedbQuery;

impl RedbStorage {
    /// Store a heuristic in cache
    pub async fn store_heuristic(&self, heuristic: &Heuristic) -> Result<()> {
        debug!("Storing heuristic in cache: {}", heuristic.heuristic_id);
        let db = Arc::clone(&self.db);
        let heuristic_id = heuristic.heuristic_id.to_string();
        let heuristic_bytes = postcard::to_allocvec(heuristic)
            .map_err(|e| Error::Storage(format!("Failed to serialize heuristic: {}", e)))?;

        tokio::task::spawn_blocking(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {}", e)))?;

            {
                let mut table = write_txn.open_table(HEURISTICS_TABLE).map_err(|e| {
                    Error::Storage(format!("Failed to open heuristics table: {}", e))
                })?;

                table
                    .insert(heuristic_id.as_str(), heuristic_bytes.as_slice())
                    .map_err(|e| Error::Storage(format!("Failed to insert heuristic: {}", e)))?;
            }

            write_txn
                .commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;

            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;

        info!("Successfully cached heuristic: {}", heuristic.heuristic_id);
        Ok(())
    }

    /// Retrieve a heuristic from cache
    pub async fn get_heuristic(&self, heuristic_id: Uuid) -> Result<Option<Heuristic>> {
        debug!("Retrieving heuristic from cache: {}", heuristic_id);
        let db = Arc::clone(&self.db);
        let heuristic_id_str = heuristic_id.to_string();

        tokio::task::spawn_blocking(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;

            let table = read_txn
                .open_table(HEURISTICS_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open heuristics table: {}", e)))?;

            match table
                .get(heuristic_id_str.as_str())
                .map_err(|e| Error::Storage(format!("Failed to get heuristic: {}", e)))?
            {
                Some(bytes_guard) => {
                    let _bytes = bytes_guard.value();
                    let heuristic: Heuristic =
                        postcard::from_bytes(bytes_guard.value()).map_err(|e| {
                            Error::Storage(format!("Failed to deserialize heuristic: {}", e))
                        })?;
                    Ok(Some(heuristic))
                }
                None => Ok(None),
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }

    /// Get all heuristics from cache (with optional limit)
    pub async fn get_all_heuristics(&self, query: &RedbQuery) -> Result<Vec<Heuristic>> {
        debug!("Retrieving all heuristics from cache");
        let db = Arc::clone(&self.db);
        let limit = query.limit;

        tokio::task::spawn_blocking(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;

            let table = read_txn
                .open_table(HEURISTICS_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open heuristics table: {}", e)))?;

            let mut heuristics = Vec::new();
            let iter = table
                .iter()
                .map_err(|e| Error::Storage(format!("Failed to iterate heuristics: {}", e)))?;

            for (count, result) in iter.enumerate() {
                if let Some(max) = limit {
                    if count >= max {
                        break;
                    }
                }

                let (_, bytes_guard) = result.map_err(|e| {
                    Error::Storage(format!("Failed to read heuristic entry: {}", e))
                })?;

                let heuristic: Heuristic =
                    postcard::from_bytes(bytes_guard.value()).map_err(|e| {
                        Error::Storage(format!("Failed to deserialize heuristic: {}", e))
                    })?;

                heuristics.push(heuristic);
            }

            Ok(heuristics)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }
}
