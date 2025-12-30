//! Pattern clustering and deduplication
//!
//! This module provides functionality for:
//! - K-means clustering of episodes by pattern similarity
//! - Pattern deduplication based on similarity scores
//! - Pattern merging with confidence scoring

use std::collections::HashMap;

use crate::episode::Episode;
use crate::pattern::Pattern;
use crate::types::TaskContext;

/// Configuration for pattern clustering
#[derive(Debug, Clone)]
pub struct ClusteringConfig {
    /// Minimum similarity score for patterns to be considered duplicates (0.0 to 1.0)
    pub deduplication_threshold: f32,
    /// Number of clusters for k-means (if 0, auto-determine)
    pub num_clusters: usize,
    /// Maximum iterations for k-means convergence
    pub max_iterations: usize,
    /// Minimum confidence score to keep a pattern
    pub min_confidence: f32,
}

impl Default for ClusteringConfig {
    fn default() -> Self {
        Self {
            deduplication_threshold: 0.85, // 85% similarity = duplicate
            num_clusters: 0,               // Auto-determine
            max_iterations: 100,
            min_confidence: 0.5, // Keep patterns with confidence > 0.5
        }
    }
}

/// Pattern clustering engine for grouping and deduplicating patterns
pub struct PatternClusterer {
    config: ClusteringConfig,
}

impl PatternClusterer {
    /// Create a new pattern clusterer with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ClusteringConfig::default(),
        }
    }

    /// Create a clusterer with custom configuration
    #[must_use]
    pub fn with_config(config: ClusteringConfig) -> Self {
        Self { config }
    }

    /// Deduplicate patterns by merging similar ones
    ///
    /// Returns deduplicated patterns with merged statistics
    #[must_use]
    pub fn deduplicate_patterns(&self, patterns: Vec<Pattern>) -> Vec<Pattern> {
        if patterns.is_empty() {
            return Vec::new();
        }

        let mut deduplicated: Vec<Pattern> = Vec::new();

        for pattern in patterns {
            let mut merged = false;

            // Check if this pattern is similar to any existing pattern
            for existing in &mut deduplicated {
                let similarity = pattern.similarity_score(existing);

                if similarity >= self.config.deduplication_threshold {
                    // Merge the patterns
                    existing.merge_with(&pattern);
                    merged = true;
                    break;
                }
            }

            // If not merged with any existing pattern, add as new
            if !merged {
                deduplicated.push(pattern);
            }
        }

        // Filter by minimum confidence
        deduplicated
            .into_iter()
            .filter(|p| p.confidence() >= self.config.min_confidence)
            .collect()
    }

    /// Group patterns by similarity key for exact duplicates
    ///
    /// Returns a map of similarity key to list of patterns
    #[must_use]
    pub fn group_by_similarity_key(&self, patterns: Vec<Pattern>) -> HashMap<String, Vec<Pattern>> {
        let mut groups: HashMap<String, Vec<Pattern>> = HashMap::new();

        for pattern in patterns {
            let key = pattern.similarity_key();
            groups.entry(key).or_default().push(pattern);
        }

        groups
    }

    /// Cluster episodes by their patterns using k-means
    ///
    /// Groups episodes with similar patterns together
    #[must_use]
    pub fn cluster_episodes(&self, episodes: Vec<Episode>) -> Vec<EpisodeCluster> {
        if episodes.is_empty() {
            return Vec::new();
        }

        // Determine number of clusters
        let k = if self.config.num_clusters > 0 {
            self.config.num_clusters
        } else {
            // Auto-determine: sqrt(n/2)
            ((episodes.len() as f32 / 2.0).sqrt().ceil() as usize).max(1)
        };

        // Initialize clusters with random episodes as centroids
        let mut clusters = self.initialize_clusters(&episodes, k);

        // K-means iterations
        for _iteration in 0..self.config.max_iterations {
            let mut changed = false;

            // Assignment step: assign each episode to nearest cluster
            let mut new_assignments: Vec<Vec<Episode>> = vec![Vec::new(); k];

            for episode in &episodes {
                let nearest_cluster = self.find_nearest_cluster(episode, &clusters);
                new_assignments[nearest_cluster].push(episode.clone());
            }

            // Update step: recalculate centroids
            for (i, cluster) in clusters.iter_mut().enumerate() {
                if !new_assignments[i].is_empty() {
                    let new_centroid = self.calculate_centroid(&new_assignments[i]);

                    // Check if centroid changed
                    #[allow(clippy::excessive_nesting)]
                    if !self.centroids_equal(&cluster.centroid, &new_centroid) {
                        cluster.centroid = new_centroid;
                        cluster.episodes = new_assignments[i].clone();
                        changed = true;
                    }
                }
            }

            // If no changes, we've converged
            if !changed {
                break;
            }
        }

        // Filter out empty clusters
        clusters
            .into_iter()
            .filter(|c| !c.episodes.is_empty())
            .collect()
    }

    /// Find similar patterns to a given pattern
    ///
    /// Returns patterns sorted by similarity score (highest first)
    #[must_use]
    pub fn find_similar_patterns(
        &self,
        target: &Pattern,
        candidates: &[Pattern],
        limit: usize,
    ) -> Vec<(Pattern, f32)> {
        let mut similarities: Vec<(Pattern, f32)> = candidates
            .iter()
            .map(|p| (p.clone(), target.similarity_score(p)))
            .filter(|(_, score)| *score > 0.0)
            .collect();

        // Sort by similarity (highest first)
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top N
        similarities.into_iter().take(limit).collect()
    }

    /// Initialize clusters with k random episodes as centroids
    fn initialize_clusters(&self, episodes: &[Episode], k: usize) -> Vec<EpisodeCluster> {
        let mut clusters = Vec::new();

        // Use evenly spaced episodes as initial centroids
        let step = episodes.len() / k.max(1);

        for i in 0..k {
            let idx = (i * step).min(episodes.len() - 1);
            let episode = &episodes[idx];

            clusters.push(EpisodeCluster {
                centroid: ClusterCentroid::from_episode(episode),
                episodes: vec![episode.clone()],
            });
        }

        clusters
    }

    /// Find the nearest cluster for an episode
    fn find_nearest_cluster(&self, episode: &Episode, clusters: &[EpisodeCluster]) -> usize {
        let mut min_distance = f32::MAX;
        let mut nearest = 0;

        for (i, cluster) in clusters.iter().enumerate() {
            let distance = self.episode_distance(episode, &cluster.centroid);
            if distance < min_distance {
                min_distance = distance;
                nearest = i;
            }
        }

        nearest
    }

    /// Calculate distance between episode and cluster centroid
    fn episode_distance(&self, episode: &Episode, centroid: &ClusterCentroid) -> f32 {
        // Distance based on:
        // 1. Context similarity (domain, language, tags)
        // 2. Number of steps difference
        // 3. Outcome similarity

        let context_dist = 1.0 - self.context_distance(&episode.context, &centroid.context);

        let steps_dist =
            (episode.steps.len() as f32 - centroid.avg_steps).abs() / centroid.avg_steps.max(1.0);

        let outcome_dist = if episode.outcome.is_some() == centroid.has_outcome {
            0.0
        } else {
            1.0
        };

        // Weighted combination
        context_dist * 0.5 + steps_dist * 0.3 + outcome_dist * 0.2
    }

    /// Calculate context distance (0.0 = identical, 1.0 = completely different)
    fn context_distance(&self, ctx1: &TaskContext, ctx2: &TaskContext) -> f32 {
        let mut distance = 0.0;

        // Domain
        if ctx1.domain != ctx2.domain {
            distance += 0.4;
        }

        // Language
        if ctx1.language != ctx2.language {
            distance += 0.3;
        }

        // Tags (Jaccard distance)
        let common_tags: Vec<_> = ctx1.tags.iter().filter(|t| ctx2.tags.contains(t)).collect();

        let total_unique = ctx1
            .tags
            .iter()
            .chain(ctx2.tags.iter())
            .collect::<std::collections::HashSet<_>>()
            .len();

        let tag_similarity = if total_unique > 0 {
            common_tags.len() as f32 / total_unique as f32
        } else {
            1.0
        };

        distance += (1.0 - tag_similarity) * 0.3;

        distance
    }

    /// Calculate centroid from a group of episodes
    fn calculate_centroid(&self, episodes: &[Episode]) -> ClusterCentroid {
        if episodes.is_empty() {
            return ClusterCentroid::default();
        }

        // Use the most common context
        let representative_context = episodes[0].context.clone();

        // Average number of steps
        let avg_steps =
            episodes.iter().map(|e| e.steps.len()).sum::<usize>() as f32 / episodes.len() as f32;

        // Check if most have outcomes
        let outcome_count = episodes.iter().filter(|e| e.outcome.is_some()).count();
        let has_outcome = outcome_count > episodes.len() / 2;

        ClusterCentroid {
            context: representative_context,
            avg_steps,
            has_outcome,
        }
    }

    /// Check if two centroids are equal (within tolerance)
    fn centroids_equal(&self, c1: &ClusterCentroid, c2: &ClusterCentroid) -> bool {
        c1.context.domain == c2.context.domain
            && (c1.avg_steps - c2.avg_steps).abs() < 0.5
            && c1.has_outcome == c2.has_outcome
    }
}

impl Default for PatternClusterer {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a cluster centroid for k-means clustering
#[derive(Debug, Clone)]
pub struct ClusterCentroid {
    /// Representative context for this cluster
    pub context: TaskContext,
    /// Average number of steps in this cluster
    pub avg_steps: f32,
    /// Whether episodes in this cluster typically have outcomes
    pub has_outcome: bool,
}

impl ClusterCentroid {
    /// Create a centroid from a single episode
    fn from_episode(episode: &Episode) -> Self {
        Self {
            context: episode.context.clone(),
            avg_steps: episode.steps.len() as f32,
            has_outcome: episode.outcome.is_some(),
        }
    }
}

impl Default for ClusterCentroid {
    fn default() -> Self {
        Self {
            context: TaskContext::default(),
            avg_steps: 0.0,
            has_outcome: false,
        }
    }
}

/// A cluster of similar episodes
#[derive(Debug, Clone)]
pub struct EpisodeCluster {
    /// Centroid representing the cluster
    pub centroid: ClusterCentroid,
    /// Episodes in this cluster
    pub episodes: Vec<Episode>,
}

impl EpisodeCluster {
    /// Get the size of this cluster
    #[must_use]
    pub fn size(&self) -> usize {
        self.episodes.len()
    }

    /// Calculate the average success rate for this cluster
    #[must_use]
    pub fn success_rate(&self) -> f32 {
        if self.episodes.is_empty() {
            return 0.0;
        }

        let successes = self
            .episodes
            .iter()
            .filter(|e| {
                e.outcome
                    .as_ref()
                    .is_some_and(|o| matches!(o, crate::types::TaskOutcome::Success { .. }))
            })
            .count();

        successes as f32 / self.episodes.len() as f32
    }

    /// Get common pattern IDs across episodes in this cluster
    ///
    /// Returns pattern IDs that appear in at least 30% of episodes (minimum 2 occurrences).
    /// The returned IDs are sorted by occurrence frequency (most common first).
    ///
    /// Note: Episodes store pattern IDs only. To get full Pattern objects,
    /// use a `PatternStorage` to look up these IDs.
    #[must_use]
    pub fn extract_common_patterns(&self) -> Vec<crate::episode::PatternId> {
        use std::collections::HashMap;

        if self.episodes.is_empty() {
            return Vec::new();
        }

        // Collect all pattern IDs from all episodes with occurrence counts
        let mut pattern_occurrences: HashMap<crate::episode::PatternId, usize> = HashMap::new();

        for episode in &self.episodes {
            for pattern_id in &episode.patterns {
                *pattern_occurrences.entry(*pattern_id).or_insert(0) += 1;
            }
        }

        // Filter patterns that appear in at least 30% of episodes (common threshold)
        let min_occurrence = (self.episodes.len() as f32 * 0.3).ceil() as usize;

        let mut common_patterns: Vec<(crate::episode::PatternId, usize)> = pattern_occurrences
            .into_iter()
            .filter(|(_, count)| *count >= min_occurrence.max(2)) // At least 2 occurrences
            .collect();

        // Sort by occurrence count (higher count first)
        common_patterns.sort_by(|a, b| b.1.cmp(&a.1));

        // Return just the pattern IDs
        common_patterns.into_iter().map(|(id, _)| id).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::PatternEffectiveness;
    use crate::types::{ComplexityLevel, ExecutionResult, TaskOutcome};
    use crate::ExecutionStep;
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_pattern_tool_sequence(tools: Vec<&str>, domain: &str) -> Pattern {
        Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: tools.iter().map(|s| (*s).to_string()).collect(),
            context: TaskContext {
                domain: domain.to_string(),
                language: Some("rust".to_string()),
                complexity: ComplexityLevel::Moderate,
                framework: None,
                tags: vec!["test".to_string()],
            },
            success_rate: 0.9,
            avg_latency: chrono::Duration::milliseconds(100),
            occurrence_count: 5,
            effectiveness: PatternEffectiveness::default(),
        }
    }

    #[test]
    fn test_pattern_deduplication() {
        let clusterer = PatternClusterer::new();

        // Create similar patterns
        let p1 = create_test_pattern_tool_sequence(vec!["read", "write"], "web-api");
        let p2 = create_test_pattern_tool_sequence(vec!["read", "write"], "web-api");
        let p3 = create_test_pattern_tool_sequence(vec!["compile", "test"], "cli");

        let patterns = vec![p1, p2, p3];

        let deduplicated = clusterer.deduplicate_patterns(patterns);

        // Should merge p1 and p2, keep p3 separate
        assert_eq!(deduplicated.len(), 2);
    }

    #[test]
    fn test_similarity_key_grouping() {
        let clusterer = PatternClusterer::new();

        let p1 = create_test_pattern_tool_sequence(vec!["read", "write"], "web-api");
        let p2 = create_test_pattern_tool_sequence(vec!["read", "write"], "web-api");
        let p3 = create_test_pattern_tool_sequence(vec!["compile", "test"], "cli");

        let patterns = vec![p1, p2, p3];

        let groups = clusterer.group_by_similarity_key(patterns);

        // Should have 2 groups (identical patterns grouped together)
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn test_find_similar_patterns() {
        let clusterer = PatternClusterer::new();

        let target = create_test_pattern_tool_sequence(vec!["read", "write"], "web-api");
        let p1 = create_test_pattern_tool_sequence(vec!["read", "write", "close"], "web-api");
        let p2 = create_test_pattern_tool_sequence(vec!["compile", "test"], "cli");
        let p3 = create_test_pattern_tool_sequence(vec!["read", "update"], "web-api");

        let candidates = vec![p1, p2, p3];

        let similar = clusterer.find_similar_patterns(&target, &candidates, 2);

        // Should find 2 similar patterns, ordered by similarity
        assert_eq!(similar.len(), 2);
        // First should have higher similarity score
        assert!(similar[0].1 >= similar[1].1);
    }

    #[test]
    fn test_episode_clustering() {
        let clusterer = PatternClusterer::with_config(ClusteringConfig {
            num_clusters: 2,
            ..ClusteringConfig::default()
        });

        // Create test episodes with different contexts
        let episodes = vec![
            create_test_episode("web-api", 5),
            create_test_episode("web-api", 6),
            create_test_episode("cli", 3),
            create_test_episode("cli", 4),
        ];

        let clusters = clusterer.cluster_episodes(episodes);

        // Should create 2 clusters
        assert!(!clusters.is_empty());
        assert!(clusters.len() <= 2);

        // Each cluster should have episodes
        for cluster in &clusters {
            assert!(cluster.size() > 0);
        }
    }

    #[test]
    fn test_cluster_success_rate() {
        let cluster = EpisodeCluster {
            centroid: ClusterCentroid::default(),
            episodes: vec![
                create_successful_episode(),
                create_successful_episode(),
                create_failed_episode(),
            ],
        };

        let success_rate = cluster.success_rate();

        // 2 out of 3 succeeded
        assert!((success_rate - 0.666).abs() < 0.01);
    }

    // Helper functions for tests
    fn create_test_episode(domain: &str, step_count: usize) -> Episode {
        let mut episode = Episode {
            episode_id: Uuid::new_v4(),
            task_type: crate::types::TaskType::CodeGeneration,
            task_description: "Test task".to_string(),
            context: TaskContext {
                domain: domain.to_string(),
                language: Some("rust".to_string()),
                complexity: ComplexityLevel::Moderate,
                framework: None,
                tags: vec![],
            },
            start_time: Utc::now(),
            end_time: None,
            steps: Vec::new(),
            outcome: None,
            reward: None,
            reflection: None,
            patterns: Vec::new(),
            heuristics: Vec::new(),
            applied_patterns: Vec::new(),
            salient_features: None,
            metadata: std::collections::HashMap::new(),
        };

        for i in 0..step_count {
            episode.steps.push(ExecutionStep {
                step_number: i + 1,
                tool: format!("tool_{i}"),
                action: format!("action_{i}"),
                timestamp: Utc::now(),
                result: Some(ExecutionResult::Success {
                    output: "ok".to_string(),
                }),
                latency_ms: 100,
                tokens_used: None,
                metadata: std::collections::HashMap::new(),
                parameters: serde_json::Value::Null,
            });
        }

        episode
    }

    fn create_successful_episode() -> Episode {
        let mut episode = create_test_episode("test", 3);
        episode.outcome = Some(TaskOutcome::Success {
            verdict: "Success".to_string(),
            artifacts: vec![],
        });
        episode
    }

    fn create_failed_episode() -> Episode {
        let mut episode = create_test_episode("test", 3);
        episode.outcome = Some(TaskOutcome::Failure {
            reason: "Failed".to_string(),
            error_details: None,
        });
        episode
    }

    #[test]
    fn test_extract_common_patterns() {
        use crate::types::TaskType;

        // Create test episodes
        let mut episodes = vec![];

        // Common pattern ID that appears in multiple episodes
        let common_pattern_id = Uuid::new_v4();
        let rare_pattern_id = Uuid::new_v4();

        // Create 3 episodes, 2 with the common pattern
        for i in 0..3 {
            let mut episode = Episode::new(
                format!("Task {i}"),
                TaskContext {
                    domain: "test".to_string(),
                    language: None,
                    complexity: ComplexityLevel::Simple,
                    framework: None,
                    tags: vec![],
                },
                TaskType::Testing,
            );

            // First 2 episodes have the common pattern
            if i < 2 {
                episode.patterns.push(common_pattern_id);
            }

            // Only the first episode has the rare pattern
            if i == 0 {
                episode.patterns.push(rare_pattern_id);
            }

            episodes.push(episode);
        }

        let cluster = EpisodeCluster {
            centroid: ClusterCentroid::default(),
            episodes,
        };

        let common = cluster.extract_common_patterns();

        // Should find the common pattern (appears in 2/3 episodes = 66%)
        // The rare pattern should NOT appear (only in 1/3 = 33%)
        assert_eq!(common.len(), 1, "Should extract exactly one common pattern");
        assert_eq!(
            common[0], common_pattern_id,
            "Should extract the pattern that appears in 2 episodes"
        );
    }
}
