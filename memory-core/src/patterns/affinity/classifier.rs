//! Pattern Affinity Classifier for routing-drift protection.

use crate::episode::Episode;
use crate::pattern::Pattern;

use super::types::{
    DEFAULT_AFFINITY_THRESHOLD, DEFAULT_MIN_SUCCESS_RATE, EpisodeAssignmentGuard, RelativeAffinity,
};

/// Pattern Affinity Classifier for routing-drift protection.
///
/// Central component that computes affinity scores and provides
/// gating decisions for pattern extraction pipeline.
#[derive(Debug, Clone)]
pub struct PatternAffinityClassifier {
    /// Threshold for considering an episode ambiguous
    affinity_threshold: f32,
    /// Minimum success rate for pattern mutation
    min_success_rate: f32,
}

impl Default for PatternAffinityClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternAffinityClassifier {
    /// Create classifier with default thresholds.
    #[must_use]
    pub fn new() -> Self {
        Self {
            affinity_threshold: DEFAULT_AFFINITY_THRESHOLD,
            min_success_rate: DEFAULT_MIN_SUCCESS_RATE,
        }
    }

    /// Create classifier with custom thresholds.
    #[must_use]
    pub fn with_config(affinity_threshold: f32, min_success_rate: f32) -> Self {
        Self {
            affinity_threshold,
            min_success_rate,
        }
    }

    /// Compute relative affinity for an episode against pattern clusters.
    ///
    /// This is the core Drel computation from DyMoE:
    /// - Compares episode embedding to centroids of old and new patterns
    /// - Returns relative difference to detect ambiguous episodes
    #[must_use]
    pub fn compute_affinity(
        &self,
        episode: &Episode,
        old_patterns: &[Pattern],
        new_patterns: &[Pattern],
        episode_embedding: Option<&[f32]>,
    ) -> RelativeAffinity {
        RelativeAffinity::compute(episode, old_patterns, new_patterns, episode_embedding)
    }

    /// Create assignment guard for an episode.
    ///
    /// Combines episode's success rate with computed affinity clarity.
    #[must_use]
    pub fn create_guard(
        &self,
        episode: &Episode,
        old_patterns: &[Pattern],
        new_patterns: &[Pattern],
        episode_embedding: Option<&[f32]>,
    ) -> EpisodeAssignmentGuard {
        let affinity =
            self.compute_affinity(episode, old_patterns, new_patterns, episode_embedding);
        let success_rate = episode
            .reward
            .as_ref()
            .map(|r| r.total / 2.0)
            .unwrap_or(0.5);

        EpisodeAssignmentGuard::with_thresholds(
            success_rate,
            affinity.clarity(),
            self.min_success_rate,
            self.affinity_threshold,
        )
    }

    /// Check if episode should be gated from pattern mutation.
    ///
    /// Returns true if the episode is ambiguous and should NOT
    /// mutate existing high-performing patterns.
    #[must_use]
    pub fn should_gate_episode(
        &self,
        episode: &Episode,
        old_patterns: &[Pattern],
        new_patterns: &[Pattern],
        episode_embedding: Option<&[f32]>,
    ) -> bool {
        let affinity =
            self.compute_affinity(episode, old_patterns, new_patterns, episode_embedding);
        affinity.is_ambiguous(self.affinity_threshold)
    }

    /// Get the configured affinity threshold.
    #[must_use]
    pub fn affinity_threshold(&self) -> f32 {
        self.affinity_threshold
    }

    /// Get the configured minimum success rate.
    #[must_use]
    pub fn min_success_rate(&self) -> f32 {
        self.min_success_rate
    }
}
