//! Embedding storage backend helper implementation for redb cache

use crate::{EMBEDDINGS_TABLE, RedbStorage};
use memory_core::{Error, Result};
use redb::ReadableTable;
use std::sync::Arc;
use tracing::{debug, info};

impl RedbStorage {
    /// Store embedding implementation
    pub async fn store_embedding_impl(&self, id: &str, embedding: Vec<f32>) -> Result<()> {
        debug!("Storing embedding via StorageBackend: {}", id);

        // Validate embedding size
        let embedding_bytes = postcard::to_allocvec(&embedding)
            .map_err(|e| Error::Storage(format!("Failed to serialize embedding: {}", e)))?;

        if embedding_bytes.len() as u64 > crate::MAX_EMBEDDING_SIZE {
            return Err(Error::Storage(format!(
                "Embedding size {} exceeds maximum of {}",
                embedding_bytes.len(),
                crate::MAX_EMBEDDING_SIZE
            )));
        }

        let db = Arc::clone(&self.db);
        let id_str = id.to_string();

        tokio::task::spawn_blocking(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {}", e)))?;

            {
                let mut table = write_txn.open_table(EMBEDDINGS_TABLE).map_err(|e| {
                    Error::Storage(format!("Failed to open embeddings table: {}", e))
                })?;

                table
                    .insert(id_str.as_str(), embedding_bytes.as_slice())
                    .map_err(|e| Error::Storage(format!("Failed to insert embedding: {}", e)))?;
            }

            write_txn
                .commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;

            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;

        info!("Successfully stored embedding: {}", id);
        Ok(())
    }

    /// Retrieve embedding implementation
    pub async fn get_embedding_impl(&self, id: &str) -> Result<Option<Vec<f32>>> {
        debug!("Retrieving embedding via StorageBackend: {}", id);

        let db = Arc::clone(&self.db);
        let id_str = id.to_string();

        let result = tokio::task::spawn_blocking(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;

            let table = read_txn
                .open_table(EMBEDDINGS_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open embeddings table: {}", e)))?;

            match table
                .get(id_str.as_str())
                .map_err(|e| Error::Storage(format!("Failed to get embedding: {}", e)))?
            {
                Some(bytes_guard) => {
                    let _bytes = bytes_guard.value();

                    // Validate size before deserializing
                    if _bytes.len() as u64 > crate::MAX_EMBEDDING_SIZE {
                        return Err(Error::Storage(format!(
                            "Embedding size {} exceeds maximum of {}",
                            _bytes.len(),
                            crate::MAX_EMBEDDING_SIZE
                        )));
                    }

                    let embedding: Vec<f32> =
                        postcard::from_bytes(bytes_guard.value()).map_err(|e| {
                            Error::Storage(format!("Failed to deserialize embedding: {}", e))
                        })?;
                    Ok::<Option<Vec<f32>>, Error>(Some(embedding))
                }
                None => Ok::<Option<Vec<f32>>, Error>(None),
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;

        Ok(result)
    }

    /// Delete embedding implementation
    pub async fn delete_embedding_impl(&self, id: &str) -> Result<bool> {
        debug!("Deleting embedding via StorageBackend: {}", id);

        let db = Arc::clone(&self.db);
        let id_str = id.to_string();

        let result = tokio::task::spawn_blocking(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {}", e)))?;

            let existed = {
                let mut table = write_txn.open_table(EMBEDDINGS_TABLE).map_err(|e| {
                    Error::Storage(format!("Failed to open embeddings table: {}", e))
                })?;

                let existed = table
                    .get(id_str.as_str())
                    .map_err(|e| Error::Storage(format!("Failed to check embedding: {}", e)))?
                    .is_some();

                if existed {
                    table.remove(id_str.as_str()).map_err(|e| {
                        Error::Storage(format!("Failed to delete embedding: {}", e))
                    })?;
                }

                existed
            };

            write_txn
                .commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;

            Ok::<bool, Error>(existed)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;

        if result {
            info!("Deleted embedding: {}", id);
        } else {
            debug!("Embedding not found for deletion: {}", id);
        }

        Ok(result)
    }

    /// Store multiple embeddings in batch implementation
    pub async fn store_embeddings_batch_impl(
        &self,
        embeddings: Vec<(String, Vec<f32>)>,
    ) -> Result<()> {
        debug!("Storing {} embeddings in batch", embeddings.len());

        if embeddings.is_empty() {
            return Ok(());
        }

        let db = Arc::clone(&self.db);
        let count = embeddings.len();

        tokio::task::spawn_blocking(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {}", e)))?;

            {
                let mut table = write_txn.open_table(EMBEDDINGS_TABLE).map_err(|e| {
                    Error::Storage(format!("Failed to open embeddings table: {}", e))
                })?;

                for (id, embedding) in embeddings {
                    let embedding_bytes = postcard::to_allocvec(&embedding).map_err(|e| {
                        Error::Storage(format!("Failed to serialize embedding: {}", e))
                    })?;

                    // Validate size
                    if embedding_bytes.len() as u64 > crate::MAX_EMBEDDING_SIZE {
                        return Err(Error::Storage(format!(
                            "Embedding size {} exceeds maximum of {}",
                            embedding_bytes.len(),
                            crate::MAX_EMBEDDING_SIZE
                        )));
                    }

                    table
                        .insert(id.as_str(), embedding_bytes.as_slice())
                        .map_err(|e| {
                            Error::Storage(format!("Failed to insert embedding: {}", e))
                        })?;
                }
            }

            write_txn
                .commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;

            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;

        info!("Successfully stored {} embeddings in batch", count);
        Ok(())
    }

    /// Get multiple embeddings in batch implementation
    pub async fn get_embeddings_batch_impl(&self, ids: &[String]) -> Result<Vec<Option<Vec<f32>>>> {
        debug!("Retrieving {} embeddings in batch", ids.len());

        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let db = Arc::clone(&self.db);
        let ids_clone = ids.to_vec();

        let results_map = tokio::task::spawn_blocking(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;

            let table = read_txn
                .open_table(EMBEDDINGS_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open embeddings table: {}", e)))?;

            let mut results_map = std::collections::HashMap::new();

            for id in &ids_clone {
                match table
                    .get(id.as_str())
                    .map_err(|e| Error::Storage(format!("Failed to get embedding: {}", e)))?
                {
                    Some(bytes_guard) => {
                        let _bytes = bytes_guard.value();

                        // Validate size before deserializing
                        if _bytes.len() as u64 <= crate::MAX_EMBEDDING_SIZE {
                            let embedding: Vec<f32> = postcard::from_bytes(bytes_guard.value())
                                .map_err(|e| {
                                    Error::Storage(format!(
                                        "Failed to deserialize embedding: {}",
                                        e
                                    ))
                                })?;
                            results_map.insert(id.clone(), Some(embedding));
                        } else {
                            results_map.insert(id.clone(), None);
                        }
                    }
                    None => {
                        results_map.insert(id.clone(), None);
                    }
                }
            }

            Ok::<std::collections::HashMap<String, Option<Vec<f32>>>, Error>(results_map)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;

        // Map results to maintain original order
        let results: Vec<Option<Vec<f32>>> = ids
            .iter()
            .map(|id| results_map.get(id).and_then(|o| o.clone()))
            .collect();

        info!("Retrieved {} embeddings from batch request", results.len());
        Ok(results)
    }
}
