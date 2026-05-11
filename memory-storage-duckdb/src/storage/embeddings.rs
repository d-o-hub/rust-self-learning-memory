use crate::DuckDbStorage;
use do_memory_core::{Error, Result};
use duckdb::params;
use std::sync::Arc;

impl DuckDbStorage {
    pub(crate) async fn store_embedding_internal(
        &self,
        item_id: &str,
        item_type: &str,
        embedding: &[f32],
    ) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        let item_id = item_id.to_string();
        let item_type = item_type.to_string();
        let embedding = embedding.to_vec();
        let dimension = i32::try_from(embedding.len()).map_err(|e| {
            Error::Storage(format!("Embedding dimension overflow for {item_id}: {e}"))
        })?;

        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let embedding_json = serde_json::to_string(&embedding)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;

            conn.execute(
                "INSERT OR REPLACE INTO embeddings (
                    embedding_id, item_id, item_type, embedding_data, embedding_vector, dimension, model
                ) VALUES (?, ?, ?, ?, CAST(? AS FLOAT[]), ?, ?)",
                params![
                    format!("{}:{}", item_type, item_id),
                    item_id,
                    item_type,
                    "{}", // embedding_data placeholder
                    embedding_json,
                    dimension,
                    "default", // model placeholder
                ],
            )
            .map_err(|e| Error::Storage(format!("Failed to store embedding: {e}")))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(())
    }

    #[allow(dead_code)]
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
                let query_embedding_json = serde_json::to_string(&query_embedding)
                    .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;

                let mut stmt = conn
                    .prepare(
                        "SELECT item_id, list_cosine_similarity(embedding_vector, CAST(? AS FLOAT[])) as score
                         FROM embeddings
                         ORDER BY score DESC
                         LIMIT ?",
                    )
                    .map_err(|e| Error::Storage(e.to_string()))?;

                let mut rows = stmt
                    .query(params![query_embedding_json, limit])
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

    pub(crate) async fn get_embedding_internal(&self, item_id: &str) -> Result<Option<Vec<f32>>> {
        let conn_arc = Arc::clone(&self.conn);
        let item_id = item_id.to_string();
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare(
                    "SELECT CAST(embedding_vector AS VARCHAR) FROM embeddings WHERE item_id = ?",
                )
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt
                .query(params![item_id])
                .map_err(|e| Error::Storage(e.to_string()))?;

            if let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let vector_json: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                let embedding: Vec<f32> = serde_json::from_str(&vector_json)
                    .map_err(|e| Error::Storage(format!("Failed to deserialize embedding: {e}")))?;
                Ok::<Option<Vec<f32>>, Error>(Some(embedding))
            } else {
                Ok::<Option<Vec<f32>>, Error>(None)
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    pub(crate) async fn delete_embedding_internal(&self, item_id: &str) -> Result<bool> {
        let conn_arc = Arc::clone(&self.conn);
        let item_id = item_id.to_string();
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let rows_affected = conn
                .execute("DELETE FROM embeddings WHERE item_id = ?", params![item_id])
                .map_err(|e| Error::Storage(e.to_string()))?;
            Ok::<bool, Error>(rows_affected > 0)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    pub(crate) async fn store_embeddings_batch_internal(
        &self,
        embeddings: Vec<(String, Vec<f32>)>,
    ) -> Result<()> {
        for (item_id, embedding) in embeddings {
            self.store_embedding_internal(&item_id, "embedding", &embedding)
                .await?;
        }
        Ok(())
    }

    pub(crate) async fn get_embeddings_batch_internal(
        &self,
        ids: &[String],
    ) -> Result<Vec<Option<Vec<f32>>>> {
        let mut results = Vec::with_capacity(ids.len());
        for id in ids {
            results.push(self.get_embedding_internal(id).await?);
        }
        Ok(results)
    }
}
