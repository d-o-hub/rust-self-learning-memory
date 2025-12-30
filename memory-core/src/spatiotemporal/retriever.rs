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

use crate::episode::Episode;
use crate::types::TaskType;
use anyhow::Result;
use chrono::Utc;
use tracing::{debug, instrument};
use uuid::Uuid;

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
    /// Create a new hierarchical retriever with default settings.
    ///
    /// Default settings:
    /// - `temporal_bias_weight`: 0.3 (30% weight on recency)
    /// - `max_clusters_to_search`: 5 clusters
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::spatiotemporal::HierarchicalRetriever;
    ///
    /// let retriever = HierarchicalRetriever::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            temporal_bias_weight: 0.3,
            max_clusters_to_search: 5,
        }
    }

    /// Create a retriever with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `temporal_bias_weight` - Weight for temporal bias (0.0-1.0)
    /// * `max_clusters_to_search` - Maximum clusters to search
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::spatiotemporal::HierarchicalRetriever;
    ///
    /// // More aggressive temporal bias
    /// let retriever = HierarchicalRetriever::with_config(0.5, 10);
    /// ```
    #[must_use]
    pub fn with_config(temporal_bias_weight: f32, max_clusters_to_search: usize) -> Self {
        Self {
            temporal_bias_weight: temporal_bias_weight.clamp(0.0, 1.0),
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
        query_domain = ?query.domain,
        query_task_type = ?query.task_type,
        total_episodes = all_episodes.len(),
        limit = query.limit
    ))]
    pub async fn retrieve(
        &self,
        query: &RetrievalQuery,
        all_episodes: &[Episode],
    ) -> Result<Vec<ScoredEpisode>> {
        debug!("Starting hierarchical retrieval");

        // Level 1: Filter by domain
        let domain_candidates = self.filter_by_domain(all_episodes, query);
        debug!(
            domain_filtered = domain_candidates.len(),
            "Level 1: Domain filtering complete"
        );

        // Level 2: Filter by task type
        let task_type_candidates = self.filter_by_task_type(&domain_candidates, query);
        debug!(
            task_type_filtered = task_type_candidates.len(),
            "Level 2: Task type filtering complete"
        );

        // Level 3: Select temporal clusters (recent bias)
        let temporal_candidates = self.select_temporal_clusters(&task_type_candidates, query);
        debug!(
            temporal_selected = temporal_candidates.len(),
            "Level 3: Temporal cluster selection complete"
        );

        // Level 4: Fine-grained similarity scoring
        let scored = self.score_episodes(&temporal_candidates, query);
        debug!(
            scored_count = scored.len(),
            "Level 4: Similarity scoring complete"
        );

        // Combine scores and rank
        let mut ranked = self.rank_by_combined_score(scored);

        // Limit results
        ranked.truncate(query.limit);

        debug!(
            final_count = ranked.len(),
            "Hierarchical retrieval complete"
        );

        Ok(ranked)
    }

    /// Level 1: Filter episodes by domain.
    ///
    /// If domain is specified in query, only return episodes from that domain.
    /// Otherwise, return all episodes.
    fn filter_by_domain<'a>(
        &self,
        episodes: &'a [Episode],
        query: &RetrievalQuery,
    ) -> Vec<&'a Episode> {
        if let Some(ref domain) = query.domain {
            episodes
                .iter()
                .filter(|ep| ep.context.domain == *domain)
                .collect()
        } else {
            episodes.iter().collect()
        }
    }

    /// Level 2: Filter episodes by task type.
    ///
    /// If task type is specified in query, only return episodes of that type.
    /// Otherwise, return all candidates.
    fn filter_by_task_type<'a>(
        &self,
        candidates: &[&'a Episode],
        query: &RetrievalQuery,
    ) -> Vec<&'a Episode> {
        if let Some(task_type) = query.task_type {
            candidates
                .iter()
                .filter(|ep| ep.task_type == task_type)
                .copied()
                .collect()
        } else {
            candidates.to_vec()
        }
    }

    /// Level 3: Select temporal clusters with recency bias.
    ///
    /// Groups episodes into temporal clusters and selects the most recent ones.
    /// Applies temporal bias to favor recent episodes in scoring.
    fn select_temporal_clusters<'a>(
        &self,
        candidates: &[&'a Episode],
        _query: &RetrievalQuery,
    ) -> Vec<&'a Episode> {
        if candidates.is_empty() {
            return vec![];
        }

        // Sort by recency (newest first)
        let mut sorted: Vec<_> = candidates.to_vec();
        sorted.sort_by(|a, b| b.start_time.cmp(&a.start_time));

        // For now, take top-k most recent episodes
        // Future: implement proper temporal clustering (weekly/monthly buckets)
        let cluster_size = candidates.len() / self.max_clusters_to_search.max(1);
        let take_count = cluster_size.max(10).min(candidates.len());

        sorted.into_iter().take(take_count).collect()
    }

    /// Level 4: Score episodes by similarity.
    ///
    /// Calculates fine-grained similarity scores for candidates.
    /// Uses embedding-based similarity when available, falls back to text similarity.
    fn score_episodes(
        &self,
        candidates: &[&Episode],
        query: &RetrievalQuery,
    ) -> Vec<ScoredEpisode> {
        let now = Utc::now();

        let scored: Vec<ScoredEpisode> = candidates
            .iter()
            .map(|episode| {
                // Level 1 score: Domain match
                let level_1_score = if let Some(ref domain) = query.domain {
                    if episode.context.domain == *domain {
                        1.0
                    } else {
                        0.0
                    }
                } else {
                    0.5 // Neutral if no domain specified
                };

                // Level 2 score: Task type match
                let level_2_score = if let Some(task_type) = query.task_type {
                    if episode.task_type == task_type {
                        1.0
                    } else {
                        0.0
                    }
                } else {
                    0.5 // Neutral if no task type specified
                };

                // Level 3 score: Temporal proximity (newer is better)
                let age_seconds = (now - episode.start_time).num_seconds().max(0) as f32;
                let max_age_seconds = 30.0 * 24.0 * 3600.0; // 30 days
                let level_3_score = 1.0 - (age_seconds / max_age_seconds).min(1.0);

                // Level 4 score: Embedding similarity (if available) or text similarity
                let level_4_score = if let Some(ref query_emb) = query.query_embedding {
                    // Generate episode embedding (simple metadata-based for now)
                    let episode_emb = generate_episode_embedding(episode);

                    // Calculate cosine similarity between query and episode embeddings
                    // Note: cosine_similarity returns a value in [-1, 1], normalize to [0, 1]
                    let similarity = crate::embeddings::cosine_similarity(query_emb, &episode_emb);
                    (similarity + 1.0) / 2.0 // Normalize from [-1, 1] to [0, 1]
                } else {
                    // Fallback to text-based similarity
                    calculate_text_similarity(
                        &query.query_text.to_lowercase(),
                        &episode.task_description.to_lowercase(),
                    )
                };

                // Combined relevance score
                // Weights: domain (0.3), task_type (0.3), temporal (temporal_bias_weight), similarity (1 - temporal_bias - 0.6)
                let temporal_weight = self.temporal_bias_weight;
                let similarity_weight = 1.0 - temporal_weight - 0.6;

                let relevance_score = 0.3 * level_1_score
                    + 0.3 * level_2_score
                    + temporal_weight * level_3_score
                    + similarity_weight.max(0.1) * level_4_score;

                ScoredEpisode {
                    episode_id: episode.episode_id,
                    relevance_score,
                    level_1_score,
                    level_2_score,
                    level_3_score,
                    level_4_score,
                }
            })
            .collect();

        scored
    }

    /// Rank scored episodes by combined relevance score.
    ///
    /// Sorts episodes in descending order of relevance.
    fn rank_by_combined_score(&self, mut scored: Vec<ScoredEpisode>) -> Vec<ScoredEpisode> {
        scored.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        scored
    }
}

/// Query for hierarchical retrieval.
///
/// Specifies the search criteria and parameters for retrieving relevant episodes.
///
/// # Fields
///
/// * `query_text` - Text description of the query task
/// * `query_embedding` - Pre-computed embedding (optional, for future use)
/// * `domain` - Filter by domain (optional)
/// * `task_type` - Filter by task type (optional)
/// * `limit` - Maximum number of results to return
///
/// # Examples
///
/// ```
/// use memory_core::spatiotemporal::RetrievalQuery;
/// use memory_core::TaskType;
///
/// // Domain-specific query
/// let query = RetrievalQuery {
///     query_text: "Implement OAuth2".to_string(),
///     query_embedding: None,
///     domain: Some("web-api".to_string()),
///     task_type: Some(TaskType::CodeGeneration),
///     limit: 5,
/// };
///
/// // General query
/// let general_query = RetrievalQuery {
///     query_text: "Fix bug in authentication".to_string(),
///     query_embedding: None,
///     domain: None,
///     task_type: None,
///     limit: 10,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct RetrievalQuery {
    /// Text description of what to search for
    pub query_text: String,
    /// Pre-computed query embedding (optional)
    pub query_embedding: Option<Vec<f32>>,
    /// Filter by domain (optional)
    pub domain: Option<String>,
    /// Filter by task type (optional)
    pub task_type: Option<TaskType>,
    /// Maximum number of results
    pub limit: usize,
}

/// Episode with hierarchical relevance scores.
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
/// use memory_core::spatiotemporal::retriever::ScoredEpisode;
/// use uuid::Uuid;
///
/// let scored = ScoredEpisode {
///     episode_id: Uuid::new_v4(),
///     relevance_score: 0.85,
///     level_1_score: 1.0,  // Perfect domain match
///     level_2_score: 1.0,  // Perfect task type match
///     level_3_score: 0.9,  // Very recent
///     level_4_score: 0.75, // Good similarity
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ScoredEpisode {
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

/// Generate a simple embedding for an episode based on its metadata.
///
/// Creates a feature vector encoding episode characteristics for similarity
/// comparison. This is a lightweight alternative to full semantic embeddings.
///
/// # Arguments
///
/// * `episode` - The episode to generate an embedding for
///
/// # Returns
///
/// Feature vector with 10 dimensions encoding episode properties
fn generate_episode_embedding(episode: &Episode) -> Vec<f32> {
    let mut embedding = Vec::with_capacity(10);

    // Domain hash
    let domain_hash = episode
        .context
        .domain
        .chars()
        .fold(0u32, |acc, c| acc.wrapping_add(c as u32));
    embedding.push((domain_hash % 100) as f32 / 100.0);

    // Task type encoding
    embedding.push(match episode.task_type {
        crate::types::TaskType::CodeGeneration => 0.9,
        crate::types::TaskType::Analysis => 0.7,
        crate::types::TaskType::Testing => 0.5,
        crate::types::TaskType::Debugging => 0.3,
        crate::types::TaskType::Refactoring => 0.2,
        crate::types::TaskType::Documentation => 0.1,
        crate::types::TaskType::Other => 0.0,
    });

    // Complexity encoding
    embedding.push(match episode.context.complexity {
        crate::types::ComplexityLevel::Simple => 0.2,
        crate::types::ComplexityLevel::Moderate => 0.5,
        crate::types::ComplexityLevel::Complex => 0.8,
    });

    // Language/framework presence
    embedding.push(if episode.context.language.is_some() {
        1.0
    } else {
        0.0
    });
    embedding.push(if episode.context.framework.is_some() {
        1.0
    } else {
        0.0
    });

    // Number of steps (normalized)
    let step_count = episode.steps.len().min(50) as f32 / 50.0;
    embedding.push(step_count);

    // Reward component (if available)
    let reward_value = episode.reward.as_ref().map_or(0.5, |r| r.total.min(1.0));
    embedding.push(reward_value);

    // Duration component
    if let Some(end) = episode.end_time {
        let duration = end - episode.start_time;
        let duration_secs = duration.num_seconds().clamp(0, 3600) as f32 / 3600.0;
        embedding.push(duration_secs);
    } else {
        embedding.push(0.5);
    }

    // Tag count (normalized)
    let tag_count = episode.context.tags.len().min(10) as f32 / 10.0;
    embedding.push(tag_count);

    // Outcome encoding
    embedding.push(match &episode.outcome {
        Some(crate::types::TaskOutcome::Success { .. }) => 1.0,
        Some(crate::types::TaskOutcome::PartialSuccess { .. }) => 0.5,
        Some(crate::types::TaskOutcome::Failure { .. }) => 0.0,
        None => 0.5,
    });

    embedding
}

/// Calculate text similarity using keyword overlap.
///
/// Simple similarity metric based on common words between texts.
/// Used as fallback when embeddings are not available.
///
/// # Arguments
///
/// * `query` - Query text (normalized/lowercase)
/// * `text` - Episode text (normalized/lowercase)
///
/// # Returns
///
/// Similarity score between 0.0 and 1.0
fn calculate_text_similarity(query: &str, text: &str) -> f32 {
    let query_words: Vec<&str> = query.split_whitespace().collect();
    let text_words: Vec<&str> = text.split_whitespace().collect();

    if query_words.is_empty() || text_words.is_empty() {
        return 0.0;
    }

    // Count common words
    let common_count = query_words
        .iter()
        .filter(|word| text_words.contains(word))
        .count();

    // Jaccard similarity
    let union_count = query_words.len() + text_words.len() - common_count;
    if union_count == 0 {
        0.0
    } else {
        common_count as f32 / union_count as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ComplexityLevel;
    use crate::types::TaskContext;
    use chrono::Duration;

    fn create_test_episode(
        domain: &str,
        task_type: TaskType,
        description: &str,
        age_days: i64,
    ) -> Episode {
        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: None,
            complexity: ComplexityLevel::Moderate,
            domain: domain.to_string(),
            tags: vec![],
        };

        let mut episode = Episode::new(description.to_string(), context, task_type);
        episode.start_time = Utc::now() - Duration::days(age_days);
        episode
    }

    #[test]
    fn test_retriever_creation() {
        let retriever = HierarchicalRetriever::new();
        assert_eq!(retriever.temporal_bias_weight, 0.3);
        assert_eq!(retriever.max_clusters_to_search, 5);

        let custom = HierarchicalRetriever::with_config(0.5, 10);
        assert_eq!(custom.temporal_bias_weight, 0.5);
        assert_eq!(custom.max_clusters_to_search, 10);
    }

    #[test]
    fn test_domain_filtering() {
        let retriever = HierarchicalRetriever::new();

        let episodes = vec![
            create_test_episode("web-api", TaskType::CodeGeneration, "API endpoint", 1),
            create_test_episode("data-science", TaskType::Analysis, "Data analysis", 1),
            create_test_episode("web-api", TaskType::Testing, "API test", 1),
        ];

        let query = RetrievalQuery {
            query_text: "test".to_string(),
            query_embedding: None,
            domain: Some("web-api".to_string()),
            task_type: None,
            limit: 10,
        };

        let filtered = retriever.filter_by_domain(&episodes, &query);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|ep| ep.context.domain == "web-api"));
    }

    #[test]
    fn test_domain_filtering_no_filter() {
        let retriever = HierarchicalRetriever::new();

        let episodes = vec![
            create_test_episode("web-api", TaskType::CodeGeneration, "API endpoint", 1),
            create_test_episode("data-science", TaskType::Analysis, "Data analysis", 1),
        ];

        let query = RetrievalQuery {
            query_text: "test".to_string(),
            query_embedding: None,
            domain: None,
            task_type: None,
            limit: 10,
        };

        let filtered = retriever.filter_by_domain(&episodes, &query);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_task_type_filtering() {
        let retriever = HierarchicalRetriever::new();

        let episodes = [
            create_test_episode("web-api", TaskType::CodeGeneration, "Write code", 1),
            create_test_episode("web-api", TaskType::Testing, "Write tests", 1),
            create_test_episode("web-api", TaskType::CodeGeneration, "More code", 1),
        ];

        let candidates: Vec<&Episode> = episodes.iter().collect();

        let query = RetrievalQuery {
            query_text: "code".to_string(),
            query_embedding: None,
            domain: None,
            task_type: Some(TaskType::CodeGeneration),
            limit: 10,
        };

        let filtered = retriever.filter_by_task_type(&candidates, &query);
        assert_eq!(filtered.len(), 2);
        assert!(filtered
            .iter()
            .all(|ep| ep.task_type == TaskType::CodeGeneration));
    }

    #[test]
    fn test_task_type_filtering_no_filter() {
        let retriever = HierarchicalRetriever::new();

        let episodes = [
            create_test_episode("web-api", TaskType::CodeGeneration, "Write code", 1),
            create_test_episode("web-api", TaskType::Testing, "Write tests", 1),
        ];

        let candidates: Vec<&Episode> = episodes.iter().collect();

        let query = RetrievalQuery {
            query_text: "code".to_string(),
            query_embedding: None,
            domain: None,
            task_type: None,
            limit: 10,
        };

        let filtered = retriever.filter_by_task_type(&candidates, &query);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_temporal_clustering_favors_recent() {
        let retriever = HierarchicalRetriever::new();

        let episodes = [
            create_test_episode("web-api", TaskType::CodeGeneration, "Old", 30),
            create_test_episode("web-api", TaskType::CodeGeneration, "Recent", 1),
            create_test_episode("web-api", TaskType::CodeGeneration, "Very old", 60),
        ];

        let candidates: Vec<&Episode> = episodes.iter().collect();

        let query = RetrievalQuery {
            query_text: "test".to_string(),
            query_embedding: None,
            domain: None,
            task_type: None,
            limit: 10,
        };

        let selected = retriever.select_temporal_clusters(&candidates, &query);

        // Should select all (only 3 episodes)
        assert_eq!(selected.len(), 3);

        // Most recent should be first
        assert_eq!(selected[0].task_description, "Recent");
    }

    #[test]
    fn test_scoring_domain_match() {
        let retriever = HierarchicalRetriever::new();

        let episodes = [
            create_test_episode("web-api", TaskType::CodeGeneration, "API endpoint", 1),
            create_test_episode("data-science", TaskType::CodeGeneration, "Data pipeline", 1),
        ];

        let candidates: Vec<&Episode> = episodes.iter().collect();

        let query = RetrievalQuery {
            query_text: "endpoint".to_string(),
            query_embedding: None,
            domain: Some("web-api".to_string()),
            task_type: None,
            limit: 10,
        };

        let scored = retriever.score_episodes(&candidates, &query);

        // web-api episode should score higher due to domain match
        assert_eq!(scored.len(), 2);
        assert!(scored[0].level_1_score > scored[1].level_1_score);
    }

    #[test]
    fn test_scoring_task_type_match() {
        let retriever = HierarchicalRetriever::new();

        let episodes = [
            create_test_episode("web-api", TaskType::CodeGeneration, "Write code", 1),
            create_test_episode("web-api", TaskType::Testing, "Write tests", 1),
        ];

        let candidates: Vec<&Episode> = episodes.iter().collect();

        let query = RetrievalQuery {
            query_text: "write".to_string(),
            query_embedding: None,
            domain: None,
            task_type: Some(TaskType::CodeGeneration),
            limit: 10,
        };

        let scored = retriever.score_episodes(&candidates, &query);

        // CodeGeneration episode should score higher due to task type match
        assert_eq!(scored.len(), 2);
        assert!(scored[0].level_2_score > scored[1].level_2_score);
    }

    #[test]
    fn test_scoring_temporal_proximity() {
        let retriever = HierarchicalRetriever::new();

        let episodes = [
            create_test_episode("web-api", TaskType::CodeGeneration, "Recent task", 1),
            create_test_episode("web-api", TaskType::CodeGeneration, "Old task", 30),
        ];

        let candidates: Vec<&Episode> = episodes.iter().collect();

        let query = RetrievalQuery {
            query_text: "task".to_string(),
            query_embedding: None,
            domain: None,
            task_type: None,
            limit: 10,
        };

        let scored = retriever.score_episodes(&candidates, &query);

        // Recent episode should have higher temporal score
        assert_eq!(scored.len(), 2);
        assert!(scored[0].level_3_score > scored[1].level_3_score);
    }

    #[test]
    fn test_text_similarity() {
        assert_eq!(
            calculate_text_similarity("implement auth", "implement auth"),
            1.0
        );

        let sim1 = calculate_text_similarity("implement oauth", "implement authentication");
        assert!(sim1 > 0.0 && sim1 < 1.0);

        let sim2 = calculate_text_similarity("web api", "data science");
        assert_eq!(sim2, 0.0);

        let sim3 = calculate_text_similarity("", "test");
        assert_eq!(sim3, 0.0);
    }

    #[tokio::test]
    async fn test_full_retrieval_workflow() {
        let retriever = HierarchicalRetriever::new();

        let episodes = vec![
            create_test_episode(
                "web-api",
                TaskType::CodeGeneration,
                "implement oauth2 api",
                1,
            ),
            create_test_episode(
                "web-api",
                TaskType::CodeGeneration,
                "create rest endpoint",
                5,
            ),
            create_test_episode("data-science", TaskType::Analysis, "analyze data trends", 2),
            create_test_episode("web-api", TaskType::Testing, "test authentication", 3),
        ];

        let query = RetrievalQuery {
            query_text: "implement authentication api".to_string(),
            query_embedding: None,
            domain: Some("web-api".to_string()),
            task_type: Some(TaskType::CodeGeneration),
            limit: 2,
        };

        let scored = retriever.retrieve(&query, &episodes).await.unwrap();

        // Should return 2 results (limit)
        assert_eq!(scored.len(), 2);

        // Results should be sorted by relevance
        assert!(scored[0].relevance_score >= scored[1].relevance_score);

        // All results should match domain and task type
        for result in &scored {
            let episode = episodes
                .iter()
                .find(|ep| ep.episode_id == result.episode_id)
                .unwrap();
            assert_eq!(episode.context.domain, "web-api");
            assert_eq!(episode.task_type, TaskType::CodeGeneration);
        }
    }

    #[tokio::test]
    async fn test_retrieval_with_no_filters() {
        let retriever = HierarchicalRetriever::new();

        let episodes = vec![
            create_test_episode("web-api", TaskType::CodeGeneration, "implement api", 1),
            create_test_episode("data-science", TaskType::Analysis, "analyze data", 2),
        ];

        let query = RetrievalQuery {
            query_text: "implement".to_string(),
            query_embedding: None,
            domain: None,
            task_type: None,
            limit: 10,
        };

        let scored = retriever.retrieve(&query, &episodes).await.unwrap();

        // Should return all episodes
        assert_eq!(scored.len(), 2);
    }

    #[tokio::test]
    async fn test_retrieval_empty_episodes() {
        let retriever = HierarchicalRetriever::new();

        let episodes = vec![];

        let query = RetrievalQuery {
            query_text: "test".to_string(),
            query_embedding: None,
            domain: Some("web-api".to_string()),
            task_type: None,
            limit: 5,
        };

        let scored = retriever.retrieve(&query, &episodes).await.unwrap();

        assert_eq!(scored.len(), 0);
    }

    #[test]
    fn test_combined_score_calculation() {
        let retriever = HierarchicalRetriever::new();

        let episodes = [create_test_episode(
            "web-api",
            TaskType::CodeGeneration,
            "implement authentication",
            1,
        )];

        let candidates: Vec<&Episode> = episodes.iter().collect();

        let query = RetrievalQuery {
            query_text: "implement authentication".to_string(),
            query_embedding: None,
            domain: Some("web-api".to_string()),
            task_type: Some(TaskType::CodeGeneration),
            limit: 1,
        };

        let scored = retriever.score_episodes(&candidates, &query);

        assert_eq!(scored.len(), 1);

        // Perfect match should have high relevance score
        let result = &scored[0];
        assert!(result.relevance_score > 0.7);
        assert!(result.level_1_score > 0.9); // Domain match
        assert!(result.level_2_score > 0.9); // Task type match
        assert!(result.level_3_score > 0.9); // Recent (1 day old)
        assert!(result.level_4_score > 0.5); // Text similarity
    }

    #[test]
    fn test_temporal_bias_weight_effect() {
        // High temporal bias
        let high_bias = HierarchicalRetriever::with_config(0.7, 5);

        let episodes = [
            create_test_episode("web-api", TaskType::CodeGeneration, "old task", 30),
            create_test_episode("web-api", TaskType::CodeGeneration, "new task", 1),
        ];

        let candidates: Vec<&Episode> = episodes.iter().collect();

        let query = RetrievalQuery {
            query_text: "task".to_string(),
            query_embedding: None,
            domain: Some("web-api".to_string()),
            task_type: Some(TaskType::CodeGeneration),
            limit: 2,
        };

        let scored = high_bias.score_episodes(&candidates, &query);

        // With high temporal bias, recent episode should score significantly higher
        assert!(scored[1].relevance_score > scored[0].relevance_score);
    }

    #[test]
    fn test_ranking_sorts_by_relevance() {
        let retriever = HierarchicalRetriever::new();

        let scored = vec![
            ScoredEpisode {
                episode_id: Uuid::new_v4(),
                relevance_score: 0.5,
                level_1_score: 0.5,
                level_2_score: 0.5,
                level_3_score: 0.5,
                level_4_score: 0.5,
            },
            ScoredEpisode {
                episode_id: Uuid::new_v4(),
                relevance_score: 0.9,
                level_1_score: 1.0,
                level_2_score: 1.0,
                level_3_score: 0.9,
                level_4_score: 0.8,
            },
            ScoredEpisode {
                episode_id: Uuid::new_v4(),
                relevance_score: 0.7,
                level_1_score: 0.8,
                level_2_score: 0.7,
                level_3_score: 0.6,
                level_4_score: 0.7,
            },
        ];

        let ranked = retriever.rank_by_combined_score(scored);

        assert_eq!(ranked.len(), 3);
        assert!(ranked[0].relevance_score >= ranked[1].relevance_score);
        assert!(ranked[1].relevance_score >= ranked[2].relevance_score);
        assert_eq!(ranked[0].relevance_score, 0.9);
    }
}
