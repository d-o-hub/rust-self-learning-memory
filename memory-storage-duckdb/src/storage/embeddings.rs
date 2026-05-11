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
                ) VALUES (?, ?, ?, ?, ?, ?, ?)",
                params![
                    format!("{}:{}", item_type, item_id),
                    item_id,
                    item_type,
                    "{}", // embedding_data placeholder
                    embedding,
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
}
