//! # Clustering Types
//!
//! Type definitions for pattern clustering and episode clustering.

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
    #[must_use]
    pub fn from_episode(episode: &crate::episode::Episode) -> Self {
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
    pub episodes: Vec<crate::episode::Episode>,
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
