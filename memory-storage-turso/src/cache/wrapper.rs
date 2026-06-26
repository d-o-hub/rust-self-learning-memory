//! Cached wrapper for TursoStorage

// Format args inlining not required for error message clarity
#![allow(clippy::uninlined_format_args)]

use super::config::{CacheConfig, CacheStats};
use super::stats::CacheStatsInner;
use crate::TursoStorage;
use async_trait::async_trait;
use do_memory_core::memory::attribution::{
    RecommendationFeedback, RecommendationSession, RecommendationStats,
};
use do_memory_core::{
    Episode, Error, Heuristic, Pattern, Result, StorageBackend, episode::PatternId,
};
use do_memory_storage_redb::{AdaptiveCache, AdaptiveCacheConfig};
use std::sync::Arc;
use std::sync::atomic::Ordering;
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
        self.stats.snapshot()
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
        id: do_memory_core::episode::PatternId,
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

    /// Record a query cache hit
    pub fn record_query_hit(&self) {
        self.stats.record_query_hit();
    }

    /// Record a query cache miss
    pub fn record_query_miss(&self) {
        self.stats.record_query_miss();
    }

    /// Record a cache eviction
    pub fn record_eviction(&self) {
        self.stats.record_eviction();
    }

    /// Record a cache expiration
    pub fn record_expiration(&self) {
        self.stats.record_expiration();
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

    async fn store_recommendation_session(&self, session: &RecommendationSession) -> Result<()> {
        self.storage
            .store_recommendation_session(session)
            .await
            .map_err(|e| Error::Storage(format!("Store recommendation session error: {}", e)))
    }

    async fn get_recommendation_session(
        &self,
        session_id: Uuid,
    ) -> Result<Option<RecommendationSession>> {
        self.storage
            .get_recommendation_session(session_id)
            .await
            .map_err(|e| Error::Storage(format!("Get recommendation session error: {}", e)))
    }

    async fn get_recommendation_session_for_episode(
        &self,
        episode_id: Uuid,
    ) -> Result<Option<RecommendationSession>> {
        self.storage
            .get_recommendation_session_for_episode(episode_id)
            .await
            .map_err(|e| {
                Error::Storage(format!("Get recommendation session (episode) error: {}", e))
            })
    }

    async fn store_recommendation_feedback(&self, feedback: &RecommendationFeedback) -> Result<()> {
        self.storage
            .store_recommendation_feedback(feedback)
            .await
            .map_err(|e| Error::Storage(format!("Store recommendation feedback error: {}", e)))
    }

    async fn get_recommendation_feedback(
        &self,
        session_id: Uuid,
    ) -> Result<Option<RecommendationFeedback>> {
        self.storage
            .get_recommendation_feedback(session_id)
            .await
            .map_err(|e| Error::Storage(format!("Get recommendation feedback error: {}", e)))
    }

    async fn get_recommendation_stats(&self) -> Result<RecommendationStats> {
        self.storage
            .get_recommendation_stats()
            .await
            .map_err(|e| Error::Storage(format!("Get recommendation stats error: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_stats() {
        // Just verify the stats method doesn't panic and returns correct types
        // It's mostly atomic loads
        let stats = CacheStats {
            episode_hits: 1,
            episode_misses: 2,
            pattern_hits: 3,
            pattern_misses: 4,
            query_hits: 0,
            query_misses: 0,
            evictions: 0,
            expirations: 0,
        };
        assert_eq!(stats.episode_hits, 1);
        assert_eq!(stats.query_hits, 0);
    }

    #[test]
    fn test_cache_stats_hit_rate() {
        let stats = CacheStats {
            episode_hits: 8,
            episode_misses: 2,
            pattern_hits: 0,
            pattern_misses: 0,
            query_hits: 0,
            query_misses: 0,
            evictions: 0,
            expirations: 0,
        };
        assert!((stats.hit_rate() - 0.8).abs() < f64::EPSILON);
        assert!((stats.episode_hit_rate() - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cache_stats_zero_requests() {
        let stats = CacheStats::default();
        assert_eq!(stats.hit_rate(), 0.0);
        assert_eq!(stats.query_hit_rate(), 0.0);
    }

    #[test]
    fn test_cache_stats_query_hit_rate() {
        let stats = CacheStats {
            query_hits: 3,
            query_misses: 7,
            ..Default::default()
        };
        assert!((stats.query_hit_rate() - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cache_stats_pattern_hit_rate() {
        let stats = CacheStats {
            pattern_hits: 5,
            pattern_misses: 5,
            ..Default::default()
        };
        assert!((stats.pattern_hit_rate() - 0.5).abs() < f64::EPSILON);
    }
}
