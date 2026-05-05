//! Multi-dimensional embedding storage operations
//!
//! This module provides dimension-aware embedding storage that routes embeddings
//! to dimension-specific tables for optimal vector search performance.

use crate::{Result, TursoStorage};
use do_memory_core::Error;
use tracing::{debug, info};

/// Statistics for a dimension-specific embedding table
#[derive(Debug, Clone)]
pub struct DimensionStats {
    /// The dimension of the table (384, 1024, 1536, 3072, or 0 for "other")
    pub dimension: usize,
    /// Number of embeddings in this table
    pub count: usize,
    /// Table name
    pub table_name: String,
}

impl TursoStorage {
    /// Get the table name for a given embedding dimension
    fn get_embedding_table_name(dimension: usize) -> &'static str {
        match dimension {
            384 => "embeddings_384",
            1024 => "embeddings_1024",
            1536 => "embeddings_1536",
            3072 => "embeddings_3072",
            _ => "embeddings_other",
        }
    }

    /// Store an embedding in the dimension-specific table
    ///
    /// Routes to the appropriate table based on embedding dimension:
    /// - 384 -> embeddings_384
    /// - 1024 -> embeddings_1024
    /// - 1536 -> embeddings_1536
    /// - 3072 -> embeddings_3072
    /// - other -> embeddings_other
    pub async fn store_embedding_dimension_aware(
        &self,
        item_id: &str,
        item_type: &str,
        embedding: &[f32],
    ) -> Result<()> {
        let dimension = embedding.len();
        let table_name = Self::get_embedding_table_name(dimension);
        debug!(
            "Storing embedding in {}: item_id={}, item_type={}, dimension={}",
            table_name, item_id, item_type, dimension
        );

        let (conn, _conn_id) = self.get_connection_with_id().await?;

        // Create JSON for vector32() conversion
        let embedding_json = serde_json::to_string(embedding).map_err(Error::Serialization)?;

        // Generate embedding ID
        let embedding_id = self.generate_embedding_id(item_id, item_type);

        // Use dimension-specific SQL
        let sql = format!(
            r#"
            INSERT OR REPLACE INTO {} (embedding_id, item_id, item_type, embedding_data, embedding_vector, dimension, model)
            VALUES (?, ?, ?, ?, vector32(?), ?, ?)
            "#,
            table_name
        );

        let stmt = self
            .prepared_cache
            .get_or_prepare(&conn, &sql)
            .await
            .map_err(|e| Error::Storage(format!("Failed to prepare statement: {}", e)))?;

        stmt.execute(libsql::params![
            embedding_id,
            item_id.to_string(),
            item_type.to_string(),
            embedding_json.clone(),
            embedding_json, // JSON array for vector32()
            dimension as i64,
            "default"
        ])
        .await
        .map_err(|e| Error::Storage(format!("Failed to store embedding: {}", e)))?;

        info!(
            "Successfully stored embedding in {}: {}",
            table_name, item_id
        );
        Ok(())
    }

    /// Get an embedding from dimension-specific tables
    ///
    /// Searches all dimension tables for the embedding.
    pub async fn get_embedding_dimension_aware(
        &self,
        item_id: &str,
        item_type: &str,
    ) -> Result<Option<Vec<f32>>> {
        debug!(
            "Retrieving embedding: item_id={}, item_type={}",
            item_id, item_type
        );

        // Try each dimension table in order of common sizes
        for table_name in [
            "embeddings_1536",
            "embeddings_384",
            "embeddings_1024",
            "embeddings_3072",
            "embeddings_other",
        ] {
            if let Some(embedding) = self
                .get_embedding_from_table(table_name, item_id, item_type)
                .await?
            {
                return Ok(Some(embedding));
            }
        }

        Ok(None)
    }

    /// Get embedding from a specific dimension table
    async fn get_embedding_from_table(
        &self,
        table_name: &str,
        item_id: &str,
        item_type: &str,
    ) -> Result<Option<Vec<f32>>> {
        let (conn, _conn_id) = self.get_connection_with_id().await?;

        let sql = format!(
            "SELECT embedding_data FROM {} WHERE item_id = ? AND item_type = ?",
            table_name
        );

        let stmt = self
            .prepared_cache
            .get_or_prepare(&conn, &sql)
            .await
            .map_err(|e| Error::Storage(format!("Failed to prepare statement: {}", e)))?;

        let mut rows = stmt
            .query(libsql::params![item_id.to_string(), item_type.to_string()])
            .await
            .map_err(|e| Error::Storage(format!("Failed to query embedding: {}", e)))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch embedding row: {}", e)))?
        {
            let embedding_data: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;

            // Parse JSON array
            let embedding: Vec<f32> = serde_json::from_str(&embedding_data)
                .map_err(|e| Error::Storage(format!("Failed to parse embedding: {}", e)))?;

            return Ok(Some(embedding));
        }

        Ok(None)
    }

    /// Delete an embedding from dimension-specific tables
    ///
    /// Deletes from all dimension tables (in case dimension changed).
    pub(crate) async fn delete_embedding_dimension_aware(&self, item_id: &str) -> Result<bool> {
        let (conn, _conn_id) = self.get_connection_with_id().await?;

        let mut deleted = false;

        for table_name in [
            "embeddings_384",
            "embeddings_1024",
            "embeddings_1536",
            "embeddings_3072",
            "embeddings_other",
        ] {
            let sql = format!("DELETE FROM {} WHERE item_id = ?", table_name);

            let stmt = self
                .prepared_cache
                .get_or_prepare(&conn, &sql)
                .await
                .map_err(|e| Error::Storage(format!("Failed to prepare statement: {}", e)))?;

            let rows_affected = stmt
                .execute(libsql::params![item_id.to_string()])
                .await
                .map_err(|e| Error::Storage(format!("Failed to delete embedding: {}", e)))?;

            if rows_affected > 0 {
                deleted = true;
                info!("Deleted embedding from {}: {}", table_name, item_id);
            }
        }

        Ok(deleted)
    }

    /// Delete embeddings in batch from dimension-specific tables
    pub(crate) async fn delete_embeddings_batch_dimension_aware(
        &self,
        item_ids: &[String],
    ) -> Result<usize> {
        if item_ids.is_empty() {
            return Ok(0);
        }

        let (conn, _conn_id) = self.get_connection_with_id().await?;
        let mut total_deleted = 0;

        for table_name in [
            "embeddings_384",
            "embeddings_1024",
            "embeddings_1536",
            "embeddings_3072",
            "embeddings_other",
        ] {
            // Build placeholders for IN clause
            let placeholders = item_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let sql = format!(
                "DELETE FROM {} WHERE item_id IN ({})",
                table_name, placeholders
            );

            // Build params
            let params: Vec<libsql::Value> = item_ids.iter().map(|id| id.clone().into()).collect();

            let rows_affected = conn
                .execute(&sql, libsql::params_from_iter(params))
                .await
                .map_err(|e| {
                    Error::Storage(format!(
                        "Failed to delete embeddings batch from {}: {}",
                        table_name, e
                    ))
                })?;

            if rows_affected > 0 {
                total_deleted += rows_affected as usize;
                info!("Deleted {} embeddings from {}", rows_affected, table_name);
            }
        }

        Ok(total_deleted)
    }

    /// Get statistics for all dimension tables
    pub async fn get_dimension_stats(&self) -> Result<Vec<DimensionStats>> {
        let (conn, _conn_id) = self.get_connection_with_id().await?;

        let mut stats = Vec::new();

        for (table_name, dimension) in [
            ("embeddings_384", 384),
            ("embeddings_1024", 1024),
            ("embeddings_1536", 1536),
            ("embeddings_3072", 3072),
            ("embeddings_other", 0),
        ] {
            let sql = format!("SELECT COUNT(*) FROM {}", table_name);
            let mut rows = conn
                .query(&sql, ())
                .await
                .map_err(|e| Error::Storage(format!("Failed to count embeddings: {}", e)))?;

            if let Some(row) = rows
                .next()
                .await
                .map_err(|e| Error::Storage(format!("Failed to fetch count row: {}", e)))?
            {
                let count: i64 = row
                    .get(0)
                    .map_err(|e| Error::Storage(format!("Failed to parse count: {}", e)))?;

                stats.push(DimensionStats {
                    dimension,
                    count: count as usize,
                    table_name: table_name.to_string(),
                });
            }
        }

        Ok(stats)
    }

    /// Store embeddings in batch using dimension-aware storage
    pub async fn store_embeddings_batch_dimension_aware(
        &self,
        embeddings: Vec<(String, Vec<f32>)>,
    ) -> Result<()> {
        debug!(
            "Storing embedding batch with dimension-aware storage: {} items",
            embeddings.len()
        );

        for (item_id, embedding) in embeddings {
            self.store_embedding_dimension_aware(&item_id, "embedding", &embedding)
                .await?;
        }

        info!("Successfully stored embedding batch with dimension-aware storage");
        Ok(())
    }

    /// Get embeddings in batch using dimension-aware storage
    pub async fn get_embeddings_batch_dimension_aware(
        &self,
        item_ids: &[String],
    ) -> Result<Vec<Option<Vec<f32>>>> {
        debug!(
            "Getting embedding batch with dimension-aware storage: {} items",
            item_ids.len()
        );

        let mut results = Vec::with_capacity(item_ids.len());

        for item_id in item_ids {
            let embedding = self
                .get_embedding_dimension_aware(item_id, "embedding")
                .await?;
            results.push(embedding);
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_name_mapping() {
        assert_eq!(
            TursoStorage::get_embedding_table_name(384),
            "embeddings_384"
        );
        assert_eq!(
            TursoStorage::get_embedding_table_name(1024),
            "embeddings_1024"
        );
        assert_eq!(
            TursoStorage::get_embedding_table_name(1536),
            "embeddings_1536"
        );
        assert_eq!(
            TursoStorage::get_embedding_table_name(3072),
            "embeddings_3072"
        );
        assert_eq!(
            TursoStorage::get_embedding_table_name(768),
            "embeddings_other"
        );
        assert_eq!(
            TursoStorage::get_embedding_table_name(512),
            "embeddings_other"
        );
    }
}
