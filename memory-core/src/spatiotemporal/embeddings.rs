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
#[derive(Clone)]
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

/// Helper function to compute Euclidean distance between two vectors
fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y) * (x - y))
        .sum::<f32>()
        .sqrt()
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
    /// Update gradient for a single triplet (extracted to reduce nesting)
    fn update_gradient_for_triplet(
        adapter: &TaskAdapter,
        anchor_emb: &[f32],
        positive_emb: &[f32],
        negative_emb: &[f32],
        gradient: &mut [Vec<f32>],
        dim: usize,
        margin: f32,
    ) {
        // Apply current adapter
        let anchor_adapted = adapter.adapt(anchor_emb.to_vec());
        let positive_adapted = adapter.adapt(positive_emb.to_vec());
        let negative_adapted = adapter.adapt(negative_emb.to_vec());

        // Compute distances
        let d_pos = euclidean_distance(&anchor_adapted, &positive_adapted);
        let d_neg = euclidean_distance(&anchor_adapted, &negative_adapted);

        // Triplet loss: max(0, d_pos - d_neg + margin)
        let loss = (d_pos - d_neg + margin).max(0.0);

        // Only update gradient if loss > 0 (violation of margin constraint)
        if loss > 0.0 {
            // Compute gradient for this triplet
            for (i, grad_row) in gradient.iter_mut().enumerate().take(dim) {
                for (j, grad_cell) in grad_row.iter_mut().enumerate().take(dim) {
                    // Gradient w.r.t. positive distance (increase to push apart)
                    let grad_pos = (anchor_emb[j] - positive_emb[j])
                        * (anchor_adapted[i] - positive_adapted[i]);

                    // Gradient w.r.t. negative distance (decrease to bring closer)
                    let grad_neg = (anchor_emb[j] - negative_emb[j])
                        * (anchor_adapted[i] - negative_adapted[i]);

                    // Total gradient (minimize d_pos, maximize d_neg)
                    *grad_cell += grad_pos - grad_neg;
                }
            }
        }
    }

    /// Apply gradient descent update to adaptation matrix (extracted to reduce nesting)
    fn apply_gradient_update(
        adaptation_matrix: &mut [Vec<f32>],
        gradient: &[Vec<f32>],
        learning_rate: f32,
        num_pairs: usize,
        dim: usize,
    ) {
        for (i, matrix_row) in adaptation_matrix.iter_mut().enumerate().take(dim) {
            for (j, matrix_cell) in matrix_row.iter_mut().enumerate().take(dim) {
                *matrix_cell -= learning_rate * gradient[i][j] / num_pairs as f32;
            }
        }
    }
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
    /// Uses triplet loss to learn a transformation matrix that brings similar
    /// episodes (same task type, successful outcomes) closer together while
    /// pushing dissimilar episodes (different task types or failed outcomes) apart.
    ///
    /// The algorithm:
    /// 1. Embeds anchor, positive, and negative episodes using base provider
    /// 2. Iteratively optimizes adaptation matrix using gradient descent
    /// 3. Minimizes triplet loss: max(0, d(anchor, positive) - d(anchor, negative) + margin)
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
    /// Returns error if training fails or embeddings cannot be generated
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
    /// embeddings.train_adapter(TaskType::Debugging, &pairs).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn train_adapter(
        &mut self,
        task_type: TaskType,
        contrastive_pairs: &[ContrastivePair],
    ) -> Result<()> {
        if contrastive_pairs.is_empty() {
            anyhow::bail!("Cannot train adapter with empty training set");
        }

        // Get embedding dimension from base provider
        let dim = self.base_embeddings.embedding_dimension();

        // Initialize with identity matrix
        let mut adapter = TaskAdapter::new_identity(task_type, dim);

        // Training hyperparameters
        const LEARNING_RATE: f32 = 0.01;
        const EPOCHS: usize = 100;
        const MARGIN: f32 = 0.5; // Triplet loss margin

        // Embed all episodes once to avoid redundant computation
        let mut embedded_pairs = Vec::new();
        for pair in contrastive_pairs {
            let anchor_emb = self
                .base_embeddings
                .embed_text(&pair.anchor.task_description)
                .await?;
            let positive_emb = self
                .base_embeddings
                .embed_text(&pair.positive.task_description)
                .await?;
            let negative_emb = self
                .base_embeddings
                .embed_text(&pair.negative.task_description)
                .await?;

            embedded_pairs.push((anchor_emb, positive_emb, negative_emb));
        }

        // Gradient descent optimization
        for _epoch in 0..EPOCHS {
            let mut gradient = vec![vec![0.0; dim]; dim];

            // Compute gradient across all training pairs
            for (anchor_emb, positive_emb, negative_emb) in &embedded_pairs {
                Self::update_gradient_for_triplet(
                    &adapter,
                    anchor_emb,
                    positive_emb,
                    negative_emb,
                    &mut gradient,
                    dim,
                    MARGIN,
                );
            }

            // Apply gradient descent update
            Self::apply_gradient_update(
                &mut adapter.adaptation_matrix,
                &gradient,
                LEARNING_RATE,
                contrastive_pairs.len(),
                dim,
            );
        }

        // Update training count
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
pub mod tests;
