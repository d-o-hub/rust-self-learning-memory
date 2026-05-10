use crate::DuckDbStorage;
use do_memory_core::{Error, Result};
use duckdb::params;
use std::sync::Arc;

impl DuckDbStorage {
    pub(crate) async fn store_embedding_internal(
        &self,
        id: &str,
        embedding: Vec<f32>,
    ) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        let id = id.to_string();
        let dimension = i32::try_from(embedding.len()).unwrap_or(0);
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let vector_json = serde_json::to_string(&embedding)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;

            conn.execute(
                "INSERT OR REPLACE INTO embeddings (
                    embedding_id, item_id, item_type, embedding_data, embedding_vector, dimension, model
                ) VALUES (?, ?, ?, ?, ?::FLOAT[], ?, ?)",
                params![
                    id,
                    id, // Using id as item_id for now as StorageBackend only provides id
                    "generic",
                    "{}", // embedding_data placeholder
                    vector_json,
                    dimension,
                    "default"
                ],
            )
            .map_err(|e| Error::Storage(format!("Failed to store embedding: {e}")))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(())
    }

    pub(crate) async fn get_embedding_internal(&self, id: &str) -> Result<Option<Vec<f32>>> {
        let conn_arc = Arc::clone(&self.conn);
        let id = id.to_string();
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare("SELECT CAST(embedding_vector AS VARCHAR) FROM embeddings WHERE embedding_id = ?")
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt
                .query(params![id])
                .map_err(|e| Error::Storage(e.to_string()))?;

            if let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let vector_json: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                let vector: Vec<f32> = serde_json::from_str(&vector_json)
                    .map_err(|e| Error::Storage(e.to_string()))?;
                Ok::<Option<Vec<f32>>, Error>(Some(vector))
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
            let affected = conn
                .execute("DELETE FROM embeddings WHERE embedding_id = ?", params![id])
                .map_err(|e| Error::Storage(format!("Failed to delete embedding: {e}")))?;
            Ok::<bool, Error>(affected > 0)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    pub(crate) async fn store_embeddings_batch_internal(
        &self,
        embeddings: Vec<(String, Vec<f32>)>,
    ) -> Result<()> {
        if embeddings.is_empty() {
            return Ok(());
        }
        let conn_arc = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let mut conn = conn_arc.lock();
            let tx = conn
                .transaction()
                .map_err(|e| Error::Storage(format!("Failed to begin transaction: {e}")))?;
            {
                let mut stmt = tx.prepare(
                    "INSERT OR REPLACE INTO embeddings (
                        embedding_id, item_id, item_type, embedding_data, embedding_vector, dimension, model
                    ) VALUES (?, ?, ?, ?, ?::FLOAT[], ?, ?)",
                ).map_err(|e| Error::Storage(e.to_string()))?;

                for (id, embedding) in embeddings {
                    let dimension = i32::try_from(embedding.len()).unwrap_or(0);
                    let vector_json = serde_json::to_string(&embedding)
                        .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;
                    stmt.execute(params![
                        id,
                        id,
                        "generic",
                        "{}",
                        vector_json,
                        dimension,
                        "default"
                    ])
                    .map_err(|e| Error::Storage(format!("Failed to store embedding: {e}")))?;
                }
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
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let conn_arc = Arc::clone(&self.conn);
        let ids = ids.to_vec();
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let placeholders = vec!["?"; ids.len()].join(",");
            let query = format!(
                "SELECT embedding_id, CAST(embedding_vector AS VARCHAR) FROM embeddings WHERE embedding_id IN ({placeholders})"
            );
            let mut stmt = conn
                .prepare(&query)
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt
                .query(duckdb::params_from_iter(ids.iter()))
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut results_map = std::collections::HashMap::new();
            while let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let id: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                let vector_json: String = row.get(1).map_err(|e| Error::Storage(e.to_string()))?;
                let vector: Vec<f32> = serde_json::from_str(&vector_json)
                    .map_err(|e| Error::Storage(e.to_string()))?;
                results_map.insert(id, vector);
            }

            let ordered_results: Vec<Option<Vec<f32>>> = ids
                .into_iter()
                .map(|id| results_map.remove(&id))
                .collect();
            Ok::<Vec<Option<Vec<f32>>>, Error>(ordered_results)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    // ========== Relationship Storage Methods ==========
    /// * `vector` - The query embedding vector.
    /// * `limit` - The maximum number of results to return.
    ///
    /// # Errors
    ///
    /// Returns an error if the search fails or the database query fails.
    pub async fn search_embeddings_vss(
        &self,
        vector: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<serde_json::Value>> {
        let conn_arc = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare(
                    "SELECT item_id, list_cosine_similarity(embedding_vector, ?::FLOAT[]) AS score
                FROM embeddings
                ORDER BY score DESC
                LIMIT ?",
                )
                .map_err(|e| Error::Storage(e.to_string()))?;

            let vector_json = serde_json::to_string(&vector).unwrap_or_default();
            let rows = stmt
                .query_map(
                    params![vector_json, i64::try_from(limit).unwrap_or(10)],
                    |row| {
                        let item_id: String = row.get(0)?;
                        let score: f64 = row.get(1)?;
                        let val = serde_json::json!({
                            "item_id": item_id,
                            "score": score,
                        });
                        Ok(val)
                    },
                )
                .map_err(|e| Error::Storage(e.to_string()))?;

            let results: std::result::Result<Vec<serde_json::Value>, duckdb::Error> =
                rows.collect();
            results.map_err(|e| Error::Storage(e.to_string()))
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))?
    }
}
