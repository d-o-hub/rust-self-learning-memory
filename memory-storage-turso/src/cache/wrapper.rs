//! Cached wrapper for TursoStorage

use super::config::{CacheConfig, CacheStats};
use crate::TursoStorage;
use async_trait::async_trait;
use memory_core::{episode::PatternId, Episode, Error, Heuristic, Pattern, Result, StorageBackend};
use memory_storage_redb::{AdaptiveCache, AdaptiveCacheConfig};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use uuid::Uuid;

/// Cached wrapper around TursoStorage
///
/// Provides transparent caching for episodes, patterns, and heuristics
/// using adaptive TTL based on access patterns.
pub struct CachedTursoStorage {
    /// Underlying Turso storage
    storage: Arc<TursoStorage>,

    /// Episode cache
    episode_cache: Option<AdaptiveCache<Episode>>,

    /// Pattern cache
    pattern_cache: Option<AdaptiveCache<Pattern>>,

    /// Heuristic cache
    heuristic_cache: Option<AdaptiveCache<Heuristic>>,

    /// Cache configuration
    config: CacheConfig,

    /// Cache statistics
    stats: CacheStatsInner,
}

/// Internal cache statistics with atomic counters
#[derive(Default)]
struct CacheStatsInner {
    episode_hits: AtomicU64,
    episode_misses: AtomicU64,
    pattern_hits: AtomicU64,
    pattern_misses: AtomicU64,
    heuristic_hits: AtomicU64,
    heuristic_misses: AtomicU64,
}

impl CachedTursoStorage {
    /// Create a new cached storage wrapper
    pub fn new(storage: TursoStorage, config: CacheConfig) -> Self {
        // Create episode cache if enabled
        let episode_cache = if config.enable_episode_cache {
            let cache_config = AdaptiveCacheConfig {
                max_size: config.max_episodes,
                default_ttl: config.episode_ttl,
                min_ttl: config.min_ttl,
                max_ttl: config.max_ttl,
                hot_threshold: config.hot_threshold,
                cold_threshold: config.cold_threshold,
                adaptation_rate: config.adaptation_rate,
                window_size: 20,
                cleanup_interval_secs: config.cleanup_interval_secs,
                enable_background_cleanup: config.enable_background_cleanup,
            };
            Some(AdaptiveCache::new(cache_config))
        } else {
            None
        };

        // Create pattern cache if enabled
        let pattern_cache = if config.enable_pattern_cache {
            let cache_config = AdaptiveCacheConfig {
                max_size: config.max_patterns,
                default_ttl: config.pattern_ttl,
                min_ttl: config.min_ttl,
                max_ttl: config.max_ttl,
                hot_threshold: config.hot_threshold,
                cold_threshold: config.cold_threshold,
                adaptation_rate: config.adaptation_rate,
                window_size: 20,
                cleanup_interval_secs: config.cleanup_interval_secs,
                enable_background_cleanup: config.enable_background_cleanup,
            };
            Some(AdaptiveCache::new(cache_config))
        } else {
            None
        };

        // Create heuristic cache (smaller, same config as patterns)
        let heuristic_cache = if config.enable_pattern_cache {
            let cache_config = AdaptiveCacheConfig {
                max_size: config.max_patterns / 2, // Half the size of patterns
                default_ttl: config.pattern_ttl,
                min_ttl: config.min_ttl,
                max_ttl: config.max_ttl,
                hot_threshold: config.hot_threshold,
                cold_threshold: config.cold_threshold,
                adaptation_rate: config.adaptation_rate,
                window_size: 20,
                cleanup_interval_secs: config.cleanup_interval_secs,
                enable_background_cleanup: config.enable_background_cleanup,
            };
            Some(AdaptiveCache::new(cache_config))
        } else {
            None
        };

        Self {
            storage: Arc::new(storage),
            episode_cache,
            pattern_cache,
            heuristic_cache,
            config,
            stats: CacheStatsInner::default(),
        }
    }

    /// Get the underlying storage (for operations that bypass cache)
    pub fn storage(&self) -> &TursoStorage {
        &self.storage
    }

    /// Get cache configuration
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            episode_hits: self.stats.episode_hits.load(Ordering::Relaxed),
            episode_misses: self.stats.episode_misses.load(Ordering::Relaxed),
            pattern_hits: self.stats.pattern_hits.load(Ordering::Relaxed),
            pattern_misses: self.stats.pattern_misses.load(Ordering::Relaxed),
            query_hits: 0, // Not yet implemented
            query_misses: 0,
            evictions: 0, // Requires async access, use cache_sizes() + stats for accurate count
            expirations: 0,
        }
    }

    /// Get episode with caching
    pub async fn get_episode_cached(&self, id: Uuid) -> Result<Option<Episode>> {
        // Check cache first
        if let Some(ref cache) = self.episode_cache {
            if let Some(episode) = cache.get_and_record(id).await {
                self.stats.episode_hits.fetch_add(1, Ordering::Relaxed);
                return Ok(Some(episode));
            }
        }

        // Cache miss - fetch from storage
        self.stats.episode_misses.fetch_add(1, Ordering::Relaxed);
        let episode = self.storage.get_episode(id).await?;

        // Store in cache if found
        if let (Some(ep), Some(cache)) = (&episode, &self.episode_cache) {
            cache.record_access(id, false, Some(ep.clone())).await;
        }

        Ok(episode)
    }

    /// Get pattern with caching
    pub async fn get_pattern_cached(
        &self,
        id: memory_core::episode::PatternId,
    ) -> Result<Option<Pattern>> {
        // PatternId is already a Uuid, use directly
        let cache_key = id;

        // Check cache first
        if let Some(ref cache) = self.pattern_cache {
            if let Some(pattern) = cache.get_and_record(cache_key).await {
                self.stats.pattern_hits.fetch_add(1, Ordering::Relaxed);
                return Ok(Some(pattern));
            }
        }

        // Cache miss - fetch from storage
        self.stats.pattern_misses.fetch_add(1, Ordering::Relaxed);
        let pattern = self.storage.get_pattern(id).await?;

        // Store in cache if found
        if let (Some(pat), Some(cache)) = (&pattern, &self.pattern_cache) {
            cache
                .record_access(cache_key, false, Some(pat.clone()))
                .await;
        }

        Ok(pattern)
    }

    /// Get heuristic with caching
    pub async fn get_heuristic_cached(&self, id: Uuid) -> Result<Option<Heuristic>> {
        // Check cache first
        if let Some(ref cache) = self.heuristic_cache {
            if let Some(heuristic) = cache.get_and_record(id).await {
                self.stats.heuristic_hits.fetch_add(1, Ordering::Relaxed);
                return Ok(Some(heuristic));
            }
        }

        // Cache miss - fetch from storage
        self.stats.heuristic_misses.fetch_add(1, Ordering::Relaxed);
        let heuristic = self.storage.get_heuristic(id).await?;

        // Store in cache if found
        if let (Some(h), Some(cache)) = (&heuristic, &self.heuristic_cache) {
            cache.record_access(id, false, Some(h.clone())).await;
        }

        Ok(heuristic)
    }

    /// Store episode (invalidates cache entry)
    pub async fn store_episode_cached(&self, episode: &Episode) -> Result<()> {
        // Store in database first
        self.storage.store_episode(episode).await?;

        // Invalidate cache entry
        if let Some(ref cache) = self.episode_cache {
            cache.remove(episode.episode_id).await;
        }

        Ok(())
    }

    /// Store pattern (invalidates cache entry)
    pub async fn store_pattern_cached(&self, pattern: &Pattern) -> Result<()> {
        // Store in database first
        self.storage.store_pattern(pattern).await?;

        // Invalidate cache entry
        if let Some(ref cache) = self.pattern_cache {
            cache.remove(pattern.id()).await;
        }

        Ok(())
    }

    /// Store heuristic (invalidates cache entry)
    pub async fn store_heuristic_cached(&self, heuristic: &Heuristic) -> Result<()> {
        // Store in database first
        self.storage.store_heuristic(heuristic).await?;

        // Invalidate cache entry
        if let Some(ref cache) = self.heuristic_cache {
            cache.remove(heuristic.heuristic_id).await;
        }

        Ok(())
    }

    /// Delete episode (invalidates cache entry)
    pub async fn delete_episode_cached(&self, id: Uuid) -> Result<()> {
        // Delete from database first
        self.storage.delete_episode(id).await?;

        // Invalidate cache entry
        if let Some(ref cache) = self.episode_cache {
            cache.remove(id).await;
        }

        Ok(())
    }

    /// Clear all caches
    pub async fn clear_caches(&self) {
        if let Some(ref cache) = self.episode_cache {
            cache.clear().await;
        }
        if let Some(ref cache) = self.pattern_cache {
            cache.clear().await;
        }
        if let Some(ref cache) = self.heuristic_cache {
            cache.clear().await;
        }
    }

    /// Get cache sizes
    pub async fn cache_sizes(&self) -> (usize, usize, usize) {
        let episode_size = if let Some(ref cache) = self.episode_cache {
            cache.len().await
        } else {
            0
        };

        let pattern_size = if let Some(ref cache) = self.pattern_cache {
            cache.len().await
        } else {
            0
        };

        let heuristic_size = if let Some(ref cache) = self.heuristic_cache {
            cache.len().await
        } else {
            0
        };

        (episode_size, pattern_size, heuristic_size)
    }
}

#[async_trait]
impl StorageBackend for CachedTursoStorage {
    async fn store_episode(&self, episode: &Episode) -> Result<()> {
        self.store_episode_cached(episode)
            .await
            .map_err(|e| Error::Storage(format!("Cache store error: {}", e)))
    }

    async fn get_episode(&self, id: Uuid) -> Result<Option<Episode>> {
        self.get_episode_cached(id)
            .await
            .map_err(|e| Error::Storage(format!("Cache get error: {}", e)))
    }

    async fn delete_episode(&self, id: Uuid) -> Result<()> {
        self.delete_episode_cached(id)
            .await
            .map_err(|e| Error::Storage(format!("Cache delete error: {}", e)))
    }

    async fn store_pattern(&self, pattern: &Pattern) -> Result<()> {
        self.store_pattern_cached(pattern)
            .await
            .map_err(|e| Error::Storage(format!("Cache store error: {}", e)))
    }

    async fn get_pattern(&self, id: PatternId) -> Result<Option<Pattern>> {
        self.get_pattern_cached(id)
            .await
            .map_err(|e| Error::Storage(format!("Cache get error: {}", e)))
    }

    async fn store_heuristic(&self, heuristic: &Heuristic) -> Result<()> {
        self.store_heuristic_cached(heuristic)
            .await
            .map_err(|e| Error::Storage(format!("Cache store error: {}", e)))
    }

    async fn get_heuristic(&self, id: Uuid) -> Result<Option<Heuristic>> {
        self.get_heuristic_cached(id)
            .await
            .map_err(|e| Error::Storage(format!("Cache get error: {}", e)))
    }

    async fn query_episodes_since(
        &self,
        since: chrono::DateTime<chrono::Utc>,
        limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        // Query caching not implemented for this method
        // Fall back to underlying storage
        self.storage
            .query_episodes_since(since, limit)
            .await
            .map_err(|e| Error::Storage(format!("Query error: {}", e)))
    }

    async fn query_episodes_by_metadata(
        &self,
        key: &str,
        value: &str,
        limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        // Query caching not implemented for this method
        // Fall back to underlying storage
        self.storage
            .query_episodes_by_metadata(key, value, limit)
            .await
            .map_err(|e| Error::Storage(format!("Query error: {}", e)))
    }

    async fn store_embedding(&self, id: &str, embedding: Vec<f32>) -> Result<()> {
        self.storage
            .store_embedding(id, embedding)
            .await
            .map_err(|e| Error::Storage(format!("Store embedding error: {}", e)))
    }

    async fn get_embedding(&self, id: &str) -> Result<Option<Vec<f32>>> {
        self.storage
            .get_embedding(id)
            .await
            .map_err(|e| Error::Storage(format!("Get embedding error: {}", e)))
    }

    async fn delete_embedding(&self, id: &str) -> Result<bool> {
        self.storage
            .delete_embedding(id)
            .await
            .map_err(|e| Error::Storage(format!("Delete embedding error: {}", e)))
    }

    async fn store_embeddings_batch(&self, embeddings: Vec<(String, Vec<f32>)>) -> Result<()> {
        self.storage
            .store_embeddings_batch(embeddings)
            .await
            .map_err(|e| Error::Storage(format!("Batch store embeddings error: {}", e)))
    }

    async fn get_embeddings_batch(&self, ids: &[String]) -> Result<Vec<Option<Vec<f32>>>> {
        self.storage
            .get_embeddings_batch(ids)
            .await
            .map_err(|e| Error::Storage(format!("Batch get embeddings error: {}", e)))
    }
}
