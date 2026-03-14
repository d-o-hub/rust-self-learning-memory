//! Adapter for using AdaptiveCache as a metadata-only cache
//!
//! This module provides `AdaptiveCacheAdapter`, which wraps `AdaptiveCache<()>`
//! to implement the `Cache` trait for metadata-only tracking. This enables
//! using AdaptiveCache's intelligent TTL adjustment without storing values
//! (since redb already stores the actual data).

use super::adaptive::{AdaptiveCache, AdaptiveCacheConfig, AdaptiveCacheMetrics};
use super::traits::Cache;
use super::types::CacheMetrics;
use async_trait::async_trait;
use uuid::Uuid;

/// Adapter that wraps AdaptiveCache for metadata-only caching.
///
/// This adapter uses `AdaptiveCache<()>` (unit type) to track cache metadata
/// without storing actual values. This is useful when the actual data is
/// already stored in a persistent layer (like redb), and you only need
/// metadata tracking with intelligent TTL adjustment.
///
/// # Benefits over LRUCache
///
/// - Adaptive TTL: Frequently accessed items get longer TTL
/// - Cold item detection: Rarely accessed items get shorter TTL
/// - Better memory efficiency for cold items
///
/// # Example
///
/// ```no_run
/// use memory_storage_redb::{AdaptiveCacheAdapter, AdaptiveCacheConfig};
/// use std::time::Duration;
///
/// let config = AdaptiveCacheConfig {
///     max_size: 1000,
///     default_ttl: Duration::from_secs(1800),
///     ..Default::default()
/// };
/// let cache = AdaptiveCacheAdapter::new(config);
/// ```
pub struct AdaptiveCacheAdapter {
    inner: AdaptiveCache<()>,
}

impl AdaptiveCacheAdapter {
    /// Create a new adaptive cache adapter with the given configuration
    pub fn new(config: AdaptiveCacheConfig) -> Self {
        Self {
            inner: AdaptiveCache::new(config),
        }
    }

    /// Create a new adaptive cache adapter with default configuration
    pub fn with_defaults() -> Self {
        Self::new(AdaptiveCacheConfig::default())
    }

    /// Get the inner AdaptiveCache for advanced operations
    pub fn inner(&self) -> &AdaptiveCache<()> {
        &self.inner
    }

    /// Get adaptive-specific metrics
    pub async fn get_adaptive_metrics(&self) -> AdaptiveCacheMetrics {
        self.inner.get_metrics().await
    }

    /// Get the number of hot items
    pub async fn hot_count(&self) -> usize {
        self.inner.hot_count().await
    }

    /// Get the number of cold items
    pub async fn cold_count(&self) -> usize {
        self.inner.cold_count().await
    }

    /// Get cache size (number of entries)
    pub async fn len(&self) -> usize {
        self.inner.len().await
    }

    /// Check if cache is empty
    pub async fn is_empty(&self) -> bool {
        self.inner.is_empty().await
    }
}

#[async_trait]
impl Cache for AdaptiveCacheAdapter {
    async fn record_access(&self, id: Uuid, hit: bool, _size_bytes: Option<usize>) -> bool {
        // Use () as the value for metadata-only tracking
        // On miss, we store () to track the entry
        // On hit, we just record the access
        let value = if hit { None } else { Some(()) };
        self.inner.record_access(id, hit, value).await
    }

    async fn remove(&self, id: Uuid) {
        self.inner.remove(id).await
    }

    async fn contains(&self, id: Uuid) -> bool {
        self.inner.contains(id).await
    }

    async fn get_metrics(&self) -> CacheMetrics {
        let adaptive_metrics = self.inner.get_metrics().await;
        adaptive_metrics.base
    }

    async fn clear(&self) {
        self.inner.clear().await
    }

    async fn cleanup_expired(&self) -> usize {
        self.inner.cleanup_expired().await
    }
}

impl From<AdaptiveCacheConfig> for AdaptiveCacheAdapter {
    fn from(config: AdaptiveCacheConfig) -> Self {
        Self::new(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_adapter_basic_operations() {
        let config = AdaptiveCacheConfig {
            max_size: 100,
            default_ttl: Duration::from_secs(60),
            ..Default::default()
        };
        let cache = AdaptiveCacheAdapter::new(config);

        let id = Uuid::new_v4();

        // Record a miss (new entry)
        let result = cache.record_access(id, false, Some(100)).await;
        assert!(!result); // New entry returns false

        // Check contains
        assert!(cache.contains(id).await);

        // Record a hit
        let result = cache.record_access(id, true, None).await;
        assert!(result); // Hit returns true

        // Get metrics
        let metrics = cache.get_metrics().await;
        assert_eq!(metrics.hits, 1);
        assert_eq!(metrics.misses, 1);

        // Remove
        cache.remove(id).await;
        assert!(!cache.contains(id).await);
    }

    #[tokio::test]
    async fn test_adapter_adaptive_features() {
        let config = AdaptiveCacheConfig {
            max_size: 100,
            default_ttl: Duration::from_secs(60),
            hot_threshold: 3,
            ..Default::default()
        };
        let cache = AdaptiveCacheAdapter::new(config);

        let id = Uuid::new_v4();

        // Add entry
        cache.record_access(id, false, None).await;

        // Access multiple times to make it "hot"
        for _ in 0..5 {
            cache.record_access(id, true, None).await;
        }

        // Check hot count
        assert!(cache.hot_count().await > 0);
    }

    #[tokio::test]
    async fn test_adapter_cleanup() {
        let config = AdaptiveCacheConfig {
            max_size: 100,
            default_ttl: Duration::from_millis(10), // Very short TTL
            min_ttl: Duration::from_millis(1),
            enable_background_cleanup: false,
            ..Default::default()
        };
        let cache = AdaptiveCacheAdapter::new(config);

        let id = Uuid::new_v4();
        cache.record_access(id, false, None).await;

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Cleanup expired entries
        let removed = cache.cleanup_expired().await;
        assert!(removed > 0);
        assert!(!cache.contains(id).await);
    }
}
