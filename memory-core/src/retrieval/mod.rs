//! Episodic memory retrieval with caching and hybrid search.
//!
//! This module provides efficient retrieval of episodes with:
//! - LRU caching and TTL for query results
//! - BM25 keyword search (via CSM crate, first-tier, no API calls)
//! - ConceptGraph ontology expansion (via CSM crate, synonym matching)
//! - HDC hyperdimensional vectors (via CSM crate)
//! - Cascading retrieval pipeline (WG-131)

pub mod cache;
pub mod cascade;

// Re-export CSM types when csm feature is enabled
#[cfg(feature = "csm")]
pub use chaotic_semantic_memory::{
    BundleAccumulator, ConceptGraph, HVec10240, encoder::TextEncoder as HdcEncoder,
};

#[cfg(feature = "csm")]
pub use chaotic_semantic_memory::retrieval::{
    Bm25Config, Bm25Index, HybridConfig, HybridMode, compute_weights, merge_results,
    normalize_scores,
};

pub use cache::{CacheKey, CacheMetrics, DEFAULT_CACHE_TTL, DEFAULT_MAX_ENTRIES, QueryCache};
pub use cascade::{CascadeConfig, CascadeResult, CascadeRetriever};
