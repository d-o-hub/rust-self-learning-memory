//! Context-aware embeddings with task-specific adaptation
//!
//! This module provides embedding adaptation for improved semantic similarity
//! within task-specific contexts. It uses contrastive learning to train
//! task-type specific adapters that transform base embeddings into spaces
//! where similar tasks are closer together.
//!
//! # Architecture
//!
//! - **Base Embeddings**: Standard embedding provider (local/OpenAI)
//! - **Task Adapters**: Linear transformations per task type
//! - **Contrastive Learning**: Train adapters using positive/negative pairs
//! - **Backward Compatibility**: Falls back to base embeddings if no adapter

use crate::embeddings::EmbeddingProvider;
use crate::episode::Episode;
use crate::types::TaskType;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

/// Context-aware embedding provider with task-specific adaptation
///
/// Wraps a base embedding provider and applies learned task-specific
/// transformations to improve semantic similarity for episodes of the
/// same task type.
///
/// # Examples
///
/// ```no_run
/// use memory_core::spatiotemporal::embeddings::ContextAwareEmbeddings;
/// use memory_core::embeddings::LocalEmbeddingProvider;
/// use memory_core::types::TaskType;
/// use std::sync::Arc;
///
/// # async fn example() -> anyhow::Result<()> {
/// let base = Arc::new(LocalEmbeddingProvider::new(Default::default()).await?);
/// let mut embeddings = ContextAwareEmbeddings::new(base);
///
/// // Get adapted embedding for a coding task
/// let embedding = embeddings
///     .get_adapted_embedding("implement REST API", Some(TaskType::CodeGeneration))
///     .await?;
/// # Ok(())
/// # }
/// ```
pub struct ContextAwareEmbeddings {
    /// Base embedding provider
    base_embeddings: Arc<dyn EmbeddingProvider>,
    /// Task-specific adaptation matrices
    task_adapters: HashMap<TaskType, TaskAdapter>,
}

/// Task-specific embedding adapter
///
/// Applies a learned linear transformation to base embeddings to improve
/// similarity within a specific task type. Trained using contrastive learning.
#[derive(Debug, Clone)]
pub struct TaskAdapter {
    /// Task type this adapter is for
    pub task_type: TaskType,
    /// Linear transformation matrix (dim × dim)
    /// Applied as: `adapted = base * adaptation_matrix`
    pub adaptation_matrix: Vec<Vec<f32>>,
    /// Number of training examples used
    pub trained_on_count: usize,
}

impl TaskAdapter {
    /// Create a new task adapter with identity matrix
    ///
    /// # Arguments
    ///
    /// * `task_type` - The task type this adapter is for
    /// * `dimension` - Embedding dimension
    ///
    /// # Returns
    ///
    /// A new adapter initialized with identity transformation
    fn new_identity(task_type: TaskType, dimension: usize) -> Self {
        let mut matrix = vec![vec![0.0; dimension]; dimension];
        #[allow(clippy::needless_range_loop)]
        for i in 0..dimension {
            matrix[i][i] = 1.0;
        }

        Self {
            task_type,
            adaptation_matrix: matrix,
            trained_on_count: 0,
        }
    }

    /// Apply this adapter to a base embedding
    ///
    /// Performs matrix multiplication: `result = embedding * adaptation_matrix`
    ///
    /// # Arguments
    ///
    /// * `base_embedding` - The base embedding vector
    ///
    /// # Returns
    ///
    /// Adapted embedding vector
    #[must_use]
    pub fn adapt(&self, base_embedding: Vec<f32>) -> Vec<f32> {
        let dim = base_embedding.len();
        let mut adapted = vec![0.0; dim];

        // Matrix multiplication: adapted[i] = sum(base[j] * matrix[j][i])
        #[allow(clippy::needless_range_loop)]
        for i in 0..dim {
            for j in 0..dim {
                adapted[i] += base_embedding[j] * self.adaptation_matrix[j][i];
            }
        }

        adapted
    }
}

/// A pair of episodes for contrastive learning
///
/// Used to train task adapters by bringing similar episodes (positives)
/// closer together and pushing dissimilar episodes (negatives) apart.
#[derive(Debug, Clone)]
pub struct ContrastivePair {
    /// Anchor episode
    pub anchor: Episode,
    /// Similar episode (same task type, successful)
    pub positive: Episode,
    /// Dissimilar episode (different task type or failed)
    pub negative: Episode,
}

impl ContextAwareEmbeddings {
    /// Create a new context-aware embedding provider
    ///
    /// # Arguments
    ///
    /// * `base_embeddings` - Base embedding provider to wrap
    ///
    /// # Returns
    ///
    /// A new `ContextAwareEmbeddings` with no trained adapters
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use memory_core::spatiotemporal::embeddings::ContextAwareEmbeddings;
    /// use memory_core::embeddings::LocalEmbeddingProvider;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let base = Arc::new(LocalEmbeddingProvider::new(Default::default()).await?);
    /// let embeddings = ContextAwareEmbeddings::new(base);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn new(base_embeddings: Arc<dyn EmbeddingProvider>) -> Self {
        Self {
            base_embeddings,
            task_adapters: HashMap::new(),
        }
    }

    /// Get adapted embedding for text with optional task type
    ///
    /// If a task type is provided and an adapter exists for it, the base
    /// embedding will be transformed using the learned adapter. Otherwise,
    /// the base embedding is returned unchanged (backward compatibility).
    ///
    /// # Arguments
    ///
    /// * `text` - Text to embed
    /// * `task_type` - Optional task type for adaptation
    ///
    /// # Returns
    ///
    /// Embedding vector (adapted if task adapter available)
    ///
    /// # Errors
    ///
    /// Returns error if base embedding generation fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use memory_core::spatiotemporal::embeddings::ContextAwareEmbeddings;
    /// use memory_core::embeddings::LocalEmbeddingProvider;
    /// use memory_core::types::TaskType;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// # let base = Arc::new(LocalEmbeddingProvider::new(Default::default()).await?);
    /// let embeddings = ContextAwareEmbeddings::new(base);
    ///
    /// // Get base embedding (no adapter trained yet)
    /// let base = embeddings.get_adapted_embedding("debug error", None).await?;
    ///
    /// // Get adapted embedding (if adapter exists)
    /// let adapted = embeddings
    ///     .get_adapted_embedding("debug error", Some(TaskType::Debugging))
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_adapted_embedding(
        &self,
        text: &str,
        task_type: Option<TaskType>,
    ) -> Result<Vec<f32>> {
        // Get base embedding
        let base_embedding = self.base_embeddings.embed_text(text).await?;

        // Apply task-specific adaptation if available
        if let Some(task) = task_type {
            if let Some(adapter) = self.task_adapters.get(&task) {
                return Ok(adapter.adapt(base_embedding));
            }
        }

        // Fallback to base embedding
        Ok(base_embedding)
    }

    /// Train a task-specific adapter using contrastive learning
    ///
    /// This simplified implementation creates an identity matrix as a baseline.
    /// Full contrastive learning optimization (minimizing positive distances,
    /// maximizing negative distances) can be added in Phase 4.
    ///
    /// # Arguments
    ///
    /// * `task_type` - Task type to train adapter for
    /// * `contrastive_pairs` - Training pairs (anchor, positive, negative)
    ///
    /// # Returns
    ///
    /// Ok if training succeeds
    ///
    /// # Errors
    ///
    /// Returns error if training fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use memory_core::spatiotemporal::embeddings::{ContextAwareEmbeddings, ContrastivePair};
    /// use memory_core::embeddings::LocalEmbeddingProvider;
    /// use memory_core::types::TaskType;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// # let base = Arc::new(LocalEmbeddingProvider::new(Default::default()).await?);
    /// let mut embeddings = ContextAwareEmbeddings::new(base);
    ///
    /// // Create training pairs (in practice, from real episode data)
    /// let pairs: Vec<ContrastivePair> = vec![];
    ///
    /// // Train adapter for debugging tasks
    /// embeddings.train_adapter(TaskType::Debugging, &pairs)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn train_adapter(
        &mut self,
        task_type: TaskType,
        contrastive_pairs: &[ContrastivePair],
    ) -> Result<()> {
        if contrastive_pairs.is_empty() {
            anyhow::bail!("Cannot train adapter with empty training set");
        }

        // Get embedding dimension from base provider
        let dim = self.base_embeddings.embedding_dimension();

        // Create identity matrix as baseline (MVP implementation)
        // TODO: Implement full contrastive learning optimization in Phase 4
        // This would involve:
        // 1. Computing embeddings for anchor, positive, negative
        // 2. Learning transformation matrix M that:
        //    - Minimizes distance(M * anchor, M * positive)
        //    - Maximizes distance(M * anchor, M * negative)
        // 3. Using gradient descent or similar optimization
        let adapter = TaskAdapter::new_identity(task_type, dim);

        // Update adapter with training count
        let mut adapter = adapter;
        adapter.trained_on_count = contrastive_pairs.len();

        self.task_adapters.insert(task_type, adapter);

        Ok(())
    }

    /// Check if an adapter exists for a task type
    ///
    /// # Arguments
    ///
    /// * `task_type` - Task type to check
    ///
    /// # Returns
    ///
    /// `true` if adapter exists, `false` otherwise
    #[must_use]
    pub fn has_adapter(&self, task_type: TaskType) -> bool {
        self.task_adapters.contains_key(&task_type)
    }

    /// Get the number of trained adapters
    ///
    /// # Returns
    ///
    /// Count of task types with trained adapters
    #[must_use]
    pub fn adapter_count(&self) -> usize {
        self.task_adapters.len()
    }

    /// Get reference to a task adapter
    ///
    /// # Arguments
    ///
    /// * `task_type` - Task type to get adapter for
    ///
    /// # Returns
    ///
    /// Reference to adapter if it exists
    #[must_use]
    pub fn get_adapter(&self, task_type: TaskType) -> Option<&TaskAdapter> {
        self.task_adapters.get(&task_type)
    }

    /// Get the base embedding provider
    ///
    /// # Returns
    ///
    /// Reference to base embedding provider
    #[must_use]
    pub fn base_provider(&self) -> &Arc<dyn EmbeddingProvider> {
        &self.base_embeddings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embeddings::EmbeddingProvider;
    use crate::types::{ComplexityLevel, TaskContext, TaskOutcome};
    use async_trait::async_trait;

    // Mock embedding provider for testing
    struct MockEmbeddingProvider {
        dimension: usize,
    }

    impl MockEmbeddingProvider {
        fn new(dimension: usize) -> Self {
            Self { dimension }
        }
    }

    #[async_trait]
    impl EmbeddingProvider for MockEmbeddingProvider {
        async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
            // Create a deterministic embedding based on text length
            let mut embedding = vec![0.0; self.dimension];
            let text_len = text.len() as f32;
            for i in 0..self.dimension {
                embedding[i] = (i as f32 + text_len) / 100.0;
            }
            Ok(embedding)
        }

        async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
            let mut embeddings = Vec::new();
            for text in texts {
                embeddings.push(self.embed_text(text).await?);
            }
            Ok(embeddings)
        }

        async fn similarity(&self, text1: &str, text2: &str) -> Result<f32> {
            let emb1 = self.embed_text(text1).await?;
            let emb2 = self.embed_text(text2).await?;
            Ok(cosine_similarity(&emb1, &emb2))
        }

        fn embedding_dimension(&self) -> usize {
            self.dimension
        }

        fn model_name(&self) -> &str {
            "mock-model"
        }
    }

    // Helper function for cosine similarity
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot / (norm_a * norm_b)
        }
    }

    fn create_test_episode(task_type: TaskType, description: &str) -> Episode {
        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: None,
            complexity: ComplexityLevel::Moderate,
            domain: "testing".to_string(),
            tags: vec![],
        };

        let mut episode = Episode::new(description.to_string(), context, task_type);
        episode.complete(TaskOutcome::Success {
            verdict: "Success".to_string(),
            artifacts: vec![],
        });
        episode
    }

    #[test]
    fn test_context_aware_embeddings_creation() {
        let provider = Arc::new(MockEmbeddingProvider::new(128));
        let embeddings = ContextAwareEmbeddings::new(provider);

        assert_eq!(embeddings.adapter_count(), 0);
        assert!(!embeddings.has_adapter(TaskType::CodeGeneration));
    }

    #[tokio::test]
    async fn test_base_embedding_fallback() {
        let provider = Arc::new(MockEmbeddingProvider::new(128));
        let embeddings = ContextAwareEmbeddings::new(provider.clone());

        // Without adapter, should return base embedding
        let text = "implement REST API";
        let adapted = embeddings
            .get_adapted_embedding(text, Some(TaskType::CodeGeneration))
            .await
            .unwrap();
        let base = provider.embed_text(text).await.unwrap();

        assert_eq!(adapted.len(), base.len());
        assert_eq!(adapted, base);
    }

    #[test]
    fn test_task_adapter_creation() {
        let adapter = TaskAdapter::new_identity(TaskType::Debugging, 4);

        assert_eq!(adapter.task_type, TaskType::Debugging);
        assert_eq!(adapter.adaptation_matrix.len(), 4);
        assert_eq!(adapter.adaptation_matrix[0].len(), 4);
        assert_eq!(adapter.trained_on_count, 0);

        // Check identity matrix
        for i in 0..4 {
            for j in 0..4 {
                if i == j {
                    assert_eq!(adapter.adaptation_matrix[i][j], 1.0);
                } else {
                    assert_eq!(adapter.adaptation_matrix[i][j], 0.0);
                }
            }
        }
    }

    #[test]
    fn test_adapter_identity_transformation() {
        let adapter = TaskAdapter::new_identity(TaskType::Testing, 4);
        let input = vec![1.0, 2.0, 3.0, 4.0];
        let output = adapter.adapt(input.clone());

        // Identity transformation should return same vector
        assert_eq!(output, input);
    }

    #[test]
    fn test_train_adapter_success() {
        let provider = Arc::new(MockEmbeddingProvider::new(128));
        let mut embeddings = ContextAwareEmbeddings::new(provider);

        // Create training pairs
        let pairs = vec![
            ContrastivePair {
                anchor: create_test_episode(TaskType::CodeGeneration, "task 1"),
                positive: create_test_episode(TaskType::CodeGeneration, "task 2"),
                negative: create_test_episode(TaskType::Debugging, "task 3"),
            },
            ContrastivePair {
                anchor: create_test_episode(TaskType::CodeGeneration, "task 4"),
                positive: create_test_episode(TaskType::CodeGeneration, "task 5"),
                negative: create_test_episode(TaskType::Testing, "task 6"),
            },
        ];

        let result = embeddings.train_adapter(TaskType::CodeGeneration, &pairs);
        assert!(result.is_ok());
        assert!(embeddings.has_adapter(TaskType::CodeGeneration));
        assert_eq!(embeddings.adapter_count(), 1);

        let adapter = embeddings.get_adapter(TaskType::CodeGeneration).unwrap();
        assert_eq!(adapter.trained_on_count, 2);
    }

    #[test]
    fn test_train_adapter_empty_pairs() {
        let provider = Arc::new(MockEmbeddingProvider::new(128));
        let mut embeddings = ContextAwareEmbeddings::new(provider);

        let result = embeddings.train_adapter(TaskType::Debugging, &[]);
        assert!(result.is_err());
        assert!(!embeddings.has_adapter(TaskType::Debugging));
    }

    #[tokio::test]
    async fn test_adapted_embedding_generation() {
        let provider = Arc::new(MockEmbeddingProvider::new(128));
        let mut embeddings = ContextAwareEmbeddings::new(provider.clone());

        // Train adapter
        let pairs = vec![ContrastivePair {
            anchor: create_test_episode(TaskType::Refactoring, "refactor 1"),
            positive: create_test_episode(TaskType::Refactoring, "refactor 2"),
            negative: create_test_episode(TaskType::Analysis, "analyze 1"),
        }];

        embeddings
            .train_adapter(TaskType::Refactoring, &pairs)
            .unwrap();

        // Get adapted embedding (with identity matrix, should equal base)
        let text = "refactor code";
        let adapted = embeddings
            .get_adapted_embedding(text, Some(TaskType::Refactoring))
            .await
            .unwrap();

        assert_eq!(adapted.len(), 128);

        // With identity matrix, adapted should equal base
        let base = provider.embed_text(text).await.unwrap();
        assert_eq!(adapted, base);
    }

    #[tokio::test]
    async fn test_backward_compatibility() {
        let provider = Arc::new(MockEmbeddingProvider::new(128));
        let embeddings = ContextAwareEmbeddings::new(provider.clone());

        // Without any trained adapters, should behave like base provider
        let text = "test task";
        let adapted_none = embeddings.get_adapted_embedding(text, None).await.unwrap();
        let adapted_some = embeddings
            .get_adapted_embedding(text, Some(TaskType::Testing))
            .await
            .unwrap();
        let base = provider.embed_text(text).await.unwrap();

        assert_eq!(adapted_none, base);
        assert_eq!(adapted_some, base);
    }

    #[test]
    fn test_multiple_adapters() {
        let provider = Arc::new(MockEmbeddingProvider::new(64));
        let mut embeddings = ContextAwareEmbeddings::new(provider);

        // Train adapters for different task types
        let coding_pairs = vec![ContrastivePair {
            anchor: create_test_episode(TaskType::CodeGeneration, "code 1"),
            positive: create_test_episode(TaskType::CodeGeneration, "code 2"),
            negative: create_test_episode(TaskType::Testing, "test 1"),
        }];

        let debug_pairs = vec![ContrastivePair {
            anchor: create_test_episode(TaskType::Debugging, "debug 1"),
            positive: create_test_episode(TaskType::Debugging, "debug 2"),
            negative: create_test_episode(TaskType::CodeGeneration, "code 3"),
        }];

        embeddings
            .train_adapter(TaskType::CodeGeneration, &coding_pairs)
            .unwrap();
        embeddings
            .train_adapter(TaskType::Debugging, &debug_pairs)
            .unwrap();

        assert_eq!(embeddings.adapter_count(), 2);
        assert!(embeddings.has_adapter(TaskType::CodeGeneration));
        assert!(embeddings.has_adapter(TaskType::Debugging));
        assert!(!embeddings.has_adapter(TaskType::Testing));
    }

    #[test]
    fn test_contrastive_pair_structure() {
        let anchor = create_test_episode(TaskType::CodeGeneration, "anchor");
        let positive = create_test_episode(TaskType::CodeGeneration, "positive");
        let negative = create_test_episode(TaskType::Debugging, "negative");

        let pair = ContrastivePair {
            anchor: anchor.clone(),
            positive: positive.clone(),
            negative: negative.clone(),
        };

        assert_eq!(pair.anchor.task_type, anchor.task_type);
        assert_eq!(pair.positive.task_type, positive.task_type);
        assert_eq!(pair.negative.task_type, negative.task_type);
    }

    #[tokio::test]
    async fn test_embedding_dimension_consistency() {
        let provider = Arc::new(MockEmbeddingProvider::new(256));
        let mut embeddings = ContextAwareEmbeddings::new(provider);

        // Train adapter
        let pairs = vec![ContrastivePair {
            anchor: create_test_episode(TaskType::Analysis, "analyze 1"),
            positive: create_test_episode(TaskType::Analysis, "analyze 2"),
            negative: create_test_episode(TaskType::Testing, "test 1"),
        }];

        embeddings
            .train_adapter(TaskType::Analysis, &pairs)
            .unwrap();

        // Check that adapted embeddings maintain dimension
        let embedding = embeddings
            .get_adapted_embedding("analyze data", Some(TaskType::Analysis))
            .await
            .unwrap();

        assert_eq!(embedding.len(), 256);
    }
}
