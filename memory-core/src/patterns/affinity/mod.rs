//! Pattern Affinity Classification for DyMoE Routing-Drift Protection
//!
//! Inspired by LLaVA-DyMoE (CVPR 2026), this module implements routing-drift
//! protection to prevent ambiguous episodes from corrupting established
//! high-success-rate pattern clusters during pattern extraction.
//!
//! Key concepts:
//! - **Drel (relative affinity)**: Measures how ambiguous an episode is relative
//!   to old vs new pattern clusters. Drel ≈ 0 → ambiguous episode.
//! - **Episode Assignment Guard**: Two-dimensional gate combining success_rate
//!   and affinity_clarity to control pattern mutation.
//!
//! Reference: <https://zhaoc5.github.io/DyMoE/> (Section 3.1-3.2)

use crate::episode::Episode;
use crate::pattern::Pattern;
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
        episode: &Episode,
        old_patterns: &[Pattern],
        new_patterns: &[Pattern],
        episode_embedding: Option<&[f32]>,
    ) -> Self {
        let score_old = max_cosine_similarity(episode, old_patterns, episode_embedding);
        let score_new = max_cosine_similarity(episode, new_patterns, episode_embedding);

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
        // More permissive - only need minimum success rate
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

/// Compute max cosine similarity between an episode and pattern set.
///
/// Finds the pattern with highest embedding similarity to the episode.
fn max_cosine_similarity(
    episode: &Episode,
    patterns: &[Pattern],
    episode_embedding: Option<&[f32]>,
) -> f32 {
    if patterns.is_empty() {
        return 0.0;
    }

    // Use episode embedding if provided, otherwise fall back to context similarity
    let ep_emb = episode_embedding;

    patterns
        .iter()
        .map(|pattern| {
            // Try to get pattern embedding (if available)
            pattern_embedding_similarity(ep_emb, pattern)
                .unwrap_or_else(|| context_similarity(episode, pattern))
        })
        .fold(0.0, f32::max)
}

/// Compute embedding-based similarity if embeddings are available.
fn pattern_embedding_similarity(
    _episode_embedding: Option<&[f32]>,
    _pattern: &Pattern,
) -> Option<f32> {
    // Check if pattern has an embedding (stored in effectiveness metadata)
    // For now, we use context-based similarity as patterns don't store embeddings directly
    // This could be enhanced to use pattern centroids when available
    None
}

/// Compute context-based similarity as fallback.
///
/// Uses task context features (domain, tags) for similarity estimation.
fn context_similarity(episode: &Episode, pattern: &Pattern) -> f32 {
    let ep_context = &episode.context;
    let pat_context = pattern.context();

    match pat_context {
        Some(pat_ctx) => {
            let mut score = 0.0;
            let mut components = 0;

            // Domain match
            if ep_context.domain == pat_ctx.domain {
                score += 1.0;
            }
            components += 1;

            // Tag overlap (Jaccard)
            let ep_tags: std::collections::HashSet<_> = ep_context.tags.iter().collect();
            let pat_tags: std::collections::HashSet<_> = pat_ctx.tags.iter().collect();
            let intersection = ep_tags.intersection(&pat_tags).count();
            let union = ep_tags.union(&pat_tags).count();
            if union > 0 {
                score += intersection as f32 / union as f32;
                components += 1;
            }

            // Language match
            if ep_context.language == pat_ctx.language {
                score += 0.5;
                components += 1;
            }

            score / components as f32
        }
        None => 0.3, // Neutral for patterns without context
    }
}

#[cfg(test)]
mod tests;
