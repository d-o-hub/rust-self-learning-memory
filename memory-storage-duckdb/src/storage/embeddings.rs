use crate::DuckDbStorage;
use do_memory_core::{Error, Result};
use duckdb::params;
use std::sync::Arc;

impl DuckDbStorage {
    pub(crate) async fn store_embedding_internal(
        &self,
        id: &str,
        embedding: &[f32],
    ) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        let id = id.to_string();
        let embedding = embedding.to_vec();
        let dimension = i32::try_from(embedding.len()).map_err(|e| {
            Error::Storage(format!("Embedding dimension overflow for {id}: {e}"))
        })?;

        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();

            conn.execute(
                "INSERT OR REPLACE INTO embeddings (
                    embedding_id, item_id, item_type, embedding_data, embedding_vector, dimension, model
                ) VALUES (?, ?, ?, ?, ?, ?, ?)",
                params![
                    id,
                    id,
                    "default",
                    "{}",
                    embedding,
                    dimension,
                    "default",
                ],
            )
            .map_err(|e| Error::Storage(format!("Failed to store embedding: {e}")))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(())
    }

    pub(crate) async fn search_embeddings_internal(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<(String, f32)>> {
        #[cfg(feature = "vss")]
        {
            let conn_arc = Arc::clone(&self.conn);
            let query_embedding = query_embedding.to_vec();
            let limit = i64::try_from(limit).unwrap_or(100);

            let res = tokio::task::spawn_blocking(move || {
                let conn = conn_arc.lock();
                let mut stmt = conn
                    .prepare(
                        "SELECT item_id, list_cosine_similarity(embedding_vector, ?::FLOAT[]) as score
                         FROM embeddings
                         ORDER BY score DESC
                         LIMIT ?",
                    )
                    .map_err(|e| Error::Storage(e.to_string()))?;

                let mut rows = stmt
                    .query(params![query_embedding, limit])
                    .map_err(|e| Error::Storage(e.to_string()))?;

                let mut results = Vec::new();
                while let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                    let id: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                    let score: f32 = row.get(1).map_err(|e| Error::Storage(e.to_string()))?;
                    results.push((id, score));
                }
                Ok::<Vec<(String, f32)>, Error>(results)
            })
            .await
            .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
            Ok(res)
        }

        #[cfg(not(feature = "vss"))]
        {
            let _ = query_embedding;
            let _ = limit;
            Err(Error::Storage("VSS feature not enabled".to_string()))
        }
    }

    pub(crate) async fn get_embeddings_internal(&self, item_id: &str) -> Result<Option<Vec<f32>>> {
        let conn_arc = Arc::clone(&self.conn);
        let item_id = item_id.to_string();
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare("SELECT embedding_vector FROM embeddings WHERE item_id = ?")
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt
                .query(params![item_id])
                .map_err(|e| Error::Storage(e.to_string()))?;

            if let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let embedding: Vec<f32> = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                Ok::<Option<Vec<f32>>, Error>(Some(embedding))
            } else {
                Ok::<Option<Vec<f32>>, Error>(None)
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    pub(crate) async fn delete_embedding_internal(&self, id: &str) -> Result<bool> {
        let conn_arc = Arc::clone(&self.conn);
        let id = id.to_string();
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let rows_changed = conn
                .execute("DELETE FROM embeddings WHERE item_id = ?", params![id])
                .map_err(|e| Error::Storage(e.to_string()))?;
            Ok::<bool, Error>(rows_changed > 0)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    pub(crate) async fn store_embeddings_batch_internal(
        &self,
        embeddings: Vec<(String, Vec<f32>)>,
    ) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let mut conn = conn_arc.lock();
            let tx = conn
                .transaction()
                .map_err(|e| Error::Storage(format!("Failed to start transaction: {e}")))?;

            for (id, embedding) in embeddings {
                let dimension = i32::try_from(embedding.len()).map_err(|e| {
                    Error::Storage(format!("Embedding dimension overflow for {id}: {e}"))
                })?;

                tx.execute(
                    "INSERT OR REPLACE INTO embeddings (
                        embedding_id, item_id, item_type, embedding_data, embedding_vector, dimension, model
                    ) VALUES (?, ?, ?, ?, ?, ?, ?)",
                    params![
                        id,
                        id,
                        "default",
                        "{}",
                        embedding,
                        dimension,
                        "default",
                    ],
                )
                .map_err(|e| Error::Storage(format!("Failed to store embedding in batch: {e}")))?;
            }

            tx.commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {e}")))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(())
    }

    pub(crate) async fn get_embeddings_batch_internal(
        &self,
        ids: &[String],
    ) -> Result<Vec<Option<Vec<f32>>>> {
        let conn_arc = Arc::clone(&self.conn);
        let ids = ids.to_vec();
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut results = Vec::with_capacity(ids.len());
            let mut stmt = conn
                .prepare("SELECT embedding_vector FROM embeddings WHERE item_id = ?")
                .map_err(|e| Error::Storage(e.to_string()))?;

            for id in ids {
                let mut rows = stmt
                    .query(params![id])
                    .map_err(|e| Error::Storage(e.to_string()))?;

                if let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                    let embedding: Vec<f32> = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                    results.push(Some(embedding));
                } else {
                    results.push(None);
                }
            }
            Ok::<Vec<Option<Vec<f32>>>, Error>(results)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }
}
