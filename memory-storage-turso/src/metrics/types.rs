//! # Metrics Types and Statistics
//!
//! Type definitions and statistics structures for metrics collection.

// Type definitions and statistics structures for metrics collection.

/// Maximum number of latency samples to retain per operation
const MAX_LATENCY_SAMPLES: usize = 1000;

/// Operation types for metrics tracking
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum OperationType {
    /// Episode creation
    EpisodeCreate,
    /// Episode retrieval
    EpisodeGet,
    /// Episode list/query
    EpisodeList,
    /// Episode update
    EpisodeUpdate,
    /// Episode delete
    EpisodeDelete,
    /// Pattern extraction
    PatternExtract,
    /// Pattern retrieval
    PatternGet,
    /// Pattern list/query
    PatternList,
    /// Embedding operations
    Embedding,
    /// Generic query
    Query,
    /// Batch operations
    Batch,
    /// Connection acquisition
    ConnectionAcquire,
    /// Health check
    HealthCheck,
    /// Other/custom operation
    Custom(String),
}

impl From<&str> for OperationType {
    fn from(s: &str) -> Self {
        match s {
            "episode_create" => Self::EpisodeCreate,
            "episode_get" => Self::EpisodeGet,
            "episode_list" => Self::EpisodeList,
            "episode_update" => Self::EpisodeUpdate,
            "episode_delete" => Self::EpisodeDelete,
            "pattern_extract" => Self::PatternExtract,
            "pattern_get" => Self::PatternGet,
            "pattern_list" => Self::PatternList,
            "embedding" => Self::Embedding,
            "query" => Self::Query,
            "batch" => Self::Batch,
            "connection_acquire" => Self::ConnectionAcquire,
            "health_check" => Self::HealthCheck,
            _ => Self::Custom(s.to_string()),
        }
    }
}

impl std::fmt::Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EpisodeCreate => write!(f, "episode_create"),
            Self::EpisodeGet => write!(f, "episode_get"),
            Self::EpisodeList => write!(f, "episode_list"),
            Self::EpisodeUpdate => write!(f, "episode_update"),
            Self::EpisodeDelete => write!(f, "episode_delete"),
            Self::PatternExtract => write!(f, "pattern_extract"),
            Self::PatternGet => write!(f, "pattern_get"),
            Self::PatternList => write!(f, "pattern_list"),
            Self::Embedding => write!(f, "embedding"),
            Self::Query => write!(f, "query"),
            Self::Batch => write!(f, "batch"),
            Self::ConnectionAcquire => write!(f, "connection_acquire"),
            Self::HealthCheck => write!(f, "health_check"),
            Self::Custom(s) => write!(f, "custom:{}", s),
        }
    }
}

/// Latency statistics for an operation
#[derive(Debug, Default, Clone)]
pub struct LatencyStats {
    /// Count of samples
    pub count: u64,
    /// Minimum latency (microseconds)
    pub min_us: u64,
    /// Maximum latency (microseconds)
    pub max_us: u64,
    /// Sum of all latencies (microseconds)
    pub sum_us: u64,
    /// Latency samples for percentile calculation
    samples: Vec<u64>,
}

impl LatencyStats {
    /// Create new latency statistics
    pub fn new() -> Self {
        Self {
            count: 0,
            min_us: u64::MAX,
            max_us: 0,
            sum_us: 0,
            samples: Vec::with_capacity(MAX_LATENCY_SAMPLES),
        }
    }

    /// Record a latency observation
    pub fn record(&mut self, latency_us: u64) {
        self.count += 1;
        self.min_us = self.min_us.min(latency_us);
        self.max_us = self.max_us.max(latency_us);
        self.sum_us += latency_us;

        // Keep only the last N samples for percentile calculation
        if self.samples.len() < MAX_LATENCY_SAMPLES {
            self.samples.push(latency_us);
        } else {
            // Replace a random sample for approximate reservoir sampling
            let idx = (self.count as usize % MAX_LATENCY_SAMPLES) % MAX_LATENCY_SAMPLES;
            if idx < self.samples.len() {
                self.samples[idx] = latency_us;
            }
        }
    }

    /// Calculate average latency in microseconds
    pub fn avg_us(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum_us as f64 / self.count as f64
        }
    }

    /// Calculate percentile latency
    ///
    /// # Arguments
    ///
    /// * `percentile` - Percentile to calculate (e.g., 0.95 for P95)
    pub fn percentile(&self, percentile: f64) -> u64 {
        if self.samples.is_empty() {
            0
        } else {
            let mut samples = self.samples.clone();
            samples.sort_unstable();
            let idx = ((percentile * (samples.len() - 1) as f64) as usize).min(samples.len() - 1);
            samples[idx]
        }
    }

    /// Get P50, P95, P99 latency
    pub fn percentiles(&self) -> (u64, u64, u64) {
        (
            self.percentile(0.50),
            self.percentile(0.95),
            self.percentile(0.99),
        )
    }
}

/// Metrics for a specific operation
#[derive(Debug, Clone)]
pub struct OperationMetrics {
    /// Operation type
    pub operation: String,
    /// Total operation count
    pub total_count: u64,
    /// Success count
    pub success_count: u64,
    /// Error count
    pub error_count: u64,
    /// Latency statistics
    pub latency: LatencyStats,
}

impl OperationMetrics {
    /// Create new operation metrics
    pub fn new(operation: &str) -> Self {
        Self {
            operation: operation.to_string(),
            total_count: 0,
            success_count: 0,
            error_count: 0,
            latency: LatencyStats::new(),
        }
    }

    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_count == 0 {
            0.0
        } else {
            self.success_count as f64 / self.total_count as f64
        }
    }
}

/// Cache statistics for metrics reporting
#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    /// Cache hits
    pub hits: u64,
    /// Cache misses
    pub misses: u64,
    /// Cache evictions
    pub evictions: u64,
    /// Current cache size
    pub current_size: usize,
    /// Maximum cache size reached
    pub max_size: usize,
}

impl CacheStats {
    /// Calculate cache hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

/// Connection pool statistics for metrics reporting
#[derive(Debug, Default, Clone)]
pub struct PoolStats {
    /// Active connections
    pub active_connections: u32,
    /// Idle connections
    pub idle_connections: u32,
    /// Total connections in pool
    pub total_connections: u32,
    /// Wait count for connections
    pub wait_count: u64,
    /// Total wait time (microseconds)
    pub wait_time_us: u64,
}

impl PoolStats {
    /// Calculate pool utilization
    pub fn utilization(&self) -> f64 {
        if self.total_connections == 0 {
            0.0
        } else {
            self.active_connections as f64 / self.total_connections as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latency_stats_percentiles() {
        let mut stats = LatencyStats::new();

        // Record various latencies in microseconds
        for i in 1..=100 {
            stats.record(i * 10); // 10, 20, 30, ... 1000
        }

        assert_eq!(stats.count, 100);
        assert_eq!(stats.min_us, 10);
        assert_eq!(stats.max_us, 1000);
        assert!((stats.avg_us() - 505.0).abs() < 1.0);

        // Check percentiles
        let (p50, p95, p99) = stats.percentiles();
        assert!((p50 as f64 - 500.0).abs() < 50.0); // ~500
        assert!((p95 as f64 - 950.0).abs() < 50.0); // ~950
        assert!((p99 as f64 - 990.0).abs() < 50.0); // ~990
    }

    #[test]
    fn test_operation_type_display() {
        assert_eq!(OperationType::EpisodeCreate.to_string(), "episode_create");
        assert_eq!(
            OperationType::Custom("test".to_string()).to_string(),
            "custom:test"
        );
    }

    #[test]
    fn test_cache_stats_hit_rate() {
        let stats = CacheStats {
            hits: 8,
            misses: 2,
            current_size: 0,
            max_size: 0,
            evictions: 0,
        };
        assert!((stats.hit_rate() - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_pool_stats_utilization() {
        let stats = PoolStats {
            active_connections: 3,
            idle_connections: 2,
            total_connections: 5,
            wait_count: 0,
            wait_time_us: 0,
        };
        assert!((stats.utilization() - 0.6).abs() < 0.01);
    }
}
