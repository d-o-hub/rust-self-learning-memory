//! EmbeddingStorageBackend trait implementation for redb cache

use crate::{EMBEDDINGS_TABLE, EPISODES_TABLE, PATTERNS_TABLE, RedbStorage};
use async_trait::async_trait;
use memory_core::embeddings::{
    EmbeddingStorageBackend, SimilarityMetadata, SimilaritySearchResult, cosine_similarity,
};
use memory_core::episode::PatternId;
use memory_core::{Episode, Error, Pattern, Result};
use redb::ReadableTable;
use std::sync::Arc;
use tracing::debug;
use uuid::Uuid;

#[async_trait]
impl EmbeddingStorageBackend for RedbStorage {
    async fn store_episode_embedding(&self, episode_id: Uuid, embedding: Vec<f32>) -> Result<()> {
        debug!("Storing episode embedding: {}", episode_id);
        let key = format!("episode_{}", episode_id);
        self.store_embedding_raw(&key, &embedding).await
    }

    async fn store_pattern_embedding(
        &self,
        pattern_id: PatternId,
        embedding: Vec<f32>,
    ) -> Result<()> {
        debug!("Storing pattern embedding: {}", pattern_id);
        let key = format!("pattern_{}", pattern_id);
        self.store_embedding_raw(&key, &embedding).await
    }

    async fn get_episode_embedding(&self, episode_id: Uuid) -> Result<Option<Vec<f32>>> {
        debug!("Retrieving episode embedding: {}", episode_id);
        let key = format!("episode_{}", episode_id);
        self.get_embedding_raw(&key).await
    }

    async fn get_pattern_embedding(&self, pattern_id: PatternId) -> Result<Option<Vec<f32>>> {
        debug!("Retrieving pattern embedding: {}", pattern_id);
        let key = format!("pattern_{}", pattern_id);
        self.get_embedding_raw(&key).await
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

        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;

            let embeddings_table = read_txn
                .open_table(EMBEDDINGS_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open embeddings table: {}", e)))?;

            let episodes_table = read_txn
                .open_table(EPISODES_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open episodes table: {}", e)))?;

            let mut results = Vec::new();
            let iter = embeddings_table
                .iter()
                .map_err(|e| Error::Storage(format!("Failed to iterate embeddings: {}", e)))?;

            for result in iter {
                let (key_bytes, embedding_bytes_guard) = result.map_err(|e| {
                    Error::Storage(format!("Failed to read embedding entry: {}", e))
                })?;

                let key = key_bytes.value();

                // Only process episode embeddings
                if !key.starts_with("episode_") {
                    continue;
                }

                let embedding: Vec<f32> = postcard::from_bytes(embedding_bytes_guard.value())
                    .map_err(|e| {
                        Error::Storage(format!("Failed to deserialize embedding: {}", e))
                    })?;

                let similarity = cosine_similarity(&query_embedding, &embedding);

                if similarity >= threshold {
                    // Extract episode ID from key
                    let episode_id_str = &key[8..]; // Remove "episode_" prefix
                    if let Ok(_episode_id) = Uuid::parse_str(episode_id_str) {
                        // Try to get the episode
                        if let Some(episode_bytes) = episodes_table
                            .get(episode_id_str)
                            .map_err(|e| Error::Storage(format!("Failed to get episode: {}", e)))?
                        {
                            let episode: Episode = postcard::from_bytes(episode_bytes.value())
                                .map_err(|e| {
                                    Error::Storage(format!("Failed to deserialize episode: {}", e))
                                })?;

                            results.push(SimilaritySearchResult {
                                item: episode,
                                similarity,
                                metadata: SimilarityMetadata {
                                    embedding_model: "unknown".to_string(),
                                    embedding_timestamp: None,
                                    context: serde_json::json!({}),
                                },
                            });
                        }
                    }
                }
            }

            // Sort by similarity (highest first)
            results.sort_by(|a, b| {
                b.similarity
                    .partial_cmp(&a.similarity)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            // Limit results
            results.truncate(limit);

            Ok(results)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
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

        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;

            let embeddings_table = read_txn
                .open_table(EMBEDDINGS_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open embeddings table: {}", e)))?;

            let patterns_table = read_txn
                .open_table(PATTERNS_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open patterns table: {}", e)))?;

            let mut results = Vec::new();
            let iter = embeddings_table
                .iter()
                .map_err(|e| Error::Storage(format!("Failed to iterate embeddings: {}", e)))?;

            for result in iter {
                let (key_bytes, embedding_bytes_guard) = result.map_err(|e| {
                    Error::Storage(format!("Failed to read embedding entry: {}", e))
                })?;

                let key = key_bytes.value();

                // Only process pattern embeddings
                if !key.starts_with("pattern_") {
                    continue;
                }

                let embedding: Vec<f32> = postcard::from_bytes(embedding_bytes_guard.value())
                    .map_err(|e| {
                        Error::Storage(format!("Failed to deserialize embedding: {}", e))
                    })?;

                let similarity = cosine_similarity(&query_embedding, &embedding);

                if similarity >= threshold {
                    // Extract pattern ID from key
                    let pattern_id_str = &key[8..]; // Remove "pattern_" prefix
                    if let Ok(_pattern_id) = PatternId::parse_str(pattern_id_str) {
                        // Try to get the pattern
                        if let Some(pattern_bytes) = patterns_table
                            .get(pattern_id_str)
                            .map_err(|e| Error::Storage(format!("Failed to get pattern: {}", e)))?
                        {
                            let pattern: Pattern = postcard::from_bytes(pattern_bytes.value())
                                .map_err(|e| {
                                    Error::Storage(format!("Failed to deserialize pattern: {}", e))
                                })?;

                            results.push(SimilaritySearchResult {
                                item: pattern,
                                similarity,
                                metadata: SimilarityMetadata {
                                    embedding_model: "unknown".to_string(),
                                    embedding_timestamp: None,
                                    context: serde_json::json!({}),
                                },
                            });
                        }
                    }
                }
            }

            // Sort by similarity (highest first)
            results.sort_by(|a, b| {
                b.similarity
                    .partial_cmp(&a.similarity)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            // Limit results
            results.truncate(limit);

            Ok(results)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }
}
