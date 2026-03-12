//! Cache trait for abstracting cache implementations
//!
//! This module defines the common interface for cache implementations,
//! allowing different caching strategies (LRU, Adaptive, etc.) to be
//! used interchangeably.

use async_trait::async_trait;
use uuid::Uuid;

use super::types::CacheMetrics;

/// Common cache trait for metadata tracking
///
/// This trait abstracts the cache interface, allowing different implementations
/// (LRU, Adaptive TTL) to be used interchangeably in `RedbStorage`.
///
/// All implementations must be `Send + Sync` for thread-safe async access.
#[async_trait]
pub trait Cache: Send + Sync {
    /// Record a cache access (hit or miss)
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the cache entry
    /// * `hit` - Whether this was a cache hit (true) or miss (false)
    /// * `size_bytes` - Optional size in bytes (for size-aware eviction)
    ///
    /// # Returns
    ///
    /// `true` if the entry was found and is valid, `false` otherwise
    async fn record_access(&self, id: Uuid, hit: bool, size_bytes: Option<usize>) -> bool;

    /// Remove an entry from the cache
    async fn remove(&self, id: Uuid);

    /// Check if an entry exists and is not expired
    async fn contains(&self, id: Uuid) -> bool;

    /// Get current cache metrics
    async fn get_metrics(&self) -> CacheMetrics;

    /// Clear all entries from cache
    async fn clear(&self);

    /// Manually cleanup expired entries
    ///
    /// Returns the number of expired entries removed.
    async fn cleanup_expired(&self) -> usize;
}
