//! Search result ranking and scoring
//!
//! This module provides multi-signal ranking for search results based on:
//! - Relevance (match quality)
//! - Recency (time-based scoring)
//! - Success rate (outcome-based scoring)
//! - Completeness (episode state)
//! - Field importance (where the match was found)

use crate::episode::Episode;
use crate::search::{SearchField, SearchMode, SearchResult};
use chrono::{DateTime, Utc};

/// Weights for different ranking signals
#[derive(Debug, Clone)]
pub struct RankingWeights {
    /// Relevance score weight (how well the text matches)
    pub relevance: f64,
    /// Recency score weight (how recent the episode is)
    pub recency: f64,
    /// Success rate weight (whether episode was successful)
    pub success: f64,
    /// Completeness weight (whether episode is complete)
    pub completeness: f64,
    /// Field match weight (which field matched)
    pub field_importance: f64,
}

impl Default for RankingWeights {
    fn default() -> Self {
        Self {
            relevance: 0.40,        // 40% - Most important
            recency: 0.20,          // 20% - Moderately important
            success: 0.20,          // 20% - Moderately important
            completeness: 0.10,     // 10% - Somewhat important
            field_importance: 0.10, // 10% - Somewhat important
        }
    }
}

impl RankingWeights {
    /// Create a new set of ranking weights
    #[must_use]
    pub fn new(
        relevance: f64,
        recency: f64,
        success: f64,
        completeness: f64,
        field_importance: f64,
    ) -> Self {
        Self {
            relevance,
            recency,
            success,
            completeness,
            field_importance,
        }
    }

    /// Validate that weights sum to approximately 1.0
    pub fn validate(&self) -> Result<(), String> {
        let sum = self.relevance
            + self.recency
            + self.success
            + self.completeness
            + self.field_importance;

        if (sum - 1.0).abs() > 0.01 {
            return Err(format!(
                "Weights should sum to 1.0, got {} (difference: {})",
                sum,
                (sum - 1.0).abs()
            ));
        }

        Ok(())
    }
}

/// Calculate relevance score based on search mode and match quality
///
/// # Arguments
///
/// * `mode` - The search mode used
/// * `similarity` - The similarity score (0.0 to 1.0)
///
/// # Returns
///
/// A relevance score between 0.0 and 1.0
#[must_use]
pub fn calculate_relevance_score(mode: &SearchMode, similarity: f64) -> f64 {
    match mode {
        SearchMode::Exact => {
            // Exact matches get full score
            if similarity >= 1.0 {
                1.0
            } else {
                0.0
            }
        }
        SearchMode::Fuzzy { threshold } => {
            // Fuzzy matches: normalize above threshold
            if similarity >= *threshold {
                // Scale from threshold to 1.0
                (similarity - threshold) / (1.0 - threshold)
            } else {
                0.0
            }
        }
        SearchMode::Regex => {
            // Regex matches get high score (pattern matched)
            0.9
        }
    }
}

/// Calculate recency score based on episode age
///
/// More recent episodes get higher scores. Uses exponential decay.
///
/// # Arguments
///
/// * `start_time` - When the episode started
/// * `now` - Current time
///
/// # Returns
///
/// A recency score between 0.0 and 1.0
#[must_use]
pub fn calculate_recency_score(start_time: DateTime<Utc>, now: DateTime<Utc>) -> f64 {
    let age_days = (now - start_time).num_days() as f64;

    // Exponential decay: half-life of 30 days
    let half_life = 30.0;
    0.5_f64.powf(age_days / half_life)
}

/// Calculate success score based on episode outcome
///
/// # Arguments
///
/// * `episode` - The episode to score
///
/// # Returns
///
/// A success score between 0.0 and 1.0
#[must_use]
pub fn calculate_success_score(episode: &Episode) -> f64 {
    if let Some(ref outcome) = episode.outcome {
        match outcome {
            crate::types::TaskOutcome::Success { .. } => 1.0,
            crate::types::TaskOutcome::PartialSuccess { .. } => 0.6,
            crate::types::TaskOutcome::Failure { .. } => 0.2,
        }
    } else {
        // No outcome yet
        0.5
    }
}

/// Calculate completeness score
///
/// # Arguments
///
/// * `episode` - The episode to score
///
/// # Returns
///
/// A completeness score between 0.0 and 1.0
#[must_use]
pub fn calculate_completeness_score(episode: &Episode) -> f64 {
    if episode.is_complete() {
        1.0
    } else {
        // Partial score based on number of steps
        (episode.steps.len() as f64 * 0.1).min(0.8)
    }
}

/// Calculate field importance score based on where the match was found
///
/// # Arguments
///
/// * `field` - The field where the match was found
///
/// # Returns
///
/// A field importance score between 0.0 and 1.0
#[must_use]
pub fn calculate_field_importance_score(field: &SearchField) -> f64 {
    field.weight()
}

/// Calculate overall ranking score for an episode
///
/// # Arguments
///
/// * `episode` - The episode to score
/// * `mode` - The search mode used
/// * `similarity` - The match similarity score
/// * `matched_field` - The field where the match was found
/// * `weights` - The ranking weights to use
///
/// # Returns
///
/// A final score between 0.0 and 1.0
#[must_use]
pub fn calculate_ranking_score(
    episode: &Episode,
    mode: &SearchMode,
    similarity: f64,
    matched_field: &SearchField,
    weights: &RankingWeights,
) -> f64 {
    let now = Utc::now();

    let relevance = calculate_relevance_score(mode, similarity);
    let recency = calculate_recency_score(episode.start_time, now);
    let success = calculate_success_score(episode);
    let completeness = calculate_completeness_score(episode);
    let field_importance = calculate_field_importance_score(matched_field);

    // Weighted sum
    relevance * weights.relevance
        + recency * weights.recency
        + success * weights.success
        + completeness * weights.completeness
        + field_importance * weights.field_importance
}

/// Rank and sort search results
///
/// # Arguments
///
/// * `results` - Vector of search results to rank
///
/// # Returns
///
/// Sorted vector with highest scores first
#[must_use]
pub fn rank_search_results<T>(mut results: Vec<SearchResult<T>>) -> Vec<SearchResult<T>> {
    // Sort by score (highest first), then by recency if scores are equal
    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{TaskContext, TaskType};
    use chrono::Duration;

    #[test]
    fn test_default_weights_sum_to_one() {
        let weights = RankingWeights::default();
        assert!(weights.validate().is_ok());
    }

    #[test]
    fn test_custom_weights_validation() {
        let valid = RankingWeights::new(0.5, 0.2, 0.2, 0.05, 0.05);
        assert!(valid.validate().is_ok());

        let invalid = RankingWeights::new(0.5, 0.2, 0.2, 0.2, 0.2);
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_exact_relevance_score() {
        assert_eq!(calculate_relevance_score(&SearchMode::Exact, 1.0), 1.0);
        assert_eq!(calculate_relevance_score(&SearchMode::Exact, 0.9), 0.0);
    }

    #[test]
    fn test_fuzzy_relevance_score() {
        let mode = SearchMode::Fuzzy { threshold: 0.8 };

        // Perfect match
        assert_eq!(calculate_relevance_score(&mode, 1.0), 1.0);

        // At threshold
        assert_eq!(calculate_relevance_score(&mode, 0.8), 0.0);

        // Midway between threshold and perfect
        assert_eq!(calculate_relevance_score(&mode, 0.9), 0.5);

        // Below threshold
        assert_eq!(calculate_relevance_score(&mode, 0.7), 0.0);
    }

    #[test]
    fn test_regex_relevance_score() {
        assert_eq!(calculate_relevance_score(&SearchMode::Regex, 1.0), 0.9);
    }

    #[test]
    fn test_recency_score() {
        let now = Utc::now();

        // Recent episode (1 day old)
        let recent = now - Duration::days(1);
        let score = calculate_recency_score(recent, now);
        assert!(score > 0.9);

        // 30 days old (half-life)
        let month_old = now - Duration::days(30);
        let score = calculate_recency_score(month_old, now);
        assert!((score - 0.5).abs() < 0.01);

        // 60 days old
        let two_months_old = now - Duration::days(60);
        let score = calculate_recency_score(two_months_old, now);
        assert!((score - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_success_score() {
        let context = TaskContext::default();
        let mut episode = Episode::new("test".to_string(), context, TaskType::CodeGeneration);

        // No outcome
        assert_eq!(calculate_success_score(&episode), 0.5);

        // Success
        episode.outcome = Some(crate::types::TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });
        assert_eq!(calculate_success_score(&episode), 1.0);

        // Partial success
        episode.outcome = Some(crate::types::TaskOutcome::PartialSuccess {
            verdict: "Partial".to_string(),
            completed: vec![],
            failed: vec![],
        });
        assert_eq!(calculate_success_score(&episode), 0.6);

        // Failure
        episode.outcome = Some(crate::types::TaskOutcome::Failure {
            reason: "Failed".to_string(),
            error_details: None,
        });
        assert_eq!(calculate_success_score(&episode), 0.2);
    }

    #[test]
    fn test_completeness_score() {
        let context = TaskContext::default();
        let mut episode = Episode::new("test".to_string(), context, TaskType::CodeGeneration);

        // Incomplete, no steps
        assert_eq!(calculate_completeness_score(&episode), 0.0);

        // Add some steps but not complete
        for i in 1..=5 {
            let step =
                crate::episode::ExecutionStep::new(i, "tool".to_string(), "action".to_string());
            episode.steps.push(step);
        }
        let score = calculate_completeness_score(&episode);
        assert!(score > 0.0 && score < 1.0);

        // Complete (needs both outcome and end_time)
        episode.outcome = Some(crate::types::TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });
        episode.end_time = Some(Utc::now());
        assert_eq!(calculate_completeness_score(&episode), 1.0);
    }

    #[test]
    fn test_field_importance_score() {
        assert_eq!(
            calculate_field_importance_score(&SearchField::Description),
            1.0
        );
        assert_eq!(calculate_field_importance_score(&SearchField::Outcome), 0.8);
        assert_eq!(calculate_field_importance_score(&SearchField::Steps), 0.6);
        assert_eq!(calculate_field_importance_score(&SearchField::Tags), 0.5);
        assert_eq!(calculate_field_importance_score(&SearchField::Domain), 0.4);
    }

    #[test]
    fn test_calculate_ranking_score() {
        let context = TaskContext::default();
        let episode = Episode::new("test".to_string(), context, TaskType::CodeGeneration);

        let weights = RankingWeights::default();
        let mode = SearchMode::Exact;
        let field = SearchField::Description;

        let score = calculate_ranking_score(&episode, &mode, 1.0, &field, &weights);

        // Score should be between 0 and 1
        assert!((0.0..=1.0).contains(&score));
    }

    #[test]
    fn test_rank_search_results() {
        let low_score = SearchResult {
            item: "episode1",
            score: 0.5,
            matches: vec![],
        };
        let high_score = SearchResult {
            item: "episode2",
            score: 0.9,
            matches: vec![],
        };
        let mid_score = SearchResult {
            item: "episode3",
            score: 0.7,
            matches: vec![],
        };

        let results = vec![low_score, high_score, mid_score];
        let ranked = rank_search_results(results);

        // Should be sorted by score (highest first)
        assert_eq!(ranked[0].score, 0.9);
        assert_eq!(ranked[1].score, 0.7);
        assert_eq!(ranked[2].score, 0.5);
    }
}
