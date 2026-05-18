//! Episodic memory retrieval with caching and hybrid search.
//!
//! This module provides efficient retrieval of episodes with:
//! - LRU caching and TTL for query results
//! - BM25 keyword search (via CSM crate, first-tier, no API calls)
//! - ConceptGraph ontology expansion (via CSM crate, synonym matching)
//! - HDC hyperdimensional vectors (via CSM crate)
//! - Cascading retrieval pipeline (WG-131)
//! - Hierarchical/gist reranking for dense context (WG-118)
//! - Reconstructive retrieval windows (WG-120, E-mem-inspired)
//! - Execution-signature retrieval (WG-121, APEX-EM-inspired)
//! - Scope-before-search shard routing (WG-122, ShardMemo-inspired)

pub mod cache;
pub mod cascade;
pub mod gist;
pub mod shard;
pub mod signature;
pub mod windows;

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
pub use gist::{EpisodeGist, GistExtractor, GistScoredItem, HierarchicalReranker, RerankConfig};
pub use shard::{EpisodeMetadata, RoutingResult, ScopeFilter, ShardConfig, ShardRouter, TimeRange};
pub use signature::{
    ExecutionSignature, QuerySignature, SignatureConfig, SignatureMatch, SignatureMatcher,
    StepPattern,
};
pub use windows::{
    ContextWindow, RetrievalHit, WindowConfig, WindowExpander, WindowExpansionResult,
    merge_overlapping_windows,
};
