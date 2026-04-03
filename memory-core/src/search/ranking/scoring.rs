//! Ranking score calculations for search results
//!
//! Multi-signal scoring based on relevance, recency, success rate,
//! completeness, and field importance.

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
            if similarity >= 1.0 { 1.0 } else { 0.0 }
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
