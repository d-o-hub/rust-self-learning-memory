//! Types and constants for pattern affinity classification.

use serde::{Deserialize, Serialize};

/// Default threshold for affinity clarity gating.
/// Episodes with Drel < this threshold are considered ambiguous.
pub const DEFAULT_AFFINITY_THRESHOLD: f32 = 0.25;

/// Default minimum success rate for pattern mutation.
pub const DEFAULT_MIN_SUCCESS_RATE: f32 = 0.70;

/// Relative affinity between new and old pattern clusters.
///
/// Drel measures how clearly an episode belongs to new vs old patterns.
/// - Drel ≈ 0: Episode is ambiguous (similar scores to both clusters)
/// - Drel ≈ 1: Episode clearly belongs to one cluster
///
/// Formula: Drel = |score_new - score_old| / max(score_new, score_old)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RelativeAffinity {
    /// Affinity score to old/existing patterns
    pub score_old: f32,
    /// Affinity score to new/potential patterns
    pub score_new: f32,
    /// Relative difference (computed)
    pub drel: f32,
}

impl RelativeAffinity {
    /// Compute relative affinity between episode and pattern clusters.
    ///
    /// Uses max cosine similarity to find the best match in each cluster.
    pub fn compute(
        episode: &crate::episode::Episode,
        old_patterns: &[crate::pattern::Pattern],
        new_patterns: &[crate::pattern::Pattern],
        episode_embedding: Option<&[f32]>,
    ) -> Self {
        let score_old =
            super::computation::max_cosine_similarity(episode, old_patterns, episode_embedding);
        let score_new =
            super::computation::max_cosine_similarity(episode, new_patterns, episode_embedding);

        let denom = score_old.max(score_new).max(1e-6);
        let drel = (score_new - score_old).abs() / denom;

        Self {
            score_old,
            score_new,
            drel,
        }
    }

    /// Check if this affinity is ambiguous (Drel < threshold).
    #[must_use]
    pub fn is_ambiguous(&self, threshold: f32) -> bool {
        self.drel < threshold
    }

    /// Get the clarity score (1 - Drel normalized).
    /// Higher clarity means the episode clearly belongs to one cluster.
    #[must_use]
    pub fn clarity(&self) -> f32 {
        1.0 - self.drel
    }
}

/// Two-dimensional guard for episode-to-pattern assignment.
///
/// Combines success rate and affinity clarity to prevent:
/// 1. Low-success episodes from corrupting patterns
/// 2. Ambiguous episodes from causing routing drift
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EpisodeAssignmentGuard {
    /// Episode's success/reward score (existing dimension)
    pub success_rate: f32,
    /// Affinity clarity from Drel computation (new dimension)
    pub affinity_clarity: f32,
    /// Minimum success rate threshold
    pub min_success_rate: f32,
    /// Minimum affinity clarity threshold
    pub min_affinity_clarity: f32,
}

impl EpisodeAssignmentGuard {
    /// Create a guard with default thresholds.
    #[must_use]
    pub fn new(success_rate: f32, affinity_clarity: f32) -> Self {
        Self {
            success_rate,
            affinity_clarity,
            min_success_rate: DEFAULT_MIN_SUCCESS_RATE,
            min_affinity_clarity: DEFAULT_AFFINITY_THRESHOLD,
        }
    }

    /// Create a guard with custom thresholds.
    #[must_use]
    pub fn with_thresholds(
        success_rate: f32,
        affinity_clarity: f32,
        min_success_rate: f32,
        min_affinity_clarity: f32,
    ) -> Self {
        Self {
            success_rate,
            affinity_clarity,
            min_success_rate,
            min_affinity_clarity,
        }
    }

    /// Check if this episode should be allowed to mutate patterns.
    ///
    /// Returns true only if BOTH dimensions pass their thresholds:
    /// - success_rate >= min_success_rate
    /// - affinity_clarity >= min_affinity_clarity
    #[must_use]
    pub fn allows_mutation(&self) -> bool {
        self.success_rate >= self.min_success_rate
            && self.affinity_clarity >= self.min_affinity_clarity
    }

    /// Check if episode can be used for read-only pattern matching.
    /// Episodes that fail mutation gate can still be used for retrieval.
    #[must_use]
    pub fn allows_retrieval(&self) -> bool {
        self.success_rate >= self.min_success_rate * 0.5
    }

    /// Get the reason for rejection (if mutation is not allowed).
    #[must_use]
    pub fn rejection_reason(&self) -> Option<RejectionReason> {
        if self.success_rate < self.min_success_rate {
            return Some(RejectionReason::LowSuccessRate {
                actual: self.success_rate,
                required: self.min_success_rate,
            });
        }
        if self.affinity_clarity < self.min_affinity_clarity {
            return Some(RejectionReason::AmbiguousAffinity {
                actual: self.affinity_clarity,
                required: self.min_affinity_clarity,
            });
        }
        None
    }
}

/// Reason why an episode was rejected from pattern mutation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RejectionReason {
    /// Episode success rate below threshold
    LowSuccessRate { actual: f32, required: f32 },
    /// Episode affinity is ambiguous (routing drift risk)
    AmbiguousAffinity { actual: f32, required: f32 },
}
