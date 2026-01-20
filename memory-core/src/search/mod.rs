//! Search capabilities for memory retrieval
//!
//! This module provides various search algorithms and utilities for
//! retrieving relevant episodes and patterns from memory.

pub mod fuzzy;
pub mod ranking;
pub mod regex;
pub mod types;

#[cfg(feature = "hybrid_search")]
pub mod hybrid;

pub use fuzzy::{best_fuzzy_match, fuzzy_match, fuzzy_search_in_text};
pub use ranking::{
    calculate_completeness_score, calculate_field_importance_score, calculate_ranking_score,
    calculate_recency_score, calculate_relevance_score, calculate_success_score,
    rank_search_results, RankingWeights,
};
pub use regex::{
    regex_matches, regex_search, regex_search_case_insensitive, validate_regex_pattern,
};
pub use types::{FieldMatch, SearchField, SearchMode, SearchResult};

#[cfg(feature = "hybrid_search")]
pub use hybrid::{HybridSearch, HybridSearchConfig, HybridSearchResult};
