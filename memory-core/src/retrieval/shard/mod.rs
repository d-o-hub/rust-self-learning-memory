//! Scope-before-search shard routing (WG-122).
//!
//! Inspired by ShardMemo (arXiv:2601.21545): route queries through
//! cheap scope filters before expensive vector search.
//!
//! Pre-filters candidates by metadata (tags, task type, timeframe)
//! to reduce the search space for embedding-based retrieval.

mod router;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

pub use router::ShardRouter;

/// Configuration for shard routing.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ShardConfig {
    /// Maximum candidates to pass to vector search.
    pub max_candidates: usize,
    /// Minimum candidates required before search.
    pub min_candidates: usize,
    /// Weight for tag matching in routing.
    pub tag_weight: f32,
    /// Weight for task type matching.
    pub task_type_weight: f32,
    /// Weight for timeframe matching.
    pub timeframe_weight: f32,
    /// Whether to use temporal decay for recent episodes.
    pub use_temporal_decay: bool,
    /// Days before an episode is considered stale.
    pub stale_days: u32,
}

impl Default for ShardConfig {
    fn default() -> Self {
        Self {
            max_candidates: 100,
            min_candidates: 10,
            tag_weight: 0.4,
            task_type_weight: 0.3,
            timeframe_weight: 0.3,
            use_temporal_decay: true,
            stale_days: 30,
        }
    }
}

impl ShardConfig {
    /// Create a config for large datasets.
    #[must_use]
    pub fn large_dataset() -> Self {
        Self {
            max_candidates: 500,
            min_candidates: 50,
            tag_weight: 0.5,
            task_type_weight: 0.3,
            timeframe_weight: 0.2,
            use_temporal_decay: true,
            stale_days: 60,
        }
    }

    /// Create a config for small datasets.
    #[must_use]
    pub fn small_dataset() -> Self {
        Self {
            max_candidates: 50,
            min_candidates: 5,
            tag_weight: 0.3,
            task_type_weight: 0.3,
            timeframe_weight: 0.4,
            use_temporal_decay: true,
            stale_days: 14,
        }
    }

    /// Create a config emphasizing recent episodes.
    #[must_use]
    pub fn recent_focused() -> Self {
        Self {
            max_candidates: 100,
            min_candidates: 10,
            tag_weight: 0.2,
            task_type_weight: 0.2,
            timeframe_weight: 0.6,
            use_temporal_decay: true,
            stale_days: 7,
        }
    }
}

/// Scope filter for routing queries.
///
/// Defines the metadata constraints for pre-filtering candidates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeFilter {
    /// Required tags (at least one must match).
    pub required_tags: HashSet<String>,
    /// Excluded tags (none should match).
    pub excluded_tags: HashSet<String>,
    /// Required task types.
    pub required_task_types: HashSet<String>,
    /// Time range for episodes.
    pub time_range: Option<TimeRange>,
    /// Minimum success rate threshold.
    pub min_success_rate: Option<f32>,
}

impl ScopeFilter {
    /// Create a new empty scope filter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            required_tags: HashSet::new(),
            excluded_tags: HashSet::new(),
            required_task_types: HashSet::new(),
            time_range: None,
            min_success_rate: None,
        }
    }

    /// Create a scope filter from query text.
    ///
    /// Extracts tags, task types, and time hints from text.
    #[must_use]
    pub fn from_query_text(query: &str) -> Self {
        let mut filter = Self::new();

        // Extract common tags from query
        let common_tags = [
            "bug",
            "fix",
            "feature",
            "refactor",
            "test",
            "security",
            "performance",
            "documentation",
            "api",
            "cli",
            "mcp",
            "storage",
            "core",
        ];

        for tag in common_tags {
            if query.to_lowercase().contains(tag) {
                filter.required_tags.insert(tag.to_string());
            }
        }

        // Extract task types
        let task_types = ["implementation", "debugging", "testing", "review"];
        for tt in task_types {
            if query.to_lowercase().contains(tt) {
                filter.required_task_types.insert(tt.to_string());
            }
        }

        // Extract time hints
        let lower = query.to_lowercase();
        if lower.contains("recent") || lower.contains("latest") {
            filter.time_range = Some(TimeRange::recent_days(7));
        } else if lower.contains("today") {
            filter.time_range = Some(TimeRange::today());
        } else if lower.contains("this week") {
            filter.time_range = Some(TimeRange::recent_days(7));
        } else if lower.contains("this month") {
            filter.time_range = Some(TimeRange::recent_days(30));
        }

        filter
    }

    /// Add a required tag.
    pub fn require_tag(&mut self, tag: &str) {
        self.required_tags.insert(tag.to_lowercase());
    }

    /// Add an excluded tag.
    pub fn exclude_tag(&mut self, tag: &str) {
        self.excluded_tags.insert(tag.to_lowercase());
    }

    /// Add a required task type.
    pub fn require_task_type(&mut self, task_type: &str) {
        self.required_task_types.insert(task_type.to_lowercase());
    }

    /// Set time range.
    pub fn set_time_range(&mut self, range: TimeRange) {
        self.time_range = Some(range);
    }

    /// Set minimum success rate.
    pub fn set_min_success_rate(&mut self, rate: f32) {
        self.min_success_rate = Some(rate);
    }

    /// Check if filter has any constraints.
    #[must_use]
    pub fn has_constraints(&self) -> bool {
        !self.required_tags.is_empty()
            || !self.excluded_tags.is_empty()
            || !self.required_task_types.is_empty()
            || self.time_range.is_some()
            || self.min_success_rate.is_some()
    }

    /// Get constraint count.
    #[must_use]
    pub fn constraint_count(&self) -> usize {
        self.required_tags.len()
            + self.excluded_tags.len()
            + self.required_task_types.len()
            + self.time_range.map_or(0, |_| 1)
            + self.min_success_rate.map_or(0, |_| 1)
    }
}

impl Default for ScopeFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Time range for filtering episodes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TimeRange {
    /// Start of range.
    pub start: DateTime<Utc>,
    /// End of range.
    pub end: DateTime<Utc>,
}

impl TimeRange {
    /// Create a time range from start to end.
    #[must_use]
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self { start, end }
    }

    /// Create a range for recent days.
    #[must_use]
    pub fn recent_days(days: u32) -> Self {
        let end = Utc::now();
        let start = end - chrono::Duration::days(i64::from(days));
        Self { start, end }
    }

    /// Create a range for today.
    #[must_use]
    pub fn today() -> Self {
        let now = Utc::now();
        let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
        Self {
            start: DateTime::from_naive_utc_and_offset(start, Utc),
            end: now,
        }
    }

    /// Create a range for this week.
    #[must_use]
    pub fn this_week() -> Self {
        Self::recent_days(7)
    }

    /// Check if a timestamp is within the range.
    #[must_use]
    pub fn contains(&self, timestamp: DateTime<Utc>) -> bool {
        timestamp >= self.start && timestamp <= self.end
    }

    /// Get duration in days.
    #[must_use]
    pub fn duration_days(&self) -> u32 {
        ((self.end - self.start).num_days()).max(0) as u32
    }
}

/// Episode metadata for shard routing.
///
/// Lightweight metadata extracted from episodes for efficient filtering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeMetadata {
    /// Episode ID.
    pub episode_id: Uuid,
    /// Tags associated with episode.
    pub tags: HashSet<String>,
    /// Task type.
    pub task_type: String,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Success rate of execution.
    pub success_rate: f32,
    /// Whether episode is complete.
    pub is_complete: bool,
}

impl EpisodeMetadata {
    /// Create new episode metadata.
    #[must_use]
    pub fn new(
        episode_id: Uuid,
        tags: HashSet<String>,
        task_type: String,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            episode_id,
            tags,
            task_type,
            created_at,
            success_rate: 0.0,
            is_complete: false,
        }
    }

    /// Set success rate.
    pub fn set_success_rate(&mut self, rate: f32) {
        self.success_rate = rate;
    }

    /// Mark as complete.
    pub fn mark_complete(&mut self) {
        self.is_complete = true;
    }

    /// Check if has a specific tag.
    #[must_use]
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(&tag.to_lowercase())
    }

    /// Check if matches task type.
    #[must_use]
    pub fn matches_task_type(&self, task_type: &str) -> bool {
        self.task_type.to_lowercase() == task_type.to_lowercase()
    }

    /// Get age in days.
    #[must_use]
    pub fn age_days(&self) -> u32 {
        ((Utc::now() - self.created_at).num_days()).max(0) as u32
    }
}

/// Routing result containing filtered candidates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingResult {
    /// Candidate episode IDs for vector search.
    pub candidates: Vec<Uuid>,
    /// Original candidate count before filtering.
    pub original_count: usize,
    /// Filtered candidate count.
    pub filtered_count: usize,
    /// Whether max_candidates was reached.
    pub capped: bool,
    /// Applied scope filter.
    pub filter: ScopeFilter,
    /// Routing scores for each candidate.
    pub scores: Vec<f32>,
}

impl RoutingResult {
    /// Create an empty routing result.
    #[must_use]
    pub fn empty(filter: ScopeFilter) -> Self {
        Self {
            candidates: Vec::new(),
            original_count: 0,
            filtered_count: 0,
            capped: false,
            filter,
            scores: Vec::new(),
        }
    }

    /// Get candidate count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.candidates.len()
    }

    /// Check if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }

    /// Get reduction ratio (filtered / original).
    #[must_use]
    pub fn reduction_ratio(&self) -> f32 {
        if self.original_count == 0 {
            return 0.0;
        }
        1.0 - (self.filtered_count as f32 / self.original_count as f32)
    }
}

#[cfg(test)]
mod tests;
