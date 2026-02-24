//! Spatiotemporal hierarchical indexing for episode queries.
//!
//! This module provides efficient time-based indexing of episodes using a hierarchical
//! structure: Year → Month → Day → Hour. This enables O(log n) time range queries
//! instead of O(n) linear scans.
//!
//! See the [`SpatiotemporalIndex`] for usage examples.

mod types;

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

use crate::episode::Episode;

pub use self::types::{
    DayNode, HourNode, IndexStats, MonthNode, QueryOptions, TimeBucket, TimeRange, YearNode,
};

/// A spatiotemporal index for efficient episode retrieval by time.
///
/// Organizes episodes in a hierarchical time structure:
/// Year → Month → Day → Hour
///
/// This enables efficient range queries by only traversing relevant time buckets.
#[derive(Debug, Clone, PartialEq)]
pub struct SpatiotemporalIndex {
    /// Root of the time hierarchy: year → year node
    years: HashMap<u32, YearNode>,
    /// Total number of indexed episodes
    total_episodes: usize,
    /// Index creation timestamp
    created_at: DateTime<Utc>,
    /// Last modification timestamp
    last_modified: DateTime<Utc>,
    /// Statistics for query optimization
    stats: IndexStats,
}

impl SpatiotemporalIndex {
    /// Create a new empty spatiotemporal index.
    #[must_use]
    pub fn new() -> Self {
        Self {
            years: HashMap::new(),
            total_episodes: 0,
            created_at: Utc::now(),
            last_modified: Utc::now(),
            stats: IndexStats::default(),
        }
    }

    /// Insert an episode into the index.
    ///
    /// # Arguments
    ///
    /// * `episode` - Episode to index
    ///
    /// # Performance
    ///
    /// O(h) where h is the height of the tree (typically 4 for year/month/day/hour).
    pub fn insert(&mut self, episode: &Episode) {
        let timestamp = episode.start_time;
        let (year, month, day, hour) = TimeRange::extract_time_components(timestamp);

        // Get or create year node
        let year_node = self
            .years
            .entry(year)
            .or_insert_with(|| YearNode::new(year));

        // Get or create month node
        let month_node = year_node
            .months
            .entry(month)
            .or_insert_with(|| MonthNode::new(month));

        // Get or create day node
        let day_node = month_node
            .days
            .entry(day)
            .or_insert_with(|| DayNode::new(day));

        // Get or create hour node
        let hour_node = day_node
            .hours
            .entry(hour)
            .or_insert_with(|| HourNode::new(hour));

        // Add episode to hour node
        hour_node.add_episode(episode.episode_id);

        // Update statistics up the tree
        day_node.total_episodes += 1;
        month_node.total_episodes += 1;
        year_node.total_episodes += 1;
        self.total_episodes += 1;
        self.stats.insertion_count += 1;
        self.last_modified = Utc::now();
    }

    /// Remove an episode from the index.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - ID of the episode to remove
    /// * `timestamp` - Timestamp when the episode started (for efficient lookup)
    ///
    /// # Returns
    ///
    /// `true` if the episode was found and removed.
    ///
    /// # Performance
    ///
    /// O(h) where h is the height of the tree.
    pub fn remove(&mut self, episode_id: Uuid, timestamp: DateTime<Utc>) -> bool {
        let (year, month, day, hour) = TimeRange::extract_time_components(timestamp);

        let Some(year_node) = self.years.get_mut(&year) else {
            return false;
        };

        let Some(month_node) = year_node.months.get_mut(&month) else {
            return false;
        };

        let Some(day_node) = month_node.days.get_mut(&day) else {
            return false;
        };

        let Some(hour_node) = day_node.hours.get_mut(&hour) else {
            return false;
        };

        let removed = hour_node.remove_episode(episode_id);

        if removed {
            // Update statistics up the tree
            day_node.total_episodes = day_node.total_episodes.saturating_sub(1);
            month_node.total_episodes = month_node.total_episodes.saturating_sub(1);
            year_node.total_episodes = year_node.total_episodes.saturating_sub(1);
            self.total_episodes = self.total_episodes.saturating_sub(1);
            self.last_modified = Utc::now();

            // Clean up empty nodes
            self.cleanup_if_empty(year, month, day, hour);
        }

        removed
    }

    /// Query episodes within a time range.
    ///
    /// # Arguments
    ///
    /// * `start` - Start of the time range (inclusive)
    /// * `end` - End of the time range (exclusive)
    /// * `limit` - Maximum number of results to return
    ///
    /// # Returns
    ///
    /// Vector of episode IDs within the time range.
    ///
    /// # Performance
    ///
    /// O(log n + k) where n is total episodes and k is results count.
    /// This is significantly faster than O(n) linear scan.
    #[must_use]
    pub fn query_range(&self, start: DateTime<Utc>, end: DateTime<Utc>, limit: usize) -> Vec<Uuid> {
        let start_time = std::time::Instant::now();
        let mut results = Vec::new();
        let range = TimeRange::from_timestamps(start, end);

        // Iterate through relevant years
        for year in range.start_year..=range.end_year {
            let Some(year_node) = self.years.get(&year) else {
                continue;
            };
            let month_start = if range.is_start_year(year) {
                range.start_month
            } else {
                1
            };
            let month_end = if range.is_end_month(year, range.end_month) {
                range.end_month
            } else {
                12
            };

            for month in month_start..=month_end {
                let Some(month_node) = year_node.months.get(&month) else {
                    continue;
                };
                let month_results =
                    Self::query_month_range(month_node, year, month, &range, limit - results.len());
                results.extend(month_results);
                if results.len() >= limit {
                    results.truncate(limit);
                    return results;
                }
            }
        }

        // Update statistics
        let elapsed = start_time.elapsed().as_micros() as f64;
        self.update_query_stats(elapsed);

        results
    }

    /// Query episodes by time bucket.
    ///
    /// # Arguments
    ///
    /// * `bucket` - Time bucket to query
    ///
    /// # Returns
    ///
    /// Vector of episode IDs in the specified time bucket.
    ///
    /// # Example
    ///
    /// ```
    /// use memory_core::indexing::spatiotemporal::{SpatiotemporalIndex, TimeBucket};
    ///
    /// let index = SpatiotemporalIndex::new();
    ///
    /// // Get all episodes from March 2024
    /// let bucket = TimeBucket::Month { year: 2024, month: 3 };
    /// let episodes = index.query_bucket(&bucket);
    /// ```
    #[must_use]
    pub fn query_bucket(&self, bucket: &TimeBucket) -> Vec<Uuid> {
        match bucket {
            TimeBucket::Year(year) => self.query_year(*year),
            TimeBucket::Month { year, month } => self.query_month(*year, *month),
            TimeBucket::Day { year, month, day } => self.query_day(*year, *month, *day),
            TimeBucket::Hour {
                year,
                month,
                day,
                hour,
            } => self.query_hour(*year, *month, *day, *hour),
        }
    }

    /// Query all episodes in a specific year.
    #[must_use]
    pub fn query_year(&self, year: u32) -> Vec<Uuid> {
        self.years.get(&year).map_or_else(Vec::new, |year_node| {
            Self::collect_episodes_from_year(year_node)
        })
    }

    /// Query all episodes in a specific month.
    #[must_use]
    pub fn query_month(&self, year: u32, month: u8) -> Vec<Uuid> {
        self.years
            .get(&year)
            .and_then(|year_node| year_node.months.get(&month))
            .map_or_else(Vec::new, |month_node| {
                Self::collect_episodes_from_month(month_node)
            })
    }

    /// Query all episodes on a specific day.
    #[must_use]
    pub fn query_day(&self, year: u32, month: u8, day: u8) -> Vec<Uuid> {
        self.years
            .get(&year)
            .and_then(|year_node| year_node.months.get(&month))
            .and_then(|month_node| month_node.days.get(&day))
            .map_or_else(Vec::new, |day_node| {
                Self::collect_episodes_from_day(day_node)
            })
    }

    /// Query all episodes in a specific hour.
    #[must_use]
    pub fn query_hour(&self, year: u32, month: u8, day: u8, hour: u8) -> Vec<Uuid> {
        self.years
            .get(&year)
            .and_then(|year_node| year_node.months.get(&month))
            .and_then(|month_node| month_node.days.get(&day))
            .and_then(|day_node| day_node.hours.get(&hour))
            .map_or_else(Vec::new, |hour_node| hour_node.episodes.clone())
    }

    /// Get the total number of indexed episodes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.total_episodes
    }

    /// Check if the index is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.total_episodes == 0
    }

    /// Get index statistics.
    #[must_use]
    pub fn stats(&self) -> IndexStats {
        self.stats
    }

    /// Clear all index data.
    pub fn clear(&mut self) {
        self.years.clear();
        self.total_episodes = 0;
        self.last_modified = Utc::now();
        self.stats = IndexStats::default();
    }

    /// Get the number of years in the index.
    #[must_use]
    pub fn year_count(&self) -> usize {
        self.years.len()
    }

    /// Get memory usage estimate in bytes.
    #[must_use]
    pub fn memory_usage_estimate(&self) -> usize {
        let mut total = std::mem::size_of::<Self>();
        for year_node in self.years.values() {
            total += Self::calculate_year_node_size(year_node);
        }
        total
    }

    // Private helper methods

    /// Collect all episodes from a day node and its hour children.
    fn collect_episodes_from_day(day_node: &DayNode) -> Vec<Uuid> {
        let mut results = Vec::new();
        results.extend(&day_node.episodes);
        for hour_node in day_node.hours.values() {
            results.extend(&hour_node.episodes);
        }
        results
    }

    /// Collect all episodes from a month node and its day/hour children.
    fn collect_episodes_from_month(month_node: &MonthNode) -> Vec<Uuid> {
        let mut results = Vec::new();
        results.extend(&month_node.episodes);
        for day_node in month_node.days.values() {
            results.extend(Self::collect_episodes_from_day(day_node));
        }
        results
    }

    /// Collect all episodes from a year node and its month/day/hour children.
    fn collect_episodes_from_year(year_node: &YearNode) -> Vec<Uuid> {
        let mut results = Vec::new();
        results.extend(&year_node.episodes);
        for month_node in year_node.months.values() {
            results.extend(Self::collect_episodes_from_month(month_node));
        }
        results
    }

    /// Calculate memory size of an hour node.
    fn calculate_hour_node_size(hour_node: &HourNode) -> usize {
        std::mem::size_of::<HourNode>() + hour_node.episodes.len() * std::mem::size_of::<Uuid>()
    }

    /// Calculate memory size of a day node.
    fn calculate_day_node_size(day_node: &DayNode) -> usize {
        let mut total =
            std::mem::size_of::<DayNode>() + day_node.episodes.len() * std::mem::size_of::<Uuid>();
        for hour_node in day_node.hours.values() {
            total += Self::calculate_hour_node_size(hour_node);
        }
        total
    }

    /// Calculate memory size of a month node.
    fn calculate_month_node_size(month_node: &MonthNode) -> usize {
        let mut total = std::mem::size_of::<MonthNode>()
            + month_node.episodes.len() * std::mem::size_of::<Uuid>();
        for day_node in month_node.days.values() {
            total += Self::calculate_day_node_size(day_node);
        }
        total
    }

    /// Calculate memory size of a year node.
    fn calculate_year_node_size(year_node: &YearNode) -> usize {
        let mut total = std::mem::size_of::<YearNode>()
            + year_node.episodes.len() * std::mem::size_of::<Uuid>();
        for month_node in year_node.months.values() {
            total += Self::calculate_month_node_size(month_node);
        }
        total
    }

    /// Query episodes within a specific time range for a month.
    fn query_month_range(
        month_node: &MonthNode,
        year: u32,
        month: u8,
        range: &TimeRange,
        limit: usize,
    ) -> Vec<Uuid> {
        let mut results = Vec::new();
        let day_start = if range.is_start_month(year, month) {
            range.start_day
        } else {
            1
        };
        let day_end = if range.is_end_month(year, month) {
            range.end_day
        } else {
            31
        };

        for day in day_start..=day_end {
            let Some(day_node) = month_node.days.get(&day) else {
                continue;
            };

            let hour_start = if range.is_start_day(year, month, day) {
                range.start_hour
            } else {
                0
            };
            let hour_end = if range.is_end_day(year, month, day) {
                range.end_hour
            } else {
                23
            };

            for hour in hour_start..=hour_end {
                let Some(hour_node) = day_node.hours.get(&hour) else {
                    continue;
                };
                results.extend(&hour_node.episodes);
                if results.len() >= limit {
                    return results[..limit.min(results.len())].to_vec();
                }
            }
        }
        results
    }

    /// Update query statistics.
    fn update_query_stats(&self, elapsed_us: f64) {
        // This is a simplified version - in production, use atomic operations
        // or a separate statistics thread
        let _ = elapsed_us;
    }

    /// Clean up empty nodes after removal.
    fn cleanup_if_empty(&mut self, year: u32, month: u8, day: u8, hour: u8) {
        // Check if hour node is empty and needs removal
        let should_remove = self
            .years
            .get(&year)
            .and_then(|y| y.months.get(&month))
            .and_then(|m| m.days.get(&day))
            .and_then(|d| d.hours.get(&hour))
            .is_some_and(|h| h.is_empty());

        if !should_remove {
            return;
        }

        // Remove empty hour node
        let Some(year_node) = self.years.get_mut(&year) else {
            return;
        };
        let Some(month_node) = year_node.months.get_mut(&month) else {
            return;
        };
        let Some(day_node) = month_node.days.get_mut(&day) else {
            return;
        };
        day_node.hours.remove(&hour);
    }
}

impl Default for SpatiotemporalIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
