//! Procedural memory storage operations for redb

use crate::{PROCEDURAL_TABLE, RedbStorage, with_db_timeout};
use do_memory_core::procedural::ProceduralMemory;
use do_memory_core::{Error, Result};
use tracing::debug;
use uuid::Uuid;

impl RedbStorage {
    /// Store a procedural memory
    pub async fn store_procedural(&self, procedural: &ProceduralMemory) -> Result<()> {
        let id = procedural.id.to_string();
        debug!("Caching procedural memory in redb: {}", id);

        // Serialize to bytes using postcard (consistent with other redb implementations)
        let data = postcard::to_allocvec(procedural).map_err(|e| {
            Error::Serialization(serde_json::to_error(e))
        })?;

        let db = self.db.clone();
        with_db_timeout(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {}", e)))?;

            {
                let mut table = write_txn.open_table(PROCEDURAL_TABLE).map_err(|e| {
                    Error::Storage(format!("Failed to open procedural table: {}", e))
                })?;
                table
                    .insert(id.as_str(), data.as_slice())
                    .map_err(|e| Error::Storage(format!("Failed to insert procedural: {}", e)))?;
            }

            write_txn
                .commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;
            Ok(())
        })
        .await
    }

    /// Retrieve a procedural memory by ID
    pub async fn get_procedural(&self, id: Uuid) -> Result<Option<ProceduralMemory>> {
        let id_str = id.to_string();
        debug!("Retrieving procedural memory from redb: {}", id_str);

        let db = self.db.clone();
        with_db_timeout(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;

            let table = read_txn.open_table(PROCEDURAL_TABLE).map_err(|e| {
                Error::Storage(format!("Failed to open procedural table: {}", e))
            })?;

            let result = table
                .get(id_str.as_str())
                .map_err(|e| Error::Storage(format!("Failed to get procedural: {}", e)))?;

            if let Some(data) = result {
                let procedural: ProceduralMemory = postcard::from_bytes(data.value()).map_err(|e| {
                    Error::Serialization(serde_json::to_error(e))
                })?;
                Ok(Some(procedural))
            } else {
                Ok(None)
            }
        })
        .await
    }

    /// Delete a procedural memory by ID
    pub async fn delete_procedural(&self, id: Uuid) -> Result<()> {
        let id_str = id.to_string();
        debug!("Deleting procedural memory from redb: {}", id_str);

        let db = self.db.clone();
        with_db_timeout(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {}", e)))?;

            {
                let mut table = write_txn.open_table(PROCEDURAL_TABLE).map_err(|e| {
                    Error::Storage(format!("Failed to open procedural table: {}", e))
                })?;
                table
                    .remove(id_str.as_str())
                    .map_err(|e| Error::Storage(format!("Failed to remove procedural: {}", e)))?;
            }

            write_txn
                .commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;
            Ok(())
        })
        .await
    }

    /// Query all procedural memories
    pub async fn query_procedural(&self, limit: Option<usize>) -> Result<Vec<ProceduralMemory>> {
        debug!("Querying procedural memories from redb");
        let limit = do_memory_core::apply_query_limit(limit);

        let db = self.db.clone();
        with_db_timeout(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;

            let table = read_txn.open_table(PROCEDURAL_TABLE).map_err(|e| {
                Error::Storage(format!("Failed to open procedural table: {}", e))
            })?;

            let mut results = Vec::new();
            for item in table
                .iter()
                .map_err(|e| Error::Storage(format!("Failed to iterate procedural table: {}", e)))?
            {
                if results.len() >= limit {
                    break;
                }

                let (_key, value) = item.map_err(|e| {
                    Error::Storage(format!("Failed to read procedural table item: {}", e))
                })?;

                let procedural: ProceduralMemory =
                    postcard::from_bytes(value.value()).map_err(|e| {
                        Error::Serialization(serde_json::to_error(e))
                    })?;
                results.push(procedural);
            }

            // Sort by updated_at descending
            results.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

            Ok(results)
        })
        .await
    }
}
