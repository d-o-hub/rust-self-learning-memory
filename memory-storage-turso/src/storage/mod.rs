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
pub mod capacity;
pub mod episodes;
pub mod heuristics;
pub mod monitoring;
pub mod patterns;
pub mod search;

pub use episodes::EpisodeQuery;
#[allow(unused)]
pub use patterns::PatternMetadata;
pub use patterns::PatternQuery;

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

        let embedding_json =
            serde_json::to_string(embedding).map_err(memory_core::Error::Serialization)?;

        let sql = r#"
            INSERT OR REPLACE INTO embeddings (embedding_id, item_id, item_type, embedding_data, dimension, model) VALUES (?, ?, ?, ?, ?, ?)
        "#;

        let embedding_id = self.generate_embedding_id(item_id, item_type);

        conn.execute(
            sql,
            libsql::params![
                embedding_id,
                item_id.to_string(),
                item_type.to_string(),
                embedding_json,
                embedding.len() as i64,
                "default"
            ],
        )
        .await
        .map_err(|e| memory_core::Error::Storage(format!("Failed to store embedding: {}", e)))?;

        info!("Successfully stored embedding: {}", item_id);
        Ok(())
    }

    /// Get an embedding (internal implementation)
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

        let sql = "SELECT embedding_data FROM embeddings WHERE item_id = ? AND item_type = ?";

        let mut rows = conn
            .query(
                sql,
                libsql::params![item_id.to_string(), item_type.to_string()],
            )
            .await
            .map_err(|e| {
                memory_core::Error::Storage(format!("Failed to query embedding: {}", e))
            })?;

        if let Some(row) = rows.next().await.map_err(|e| {
            memory_core::Error::Storage(format!("Failed to fetch embedding row: {}", e))
        })? {
            let embedding_json: String = row
                .get(0)
                .map_err(|e| memory_core::Error::Storage(e.to_string()))?;
            let embedding: Vec<f32> = serde_json::from_str(&embedding_json).map_err(|e| {
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

        let sql = "DELETE FROM embeddings WHERE item_id = ?";

        let rows_affected = conn
            .execute(sql, libsql::params![item_id.to_string()])
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

        let sql = r#"
            INSERT OR REPLACE INTO embeddings (embedding_id, item_id, item_type, embedding_data, dimension, model) VALUES (?, ?, ?, ?, ?, ?)
        "#;

        for (item_id, embedding) in embeddings {
            let embedding_json =
                serde_json::to_string(&embedding).map_err(memory_core::Error::Serialization)?;

            let embedding_id = self.generate_embedding_id(&item_id, "embedding");

            conn.execute(
                sql,
                libsql::params![
                    embedding_id,
                    item_id,
                    "embedding",
                    embedding_json,
                    embedding.len() as i64,
                    "default"
                ],
            )
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
