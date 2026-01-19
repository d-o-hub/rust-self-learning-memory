//! Semantic pattern search and recommendation engine
//!
//! Provides intelligent pattern discovery using semantic embeddings,
//! multi-signal ranking, and contextual filtering.

pub mod recommendation;
pub mod scoring;

pub use recommendation::{
    discover_analogous_patterns, recommend_patterns_for_task, search_patterns_semantic,
    PatternSearchResult,
};
pub use scoring::{ScoreBreakdown, SearchConfig};
