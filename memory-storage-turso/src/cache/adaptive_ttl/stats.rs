use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// Statistics for the adaptive TTL cache
#[derive(Debug, Default)]
pub struct CacheStats {
    /// Total number of cache hits
    hits: AtomicU64,
    /// Total number of cache misses
    misses: AtomicU64,
    /// Total number of evictions (size limit)
    evictions: AtomicU64,
    /// Total number of TTL-based expirations
    ttl_expirations: AtomicU64,
    /// Total number of explicit removals
    removals: AtomicU64,
    /// Total number of TTL adaptations
    ttl_adaptations: AtomicU64,
    /// Sum of all TTL values (for calculating average)
    ttl_sum_micros: AtomicU64,
    /// Number of TTL samples
    ttl_samples: AtomicU64,
    /// Current number of entries
    entry_count: AtomicU64,
    /// Peak number of entries
    peak_entries: AtomicU64,
    /// Total cleanup operations performed
    cleanup_operations: AtomicU64,
    /// Total bytes evicted (estimated)
    bytes_evicted: AtomicU64,
}

impl CacheStats {
    /// Create new empty stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a cache hit
    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache miss
    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Record an eviction
    pub fn record_eviction(&self, estimated_bytes: u64) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
        self.bytes_evicted
            .fetch_add(estimated_bytes, Ordering::Relaxed);
    }

    /// Record a TTL expiration
    pub fn record_ttl_expiration(&self) {
        self.ttl_expirations.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a removal
    pub fn record_removal(&self) {
        self.removals.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a TTL adaptation
    pub fn record_ttl_adaptation(&self, ttl: Duration) {
        self.ttl_adaptations.fetch_add(1, Ordering::Relaxed);
        self.ttl_sum_micros
            .fetch_add(ttl.as_micros() as u64, Ordering::Relaxed);
        self.ttl_samples.fetch_add(1, Ordering::Relaxed);
    }

    /// Update entry count
    pub fn update_entry_count(&self, count: usize) {
        let count_u64 = count as u64;
        self.entry_count.store(count_u64, Ordering::Relaxed);

        let mut peak = self.peak_entries.load(Ordering::Relaxed);
        while count_u64 > peak {
            match self.peak_entries.compare_exchange_weak(
                peak,
                count_u64,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => peak = actual,
            }
        }
    }

    /// Record a cleanup operation
    pub fn record_cleanup(&self) {
        self.cleanup_operations.fetch_add(1, Ordering::Relaxed);
    }

    /// Get the current hit rate (0.0 - 1.0)
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;

        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    /// Get the current hit rate as a percentage
    pub fn hit_rate_percent(&self) -> f64 {
        self.hit_rate() * 100.0
    }

    /// Get total number of hits
    pub fn hits(&self) -> u64 {
        self.hits.load(Ordering::Relaxed)
    }

    /// Get total number of misses
    pub fn misses(&self) -> u64 {
        self.misses.load(Ordering::Relaxed)
    }

    /// Get total number of evictions
    pub fn evictions(&self) -> u64 {
        self.evictions.load(Ordering::Relaxed)
    }

    /// Get total number of TTL expirations
    pub fn ttl_expirations(&self) -> u64 {
        self.ttl_expirations.load(Ordering::Relaxed)
    }

    /// Get total number of removals
    pub fn removals(&self) -> u64 {
        self.removals.load(Ordering::Relaxed)
    }

    /// Get total number of TTL adaptations
    pub fn ttl_adaptations(&self) -> u64 {
        self.ttl_adaptations.load(Ordering::Relaxed)
    }

    /// Get average TTL in seconds
    pub fn average_ttl_secs(&self) -> f64 {
        let sum = self.ttl_sum_micros.load(Ordering::Relaxed);
        let samples = self.ttl_samples.load(Ordering::Relaxed);

        if samples == 0 {
            0.0
        } else {
            (sum as f64 / samples as f64) / 1_000_000.0
        }
    }

    /// Get current entry count
    pub fn entry_count(&self) -> usize {
        self.entry_count.load(Ordering::Relaxed) as usize
    }

    /// Get peak entry count
    pub fn peak_entries(&self) -> usize {
        self.peak_entries.load(Ordering::Relaxed) as usize
    }

    /// Get total cleanup operations
    pub fn cleanup_operations(&self) -> u64 {
        self.cleanup_operations.load(Ordering::Relaxed)
    }

    /// Get total bytes evicted
    pub fn bytes_evicted(&self) -> u64 {
        self.bytes_evicted.load(Ordering::Relaxed)
    }

    /// Get a snapshot of all statistics
    pub fn snapshot(&self) -> CacheStatsSnapshot {
        CacheStatsSnapshot {
            hits: self.hits(),
            misses: self.misses(),
            evictions: self.evictions(),
            ttl_expirations: self.ttl_expirations(),
            removals: self.removals(),
            ttl_adaptations: self.ttl_adaptations(),
            hit_rate: self.hit_rate(),
            hit_rate_percent: self.hit_rate_percent(),
            average_ttl_secs: self.average_ttl_secs(),
            entry_count: self.entry_count(),
            peak_entries: self.peak_entries(),
            cleanup_operations: self.cleanup_operations(),
            bytes_evicted: self.bytes_evicted(),
        }
    }
}

/// A snapshot of cache statistics at a point in time
#[derive(Debug, Clone)]
pub struct CacheStatsSnapshot {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub ttl_expirations: u64,
    pub removals: u64,
    pub ttl_adaptations: u64,
    pub hit_rate: f64,
    pub hit_rate_percent: f64,
    pub average_ttl_secs: f64,
    pub entry_count: usize,
    pub peak_entries: usize,
    pub cleanup_operations: u64,
    pub bytes_evicted: u64,
}

impl CacheStatsSnapshot {
    pub fn is_effective(&self) -> bool {
        self.hit_rate > 0.8
    }

    pub fn total_operations(&self) -> u64 {
        self.hits + self.misses
    }

    pub fn eviction_rate(&self) -> f64 {
        let total = self.total_operations();
        if total == 0 {
            0.0
        } else {
            (self.evictions as f64 / total as f64) * 1000.0
        }
    }
}
