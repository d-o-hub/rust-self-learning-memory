//! Search capabilities for memory retrieval
//!
//! This module provides various search algorithms and utilities for
//! retrieving relevant episodes and patterns from memory.

#[cfg(feature = "hybrid_search")]
pub mod hybrid;

#[cfg(feature = "hybrid_search")]
pub use hybrid::{HybridSearch, HybridSearchConfig, HybridSearchResult};
