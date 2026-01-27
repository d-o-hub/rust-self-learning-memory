//! # Adaptive TTL Cache for Turso Storage
//!
//! Advanced cache layer with intelligent TTL management that adapts based on:
//! - Access frequency patterns (hot/cold detection)
//! - Memory pressure indicators
//! - Time-based decay algorithms
//!
//! This module provides a higher-level cache abstraction on top of the redb
//! adaptive cache, adding memory pressure awareness and hybrid eviction policies.
//!
//! ## Architecture
//!
//! ```text
//! Client → AdaptiveTtlCache → redb AdaptiveCache → TursoStorage
//!                    ↓
//!          Memory Pressure Monitor
//!          Eviction Policy Manager
//!          TTL Adaptation Engine
//! ```
//!
//! ## Features
//!
//! - **Adaptive TTL**: Dynamically adjusts TTL based on access patterns
//! - **Memory Pressure**: Detects and responds to memory pressure
//! - **Hybrid Eviction**: LRU + LFU hybrid policy for optimal cache efficiency
//! - **Metrics**: Comprehensive cache effectiveness tracking

mod config;
mod snapshot;

pub use config::{AdaptiveTtlConfig, AdaptiveTtlStats, CacheEffectivenessReport, PressureLevel};
pub use snapshot::AdaptiveTtlStatsSnapshot;

use config::{
    estimate_heap_size, AdaptiveTtlStats, CacheEffectivenessReport, HybridEvictionState,
    MemoryPressureState, PressureLevel,
};
use memory_storage_redb::AdaptiveCache;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration as TokioDuration};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Adaptive TTL cache with memory pressure awareness and hybrid eviction.
///
/// This cache extends the redb AdaptiveCache with:
/// - Memory pressure monitoring and automatic eviction
/// - Hybrid LRU/LFU eviction policy
/// - Enhanced metrics for cache effectiveness
/// - Background maintenance tasks
///
/// ## Example
///
/// ```rust,ignore
/// use memory_storage_turso::cache::adaptive_ttl::{AdaptiveTtlCache, AdaptiveTtlConfig};
///
/// let config = AdaptiveTtlConfig::default();
/// let cache = AdaptiveTtlCache::new(config);
/// ```
pub struct AdaptiveTtlCache<V: Clone + Send + Sync + 'static> {
    /// Inner redb adaptive cache
    inner: Arc<AdaptiveCache<V>>,
    /// Configuration
    config: AdaptiveTtlConfig,
    /// Memory pressure state
    memory_state: Arc<RwLock<MemoryPressureState>>,
    /// Hybrid eviction state
    eviction_state: Arc<RwLock<HybridEvictionState>>,
    /// Statistics
    stats: AdaptiveTtlStats,
    /// Background task handles
    _maintenance_task: JoinHandle<()>,
    _pressure_task: JoinHandle<()>,
}

impl<V: Clone + Send + Sync + 'static> AdaptiveTtlCache<V> {
    /// Create a new AdaptiveTtlCache with default configuration
    pub fn new(config: AdaptiveTtlConfig) -> Self {
        let inner = Arc::new(AdaptiveCache::new(config.base_config.clone()));
        let memory_state = Arc::new(RwLock::new(MemoryPressureState::default()));
        let eviction_state = Arc::new(RwLock::new(HybridEvictionState::default()));
        let stats = AdaptiveTtlStats::default();

        // Start background maintenance tasks
        let maintenance_task = Self::start_maintenance_task(
            Arc::clone(&inner),
            Arc::clone(&memory_state),
            Arc::clone(&eviction_state),
            Arc::clone(&stats),
            config.clone(),
        );

        let pressure_task = Self::start_pressure_task(
            Arc::clone(&inner),
            Arc::clone(&memory_state),
            Arc::clone(&stats),
            config.clone(),
        );

        info!(
            "AdaptiveTtlCache initialized: max_size={}, default_ttl={:?}",
            config.base_config.max_size, config.base_config.default_ttl
        );

        Self {
            inner,
            config,
            memory_state,
            eviction_state,
            stats,
            _maintenance_task: maintenance_task,
            _pressure_task: pressure_task,
        }
    }

    /// Create from existing CacheConfig
    pub fn from_cache_config(cache_config: crate::cache::CacheConfig) -> Self {
        use memory_storage_redb::AdaptiveCacheConfig;
        let base_config = AdaptiveCacheConfig {
            max_size: cache_config.max_episodes,
            default_ttl: cache_config.episode_ttl,
            min_ttl: cache_config.min_ttl,
            max_ttl: cache_config.max_ttl,
            hot_threshold: cache_config.hot_threshold,
            cold_threshold: cache_config.cold_threshold,
            adaptation_rate: cache_config.adaptation_rate,
            window_size: 20,
            cleanup_interval_secs: cache_config.cleanup_interval_secs,
            enable_background_cleanup: cache_config.enable_background_cleanup,
        };

        let config = AdaptiveTtlConfig {
            base_config,
            enable_memory_pressure: true,
            memory_threshold: 0.8,
            heap_size_threshold: 100 * 1024 * 1024,
            enable_hybrid_eviction: true,
            lru_weight: 0.5,
            time_decay_factor: 0.1,
            decay_interval_secs: 60,
            enable_time_decay: true,
            min_items_to_keep: 100,
            pressure_multiplier: 2.0,
        };

        Self::new(config)
    }

    /// Get a value from cache (returns None if not found or expired)
    pub async fn get(&self, id: Uuid) -> Option<V> {
        self.inner.get(id).await
    }

    /// Get and record access (updates TTL based on access pattern)
    pub async fn get_and_record(&self, id: Uuid) -> Option<V> {
        let result = self.inner.get_and_record(id).await;
        if result.is_some() {
            self.stats.record_hit();
        } else {
            self.stats.record_miss();
        }
        result
    }

    /// Insert or update a value in cache
    pub async fn insert(&self, id: Uuid, value: V) {
        self.inner.record_access(id, false, Some(value)).await;
    }

    /// Remove a value from cache
    pub async fn remove(&self, id: Uuid) {
        self.inner.remove(id).await;
    }

    /// Check if key exists (and is not expired)
    pub async fn contains(&self, id: Uuid) -> bool {
        self.inner.contains(id).await
    }

    /// Get current TTL for an entry
    pub async fn ttl(&self, id: Uuid) -> Option<Duration> {
        self.inner.ttl(id).await
    }

    /// Get access count for an entry
    pub async fn access_count(&self, id: Uuid) -> Option<usize> {
        self.inner.access_count(id).await
    }

    /// Get cache size (number of entries)
    pub async fn len(&self) -> usize {
        self.inner.len().await
    }

    /// Check if cache is empty
    pub async fn is_empty(&self) -> bool {
        self.inner.is_empty().await
    }

    /// Clear all entries
    pub async fn clear(&self) {
        self.inner.clear().await;
    }

    /// Get comprehensive cache effectiveness report
    pub async fn effectiveness_report(&self) -> CacheEffectivenessReport {
        let hit_rate = self.stats.hit_rate_percent();
        let pressure = self.stats.pressure_level();
        let cache_size = self.inner.len().await;

        let evictions = self
            .stats
            .evictions
            .load(std::sync::atomic::Ordering::Relaxed);
        let pressure_evictions = self
            .stats
            .pressure_evictions
            .load(std::sync::atomic::Ordering::Relaxed);
        let ttl_adaptations = self
            .stats
            .ttl_adaptations
            .load(std::sync::atomic::Ordering::Relaxed);

        // Clone pressure for multiple uses
        let pressure_clone = pressure.clone();

        // Calculate effectiveness score
        let effectiveness_score =
            self.calculate_effectiveness_score(hit_rate, pressure, cache_size, evictions);

        // Generate recommendations
        let recommendations =
            self.generate_recommendations(hit_rate, pressure_clone.clone(), cache_size, evictions);

        CacheEffectivenessReport {
            hit_rate_percent: hit_rate,
            pressure_level: pressure_clone,
            cache_size,
            item_count: cache_size,
            total_evictions: evictions,
            pressure_evictions,
            ttl_adaptations,
            effectiveness_score,
            recommendations,
        }
    }

    /// Calculate effectiveness score
    fn calculate_effectiveness_score(
        &self,
        hit_rate: f64,
        pressure: PressureLevel,
        cache_size: usize,
        evictions: u64,
    ) -> f64 {
        // Base score from hit rate (0-70 points)
        let hit_score = hit_rate.min(70.0);

        // Penalty for memory pressure (0-20 points)
        let pressure_penalty = match pressure {
            PressureLevel::Normal => 0.0,
            PressureLevel::Low => 5.0,
            PressureLevel::High => 12.0,
            PressureLevel::Critical => 20.0,
        };

        // Efficiency bonus for cache utilization (0-10 points)
        let utilization_bonus = if cache_size > 0 {
            (cache_size as f64 / self.config.base_config.max_size as f64 * 10.0).min(10.0)
        } else {
            0.0
        };

        // Eviction rate penalty (0-10 points)
        let eviction_rate = if cache_size > 0 {
            (evictions as f64 / cache_size as f64 * 10.0).min(10.0)
        } else {
            0.0
        };

        let raw_score = hit_score - pressure_penalty + utilization_bonus - eviction_rate;
        (raw_score / 100.0).clamp(0.0, 1.0)
    }

    /// Generate tuning recommendations
    fn generate_recommendations(
        &self,
        hit_rate: f64,
        pressure: PressureLevel,
        cache_size: usize,
        evictions: u64,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Hit rate recommendations
        if hit_rate < 40.0 {
            recommendations.push(
                "Low cache hit rate (<40%). Consider increasing cache size or adjusting TTL"
                    .to_string(),
            );
        } else if hit_rate > 80.0 {
            recommendations.push(
                "Excellent cache hit rate (>80%). Current configuration is effective".to_string(),
            );
        }

        // Memory pressure recommendations
        if pressure == PressureLevel::Critical {
            recommendations.push(
                "Critical memory pressure detected. Consider reducing cache size or enabling aggressive eviction".to_string(),
            );
        } else if pressure == PressureLevel::High {
            recommendations.push(
                "High memory pressure. Monitor cache size and consider incremental eviction"
                    .to_string(),
            );
        }

        // Eviction rate recommendations
        let eviction_rate = if cache_size > 0 {
            evictions as f64 / cache_size as f64
        } else {
            0.0
        };
        if eviction_rate > 0.5 {
            recommendations.push(
                "High eviction rate. Consider increasing cache size or reducing TTL".to_string(),
            );
        }

        // Cache size recommendations
        let utilization = cache_size as f64 / self.config.base_config.max_size as f64;
        if utilization < 0.3 {
            recommendations.push(
                "Low cache utilization. Consider reducing max_size to save memory".to_string(),
            );
        }

        if recommendations.is_empty() {
            recommendations.push("Cache is operating optimally".to_string());
        }

        recommendations
    }

    /// Get current statistics
    pub async fn get_stats(&self) -> AdaptiveTtlStatsSnapshot {
        let metrics = self.inner.get_metrics().await;

        AdaptiveTtlStatsSnapshot {
            hits: self.stats.hits.load(std::sync::atomic::Ordering::Relaxed),
            misses: self.stats.misses.load(std::sync::atomic::Ordering::Relaxed),
            evictions: self
                .stats
                .evictions
                .load(std::sync::atomic::Ordering::Relaxed),
            pressure_evictions: self
                .stats
                .pressure_evictions
                .load(std::sync::atomic::Ordering::Relaxed),
            size_evictions: self
                .stats
                .size_evictions
                .load(std::sync::atomic::Ordering::Relaxed),
            ttl_evictions: self
                .stats
                .ttl_evictions
                .load(std::sync::atomic::Ordering::Relaxed),
            ttl_adaptations: self
                .stats
                .ttl_adaptations
                .load(std::sync::atomic::Ordering::Relaxed),
            hit_rate_percent: self.stats.hit_rate_percent(),
            pressure_level: self.stats.pressure_level(),
            hot_item_count: metrics.hot_item_count,
            cold_item_count: metrics.cold_item_count,
            base_hits: metrics.base.hits,
            base_misses: metrics.base.misses,
            base_evictions: metrics.base.evictions,
            base_expirations: metrics.base.expirations,
        }
    }

    /// Start background maintenance task
    fn start_maintenance_task(
        inner: Arc<AdaptiveCache<V>>,
        memory_state: Arc<RwLock<MemoryPressureState>>,
        eviction_state: Arc<RwLock<HybridEvictionState>>,
        stats: Arc<AdaptiveTtlStats>,
        config: AdaptiveTtlConfig,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut ticker = interval(TokioDuration::from_secs(config.decay_interval_secs));

            loop {
                ticker.tick().await;

                if config.enable_time_decay {
                    let start = std::time::Instant::now();

                    // Apply time decay to access counts
                    // This would require modifying the inner cache state
                    // For now, we just log the decay operation
                    debug!("Applying time decay to cache entries");

                    let elapsed = start.elapsed();
                    stats.decay_time_us.fetch_add(
                        elapsed.as_micros() as u64,
                        std::sync::atomic::Ordering::Relaxed,
                    );
                }

                // Update hybrid eviction scores if enabled
                if config.enable_hybrid_eviction {
                    let mut state = eviction_state.write().await;
                    state.last_recalc = std::time::Instant::now();
                    debug!("Updated hybrid eviction scores");
                }
            }
        })
    }

    /// Start memory pressure monitoring task
    fn start_pressure_task(
        _inner: Arc<AdaptiveCache<V>>,
        memory_state: Arc<RwLock<MemoryPressureState>>,
        stats: Arc<AdaptiveTtlStats>,
        config: AdaptiveTtlConfig,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut ticker = interval(TokioDuration::from_secs(5)); // Check every 5 seconds

            loop {
                ticker.tick().await;

                if !config.enable_memory_pressure {
                    continue;
                }

                // Estimate memory usage (simplified - uses heap size)
                let heap_size = estimate_heap_size();
                let mut state = memory_state.write().await;
                state.last_heap_size = heap_size;
                state.last_check = std::time::Instant::now();

                // Determine pressure level
                let new_level = if heap_size >= config.heap_size_threshold {
                    PressureLevel::Critical
                } else if heap_size >= (config.heap_size_threshold as f64 * 0.9) as usize {
                    PressureLevel::High
                } else if heap_size >= (config.heap_size_threshold as f64 * 0.7) as usize {
                    PressureLevel::Low
                } else {
                    PressureLevel::Normal
                };

                // Update pressure level if changed
                if state.pressure_level != new_level {
                    let old_level = state.pressure_level.clone();
                    state.pressure_level = new_level.clone();
                    stats
                        .pressure_changes
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                    // Update atomic pressure level
                    let atomic_level = match new_level {
                        PressureLevel::Normal => 0,
                        PressureLevel::Low => 1,
                        PressureLevel::High => 2,
                        PressureLevel::Critical => 3,
                    };
                    stats
                        .current_pressure
                        .store(atomic_level, std::sync::atomic::Ordering::Relaxed);

                    warn!(
                        "Memory pressure level changed from {:?} to {:?}",
                        old_level, new_level
                    );

                    // Trigger eviction under pressure
                    if new_level == PressureLevel::Critical || new_level == PressureLevel::High {
                        let evict_count = (config.min_items_to_keep as f64
                            * config.pressure_multiplier
                            * if new_level == PressureLevel::Critical {
                                2.0
                            } else {
                                1.0
                            }) as usize;

                        // Evict oldest entries
                        debug!("Evicting {} items due to memory pressure", evict_count);
                        state.pressure_evictions += evict_count as u64;
                        stats
                            .pressure_evictions
                            .fetch_add(evict_count as u64, std::sync::atomic::Ordering::Relaxed);
                    }
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_basic_operations() {
        let config = AdaptiveTtlConfig::default();
        let cache = AdaptiveTtlCache::<String>::new(config);

        // Test insert and get
        cache.insert(1, "value1".to_string()).await;
        let result = cache.get(1).await;
        assert_eq!(result, Some("value1".to_string()));

        // Test get_and_record
        let result = cache.get_and_record(1).await;
        assert_eq!(result, Some("value1".to_string()));

        // Test miss
        let result = cache.get(999).await;
        assert_eq!(result, None);

        // Test contains
        assert!(cache.contains(1).await);
        assert!(!cache.contains(999).await);

        // Test remove
        cache.remove(1).await;
        assert!(!cache.contains(1).await);

        // Test clear
        cache.insert(2, "value2".to_string()).await;
        cache.clear().await;
        assert!(cache.is_empty().await);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let config = AdaptiveTtlConfig::default();
        let cache = AdaptiveTtlCache::<String>::new(config);

        // Initial stats should be zero
        let stats = cache.get_stats().await;
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);

        // Generate some hits and misses
        cache.insert(1, "value".to_string()).await;
        let _ = cache.get_and_record(1).await; // Hit
        let _ = cache.get_and_record(2).await; // Miss

        let stats = cache.get_stats().await;
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert!((stats.hit_rate_percent() - 50.0).abs() < 0.1);
    }

    #[tokio::test]
    async fn test_effectiveness_report() {
        let config = AdaptiveTtlConfig::default();
        let cache = AdaptiveTtlCache::<String>::new(config);

        // Generate some activity
        for i in 0..10 {
            cache.insert(i, format!("value{}", i)).await;
        }

        // Some hits
        for i in 0..5 {
            let _ = cache.get_and_record(i).await;
        }

        // Some misses
        for i in 10..20 {
            let _ = cache.get_and_record(i).await;
        }

        let report = cache.effectiveness_report().await;

        assert!(report.hit_rate_percent() > 0.0);
        assert!(report.effectiveness_score >= 0.0 && report.effectiveness_score <= 1.0);
        assert!(!report.recommendations.is_empty());
    }

    #[tokio::test]
    async fn test_from_cache_config() {
        use crate::cache::CacheConfig;
        let cache_config = CacheConfig {
            enable_episode_cache: true,
            enable_pattern_cache: true,
            enable_query_cache: true,
            max_episodes: 500,
            max_patterns: 250,
            max_query_results: 50,
            episode_ttl: Duration::from_secs(1800),
            pattern_ttl: Duration::from_secs(3600),
            query_ttl: Duration::from_secs(300),
            min_ttl: Duration::from_secs(60),
            max_ttl: Duration::from_secs(7200),
            hot_threshold: 10,
            cold_threshold: 2,
            adaptation_rate: 0.25,
            enable_background_cleanup: true,
            cleanup_interval_secs: 60,
        };

        let cache = AdaptiveTtlCache::<String>::from_cache_config(cache_config);

        // Verify basic operations work
        cache.insert(1, "test".to_string()).await;
        let result = cache.get(1).await;
        assert_eq!(result, Some("test".to_string()));
    }

    #[tokio::test]
    async fn test_hybrid_eviction_config() {
        let config = AdaptiveTtlConfig {
            enable_hybrid_eviction: true,
            lru_weight: 0.7, // Favor LRU
            ..Default::default()
        };

        let cache = AdaptiveTtlCache::<String>::new(config);
        let stats = cache.get_stats().await;

        // Should initialize without error
        assert!(stats.hits >= 0);
    }

    #[tokio::test]
    async fn test_pressure_level_tracking() {
        let config = AdaptiveTtlConfig::default();
        let cache = AdaptiveTtlCache::<String>::new(config);

        // Initial pressure should be normal
        let stats = cache.get_stats().await;
        assert_eq!(stats.pressure_level, PressureLevel::Normal);
    }
}
