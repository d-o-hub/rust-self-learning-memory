//! Configuration and state types for Adaptive TTL cache

use crate::cache::CacheConfig;
use memory_storage_redb::{AdaptiveCache, AdaptiveCacheConfig};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;
use tracing::warn;
use uuid::Uuid;

/// Configuration for AdaptiveTtlCache
///
/// Provides fine-grained control over TTL adaptation, memory pressure
/// response, and eviction policies.
#[derive(Debug, Clone)]
pub struct AdaptiveTtlConfig {
    /// Base adaptive cache configuration
    pub base_config: AdaptiveCacheConfig,
    /// Enable memory pressure monitoring
    pub enable_memory_pressure: bool,
    /// Memory threshold percentage (0.0 - 1.0) to trigger eviction
    pub memory_threshold: f64,
    /// Heap size threshold in bytes for pressure detection
    pub heap_size_threshold: usize,
    /// Enable hybrid LRU/LFU eviction
    pub enable_hybrid_eviction: bool,
    /// LRU weight in hybrid policy (0.0 = pure LFU, 1.0 = pure LRU)
    pub lru_weight: f64,
    /// Time decay factor for access frequency (0.0 - 1.0)
    /// Higher values mean faster decay
    pub time_decay_factor: f64,
    /// Decay interval in seconds
    pub decay_interval_secs: u64,
    /// Enable time-based decay of access counts
    pub enable_time_decay: bool,
    /// Minimum items to keep regardless of memory pressure
    pub min_items_to_keep: usize,
    /// Aggressive eviction multiplier under pressure
    pub pressure_multiplier: f64,
}

impl Default for AdaptiveTtlConfig {
    fn default() -> Self {
        Self {
            base_config: AdaptiveCacheConfig::default(),
            enable_memory_pressure: true,
            memory_threshold: 0.8, // 80% memory usage triggers eviction
            heap_size_threshold: 100 * 1024 * 1024, // 100MB
            enable_hybrid_eviction: true,
            lru_weight: 0.5,        // Equal LRU/LFU
            time_decay_factor: 0.1, // 10% decay per interval
            decay_interval_secs: 60,
            enable_time_decay: true,
            min_items_to_keep: 100,
            pressure_multiplier: 2.0,
        }
    }
}

/// Conversion from AdaptiveTtlConfig to standard CacheConfig
impl From<AdaptiveTtlConfig> for CacheConfig {
    fn from(config: AdaptiveTtlConfig) -> Self {
        CacheConfig {
            enable_episode_cache: true,
            enable_pattern_cache: true,
            enable_query_cache: true,
            max_episodes: config.base_config.max_size,
            max_patterns: config.base_config.max_size / 2,
            max_query_results: config.base_config.max_size / 10,
            episode_ttl: config.base_config.default_ttl,
            pattern_ttl: config.base_config.default_ttl,
            query_ttl: Duration::from_secs(300),
            min_ttl: config.base_config.min_ttl,
            max_ttl: config.base_config.max_ttl,
            hot_threshold: config.base_config.hot_threshold,
            cold_threshold: config.base_config.cold_threshold,
            adaptation_rate: config.base_config.adaptation_rate,
            enable_background_cleanup: config.base_config.enable_background_cleanup,
            cleanup_interval_secs: config.base_config.cleanup_interval_secs,
        }
    }
}

/// Memory pressure detection state
#[derive(Debug)]
struct MemoryPressureState {
    /// Last measured heap size
    last_heap_size: usize,
    /// Current pressure level
    pressure_level: PressureLevel,
    /// Number of evictions due to memory pressure
    pressure_evictions: u64,
    /// Timestamp of last pressure check
    last_check: std::time::Instant,
}

impl Default for MemoryPressureState {
    fn default() -> Self {
        Self {
            last_heap_size: 0,
            pressure_level: PressureLevel::Normal,
            pressure_evictions: 0,
            last_check: std::time::Instant::now(),
        }
    }
}

/// Memory pressure levels
#[derive(Debug, Clone, PartialEq)]
pub enum PressureLevel {
    /// Normal operation, no pressure
    Normal,
    /// Slightly elevated memory usage
    Low,
    /// High memory usage, active eviction
    High,
    /// Critical memory, aggressive eviction
    Critical,
}

impl Default for PressureLevel {
    fn default() -> Self {
        PressureLevel::Normal
    }
}

/// Hybrid eviction state combining LRU and LFU
#[derive(Debug)]
struct HybridEvictionState {
    /// LRU scores (timestamp of last access)
    lru_scores: Vec<(Uuid, u64)>, // (id, last_access_nanos)
    /// LFU scores (access count)
    lfu_scores: Vec<(Uuid, usize)>, // (id, access_count)
    /// Hybrid scores
    hybrid_scores: Vec<(Uuid, f64)>, // (id, combined_score)
    /// Last recalculation time
    last_recalc: std::time::Instant,
}

impl Default for HybridEvictionState {
    fn default() -> Self {
        Self {
            lru_scores: Vec::new(),
            lfu_scores: Vec::new(),
            hybrid_scores: Vec::new(),
            last_recalc: std::time::Instant::now(),
        }
    }
}

/// Statistics for AdaptiveTtlCache
#[derive(Debug, Default)]
pub struct AdaptiveTtlStats {
    /// Total cache hits
    pub hits: AtomicU64,
    /// Total cache misses
    pub misses: AtomicU64,
    /// Total evictions
    pub evictions: AtomicU64,
    /// Evictions due to memory pressure
    pub pressure_evictions: AtomicU64,
    /// Evictions due to size limit
    pub size_evictions: AtomicU64,
    /// Evictions due to TTL expiration
    pub ttl_evictions: AtomicU64,
    /// Number of TTL adaptations
    pub ttl_adaptations: AtomicU64,
    /// Number of pressure level changes
    pub pressure_changes: AtomicU64,
    /// Total time spent on decay operations (microseconds)
    pub decay_time_us: AtomicU64,
    /// Current memory pressure level
    pub current_pressure: AtomicU64, // Store as u64 for atomic
    /// Hit rate (computed on demand)
    pub hit_rate: AtomicU64, // Stored as u64 representing percentage
}

impl AdaptiveTtlStats {
    /// Record a cache hit
    pub fn record_hit(&self) {
        self.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.update_hit_rate();
    }

    /// Record a cache miss
    pub fn record_miss(&self) {
        self.misses
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.update_hit_rate();
    }

    /// Update hit rate atomically
    fn update_hit_rate(&self) {
        let hits = self.hits.load(std::sync::atomic::Ordering::Relaxed);
        let misses = self.misses.load(std::sync::atomic::Ordering::Relaxed);
        let total = hits + misses;
        let rate = if total > 0 {
            (hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        self.hit_rate
            .store(rate as u64, std::sync::atomic::Ordering::Relaxed);
    }

    /// Get current hit rate as percentage
    pub fn hit_rate_percent(&self) -> f64 {
        self.hit_rate.load(std::sync::atomic::Ordering::Relaxed) as f64
    }

    /// Get current pressure level
    pub fn pressure_level(&self) -> PressureLevel {
        match self
            .current_pressure
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            0 => PressureLevel::Normal,
            1 => PressureLevel::Low,
            2 => PressureLevel::High,
            3 => PressureLevel::Critical,
            _ => PressureLevel::Normal,
        }
    }
}

/// Cache effectiveness report
#[derive(Debug, Clone)]
pub struct CacheEffectivenessReport {
    /// Overall hit rate percentage
    pub hit_rate_percent: f64,
    /// Current memory pressure level
    pub pressure_level: PressureLevel,
    /// Total cache size
    pub cache_size: usize,
    /// Number of items in cache
    pub item_count: usize,
    /// Total evictions
    pub total_evictions: u64,
    /// Pressure evictions
    pub pressure_evictions: u64,
    /// TTL adaptation count
    pub ttl_adaptations: u64,
    /// Cache effectiveness score (0.0 - 1.0)
    pub effectiveness_score: f64,
    /// Recommendations for tuning
    pub recommendations: Vec<String>,
}

/// Estimate current heap size (simplified)
pub(crate) fn estimate_heap_size() -> usize {
    // This is a simplified estimate
    // In production, you'd use more accurate metrics
    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                // Parse RSS value in kB
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<usize>() {
                        return kb * 1024; // Convert KB to bytes
                    }
                }
            }
        }
    }
    // Fallback: return 0 (unknown)
    0
}
