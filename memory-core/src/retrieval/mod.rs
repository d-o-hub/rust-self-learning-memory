//! Episodic memory retrieval with caching
//!
//! This module provides efficient retrieval of episodes with LRU caching and TTL.

pub mod cache;

pub use cache::{CacheKey, CacheMetrics, QueryCache, DEFAULT_CACHE_TTL, DEFAULT_MAX_ENTRIES};
