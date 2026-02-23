// ============================================================================
// Spatiotemporal Types
// ============================================================================
//!
//! Type definitions for spatiotemporal indexing.

use chrono::{DateTime, Datelike, Timelike, Utc};
use std::collections::HashMap;
use uuid::Uuid;

/// Statistics for index optimization and monitoring.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IndexStats {
    /// Total number of queries executed
    pub query_count: u64,
    /// Total number of insertions
    pub insertion_count: u64,
    /// Average query response time in microseconds
    pub avg_query_time_us: f64,
    /// Cache hit rate (0.0 to 1.0)
    pub cache_hit_rate: f64,
}

impl Default for IndexStats {
    fn default() -> Self {
        Self {
            query_count: 0,
            insertion_count: 0,
            avg_query_time_us: 0.0,
            cache_hit_rate: 0.0,
        }
    }
}

/// Node representing a year in the time hierarchy.
#[derive(Debug, Clone, PartialEq)]
pub struct YearNode {
    /// Year value (e.g., 2024)
    pub year: u32,
    /// Month nodes within this year
    pub months: HashMap<u8, MonthNode>,
    /// Episode IDs that span the entire year (rare, for long-running episodes)
    pub episodes: Vec<Uuid>,
    /// Total episodes in this year (including children)
    pub total_episodes: usize,
}

impl YearNode {
    pub fn new(year: u32) -> Self {
        Self {
            year,
            months: HashMap::new(),
            episodes: Vec::new(),
            total_episodes: 0,
        }
    }
}

/// Node representing a month in the time hierarchy.
#[derive(Debug, Clone, PartialEq)]
pub struct MonthNode {
    /// Month value (1-12)
    pub month: u8,
    /// Day nodes within this month
    pub days: HashMap<u8, DayNode>,
    /// Episode IDs that span the entire month
    pub episodes: Vec<Uuid>,
    /// Total episodes in this month (including children)
    pub total_episodes: usize,
}

impl MonthNode {
    pub fn new(month: u8) -> Self {
        Self {
            month,
            days: HashMap::new(),
            episodes: Vec::new(),
            total_episodes: 0,
        }
    }
}

/// Node representing a day in the time hierarchy.
#[derive(Debug, Clone, PartialEq)]
pub struct DayNode {
    /// Day value (1-31)
    pub day: u8,
    /// Hour nodes within this day
    pub hours: HashMap<u8, HourNode>,
    /// Episode IDs that span the entire day
    pub episodes: Vec<Uuid>,
    /// Total episodes in this day (including children)
    pub total_episodes: usize,
}

impl DayNode {
    pub fn new(day: u8) -> Self {
        Self {
            day,
            hours: HashMap::new(),
            episodes: Vec::new(),
            total_episodes: 0,
        }
    }
}

/// Node representing an hour in the time hierarchy.
#[derive(Debug, Clone, PartialEq)]
pub struct HourNode {
    /// Hour value (0-23)
    pub hour: u8,
    /// Episode IDs in this hour
    pub episodes: Vec<Uuid>,
    /// Total episodes in this hour
    pub total_episodes: usize,
}

impl HourNode {
    pub fn new(hour: u8) -> Self {
        Self {
            hour,
            episodes: Vec::new(),
            total_episodes: 0,
        }
    }

    pub fn add_episode(&mut self, episode_id: Uuid) {
        if !self.episodes.contains(&episode_id) {
            self.episodes.push(episode_id);
            self.total_episodes += 1;
        }
    }

    pub fn remove_episode(&mut self, episode_id: Uuid) -> bool {
        if let Some(pos) = self.episodes.iter().position(|&id| id == episode_id) {
            self.episodes.remove(pos);
            self.total_episodes = self.total_episodes.saturating_sub(1);
            true
        } else {
            false
        }
    }

    pub fn is_empty(&self) -> bool {
        self.episodes.is_empty()
    }
}

/// Time bucket for grouping episodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeBucket {
    /// All episodes in a specific year
    Year(u32),
    /// All episodes in a specific month
    Month { year: u32, month: u8 },
    /// All episodes on a specific day
    Day { year: u32, month: u8, day: u8 },
    /// All episodes in a specific hour
    Hour {
        year: u32,
        month: u8,
        day: u8,
        hour: u8,
    },
}

/// Query options for spatiotemporal queries.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryOptions {
    /// Maximum number of results to return
    pub limit: usize,
    /// Whether to sort results by time (most recent first)
    pub sort_by_time: bool,
    /// Whether to include episode metadata in results
    pub include_metadata: bool,
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            limit: 100,
            sort_by_time: true,
            include_metadata: false,
        }
    }
}

/// Time range parameters for queries.
#[derive(Debug, Clone, Copy)]
pub struct TimeRange {
    pub start_year: u32,
    pub start_month: u8,
    pub start_day: u8,
    pub start_hour: u8,
    pub end_year: u32,
    pub end_month: u8,
    pub end_day: u8,
    pub end_hour: u8,
}

impl TimeRange {
    /// Create a new time range from start and end timestamps.
    pub fn from_timestamps(start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        let (start_year, start_month, start_day, start_hour) = Self::extract_time_components(start);
        let (end_year, end_month, end_day, end_hour) = Self::extract_time_components(end);
        Self {
            start_year,
            start_month,
            start_day,
            start_hour,
            end_year,
            end_month,
            end_day,
            end_hour,
        }
    }

    /// Extract time components from a timestamp.
    pub fn extract_time_components(timestamp: DateTime<Utc>) -> (u32, u8, u8, u8) {
        (
            timestamp.year() as u32,
            timestamp.month() as u8,
            timestamp.day() as u8,
            timestamp.hour() as u8,
        )
    }

    /// Check if a year is the start year.
    pub fn is_start_year(&self, year: u32) -> bool {
        year == self.start_year
    }

    /// Check if a year/month is the start month.
    pub fn is_start_month(&self, year: u32, month: u8) -> bool {
        year == self.start_year && month == self.start_month
    }

    /// Check if a year/month/day is the start day.
    pub fn is_start_day(&self, year: u32, month: u8, day: u8) -> bool {
        year == self.start_year && month == self.start_month && day == self.start_day
    }

    /// Check if a year/month is the end month.
    pub fn is_end_month(&self, year: u32, month: u8) -> bool {
        year == self.end_year && month == self.end_month
    }

    /// Check if a year/month/day is the end day.
    pub fn is_end_day(&self, year: u32, month: u8, day: u8) -> bool {
        year == self.end_year && month == self.end_month && day == self.end_day
    }
}
