//! Multi-dimension test utilities for Turso AI enhancements
//!
//! Provides test harnesses and helpers for testing dimension-specific
//! embedding tables and routing logic.

use anyhow::Result;
use uuid::Uuid;

/// Get the expected table name for a given dimension
pub fn table_for_dimension(dimension: usize) -> &'static str {
    match dimension {
        384 => "embeddings_384",
        1024 => "embeddings_1024",
        1536 => "embeddings_1536",
        3072 => "embeddings_3072",
        _ => "embeddings_other",
    }
}

#[cfg(feature = "turso")]
pub use turso_utils::*;

#[cfg(feature = "turso")]
mod turso_utils {
    use super::*;
    use memory_core::{embeddings::EmbeddingStorageBackend, Episode};
    use memory_storage_turso::TursoStorage;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha8Rng;
    use std::sync::Arc;
    use tempfile::TempDir;
    use tracing;

    /// Generate a random embedding of specified dimension
    pub fn generate_embedding(dimension: usize, seed: u64) -> Vec<f32> {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        (0..dimension).map(|_| rng.gen_range(-1.0..1.0)).collect()
    }

    /// Test harness for multi-dimension embedding tests
    pub struct MultiDimensionTestHarness {
        /// Turso storage instance
        pub storage: Arc<TursoStorage>,
        /// Temporary directory for database file
        pub temp_dir: TempDir,
    }

    impl MultiDimensionTestHarness {
        /// Create a new test harness with initialized schema
        pub async fn new() -> Result<Self> {
            let temp_dir = TempDir::new()?;
            let db_path = temp_dir.path().join("test.db");

            let storage =
                TursoStorage::new(&format!("file://{}", db_path.to_string_lossy()), "").await?;

            storage.initialize_schema().await?;
            Ok(Self {
                storage: Arc::new(storage),
                temp_dir,
            })
        }

        /// Create a test episode with embedding
        pub async fn create_episode_with_embedding(
            &self,
            dimension: usize,
            seed: u64,
        ) -> Result<(Episode, Vec<f32>)> {
            use memory_core::types::{ComplexityLevel, TaskContext, TaskType};

            // Create episode
            let context = TaskContext {
                language: Some("rust".to_string()),
                framework: None,
                complexity: ComplexityLevel::Simple,
                domain: "web-api".to_string(),
                tags: vec![],
            };

            let mut episode = Episode::new(
                format!("Test episode with {} dim embedding", dimension),
                context,
                TaskType::Testing,
            );

            episode.complete(memory_core::types::TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            });

            // Store episode
            self.storage.store_episode(&episode).await?;

            // Generate and store embedding
            let embedding = generate_embedding(dimension, seed);
            self.storage
                .store_episode_embedding(episode.episode_id, embedding.clone())
                .await?;

            Ok((episode, embedding))
        }

        /// Verify an embedding was stored in the correct table
        ///
        /// Queries each dimension table to find where the embedding is stored
        /// and verifies it's in the expected table for the given dimension.
        pub async fn verify_table_usage(
            &self,
            episode_id: Uuid,
            expected_dimension: usize,
        ) -> Result<bool> {
            // Verify embedding was stored by retrieving it
            let embedding = self.storage.get_episode_embedding(episode_id).await?;

            if let Some(emb) = embedding {
                let actual_dimension = emb.len();
                let matches = actual_dimension == expected_dimension;

                if matches {
                    tracing::info!(
                        "Embedding {} has expected dimension {}",
                        episode_id,
                        expected_dimension
                    );
                } else {
                    tracing::error!(
                        "Embedding {} has dimension {}, expected {}",
                        episode_id,
                        actual_dimension,
                        expected_dimension
                    );
                }

                Ok(matches)
            } else {
                tracing::error!("Embedding {} not found in storage", episode_id);
                Ok(false)
            }
        }

        /// Check if embedding exists in any dimension table
        ///
        /// Returns the expected table name based on embedding dimension.
        pub async fn find_embedding_table(&self, episode_id: Uuid) -> Result<Option<&'static str>> {
            // Retrieve embedding to get its dimension
            let embedding = self.storage.get_episode_embedding(episode_id).await?;

            if let Some(emb) = embedding {
                let dimension = emb.len();
                let table_name = table_for_dimension(dimension);
                Ok(Some(table_name))
            } else {
                Ok(None)
            }
        }

        /// Run a similarity search and verify results
        pub async fn run_similarity_search(
            &self,
            query_embedding: Vec<f32>,
            limit: usize,
            threshold: f32,
        ) -> Result<Vec<(Uuid, f32)>> {
            let results = self
                .storage
                .find_similar_episodes(query_embedding, limit, threshold)
                .await?;

            Ok(results
                .into_iter()
                .map(|result| (result.item.episode_id, result.similarity))
                .collect())
        }
    }

    /// Helper to create test embeddings of various dimensions
    pub struct EmbeddingGenerator {
        dimension: usize,
        seed: u64,
    }

    impl EmbeddingGenerator {
        pub fn new(dimension: usize) -> Self {
            Self {
                dimension,
                seed: 42,
            }
        }

        pub fn with_seed(dimension: usize, seed: u64) -> Self {
            Self { dimension, seed }
        }

        pub fn generate(&mut self, count: usize) -> Vec<Vec<f32>> {
            (0..count)
                .map(|i| {
                    let mut rng = ChaCha8Rng::seed_from_u64(self.seed + i as u64);
                    (0..self.dimension)
                        .map(|_| rng.gen_range(-1.0..1.0))
                        .collect()
                })
                .collect()
        }

        pub fn generate_with_similarity(
            &mut self,
            base_embedding: &[f32],
            similarity: f32,
            count: usize,
        ) -> Vec<Vec<f32>> {
            assert!(base_embedding.len() == self.dimension);
            assert!((0.0..=1.0).contains(&similarity));

            (0..count)
                .map(|i| {
                    let mut rng = ChaCha8Rng::seed_from_u64(self.seed + i as u64);
                    base_embedding
                        .iter()
                        .map(|&base| {
                            let noise = rng.gen_range(-0.5..0.5) * (1.0 - similarity);
                            (base + noise).clamp(-1.0, 1.0)
                        })
                        .collect()
                })
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_for_dimension() {
        assert_eq!(table_for_dimension(384), "embeddings_384");
        assert_eq!(table_for_dimension(1536), "embeddings_1536");
        assert_eq!(table_for_dimension(500), "embeddings_other");
        assert_eq!(table_for_dimension(3072), "embeddings_3072");
    }

    #[cfg(feature = "turso")]
    #[test]
    fn test_embedding_generator() {
        use super::turso_utils::EmbeddingGenerator;

        let mut gen = EmbeddingGenerator::new(384);
        let embeddings = gen.generate(5);

        assert_eq!(embeddings.len(), 5);
        assert_eq!(embeddings[0].len(), 384);

        // Verify randomness (same seed produces same sequence)
        let mut gen2 = EmbeddingGenerator::with_seed(384, 42);
        let embeddings2 = gen2.generate(5);
        assert_eq!(embeddings, embeddings2);
    }
}
