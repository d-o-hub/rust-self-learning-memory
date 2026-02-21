//! Embedding storage operations for redb cache

use crate::{EMBEDDINGS_TABLE, RedbStorage};
use memory_core::{Error, Result};
use std::sync::Arc;
use tracing::debug;

impl RedbStorage {
    /// Store an embedding vector (internal method)
    pub async fn store_embedding_raw(&self, id: &str, embedding: &[f32]) -> Result<()> {
        debug!("Storing embedding: {}", id);
        let db = Arc::clone(&self.db);
        let id_str = id.to_string();
        let embedding_bytes = postcard::to_allocvec(embedding)
            .map_err(|e| Error::Storage(format!("Failed to serialize embedding: {}", e)))?;

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

        Ok(())
    }

    /// Retrieve an embedding vector (internal method)
    pub async fn get_embedding_raw(&self, id: &str) -> Result<Option<Vec<f32>>> {
        debug!("Retrieving embedding: {}", id);
        let db = Arc::clone(&self.db);
        let id_str = id.to_string();

        tokio::task::spawn_blocking(move || {
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
                    let embedding: Vec<f32> =
                        postcard::from_bytes(bytes_guard.value()).map_err(|e| {
                            Error::Storage(format!("Failed to deserialize embedding: {}", e))
                        })?;
                    Ok(Some(embedding))
                }
                None => Ok(None),
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }
}
