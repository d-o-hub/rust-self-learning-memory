//! Procedural memory storage operations for redb

use crate::{PROCEDURAL_MEMORIES_TABLE, RedbStorage, with_db_timeout};
use do_memory_core::{Error, ProceduralMemory, Result, TaskType};
use redb::{ReadableDatabase, ReadableTable};
use std::sync::Arc;
use uuid::Uuid;

impl RedbStorage {
    /// Store a procedural memory in redb
    pub async fn store_procedural_memory(&self, procedural: &ProceduralMemory) -> Result<()> {
        let db = Arc::clone(&self.db);
        let id = procedural.id.to_string();

        // Serialize to postcard (system standard)
        let data = postcard::to_allocvec(procedural)
            .map_err(|e| Error::Storage(format!("Failed to serialize procedural memory: {}", e)))?;

        with_db_timeout(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(e.to_string()))?;
            {
                let mut table = write_txn
                    .open_table(PROCEDURAL_MEMORIES_TABLE)
                    .map_err(|e| Error::Storage(e.to_string()))?;
                table
                    .insert(id.as_str(), data.as_slice())
                    .map_err(|e| Error::Storage(e.to_string()))?;
            }
            write_txn
                .commit()
                .map_err(|e| Error::Storage(e.to_string()))?;
            Ok(())
        })
        .await
    }

    /// Retrieve a procedural memory from redb by ID
    pub async fn get_procedural_memory(&self, id: Uuid) -> Result<Option<ProceduralMemory>> {
        let db = Arc::clone(&self.db);
        let id_str = id.to_string();

        with_db_timeout(move || {
            let read_txn = db.begin_read().map_err(|e| Error::Storage(e.to_string()))?;
            let table = read_txn
                .open_table(PROCEDURAL_MEMORIES_TABLE)
                .map_err(|e| Error::Storage(e.to_string()))?;

            let result = table
                .get(id_str.as_str())
                .map_err(|e| Error::Storage(e.to_string()))?;

            match result {
                Some(data) => {
                    let procedural: ProceduralMemory =
                        postcard::from_bytes(data.value()).map_err(|e| {
                            Error::Storage(format!(
                                "Failed to deserialize procedural memory: {}",
                                e
                            ))
                        })?;
                    Ok(Some(procedural))
                }
                None => Ok(None),
            }
        })
        .await
    }

    /// Query procedural memories from redb by task type
    pub async fn query_procedural_memories(
        &self,
        task_type: TaskType,
        limit: Option<usize>,
    ) -> Result<Vec<ProceduralMemory>> {
        let db = Arc::clone(&self.db);
        let limit = do_memory_core::storage::apply_query_limit(limit);

        with_db_timeout(move || {
            let read_txn = db.begin_read().map_err(|e| Error::Storage(e.to_string()))?;
            let table = read_txn
                .open_table(PROCEDURAL_MEMORIES_TABLE)
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut results = Vec::new();
            let iter = table.iter().map_err(|e| Error::Storage(e.to_string()))?;

            for item in iter {
                if results.len() >= limit {
                    break;
                }

                let (_, value) = item.map_err(|e| Error::Storage(e.to_string()))?;
                let procedural: ProceduralMemory =
                    postcard::from_bytes(value.value()).map_err(|e| {
                        Error::Storage(format!("Failed to deserialize procedural memory: {}", e))
                    })?;

                if procedural.task_type == task_type {
                    results.push(procedural);
                }
            }

            Ok(results)
        })
        .await
    }
}
