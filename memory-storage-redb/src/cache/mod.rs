//! LRU cache module for redb storage
//!
//! This module provides an in-memory LRU cache with TTL expiration,
//! split into logical submodules for maintainability:
//!
//! - `types`: Core type definitions (CacheConfig, CacheEntry, CacheMetrics)
//! - `state`: Internal cache state management
//! - `lru`: Main LRUCache implementation
//! - `tests`: Test suite

mod lru;
mod state;
mod types;

#[cfg(test)]
mod tests;

// Re-export public types
pub use lru::LRUCache;
pub use types::{CacheConfig, CacheMetrics};
