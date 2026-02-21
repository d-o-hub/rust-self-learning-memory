//! Hierarchical indexing module for spatiotemporal episode organization.
//!
//! This module provides efficient indexing structures for episode retrieval:
//!
//! - **Spatiotemporal Index**: Time-based hierarchical index (year/month/day/hour)
//! - **Hierarchical Index**: Multi-level index combining domain, task type, and time
//!
//! # Architecture
//!
//! ```text
//! HierarchicalIndex
//! ├── DomainLevelIndex (e.g., "web-api")
//! │   ├── TaskTypeLevelIndex (e.g., CodeGeneration)
//! │   │   └── SpatiotemporalIndex (time hierarchy)
//! │   └── TaskTypeLevelIndex (e.g., Debugging)
//! │       └── SpatiotemporalIndex
//! └── DomainLevelIndex (e.g., "data-processing")
//!     └── ...
//! ```
//!
//! # Performance
//!
//! | Operation | Time Complexity | Space Complexity |
//! |-----------|----------------|------------------|
//! | Insert    | O(log n)       | O(1) amortized   |
//! | Query     | O(log n + k)   | O(k)             |
//! | Remove    | O(log n)       | O(1)             |
//!
//! Where n is total episodes and k is results count.
//!
//! # Usage Examples
//!
//! ## Basic Spatiotemporal Index
//!
//! ```
//! use memory_core::indexing::{SpatiotemporalIndex, TimeBucket};
//! use memory_core::{Episode, TaskContext, TaskType};
//!
//! let mut index = SpatiotemporalIndex::new();
//!
//! // Insert episode
//! let episode = Episode::new(
//!     "Test task".to_string(),
//!     TaskContext::default(),
//!     TaskType::CodeGeneration,
//! );
//! index.insert(&episode);
//!
//! // Query by time range
//! let start = chrono::Utc::now() - chrono::Duration::hours(1);
//! let end = chrono::Utc::now();
//! let results = index.query_range(start, end, 100);
//!
//! // Query by time bucket
//! let bucket = TimeBucket::Month { year: 2024, month: 3 };
//! let results = index.query_bucket(&bucket);
//! ```
//!
//! ## Hierarchical Index
//!
//! ```
//! use memory_core::indexing::{HierarchicalIndex, HierarchicalQuery};
//! use memory_core::{Episode, TaskContext, TaskType};
//!
//! let mut index = HierarchicalIndex::new();
//!
//! // Insert episode
//! let episode = Episode::new(
//!     "Test task".to_string(),
//!     TaskContext::default(),
//!     TaskType::CodeGeneration,
//! );
//! index.insert(&episode);
//!
//! // Query with filters
//! let query = HierarchicalQuery::new()
//!     .with_domain("web-api")
//!     .with_task_type(TaskType::CodeGeneration)
//!     .with_limit(10);
//! let results = index.query(&query);
//! ```

pub mod hierarchical;
pub mod spatiotemporal;

pub use hierarchical::{HierarchicalIndex, HierarchicalIndexStats, HierarchicalQuery};
pub use spatiotemporal::{IndexStats, QueryOptions, SpatiotemporalIndex, TimeBucket};

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Extension trait for integrating indexing with `SelfLearningMemory`.
///
/// Provides methods to build and query hierarchical indexes from memory.
pub trait IndexableMemory {
    /// Build a hierarchical index from all episodes in memory.
    ///
    /// # Returns
    ///
    /// A hierarchical index containing all episodes.
    fn build_hierarchical_index(
        &self,
    ) -> impl std::future::Future<Output = HierarchicalIndex> + Send;

    /// Build a spatiotemporal index from all episodes in memory.
    ///
    /// # Returns
    ///
    /// A spatiotemporal index containing all episodes.
    fn build_spatiotemporal_index(
        &self,
    ) -> impl std::future::Future<Output = SpatiotemporalIndex> + Send;

    /// Query episodes by time range using the index.
    ///
    /// # Arguments
    ///
    /// * `start` - Start of the time range
    /// * `end` - End of the time range
    /// * `limit` - Maximum number of results
    ///
    /// # Returns
    ///
    /// Vector of episode IDs within the time range.
    fn query_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: usize,
    ) -> impl std::future::Future<Output = Vec<Uuid>> + Send;
}

/// Performance comparison between indexed and linear scan queries.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QueryPerformance {
    /// Time taken by indexed query in microseconds
    pub indexed_time_us: f64,
    /// Time taken by linear scan in microseconds
    pub linear_time_us: f64,
    /// Speedup factor (`linear_time` / `indexed_time`)
    pub speedup_factor: f64,
    /// Number of episodes scanned
    pub episodes_scanned: usize,
    /// Number of episodes returned
    pub results_count: usize,
}

impl QueryPerformance {
    /// Calculate the speedup factor.
    #[must_use]
    pub fn calculate_speedup(indexed_time_us: f64, linear_time_us: f64) -> f64 {
        if indexed_time_us > 0.0 {
            linear_time_us / indexed_time_us
        } else {
            1.0
        }
    }
}

/// Index metrics for monitoring and optimization.
#[derive(Debug, Clone, PartialEq)]
pub struct IndexMetrics {
    /// Total number of episodes indexed
    pub total_episodes: usize,
    /// Number of domains in the index
    pub domain_count: usize,
    /// Number of task types in the index
    pub task_type_count: usize,
    /// Memory usage in bytes
    pub memory_usage_bytes: usize,
    /// Average query time in microseconds
    pub avg_query_time_us: f64,
    /// Query speedup factor vs linear scan
    pub speedup_factor: f64,
    /// Cache hit rate (0.0 to 1.0)
    pub cache_hit_rate: f64,
}

/// Benchmark result comparing indexed vs linear scan performance.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkResult {
    /// Number of episodes in the dataset
    pub episode_count: usize,
    /// Number of queries executed
    pub query_count: usize,
    /// Average indexed query time
    pub avg_indexed_time_us: f64,
    /// Average linear scan time
    pub avg_linear_time_us: f64,
    /// Overall speedup factor
    pub overall_speedup: f64,
    /// Memory overhead percentage
    pub memory_overhead_percent: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Episode;
    use crate::types::{ComplexityLevel, TaskContext, TaskType};
    use chrono::{Datelike, Timelike};

    fn create_test_episode(domain: &str, task_type: TaskType) -> Episode {
        let context = TaskContext {
            domain: domain.to_string(),
            complexity: ComplexityLevel::Simple,
            tags: vec![],
            ..Default::default()
        };
        Episode::new("Test episode".to_string(), context, task_type)
    }

    #[test]
    fn test_query_performance_calculation() {
        let perf = QueryPerformance {
            indexed_time_us: 10.0,
            linear_time_us: 100.0,
            speedup_factor: QueryPerformance::calculate_speedup(10.0, 100.0),
            episodes_scanned: 1000,
            results_count: 10,
        };

        assert_eq!(perf.speedup_factor, 10.0);
    }

    #[test]
    fn test_hierarchical_query_builder() {
        let query = HierarchicalQuery::new()
            .with_domain("web-api")
            .with_task_type(TaskType::CodeGeneration)
            .with_limit(50);

        assert_eq!(query.domain, Some("web-api".to_string()));
        assert_eq!(query.task_type, Some(TaskType::CodeGeneration));
        assert_eq!(query.limit, 50);
    }

    #[test]
    fn test_time_bucket_variants() {
        let year_bucket = TimeBucket::Year(2024);
        assert_eq!(year_bucket, TimeBucket::Year(2024));

        let month_bucket = TimeBucket::Month {
            year: 2024,
            month: 3,
        };
        assert_eq!(
            month_bucket,
            TimeBucket::Month {
                year: 2024,
                month: 3
            }
        );

        let day_bucket = TimeBucket::Day {
            year: 2024,
            month: 3,
            day: 15,
        };
        assert_eq!(
            day_bucket,
            TimeBucket::Day {
                year: 2024,
                month: 3,
                day: 15,
            }
        );

        let hour_bucket = TimeBucket::Hour {
            year: 2024,
            month: 3,
            day: 15,
            hour: 10,
        };
        assert_eq!(
            hour_bucket,
            TimeBucket::Hour {
                year: 2024,
                month: 3,
                day: 15,
                hour: 10,
            }
        );
    }

    #[test]
    fn test_index_metrics() {
        let metrics = IndexMetrics {
            total_episodes: 1000,
            domain_count: 5,
            task_type_count: 10,
            memory_usage_bytes: 102_400,
            avg_query_time_us: 5.5,
            speedup_factor: 10.0,
            cache_hit_rate: 0.85,
        };

        assert_eq!(metrics.total_episodes, 1000);
        assert_eq!(metrics.speedup_factor, 10.0);
        assert!((metrics.cache_hit_rate - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn test_benchmark_result() {
        let result = BenchmarkResult {
            episode_count: 10000,
            query_count: 100,
            avg_indexed_time_us: 5.0,
            avg_linear_time_us: 100.0,
            overall_speedup: 20.0,
            memory_overhead_percent: 8.5,
        };

        assert_eq!(result.episode_count, 10000);
        assert_eq!(result.overall_speedup, 20.0);
        assert!((result.memory_overhead_percent - 8.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_spatiotemporal_index_basic() {
        let mut index = SpatiotemporalIndex::new();
        let episode = create_test_episode("web-api", TaskType::CodeGeneration);

        index.insert(&episode);

        assert_eq!(index.len(), 1);
        assert!(!index.is_empty());

        let year = episode.start_time.year() as u32;
        let month = episode.start_time.month() as u8;
        let day = episode.start_time.day() as u8;
        let hour = episode.start_time.hour() as u8;

        let results = index.query_hour(year, month, day, hour);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], episode.episode_id);
    }

    #[test]
    fn test_hierarchical_index_basic() {
        let mut index = HierarchicalIndex::new();
        let episode = create_test_episode("web-api", TaskType::CodeGeneration);

        index.insert(&episode);

        assert_eq!(index.len(), 1);
        assert_eq!(index.domain_count(), 1);

        let results = index.query_by_domain("web-api", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], episode.episode_id);
    }
}
