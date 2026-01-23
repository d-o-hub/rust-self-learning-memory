//! Hierarchical Retrieval Implementation
//!
//! Implements a 4-level coarse-to-fine retrieval strategy for episodic memory:
//!
//! 1. **Level 1: Domain Filtering** - Match episodes by domain (e.g., "web-api")
//! 2. **Level 2: Task Type Filtering** - Match by task type (e.g., `CodeGeneration`)
//! 3. **Level 3: Temporal Clustering** - Select recent clusters with temporal bias
//! 4. **Level 4: Similarity Scoring** - Fine-grained embedding similarity
//!
//! The retriever combines scores from all levels with configurable weights to produce
//! a final relevance ranking.

mod scoring;
mod types;

#[cfg(test)]
mod tests;

pub use types::{HierarchicalScore, RetrievalQuery};

use crate::episode::Episode;
use anyhow::Result;
use std::sync::Arc;
use tracing::{debug, instrument};

/// Hierarchical retriever for spatiotemporal episodic memory.
///
/// Implements a 4-level coarse-to-fine retrieval strategy that progressively
/// narrows down the search space from broad domain matching to fine-grained
/// similarity scoring.
///
/// # Configuration
///
/// * `temporal_bias_weight` - How much to favor recent episodes (0.0-1.0, default: 0.3)
/// * `max_clusters_to_search` - Maximum temporal clusters to search (default: 5)
///
/// # Examples
///
/// ```no_run
/// use memory_core::spatiotemporal::HierarchicalRetriever;
///
/// // Create with default settings
/// let retriever = HierarchicalRetriever::new();
///
/// // Create with custom settings
/// let custom_retriever = HierarchicalRetriever::with_config(0.4, 10);
/// ```
#[derive(Debug, Clone)]
pub struct HierarchicalRetriever {
    /// Weight for temporal bias (0.0 = no bias, 1.0 = only recent)
    temporal_bias_weight: f32,
    /// Maximum number of temporal clusters to search
    max_clusters_to_search: usize,
}

impl Default for HierarchicalRetriever {
    fn default() -> Self {
        Self::new()
    }
}

impl HierarchicalRetriever {
    /// Create a new hierarchical retriever with default configuration.
    ///
    /// Default values:
    /// - `temporal_bias_weight`: 0.3 (30% weight to recency)
    /// - `max_clusters_to_search`: 5 clusters
    #[must_use]
    pub fn new() -> Self {
        Self {
            temporal_bias_weight: 0.3,
            max_clusters_to_search: 5,
        }
    }

    /// Create a hierarchical retriever with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `temporal_bias_weight` - Weight for temporal bias (0.0-1.0)
    /// * `max_clusters_to_search` - Maximum temporal clusters to consider
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::spatiotemporal::HierarchicalRetriever;
    ///
    /// // Favor very recent episodes
    /// let recent_focused = HierarchicalRetriever::with_config(0.5, 3);
    ///
    /// // Broader temporal search
    /// let broad_search = HierarchicalRetriever::with_config(0.2, 10);
    /// ```
    #[must_use]
    pub fn with_config(temporal_bias_weight: f32, max_clusters_to_search: usize) -> Self {
        Self {
            temporal_bias_weight,
            max_clusters_to_search,
        }
    }

    /// Execute hierarchical retrieval across all 4 levels.
    ///
    /// Performs a coarse-to-fine search through the episode space:
    ///
    /// 1. Filter by domain (if specified)
    /// 2. Filter by task type (if specified)
    /// 3. Select temporal clusters (recent bias)
    /// 4. Score episodes by similarity
    ///
    /// # Arguments
    ///
    /// * `query` - The retrieval query specifying search criteria
    /// * `all_episodes` - All available episodes to search through
    ///
    /// # Returns
    ///
    /// Vector of scored episodes ranked by relevance (highest first)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use memory_core::spatiotemporal::{HierarchicalRetriever, RetrievalQuery};
    /// # use memory_core::TaskType;
    /// # async fn example() -> anyhow::Result<()> {
    /// let retriever = HierarchicalRetriever::new();
    ///
    /// let query = RetrievalQuery {
    ///     query_text: "Implement authentication".to_string(),
    ///     query_embedding: None,
    ///     domain: Some("web-api".to_string()),
    ///     task_type: Some(TaskType::CodeGeneration),
    ///     limit: 5,
    /// };
    ///
    /// // let episodes = vec![/* ... */];
    /// // let scored = retriever.retrieve(&query, &episodes).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, all_episodes), fields(
        query_text = %query.query_text,
        query_domain = ?query.domain,
        query_task_type = ?query.task_type,
        total_episodes = all_episodes.len(),
        limit = query.limit
    ))]
    pub async fn retrieve(
        &self,
        query: &RetrievalQuery,
        all_episodes: &[Arc<Episode>],
    ) -> Result<Vec<HierarchicalScore>> {
        debug!("Starting hierarchical retrieval");

        // Level 1: Domain filtering
        let domain_filtered = self.filter_by_domain(all_episodes, query);
        debug!(
            "Level 1 (domain filter): {} episodes",
            domain_filtered.len()
        );

        // Level 2: Task type filtering
        let task_filtered = self.filter_by_task_type(&domain_filtered, query);
        debug!(
            "Level 2 (task type filter): {} episodes",
            task_filtered.len()
        );

        // Level 3: Temporal clustering
        let temporal_candidates = self.select_temporal_clusters(&task_filtered, query);
        debug!(
            "Level 3 (temporal clusters): {} episodes",
            temporal_candidates.len()
        );

        // Level 4: Similarity scoring
        let scored = self.score_episodes(&temporal_candidates, query);
        debug!("Level 4 (similarity scoring): {} episodes", scored.len());

        // Rank by combined score
        let mut ranked = self.rank_by_combined_score(scored);

        // Apply limit
        ranked.truncate(query.limit);

        debug!(
            "Hierarchical retrieval complete: {} results returned",
            ranked.len()
        );

        Ok(ranked)
    }
}
