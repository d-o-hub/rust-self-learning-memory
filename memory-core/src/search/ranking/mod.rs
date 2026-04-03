//! Search result ranking and scoring
//!
//! This module provides multi-signal ranking for search results based on:
//! - Relevance (match quality)
//! - Recency (time-based scoring)
//! - Success rate (outcome-based scoring)
//! - Completeness (episode state)
//! - Field importance (where the match was found)
//! - Bayesian confidence (Wilson score interval)

mod scoring;
mod wilson;

pub use scoring::{
    RankingWeights, calculate_completeness_score, calculate_field_importance_score,
    calculate_ranking_score, calculate_recency_score, calculate_relevance_score,
    calculate_success_score, rank_search_results,
};
pub use wilson::{
    RankingItem, rank_by_wilson_score, wilson_lower_bound, wilson_upper_bound, z_scores,
};

#[cfg(test)]
mod tests;
