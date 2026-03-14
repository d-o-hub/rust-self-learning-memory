//! Adaptive TTL cache methods for TursoStorage.
//!
//! This module contains the adaptive TTL caching methods that were extracted
//! from helpers.rs to meet the 500 LOC file size limit.

#[cfg(feature = "adaptive-ttl")]
use crate::cache::{AdaptiveTTLCache, TTLConfig};
#[cfg(feature = "adaptive-ttl")]
use memory_core::Episode;

#[cfg(feature = "adaptive-ttl")]
impl super::TursoStorage {
    // ========== Adaptive TTL Cache Methods ==========

    /// Enable adaptive TTL caching for episode queries
    ///
    /// When enabled, episode query results are cached with adaptive TTL based on
    /// access patterns. Frequently accessed episodes get longer TTL, while rarely
    /// accessed ones get shorter TTL.
    ///
    /// # Arguments
    ///
    /// * `config` - TTL configuration for the cache
    ///
    /// # Returns
    ///
    /// The previous cache if one existed, or None
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use memory_storage_turso::{TursoStorage, TTLConfig};
    /// # use std::time::Duration;
    /// # async fn example(storage: &mut TursoStorage) -> anyhow::Result<()> {
    /// let config = TTLConfig::default()
    ///     .with_base_ttl(Duration::from_secs(300))
    ///     .with_max_entries(1000);
    ///
    /// storage.enable_episode_cache(config)?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "adaptive-ttl")]
    pub fn enable_episode_cache(
        &mut self,
        config: TTLConfig,
    ) -> crate::Result<Option<AdaptiveTTLCache<String, Episode>>> {
        let cache = AdaptiveTTLCache::new(config).map_err(|e| {
            crate::Error::Storage(format!("Failed to create adaptive TTL cache: {}", e))
        })?;
        Ok(self.episode_cache.replace(cache))
    }

    /// Enable adaptive TTL caching with default configuration
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use memory_storage_turso::TursoStorage;
    /// # async fn example(storage: &mut TursoStorage) -> anyhow::Result<()> {
    /// storage.enable_episode_cache_default();
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "adaptive-ttl")]
    pub fn enable_episode_cache_default(&mut self) -> crate::Result<()> {
        let cache = AdaptiveTTLCache::default_config().map_err(|e| {
            crate::Error::Storage(format!("Failed to create adaptive TTL cache: {}", e))
        })?;
        self.episode_cache = Some(cache);
        Ok(())
    }

    /// Disable and remove the episode cache
    ///
    /// # Returns
    ///
    /// The previous cache if one existed, or None
    #[cfg(feature = "adaptive-ttl")]
    pub fn disable_episode_cache(&mut self) -> Option<AdaptiveTTLCache<String, Episode>> {
        self.episode_cache.take()
    }

    /// Check if episode caching is enabled
    #[cfg(feature = "adaptive-ttl")]
    pub fn is_episode_cache_enabled(&self) -> bool {
        self.episode_cache.is_some()
    }

    /// Get episode cache statistics
    ///
    /// Returns cache hit rate, evictions, and other metrics if caching is enabled.
    #[cfg(feature = "adaptive-ttl")]
    pub fn episode_cache_stats(&self) -> Option<crate::cache::CacheStatsSnapshot> {
        self.episode_cache.as_ref().map(|c| c.stats())
    }

    /// Clear the episode cache
    #[cfg(feature = "adaptive-ttl")]
    pub async fn clear_episode_cache(&self) {
        if let Some(ref cache) = self.episode_cache {
            cache.clear().await;
        }
    }

    /// Get the number of entries in the episode cache
    #[cfg(feature = "adaptive-ttl")]
    pub async fn episode_cache_len(&self) -> Option<usize> {
        if let Some(ref cache) = self.episode_cache {
            Some(cache.len().await)
        } else {
            None
        }
    }
}
