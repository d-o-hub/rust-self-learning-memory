//! Scoring utilities for capacity management.
//!
//! Provides relevance scoring functions that combine quality and recency
//! to determine episode value for eviction decisions.

use crate::episode::Episode;

/// Calculate relevance score for an episode.
///
/// Combines quality score (from `PREMem` or reward score) with recency
/// to determine overall relevance. Higher scores are more relevant
/// and less likely to be evicted.
///
/// Formula: `relevance = (quality * 0.7) + (recency * 0.3)`
///
/// # Arguments
///
/// * `episode` - Episode to score
///
/// # Returns
///
/// Relevance score in range 0.0-1.0
///
/// # Examples
///
/// ```no_run
/// use memory_core::episodic::{CapacityManager, EvictionPolicy};
/// use memory_core::{Episode, TaskContext, TaskType, TaskOutcome};
///
/// let manager = CapacityManager::new(100, EvictionPolicy::RelevanceWeighted);
///
/// let mut episode = Episode::new(
///     "Test task".to_string(),
///     TaskContext::default(),
///     TaskType::Testing,
/// );
/// episode.complete(TaskOutcome::Success {
///     verdict: "Done".to_string(),
///     artifacts: vec![],
/// });
///
/// let score = manager.relevance_score(&episode);
/// assert!(score >= 0.0 && score <= 1.0);
/// ```
#[must_use]
pub fn calculate_relevance_score(episode: &Episode) -> f32 {
    let quality_score = extract_quality_score(episode);
    let recency_score = calculate_recency_score(episode);

    // Weight: 70% quality, 30% recency
    (quality_score * 0.7) + (recency_score * 0.3)
}

/// Extract quality score from episode.
///
/// Uses `PREMem` salient features quality score if available,
/// otherwise falls back to reward score total.
#[must_use]
pub fn extract_quality_score(episode: &Episode) -> f32 {
    // Try to get quality score from PREMem salient features
    if let Some(ref salient) = episode.salient_features {
        // Use the overall quality from salient features
        // This would need to be added to SalientFeatures
        // For now, we'll use a heuristic based on feature count
        let feature_count = salient.count();
        if feature_count > 0 {
            // Normalize: more features = higher quality (capped at 1.0)
            return (feature_count as f32 / 10.0).min(1.0);
        }
    }

    // Fall back to reward score if available
    if let Some(ref reward) = episode.reward {
        // Normalize reward total to 0.0-1.0 range
        // Typical reward totals are 0.0-2.0, so divide by 2.0
        return (reward.total / 2.0).clamp(0.0, 1.0);
    }

    // Default quality score if no information available
    0.5
}

/// Calculate recency score based on episode age.
///
/// Newer episodes get higher scores using exponential decay.
/// Episodes created in the last hour get scores near 1.0.
#[must_use]
pub fn calculate_recency_score(episode: &Episode) -> f32 {
    use chrono::Utc;

    let now = Utc::now();
    let episode_time = episode.end_time.unwrap_or(episode.start_time);

    // Calculate age in hours
    let age_duration = now.signed_duration_since(episode_time);
    let age_hours = age_duration.num_hours() as f32;

    // Exponential decay: score = e^(-age/24)
    // Episodes older than 24 hours decay exponentially
    let decay_factor = 24.0; // Half-life of 24 hours
    let score = (-age_hours / decay_factor).exp();

    score.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Episode;
    use crate::types::{ComplexityLevel, RewardScore, TaskContext, TaskOutcome, TaskType};

    fn create_test_episode(task_desc: &str) -> Episode {
        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: None,
            complexity: ComplexityLevel::Moderate,
            domain: "testing".to_string(),
            tags: vec![],
        };

        Episode::new(task_desc.to_string(), context, TaskType::Testing)
    }

    #[test]
    fn test_relevance_score_calculation() {
        let mut episode = create_test_episode("Test task");
        episode.reward = Some(RewardScore {
            total: 1.0,
            base: 1.0,
            efficiency: 1.0,
            complexity_bonus: 1.0,
            quality_multiplier: 1.0,
            learning_bonus: 0.0,
        });
        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let score = calculate_relevance_score(&episode);
        assert!((0.0..=1.0).contains(&score));
    }

    #[test]
    fn test_recency_score_new_episode() {
        let mut episode = create_test_episode("New task");
        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let score = calculate_recency_score(&episode);
        // Very recent episode should have high recency score
        assert!(score > 0.9, "Expected recency score > 0.9, got {score}");
    }

    #[test]
    fn test_recency_score_old_episode() {
        let mut episode = create_test_episode("Old task");
        // Simulate old episode (30 days ago)
        let old_time = chrono::Utc::now() - chrono::Duration::days(30);
        episode.start_time = old_time;
        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });
        // Override end_time to match old start time
        episode.end_time = Some(old_time);

        let score = calculate_recency_score(&episode);
        // Old episode should have low recency score
        assert!(score < 0.5, "Expected recency score < 0.5, got {score}");
    }

    #[test]
    fn test_quality_score_from_reward() {
        let mut episode = create_test_episode("Test task");
        episode.reward = Some(RewardScore {
            total: 1.5,
            base: 1.0,
            efficiency: 1.2,
            complexity_bonus: 1.1,
            quality_multiplier: 1.0,
            learning_bonus: 0.3,
        });

        let score = extract_quality_score(&episode);
        assert!((0.0..=1.0).contains(&score));
        // Reward total of 1.5 should map to quality ~0.75
        assert!((score - 0.75).abs() < 0.1);
    }

    #[test]
    fn test_quality_score_default() {
        let episode = create_test_episode("Test task");
        // No reward or salient features

        let score = extract_quality_score(&episode);
        assert_eq!(score, 0.5); // Default quality
    }
}
