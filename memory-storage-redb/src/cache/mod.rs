//! LRU cache module for redb storage
//!
//! This module provides an in-memory LRU cache with TTL expiration,
//! split into logical submodules for maintainability:
//!
//! - `types`: Core type definitions (CacheConfig, CacheEntry, CacheMetrics)
//! - `state`: Internal cache state management
//! - `lru`: Main LRUCache implementation
//! - `adaptive`: Adaptive TTL cache with access pattern-based TTL adjustment
//! - `traits`: Common Cache trait for interchangeable implementations
//! - `adapter`: AdaptiveCacheAdapter for metadata-only caching
//! - `tests`: Test suite

mod adapter;
mod adaptive;
mod lru;
mod state;
mod traits;
mod types;

#[cfg(test)]
mod tests;

// Re-export public types
pub use adapter::AdaptiveCacheAdapter;
pub use adaptive::{AdaptiveCache, AdaptiveCacheConfig, AdaptiveCacheMetrics};
pub use lru::LRUCache;
pub use traits::Cache;
pub use types::{CacheConfig, CacheMetrics};
