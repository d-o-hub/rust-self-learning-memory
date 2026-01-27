//! Storage operations for episodes, patterns, and heuristics
//!
//! This module is organized into submodules for different storage concerns:
//! - `episodes`: Episode CRUD operations
//! - `patterns`: Pattern CRUD operations
//! - `heuristics`: Heuristic CRUD operations
//! - `monitoring`: Monitoring and metrics storage
//! - `embeddings`: Embedding storage and retrieval
//! - `search`: Vector similarity search
//! - `capacity`: Capacity-constrained storage

use crate::TursoStorage;
use async_trait::async_trait;
use memory_core::embeddings::{EmbeddingStorageBackend, SimilaritySearchResult};
use memory_core::{episode::PatternId, Episode, Pattern, Result};
use tracing::{debug, info};
use uuid::Uuid;

// Re-export submodules
pub mod batch;
pub mod capacity;
pub mod episodes;
pub mod heuristics;
pub mod monitoring;
pub mod patterns;
pub mod search;
pub mod tag_operations;

pub use batch::BatchConfig;
pub use episodes::EpisodeQuery;
#[allow(unused)]
pub use patterns::PatternMetadata;
pub use patterns::PatternQuery;
pub use tag_operations::TagStats;

#[async_trait]
impl EmbeddingStorageBackend for TursoStorage {
    async fn store_episode_embedding(&self, episode_id: Uuid, embedding: Vec<f32>) -> Result<()> {
        debug!("Storing episode embedding: {}", episode_id);
        self._store_embedding_internal(&episode_id.to_string(), "episode", &embedding)
            .await
    }

    async fn store_pattern_embedding(
        &self,
        pattern_id: PatternId,
        embedding: Vec<f32>,
    ) -> Result<()> {
        debug!("Storing pattern embedding: {}", pattern_id);
        self._store_embedding_internal(&pattern_id.to_string(), "pattern", &embedding)
            .await
    }

    async fn get_episode_embedding(&self, episode_id: Uuid) -> Result<Option<Vec<f32>>> {
        debug!("Retrieving episode embedding: {}", episode_id);
        self._get_embedding_internal(&episode_id.to_string(), "episode")
            .await
    }

    async fn get_pattern_embedding(&self, pattern_id: PatternId) -> Result<Option<Vec<f32>>> {
        debug!("Retrieving pattern embedding: {}", pattern_id);
        self._get_embedding_internal(&pattern_id.to_string(), "pattern")
            .await
    }

    async fn find_similar_episodes(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SimilaritySearchResult<Episode>>> {
        debug!(
            "Finding similar episodes (limit: {}, threshold: {})",
            limit, threshold
        );

        let conn = self.get_connection().await?;

        // Try to use native vector search if migration is applied
        if let Ok(results) = self
            .find_similar_episodes_native(&conn, &query_embedding, limit, threshold)
            .await
        {
            info!(
                "Found {} similar episodes using native vector search",
                results.len()
            );
            return Ok(results);
        }

        // Fallback to brute-force search if migration not applied
        debug!("Falling back to brute-force search (migration not applied)");
        self.find_similar_episodes_brute_force(&query_embedding, limit, threshold)
            .await
    }

    async fn find_similar_patterns(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SimilaritySearchResult<Pattern>>> {
        debug!(
            "Finding similar patterns (limit: {}, threshold: {})",
            limit, threshold
        );

        let conn = self.get_connection().await?;

        // Try to use native vector search if migration is applied
        if let Ok(results) = self
            .find_similar_patterns_native(&conn, &query_embedding, limit, threshold)
            .await
        {
            info!(
                "Found {} similar patterns using native vector search",
                results.len()
            );
            return Ok(results);
        }

        // Fallback to brute-force search if migration not applied
        debug!("Falling back to brute-force search (migration not applied)");
        self.find_similar_patterns_brute_force(&query_embedding, limit, threshold)
            .await
    }
}

impl TursoStorage {
    /// Get the appropriate embeddings table name for a given dimension
    #[allow(dead_code)]
    pub fn get_embedding_table_for_dimension(&self, dimension: usize) -> &'static str {
        match dimension {
            384 => "embeddings_384",
            1024 => "embeddings_1024",
            1536 => "embeddings_1536",
            3072 => "embeddings_3072",
            _ => "embeddings_other",
        }
    }

    /// Get the appropriate vector index name for a given dimension
    #[allow(dead_code)]
    pub fn get_vector_index_for_dimension(&self, dimension: usize) -> Option<&'static str> {
        match dimension {
            384 => Some("idx_embeddings_384_vector"),
            1024 => Some("idx_embeddings_1024_vector"),
            1536 => Some("idx_embeddings_1536_vector"),
            3072 => Some("idx_embeddings_3072_vector"),
            _ => None,
        }
    }

    // ========== Internal Embedding Methods ==========

    /// Store an embedding (internal implementation)
    ///
    /// When compression is enabled, embeddings are compressed using the configured
    /// algorithm (LZ4, Zstd, or Gzip) to reduce network bandwidth.
    pub async fn _store_embedding_internal(
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
        let conn = self.get_connection().await?;

        // Get compression threshold from config
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
            // Convert f32 to bytes and compress
            let bytes: Vec<u8> = embedding.iter().flat_map(|&f| f.to_le_bytes()).collect();

            use crate::compression::CompressedPayload;
            let compressed = CompressedPayload::compress(&bytes, compression_threshold)?;

            if compressed.algorithm == crate::CompressionAlgorithm::None {
                // No compression, store as JSON
                serde_json::to_string(embedding).map_err(memory_core::Error::Serialization)?
            } else {
                // Store compressed data with header
                use base64::Engine;
                format!(
                    "__compressed__:{}:{}\n{}",
                    compressed.algorithm,
                    compressed.original_size,
                    base64::engine::general_purpose::STANDARD.encode(&compressed.data)
                )
            }
        } else {
            // No compression, store as JSON
            serde_json::to_string(embedding).map_err(memory_core::Error::Serialization)?
        };

        #[cfg(not(feature = "compression"))]
        let embedding_data: String =
            serde_json::to_string(embedding).map_err(memory_core::Error::Serialization)?;

        const SQL: &str = r#"
            INSERT OR REPLACE INTO embeddings (embedding_id, item_id, item_type, embedding_data, dimension, model) VALUES (?, ?, ?, ?, ?, ?)
        "#;

        let embedding_id = self.generate_embedding_id(item_id, item_type);

        // Use prepared statement cache
        let stmt = self
            .prepared_cache
            .get_or_prepare(&conn, SQL)
            .await
            .map_err(|e| {
                memory_core::Error::Storage(format!("Failed to prepare statement: {}", e))
            })?;

        stmt.execute(libsql::params![
            embedding_id,
            item_id.to_string(),
            item_type.to_string(),
            embedding_data,
            embedding.len() as i64,
            "default"
        ])
        .await
        .map_err(|e| memory_core::Error::Storage(format!("Failed to store embedding: {}", e)))?;

        info!("Successfully stored embedding: {}", item_id);
        Ok(())
    }

    /// Get an embedding (internal implementation)
    ///
    /// Automatically decompresses embeddings if they were stored compressed.
    pub async fn _get_embedding_internal(
        &self,
        item_id: &str,
        item_type: &str,
    ) -> Result<Option<Vec<f32>>> {
        debug!(
            "Retrieving embedding: item_id={}, item_type={}",
            item_id, item_type
        );
        let conn = self.get_connection().await?;

        const SQL: &str =
            "SELECT embedding_data FROM embeddings WHERE item_id = ? AND item_type = ?";

        // Use prepared statement cache
        let stmt = self
            .prepared_cache
            .get_or_prepare(&conn, SQL)
            .await
            .map_err(|e| {
                memory_core::Error::Storage(format!("Failed to prepare statement: {}", e))
            })?;

        let mut rows = stmt
            .query(libsql::params![item_id.to_string(), item_type.to_string()])
            .await
            .map_err(|e| {
                memory_core::Error::Storage(format!("Failed to query embedding: {}", e))
            })?;

        if let Some(row) = rows.next().await.map_err(|e| {
            memory_core::Error::Storage(format!("Failed to fetch embedding row: {}", e))
        })? {
            let embedding_data: String = row
                .get(0)
                .map_err(|e| memory_core::Error::Storage(e.to_string()))?;

            // Check if data is compressed (only when compression is enabled)
            #[cfg(feature = "compression")]
            let embedding: Vec<f32> =
                if let Some(remainder) = embedding_data.strip_prefix("__compressed__:") {
                    // Parse compressed format
                    let newline_pos = remainder.find('\n').ok_or_else(|| {
                        memory_core::Error::Storage(
                            "Invalid compressed data format: missing newline".to_string(),
                        )
                    })?;
                    let header = &remainder[..newline_pos];
                    let encoded_data = &remainder[newline_pos + 1..];

                    // Parse header
                    let colon_pos = header.find(':').ok_or_else(|| {
                        memory_core::Error::Storage("Invalid compressed header format".to_string())
                    })?;
                    let algorithm_str = &header[..colon_pos];
                    let original_size: usize = header[colon_pos + 1..].parse().map_err(|_| {
                        memory_core::Error::Storage(
                            "Invalid original size in compressed header".to_string(),
                        )
                    })?;

                    let algorithm = match algorithm_str {
                        "lz4" => crate::CompressionAlgorithm::Lz4,
                        "zstd" => crate::CompressionAlgorithm::Zstd,
                        "gzip" => crate::CompressionAlgorithm::Gzip,
                        _ => {
                            return Err(memory_core::Error::Storage(format!(
                                "Unknown compression algorithm: {}",
                                algorithm_str
                            )))
                        }
                    };

                    let compressed_data = base64::Engine::decode(
                        &base64::engine::general_purpose::STANDARD,
                        encoded_data,
                    )
                    .map_err(|e| {
                        memory_core::Error::Storage(format!(
                            "Failed to decode base64 compressed data: {}",
                            e
                        ))
                    })?;

                    let payload = crate::CompressedPayload {
                        original_size,
                        compressed_size: compressed_data.len(),
                        compression_ratio: compressed_data.len() as f64 / original_size as f64,
                        data: compressed_data,
                        algorithm,
                    };

                    let bytes = payload.decompress()?;
                    bytes
                        .chunks_exact(4)
                        .map(|chunk| {
                            let mut arr = [0u8; 4];
                            arr.copy_from_slice(chunk);
                            f32::from_le_bytes(arr)
                        })
                        .collect()
                } else {
                    // Not compressed, parse as JSON
                    serde_json::from_str(&embedding_data).map_err(|e| {
                        memory_core::Error::Storage(format!("Failed to parse embedding: {}", e))
                    })?
                };

            #[cfg(not(feature = "compression"))]
            let embedding: Vec<f32> = serde_json::from_str(&embedding_data).map_err(|e| {
                memory_core::Error::Storage(format!("Failed to parse embedding: {}", e))
            })?;

            Ok(Some(embedding))
        } else {
            Ok(None)
        }
    }

    /// Delete an embedding (internal implementation)
    pub async fn _delete_embedding_internal(&self, item_id: &str) -> Result<bool> {
        let conn = self.get_connection().await?;

        const SQL: &str = "DELETE FROM embeddings WHERE item_id = ?";

        // Use prepared statement cache
        let stmt = self
            .prepared_cache
            .get_or_prepare(&conn, SQL)
            .await
            .map_err(|e| {
                memory_core::Error::Storage(format!("Failed to prepare statement: {}", e))
            })?;

        let rows_affected = stmt
            .execute(libsql::params![item_id.to_string()])
            .await
            .map_err(|e| {
                memory_core::Error::Storage(format!("Failed to delete embedding: {}", e))
            })?;

        Ok(rows_affected > 0)
    }

    /// Store embeddings in batch (internal implementation)
    pub async fn _store_embeddings_batch_internal(
        &self,
        embeddings: Vec<(String, Vec<f32>)>,
    ) -> Result<()> {
        debug!("Storing embedding batch: {} items", embeddings.len());
        let conn = self.get_connection().await?;

        const SQL: &str = r#"
            INSERT OR REPLACE INTO embeddings (embedding_id, item_id, item_type, embedding_data, dimension, model) VALUES (?, ?, ?, ?, ?, ?)
        "#;

        // Use prepared statement cache
        let stmt = self
            .prepared_cache
            .get_or_prepare(&conn, SQL)
            .await
            .map_err(|e| {
                memory_core::Error::Storage(format!("Failed to prepare statement: {}", e))
            })?;

        for (item_id, embedding) in embeddings {
            let embedding_json =
                serde_json::to_string(&embedding).map_err(memory_core::Error::Serialization)?;

            let embedding_id = self.generate_embedding_id(&item_id, "embedding");

            stmt.execute(libsql::params![
                embedding_id,
                item_id,
                "embedding",
                embedding_json,
                embedding.len() as i64,
                "default"
            ])
            .await
            .map_err(|e| {
                memory_core::Error::Storage(format!("Failed to store batch embedding: {}", e))
            })?;
        }

        info!("Successfully stored embedding batch");
        Ok(())
    }

    /// Get embeddings in batch (internal implementation)
    pub async fn _get_embeddings_batch_internal(
        &self,
        item_ids: &[String],
    ) -> Result<Vec<Option<Vec<f32>>>> {
        debug!("Getting embedding batch: {} items", item_ids.len());

        let mut results = Vec::with_capacity(item_ids.len());

        for item_id in item_ids {
            let embedding = self._get_embedding_internal(item_id, "embedding").await?;
            results.push(embedding);
        }

        Ok(results)
    }

    /// Generate a deterministic embedding_id from item_id and item_type
    fn generate_embedding_id(&self, item_id: &str, item_type: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        format!("{}:{}", item_id, item_type).hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    // ========== Backend-compatible embedding methods ==========

    /// Store an embedding (backend API)
    pub async fn store_embedding_backend(&self, id: &str, embedding: Vec<f32>) -> Result<()> {
        self._store_embedding_internal(id, "embedding", &embedding)
            .await
    }

    /// Get an embedding (backend API)
    pub async fn get_embedding_backend(&self, id: &str) -> Result<Option<Vec<f32>>> {
        self._get_embedding_internal(id, "embedding").await
    }

    /// Delete an embedding (backend API)
    pub async fn delete_embedding_backend(&self, id: &str) -> Result<bool> {
        self._delete_embedding_internal(id).await
    }

    /// Store embeddings in batch (backend API)
    pub async fn store_embeddings_batch_backend(
        &self,
        embeddings: Vec<(String, Vec<f32>)>,
    ) -> Result<()> {
        self._store_embeddings_batch_internal(embeddings).await
    }

    /// Get embeddings in batch (backend API)
    pub async fn get_embeddings_batch_backend(
        &self,
        ids: &[String],
    ) -> Result<Vec<Option<Vec<f32>>>> {
        self._get_embeddings_batch_internal(ids).await
    }
}
