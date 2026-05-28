//! Internal embedding storage implementation for Turso

use crate::TursoStorage;
use do_memory_core::Result;
use tracing::debug;

#[cfg(not(feature = "turso_multi_dimension"))]
use tracing::info;

pub(crate) const MAX_SQL_PARAMS: usize = 500;

impl TursoStorage {
    /// Process item_ids in chunks to avoid exceeding SQLITE_MAX_VARIABLE_NUMBER
    pub(crate) fn chunk_item_ids(item_ids: &[String]) -> Vec<&[String]> {
        item_ids.chunks(MAX_SQL_PARAMS).collect()
    }

    /// Build placeholders for an IN clause of the given count
    pub(crate) fn in_clause_placeholders(count: usize) -> String {
        let mut placeholders = String::with_capacity(count * 2);
        for i in 0..count {
            if i > 0 {
                placeholders.push(',');
            }
            placeholders.push('?');
        }
        placeholders
    }

    /// Store an embedding (internal implementation)
    pub(crate) async fn _store_embedding_internal(
        &self,
        item_id: &str,
        item_type: &str,
        embedding: &[f32],
    ) -> Result<()> {
        #[cfg(feature = "turso_multi_dimension")]
        {
            return self
                .store_embedding_dimension_aware(item_id, item_type, embedding)
                .await;
        }

        #[cfg(not(feature = "turso_multi_dimension"))]
        {
            self._store_embedding_single_table(item_id, item_type, embedding)
                .await
        }
    }

    /// Store embedding in single embeddings table
    #[cfg(not(feature = "turso_multi_dimension"))]
    async fn _store_embedding_single_table(
        &self,
        item_id: &str,
        item_type: &str,
        embedding: &[f32],
    ) -> Result<()> {
        debug!(
            "Storing embedding: item_id={}, item_type={}, dimension={}",
            item_id,
            item_type,
            embedding.len()
        );
        let (conn, _conn_id) = self.get_connection_with_id().await?;

        #[cfg(feature = "compression")]
        let compression_threshold = self.config.compression_threshold;
        #[cfg(not(feature = "compression"))]
        let _compression_threshold = 0;

        #[cfg(feature = "compression")]
        let should_compress = self.config.compress_embeddings;
        #[cfg(not(feature = "compression"))]
        let _should_compress = false;

        #[cfg(feature = "compression")]
        let embedding_data: String = if should_compress {
            let bytes: Vec<u8> = embedding.iter().flat_map(|&f| f.to_le_bytes()).collect();
            use crate::compression::CompressedPayload;
            let compression_start = std::time::Instant::now();
            let compressed = match CompressedPayload::compress(&bytes, compression_threshold) {
                Ok(payload) => payload,
                Err(e) => {
                    if let Ok(mut stats) = self.compression_stats.lock() {
                        stats.record_failed();
                    }
                    return Err(e);
                }
            };
            let compression_time_us = compression_start.elapsed().as_micros() as u64;

            if compressed.algorithm == crate::CompressionAlgorithm::None {
                if let Ok(mut stats) = self.compression_stats.lock() {
                    stats.record_skipped();
                }
                serde_json::to_string(embedding).map_err(do_memory_core::Error::Serialization)?
            } else {
                if let Ok(mut stats) = self.compression_stats.lock() {
                    stats.record_compression(
                        bytes.len(),
                        compressed.data.len(),
                        compression_time_us,
                    );
                }
                use base64::Engine;
                format!(
                    "__compressed__:{}:{}\n{}",
                    compressed.algorithm,
                    compressed.original_size,
                    base64::engine::general_purpose::STANDARD.encode(&compressed.data)
                )
            }
        } else {
            serde_json::to_string(embedding).map_err(do_memory_core::Error::Serialization)?
        };

        #[cfg(not(feature = "compression"))]
        let embedding_data: String =
            serde_json::to_string(embedding).map_err(do_memory_core::Error::Serialization)?;

        let embedding_json_for_vector: String =
            serde_json::to_string(embedding).map_err(do_memory_core::Error::Serialization)?;

        const SQL: &str = r#"
            INSERT OR REPLACE INTO embeddings (embedding_id, item_id, item_type, embedding_data, embedding_vector, dimension, model)
            VALUES (?, ?, ?, ?, vector32(?), ?, ?)
        "#;

        let embedding_id = self.generate_embedding_id(item_id, item_type);
        let stmt = self
            .prepared_cache
            .get_or_prepare(&conn, SQL)
            .await
            .map_err(|e| {
                do_memory_core::Error::Storage(format!("Failed to prepare statement: {}", e))
            })?;
        stmt.execute(libsql::params![
            embedding_id,
            item_id.to_string(),
            item_type.to_string(),
            embedding_data,
            embedding_json_for_vector,
            embedding.len() as i64,
            "default"
        ])
        .await
        .map_err(|e| do_memory_core::Error::Storage(format!("Failed to store embedding: {}", e)))?;

        info!("Successfully stored embedding: {}", item_id);
        Ok(())
    }

    /// Get an embedding (internal implementation)
    pub(crate) async fn _get_embedding_internal(
        &self,
        item_id: &str,
        item_type: &str,
    ) -> Result<Option<Vec<f32>>> {
        debug!(
            "Retrieving embedding: item_id={}, item_type={}",
            item_id, item_type
        );
        let (conn, _conn_id) = self.get_connection_with_id().await?;

        const SQL: &str =
            "SELECT embedding_data FROM embeddings WHERE item_id = ? AND item_type = ?";

        let stmt = self
            .prepared_cache
            .get_or_prepare(&conn, SQL)
            .await
            .map_err(|e| {
                do_memory_core::Error::Storage(format!("Failed to prepare statement: {}", e))
            })?;
        let mut rows = stmt
            .query(libsql::params![item_id.to_string(), item_type.to_string()])
            .await
            .map_err(|e| {
                do_memory_core::Error::Storage(format!("Failed to query embedding: {}", e))
            })?;

        if let Some(row) = rows.next().await.map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to fetch embedding row: {}", e))
        })? {
            let embedding_data: String = row
                .get(0)
                .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?;

            let embedding = self.decode_embedding_data(&embedding_data)?;
            Ok(Some(embedding))
        } else {
            Ok(None)
        }
    }

    /// Delete an embedding
    pub(crate) async fn _delete_embedding_internal(&self, item_id: &str) -> Result<bool> {
        #[cfg(feature = "turso_multi_dimension")]
        {
            return self.delete_embedding_dimension_aware(item_id).await;
        }

        #[cfg(not(feature = "turso_multi_dimension"))]
        {
            let (conn, _conn_id) = self.get_connection_with_id().await?;
            const SQL: &str = "DELETE FROM embeddings WHERE item_id = ?";
            let stmt = self
                .prepared_cache
                .get_or_prepare(&conn, SQL)
                .await
                .map_err(|e| {
                    do_memory_core::Error::Storage(format!("Failed to prepare statement: {}", e))
                })?;
            let rows_affected = stmt
                .execute(libsql::params![item_id.to_string()])
                .await
                .map_err(|e| {
                    do_memory_core::Error::Storage(format!("Failed to delete embedding: {}", e))
                })?;
            Ok(rows_affected > 0)
        }
    }

    /// Delete embeddings in batch
    pub(crate) async fn _delete_embeddings_batch_internal(
        &self,
        item_ids: &[String],
    ) -> Result<usize> {
        if item_ids.is_empty() {
            return Ok(0);
        }

        #[cfg(feature = "turso_multi_dimension")]
        {
            return self.delete_embeddings_batch_dimension_aware(item_ids).await;
        }

        #[cfg(not(feature = "turso_multi_dimension"))]
        {
            let (conn, _conn_id) = self.get_connection_with_id().await?;
            let mut total_affected = 0usize;

            for chunk in Self::chunk_item_ids(item_ids) {
                let placeholders = Self::in_clause_placeholders(chunk.len());
                let sql = format!("DELETE FROM embeddings WHERE item_id IN ({})", placeholders);

                let params: Vec<libsql::Value> = chunk.iter().map(|id| id.clone().into()).collect();

                let rows_affected = conn
                    .execute(&sql, libsql::params_from_iter(params))
                    .await
                    .map_err(|e| {
                        do_memory_core::Error::Storage(format!(
                            "Failed to delete embeddings batch: {}",
                            e
                        ))
                    })?;

                total_affected += rows_affected as usize;
            }

            Ok(total_affected)
        }
    }

    /// Store embeddings in batch
    pub(crate) async fn _store_embeddings_batch_internal(
        &self,
        embeddings: Vec<(String, Vec<f32>)>,
    ) -> Result<()> {
        if embeddings.is_empty() {
            return Ok(());
        }

        let (conn, _conn_id) = self.get_connection_with_id().await?;

        // Start transaction for batch insertion
        conn.execute("BEGIN TRANSACTION", ())
            .await
            .map_err(|e| do_memory_core::Error::Storage(format!("Failed to begin txn: {}", e)))?;

        const SQL: &str = r#"
            INSERT OR REPLACE INTO embeddings (embedding_id, item_id, item_type, embedding_data, embedding_vector, dimension, model)
            VALUES (?, ?, ?, ?, vector32(?), ?, ?)
        "#;

        for (item_id, embedding) in &embeddings {
            let embedding_json = match serde_json::to_string(embedding) {
                Ok(json) => json,
                Err(e) => {
                    let _ = conn.execute("ROLLBACK", ()).await;
                    return Err(do_memory_core::Error::Serialization(e));
                }
            };
            let embedding_id = self.generate_embedding_id(item_id, "embedding");
            let stmt = match self.prepared_cache.get_or_prepare(&conn, SQL).await {
                Ok(stmt) => stmt,
                Err(e) => {
                    let _ = conn.execute("ROLLBACK", ()).await;
                    return Err(do_memory_core::Error::Storage(format!(
                        "Failed to prepare statement: {}",
                        e
                    )));
                }
            };
            if let Err(e) = stmt
                .execute(libsql::params![
                    embedding_id,
                    item_id.clone(),
                    "embedding",
                    embedding_json.clone(),
                    embedding_json,
                    embedding.len() as i64,
                    "default"
                ])
                .await
            {
                let _ = conn.execute("ROLLBACK", ()).await;
                return Err(do_memory_core::Error::Storage(format!(
                    "Failed to store batch: {}",
                    e
                )));
            }
        }

        if let Err(e) = conn.execute("COMMIT", ()).await {
            let _ = conn.execute("ROLLBACK", ()).await;
            return Err(do_memory_core::Error::Storage(format!(
                "Failed to commit txn: {}",
                e
            )));
        }

        Ok(())
    }

    /// Get embeddings in batch with chunked IN clause to avoid SQLITE_MAX_VARIABLE_NUMBER
    pub(crate) async fn _get_embeddings_batch_internal(
        &self,
        item_ids: &[String],
    ) -> Result<Vec<Option<Vec<f32>>>> {
        if item_ids.is_empty() {
            return Ok(Vec::new());
        }

        let (conn, _conn_id) = self.get_connection_with_id().await?;
        let mut results_map = std::collections::HashMap::new();

        for chunk in Self::chunk_item_ids(item_ids) {
            let placeholders = Self::in_clause_placeholders(chunk.len());
            let sql = format!(
                "SELECT item_id, embedding_data FROM embeddings WHERE item_type = 'embedding' AND item_id IN ({})",
                placeholders
            );

            let params: Vec<libsql::Value> = chunk.iter().map(|id| id.clone().into()).collect();

            let mut rows = conn
                .query(&sql, libsql::params_from_iter(params))
                .await
                .map_err(|e| {
                    do_memory_core::Error::Storage(format!(
                        "Failed to query embeddings batch: {}",
                        e
                    ))
                })?;

            while let Some(row) = rows.next().await.map_err(|e| {
                do_memory_core::Error::Storage(format!("Failed to fetch batch row: {}", e))
            })? {
                let item_id: String = row
                    .get(0)
                    .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?;
                let embedding_data: String = row
                    .get(1)
                    .map_err(|e| do_memory_core::Error::Storage(e.to_string()))?;

                let embedding = self.decode_embedding_data(&embedding_data)?;
                results_map.insert(item_id, embedding);
            }
        }

        let results = item_ids
            .iter()
            .map(|id| results_map.get(id).cloned())
            .collect();

        Ok(results)
    }

    /// Decode embedding data from string representation (handles compression)
    pub(crate) fn decode_embedding_data(&self, embedding_data: &str) -> Result<Vec<f32>> {
        #[cfg(feature = "compression")]
        {
            if let Some(remainder) = embedding_data.strip_prefix("__compressed__:") {
                let newline_pos = remainder
                    .find('\n')
                    .ok_or_else(|| do_memory_core::Error::Storage("Invalid format".into()))?;
                let header = &remainder[..newline_pos];
                let encoded_data = &remainder[newline_pos + 1..];
                let colon_pos = header
                    .find(':')
                    .ok_or_else(|| do_memory_core::Error::Storage("Invalid header".into()))?;
                let algorithm_str = &header[..colon_pos];
                let original_size: usize = header[colon_pos + 1..]
                    .parse()
                    .map_err(|_| do_memory_core::Error::Storage("Invalid size".into()))?;
                let algorithm = match algorithm_str {
                    "lz4" => crate::CompressionAlgorithm::Lz4,
                    "zstd" => crate::CompressionAlgorithm::Zstd,
                    "gzip" => crate::CompressionAlgorithm::Gzip,
                    _ => return Err(do_memory_core::Error::Storage("Unknown algo".into())),
                };
                let compressed_data = base64::Engine::decode(
                    &base64::engine::general_purpose::STANDARD,
                    encoded_data,
                )
                .map_err(|e| do_memory_core::Error::Storage(format!("Base64 fail: {}", e)))?;
                let payload = crate::CompressedPayload {
                    original_size,
                    compressed_size: compressed_data.len(),
                    compression_ratio: 0.0,
                    data: compressed_data,
                    algorithm,
                };
                let bytes = payload.decompress()?;
                Ok(bytes
                    .chunks_exact(4)
                    .map(|chunk| {
                        let mut arr = [0u8; 4];
                        arr.copy_from_slice(chunk);
                        f32::from_le_bytes(arr)
                    })
                    .collect())
            } else {
                serde_json::from_str(embedding_data)
                    .map_err(|e| do_memory_core::Error::Storage(e.to_string()))
            }
        }

        #[cfg(not(feature = "compression"))]
        {
            serde_json::from_str(embedding_data)
                .map_err(|e| do_memory_core::Error::Storage(e.to_string()))
        }
    }

    pub(crate) fn generate_embedding_id(&self, item_id: &str, item_type: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        format!("{}:{}", item_id, item_type).hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}
