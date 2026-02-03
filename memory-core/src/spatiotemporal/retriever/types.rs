//! Types for hierarchical retrieval
//!
//! Defines the query and result types used in spatiotemporal retrieval.

use crate::episode::Episode;
use crate::types::TaskType;
use uuid::Uuid;

/// Query for hierarchical retrieval.
///
/// Specifies the search criteria and parameters for retrieving relevant episodes.
///
/// # Fields
///
/// * `query_text` - Text description of the query task
/// * `query_embedding` - Pre-computed embedding (optional, for semantic search)
/// * `domain` - Filter by domain (optional)
/// * `task_type` - Filter by task type (optional)
/// * `limit` - Maximum number of results to return
/// * `episode_embeddings` - Pre-loaded episode embeddings for semantic similarity
///
/// # Examples
///
/// ```
/// use memory_core::spatiotemporal::retriever::RetrievalQuery;
/// use memory_core::types::TaskType;
/// use std::collections::HashMap;
/// use uuid::Uuid;
///
/// let query = RetrievalQuery {
///     query_text: "Implement authentication".to_string(),
///     query_embedding: None,
///     domain: Some("web-api".to_string()),
///     task_type: Some(TaskType::CodeGeneration),
///     limit: 5,
///     episode_embeddings: HashMap::new(),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct RetrievalQuery {
    /// Text description of the query task
    pub query_text: String,
    /// Optional pre-computed embedding
    pub query_embedding: Option<Vec<f32>>,
    /// Optional domain filter
    pub domain: Option<String>,
    /// Optional task type filter
    pub task_type: Option<TaskType>,
    /// Maximum number of results
    pub limit: usize,
    /// Pre-loaded episode embeddings for semantic similarity scoring
    /// Maps `episode_id` to embedding vector
    pub episode_embeddings: std::collections::HashMap<Uuid, Vec<f32>>,
}

/// Episode with hierarchical relevance scores from retrieval.
///
/// Contains the episode ID and scores from all 4 retrieval levels,
/// plus the combined relevance score.
///
/// # Fields
///
/// * `episode_id` - Unique identifier of the episode
/// * `relevance_score` - Combined score across all levels (0.0-1.0)
/// * `level_1_score` - Domain match score (0.0-1.0)
/// * `level_2_score` - Task type match score (0.0-1.0)
/// * `level_3_score` - Temporal proximity score (0.0-1.0)
/// * `level_4_score` - Embedding similarity score (0.0-1.0)
///
/// # Examples
///
/// ```
/// use memory_core::spatiotemporal::retriever::HierarchicalScore;
/// use uuid::Uuid;
///
/// let scored = HierarchicalScore {
///     episode_id: Uuid::new_v4(),
///     relevance_score: 0.85,
///     level_1_score: 1.0,  // Perfect domain match
///     level_2_score: 1.0,  // Perfect task type match
///     level_3_score: 0.9,  // Very recent
///     level_4_score: 0.75, // Good similarity
/// };
/// ```
#[derive(Debug, Clone)]
pub struct HierarchicalScore {
    /// Episode unique identifier
    pub episode_id: Uuid,
    /// Combined relevance score (0.0-1.0)
    pub relevance_score: f32,
    /// Level 1: Domain match score (0.0-1.0)
    pub level_1_score: f32,
    /// Level 2: Task type match score (0.0-1.0)
    pub level_2_score: f32,
    /// Level 3: Temporal proximity score (0.0-1.0)
    pub level_3_score: f32,
    /// Level 4: Embedding similarity score (0.0-1.0)
    pub level_4_score: f32,
}

/// Get episode embedding from pre-loaded map or generate fallback.
///
/// This function attempts to use a pre-loaded semantic embedding for an episode.
/// If no pre-loaded embedding is available, it falls back to a simple feature-based embedding.
///
/// # Arguments
///
/// * `episode` - The episode to get an embedding for
/// * `episode_embeddings` - Pre-loaded embeddings map
///
/// # Returns
///
/// A vector of floating-point values representing the episode embedding
pub(crate) fn get_or_generate_episode_embedding(
    episode: &Episode,
    episode_embeddings: &std::collections::HashMap<Uuid, Vec<f32>>,
) -> Vec<f32> {
    // Try to use pre-loaded semantic embedding first
    if let Some(embedding) = episode_embeddings.get(&episode.episode_id) {
        tracing::trace!(
            episode_id = %episode.episode_id,
            embedding_dim = embedding.len(),
            "Using pre-loaded semantic embedding"
        );
        return embedding.clone();
    }

    // Fallback to simple feature extraction
    tracing::trace!(
        episode_id = %episode.episode_id,
        "No pre-loaded embedding found, using fallback feature extraction"
    );

    let task_len = episode.task_description.len() as f32 / 100.0; // Normalize
    let domain_hash = episode
        .context
        .domain
        .chars()
        .map(|c| c as u32)
        .sum::<u32>() as f32
        / 1000.0;
    let steps_count = episode.steps.len() as f32 / 10.0;

    vec![task_len, domain_hash, steps_count]
}

/// Calculate text similarity between query and episode text.
///
/// Uses a simple word overlap metric:
/// `similarity = (common_words) / max(query_words, text_words)`
///
/// # Arguments
///
/// * `query` - Query text
/// * `text` - Text to compare against
///
/// # Returns
///
/// Similarity score between 0.0 and 1.0
pub(crate) fn calculate_text_similarity(query: &str, text: &str) -> f32 {
    let query_lower = query.to_lowercase();
    let text_lower = text.to_lowercase();

    let query_words: std::collections::HashSet<_> = query_lower.split_whitespace().collect();
    let text_words: std::collections::HashSet<_> = text_lower.split_whitespace().collect();

    let common = query_words.intersection(&text_words).count();
    let max_len = query_words.len().max(text_words.len());

    if max_len == 0 {
        0.0
    } else {
        common as f32 / max_len as f32
    }
}
