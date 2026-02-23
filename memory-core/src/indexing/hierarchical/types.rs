//! Types for hierarchical indexing.

use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::indexing::spatiotemporal::SpatiotemporalIndex;
use crate::types::TaskType;

/// Statistics for the hierarchical index.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HierarchicalIndexStats {
    /// Total number of queries executed
    pub query_count: u64,
    /// Number of domain-specific queries
    pub domain_query_count: u64,
    /// Number of task type queries
    pub task_type_query_count: u64,
    /// Number of temporal queries
    pub temporal_query_count: u64,
    /// Average query response time in microseconds
    pub avg_query_time_us: f64,
}

impl Default for HierarchicalIndexStats {
    fn default() -> Self {
        Self {
            query_count: 0,
            domain_query_count: 0,
            task_type_query_count: 0,
            temporal_query_count: 0,
            avg_query_time_us: 0.0,
        }
    }
}

/// Index for a specific domain.
#[derive(Debug, Clone, PartialEq)]
pub struct DomainLevelIndex {
    /// Domain name
    pub domain: String,
    /// Task type indices within this domain
    pub task_types: HashMap<TaskType, TaskTypeLevelIndex>,
    /// Temporal index for this domain
    pub temporal_index: SpatiotemporalIndex,
    /// Total episodes in this domain
    pub total_episodes: usize,
}

impl DomainLevelIndex {
    pub fn new(domain: String) -> Self {
        Self {
            domain,
            task_types: HashMap::new(),
            temporal_index: SpatiotemporalIndex::new(),
            total_episodes: 0,
        }
    }
}

/// Index for a specific task type within a domain.
#[derive(Debug, Clone, PartialEq)]
pub struct TaskTypeLevelIndex {
    /// Task type
    pub task_type: TaskType,
    /// Temporal index for this task type
    pub temporal_index: SpatiotemporalIndex,
    /// Total episodes for this task type
    pub total_episodes: usize,
}

impl TaskTypeLevelIndex {
    pub fn new(task_type: TaskType) -> Self {
        Self {
            task_type,
            temporal_index: SpatiotemporalIndex::new(),
            total_episodes: 0,
        }
    }
}

/// Query filter for hierarchical searches.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct HierarchicalQuery {
    /// Filter by domain
    pub domain: Option<String>,
    /// Filter by task type
    pub task_type: Option<TaskType>,
    /// Filter by start time
    pub start_time: Option<DateTime<Utc>>,
    /// Filter by end time
    pub end_time: Option<DateTime<Utc>>,
    /// Time bucket for bucket queries
    pub time_bucket: Option<crate::indexing::spatiotemporal::TimeBucket>,
    /// Maximum number of results
    pub limit: usize,
}

impl HierarchicalQuery {
    /// Create a new empty query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the domain filter.
    #[must_use]
    pub fn with_domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = Some(domain.into());
        self
    }

    /// Set the task type filter.
    #[must_use]
    pub fn with_task_type(mut self, task_type: TaskType) -> Self {
        self.task_type = Some(task_type);
        self
    }

    /// Set the time range filter.
    #[must_use]
    pub fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    /// Set the time bucket filter.
    #[must_use]
    pub fn with_time_bucket(mut self, bucket: crate::indexing::spatiotemporal::TimeBucket) -> Self {
        self.time_bucket = Some(bucket);
        self
    }

    /// Set the result limit.
    #[must_use]
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
}
