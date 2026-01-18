//! # Pattern Clusterer
//!
//! Pattern clustering engine for grouping and deduplicating patterns.

use crate::episode::Episode;
use crate::pattern::Pattern;
use crate::patterns::clustering::{ClusterCentroid, ClusteringConfig, EpisodeCluster};
use crate::types::TaskContext;
use std::collections::HashMap;

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
