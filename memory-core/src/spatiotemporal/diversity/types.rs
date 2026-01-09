//! ScoredEpisode type definition.

use serde::{Deserialize, Serialize};

/// Episode with relevance score and embedding for diversity calculation
///
/// Represents a candidate episode with:
/// - Unique identifier
/// - Pre-computed relevance score (from retrieval)
/// - Vector embedding for similarity calculation
///
/// # Examples
///
/// ```
/// use memory_core::spatiotemporal::ScoredEpisode;
///
/// let episode = ScoredEpisode::new(
///     "episode-123".to_string(),
///     0.85,  // 85% relevance to query
///     vec![0.1, 0.9, 0.3],  // 3D embedding vector
/// );
///
/// assert_eq!(episode.relevance_score(), 0.85);
/// assert_eq!(episode.embedding().len(), 3);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoredEpisode {
    /// Unique episode identifier
    episode_id: String,
    /// Relevance score from retrieval (0.0 to 1.0)
    relevance_score: f32,
    /// Embedding vector for similarity calculation
    embedding: Vec<f32>,
}

impl ScoredEpisode {
    /// Create a new scored episode
    ///
    /// # Arguments
    ///
    /// * `episode_id` - Unique identifier for the episode
    /// * `relevance_score` - Pre-computed relevance to query (0.0-1.0)
    /// * `embedding` - Vector embedding for similarity calculation
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::spatiotemporal::ScoredEpisode;
    ///
    /// let episode = ScoredEpisode::new(
    ///     "ep1".to_string(),
    ///     0.9,
    ///     vec![1.0, 0.0, 0.0],
    /// );
    /// ```
    #[must_use]
    pub fn new(episode_id: String, relevance_score: f32, embedding: Vec<f32>) -> Self {
        Self {
            episode_id,
            relevance_score,
            embedding,
        }
    }

    /// Get the episode ID
    #[must_use]
    pub fn episode_id(&self) -> &str {
        &self.episode_id
    }

    /// Get the relevance score
    #[must_use]
    pub fn relevance_score(&self) -> f32 {
        self.relevance_score
    }

    /// Get the embedding vector
    #[must_use]
    pub fn embedding(&self) -> &[f32] {
        &self.embedding
    }
}
