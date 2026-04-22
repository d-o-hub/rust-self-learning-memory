//! Hierarchical/gist reranking for dense context retrieval (WG-118).
//!
//! This module provides gist-based summarization and reranking to return
//! fewer, denser context items for downstream prompts.
//!
//! ## Purpose
//!
//! When retrieving episodes for LLM context, flat ranking can result in:
//! - Redundant items (similar episodes all included)
//! - Low information density (verbose descriptions)
//! - Token waste (too many low-value items)
//!
//! Gist reranking addresses these by:
//! - Extracting gist summaries (1-3 key sentences per episode)
//! - Scoring by gist density (information per token)
//! - Reranking with diversity to maximize coverage
//!
//! ## Architecture
//!
//! ```text
//! Retrieval Results (episodes + scores)
//!        |
//!        v
//!   GistExtractor (extract key points)
//!        |  - Parse episode description
//!        |  - Extract 1-3 key sentences
//!        |  - Compute gist density score
//!        v
//!   GistScoredItem (episode + gist + density)
//!        |
//!        v
//!   HierarchicalReranker (density + diversity reranking)
//!        |  - Score = relevance + density + recency
//!        |  - Apply MMR-style diversity
//!        |  - Return top-k dense items
//!        v
//!   Dense Bundle (fewer items, higher quality)
//! ```
//!
//! ## Quick Start
//!
//! ```
//! use do_memory_core::retrieval::{GistExtractor, HierarchicalReranker, RerankConfig};
//! use do_memory_core::episode::Episode;
//! use do_memory_core::TaskContext;
//! use do_memory_core::types::TaskType;
//! use std::sync::Arc;
//!
//! // Extract gist from episode description
//! let extractor = GistExtractor::default();
//! let gist = extractor.extract("Fixed authentication bug by adding JWT validation");
//! assert!(gist.key_points.len() <= 3);
//!
//! // Rerank retrieval results by gist density
//! let reranker = HierarchicalReranker::new(RerankConfig::dense());
//! let episodes = vec![
//!     (Arc::new(Episode::new("Fix bug".to_string(), TaskContext::default(), TaskType::Debugging)), 0.9),
//!     (Arc::new(Episode::new("Add feature".to_string(), TaskContext::default(), TaskType::CodeGeneration)), 0.85),
//! ];
//! let dense = reranker.rerank(episodes, 5);
//! // Returns at most 5 items, prioritized by density
//! ```

mod config;
mod extractor;
mod reranker;
mod types;

#[cfg(test)]
mod tests;

pub use config::RerankConfig;
pub use extractor::GistExtractor;
pub use reranker::{GistScoredItem, HierarchicalReranker};
pub use types::EpisodeGist;
