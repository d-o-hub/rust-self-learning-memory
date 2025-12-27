//! Domain-specific statistics for adaptive reward calibration

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Statistics for a specific task domain
///
/// Tracks performance baselines across episodes in a domain,
/// enabling adaptive reward calibration instead of fixed thresholds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainStatistics {
    /// Domain identifier (e.g., "web-api", "data-processing")
    pub domain: String,

    /// Total number of episodes in this domain
    pub episode_count: usize,

    /// Average duration in seconds
    pub avg_duration_secs: f32,

    /// Median (p50) duration in seconds
    pub p50_duration_secs: f32,

    /// 90th percentile duration in seconds
    pub p90_duration_secs: f32,

    /// Average number of steps
    pub avg_step_count: f32,

    /// Median (p50) step count
    pub p50_step_count: usize,

    /// 90th percentile step count
    pub p90_step_count: usize,

    /// Average reward across all episodes
    pub avg_reward: f32,

    /// Median (p50) reward
    pub p50_reward: f32,

    /// Standard deviation of rewards
    pub reward_std_dev: f32,

    /// When these statistics were last updated
    pub last_updated: DateTime<Utc>,

    /// Number of successful episodes (used for success rate)
    pub success_count: usize,
}

impl DomainStatistics {
    /// Create new empty statistics for a domain
    #[must_use]
    pub fn new(domain: String) -> Self {
        Self {
            domain,
            episode_count: 0,
            avg_duration_secs: 0.0,
            p50_duration_secs: 0.0,
            p90_duration_secs: 0.0,
            avg_step_count: 0.0,
            p50_step_count: 0,
            p90_step_count: 0,
            avg_reward: 0.0,
            p50_reward: 0.0,
            reward_std_dev: 0.0,
            last_updated: Utc::now(),
            success_count: 0,
        }
    }

    /// Calculate percentile from sorted values
    fn percentile(sorted_values: &[f32], percentile: f32) -> f32 {
        if sorted_values.is_empty() {
            return 0.0;
        }

        let index = ((sorted_values.len() - 1) as f32 * percentile).round() as usize;
        sorted_values[index]
    }

    /// Calculate percentile from sorted integer values
    fn percentile_usize(sorted_values: &[usize], percentile: f32) -> usize {
        if sorted_values.is_empty() {
            return 0;
        }

        let index = ((sorted_values.len() - 1) as f32 * percentile).round() as usize;
        sorted_values[index]
    }

    /// Calculate success rate for this domain
    #[must_use]
    pub fn success_rate(&self) -> f32 {
        if self.episode_count == 0 {
            0.0
        } else {
            self.success_count as f32 / self.episode_count as f32
        }
    }

    /// Check if statistics have enough data to be reliable
    ///
    /// Requires at least 5 episodes for basic reliability
    #[must_use]
    pub fn is_reliable(&self) -> bool {
        self.episode_count >= 5
    }

    /// Check if statistics are stale and need updating
    ///
    /// Statistics older than 7 days are considered stale
    #[must_use]
    pub fn is_stale(&self) -> bool {
        let age = Utc::now() - self.last_updated;
        age.num_days() > 7
    }
}

/// Collection of domain statistics with calculation utilities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DomainStatisticsCache {
    /// Map of domain -> statistics
    pub stats: HashMap<String, DomainStatistics>,
}

impl DomainStatisticsCache {
    /// Create a new empty cache
    #[must_use]
    pub fn new() -> Self {
        Self {
            stats: HashMap::new(),
        }
    }

    /// Get statistics for a domain, or None if not available
    #[must_use]
    pub fn get(&self, domain: &str) -> Option<&DomainStatistics> {
        self.stats.get(domain)
    }

    /// Get or create statistics for a domain
    pub fn get_or_create(&mut self, domain: String) -> &mut DomainStatistics {
        self.stats
            .entry(domain.clone())
            .or_insert_with(|| DomainStatistics::new(domain))
    }

    /// Calculate statistics for a domain from a list of episodes
    ///
    /// This performs a full recalculation of all statistics
    pub fn calculate_from_episodes(
        &mut self,
        domain: String,
        episodes: &[crate::episode::Episode],
    ) {
        use crate::types::TaskOutcome;

        if episodes.is_empty() {
            return;
        }

        let mut durations: Vec<f32> = Vec::new();
        let mut step_counts: Vec<usize> = Vec::new();
        let mut rewards: Vec<f32> = Vec::new();
        let mut success_count = 0;

        for episode in episodes {
            // Only include completed episodes from this domain
            if !episode.is_complete() || episode.context.domain != domain {
                continue;
            }

            // Duration
            if let Some(duration) = episode.duration() {
                durations.push(duration.num_seconds() as f32);
            }

            // Step count
            step_counts.push(episode.steps.len());

            // Reward
            if let Some(reward) = &episode.reward {
                rewards.push(reward.total);
            }

            // Success count
            if matches!(episode.outcome, Some(TaskOutcome::Success { .. })) {
                success_count += 1;
            }
        }

        if durations.is_empty() {
            return;
        }

        // Sort for percentile calculations
        durations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        step_counts.sort_unstable();
        rewards.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // Calculate averages
        let avg_duration = durations.iter().sum::<f32>() / durations.len() as f32;
        let avg_steps = step_counts.iter().sum::<usize>() as f32 / step_counts.len() as f32;
        let avg_reward = if !rewards.is_empty() {
            rewards.iter().sum::<f32>() / rewards.len() as f32
        } else {
            0.0
        };

        // Calculate standard deviation for rewards
        let reward_variance = if !rewards.is_empty() {
            let sum_sq_diff: f32 = rewards.iter().map(|r| (r - avg_reward).powi(2)).sum();
            sum_sq_diff / rewards.len() as f32
        } else {
            0.0
        };
        let reward_std_dev = reward_variance.sqrt();

        // Create or update statistics
        let stats = self.get_or_create(domain);
        stats.episode_count = durations.len();
        stats.avg_duration_secs = avg_duration;
        stats.p50_duration_secs = DomainStatistics::percentile(&durations, 0.5);
        stats.p90_duration_secs = DomainStatistics::percentile(&durations, 0.9);
        stats.avg_step_count = avg_steps;
        stats.p50_step_count = DomainStatistics::percentile_usize(&step_counts, 0.5);
        stats.p90_step_count = DomainStatistics::percentile_usize(&step_counts, 0.9);
        stats.avg_reward = avg_reward;
        stats.p50_reward = if !rewards.is_empty() {
            DomainStatistics::percentile(&rewards, 0.5)
        } else {
            0.0
        };
        stats.reward_std_dev = reward_std_dev;
        stats.last_updated = Utc::now();
        stats.success_count = success_count;
    }

    /// Update statistics incrementally with a new episode
    ///
    /// This is more efficient than full recalculation but less accurate
    pub fn update_incremental(
        &mut self,
        domain: &str,
        duration_secs: f32,
        step_count: usize,
        reward: f32,
        is_success: bool,
    ) {
        let stats = self.get_or_create(domain.to_string());

        let n = stats.episode_count as f32;
        let new_n = n + 1.0;

        // Update running averages
        stats.avg_duration_secs = (stats.avg_duration_secs * n + duration_secs) / new_n;
        stats.avg_step_count = (stats.avg_step_count * n + step_count as f32) / new_n;

        // Update reward average and std dev using Welford's online algorithm
        let old_mean = stats.avg_reward;
        stats.avg_reward = (old_mean * n + reward) / new_n;

        // Update variance (simplified)
        if n > 0.0 {
            let old_variance = stats.reward_std_dev.powi(2);
            let new_variance =
                ((n - 1.0) * old_variance + (reward - old_mean) * (reward - stats.avg_reward)) / n;
            stats.reward_std_dev = new_variance.sqrt();
        }

        // Update counts
        stats.episode_count += 1;
        if is_success {
            stats.success_count += 1;
        }

        // Update timestamp
        stats.last_updated = Utc::now();

        // Note: Percentiles can't be updated incrementally efficiently
        // They should be recalculated periodically
    }

    /// Remove stale statistics (older than 30 days with no updates)
    pub fn prune_stale(&mut self, max_age_days: i64) {
        let cutoff = Utc::now() - chrono::Duration::days(max_age_days);
        self.stats.retain(|_, stats| stats.last_updated > cutoff);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_statistics_creation() {
        let stats = DomainStatistics::new("web-api".to_string());
        assert_eq!(stats.domain, "web-api");
        assert_eq!(stats.episode_count, 0);
        assert_eq!(stats.success_rate(), 0.0);
        assert!(!stats.is_reliable());
    }

    #[test]
    fn test_percentile_calculation() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(DomainStatistics::percentile(&values, 0.5), 3.0); // Median
        assert_eq!(DomainStatistics::percentile(&values, 0.0), 1.0); // Min
        assert_eq!(DomainStatistics::percentile(&values, 1.0), 5.0); // Max
    }

    #[test]
    fn test_incremental_update() {
        let mut cache = DomainStatisticsCache::new();

        // Add first episode
        cache.update_incremental("web-api", 60.0, 10, 0.8, true);

        let stats = cache.get("web-api").unwrap();
        assert_eq!(stats.episode_count, 1);
        assert_eq!(stats.avg_duration_secs, 60.0);
        assert_eq!(stats.avg_step_count, 10.0);
        assert_eq!(stats.avg_reward, 0.8);
        assert_eq!(stats.success_count, 1);

        // Add second episode
        cache.update_incremental("web-api", 120.0, 15, 0.9, true);

        let stats = cache.get("web-api").unwrap();
        assert_eq!(stats.episode_count, 2);
        assert_eq!(stats.avg_duration_secs, 90.0); // (60 + 120) / 2
        assert_eq!(stats.avg_step_count, 12.5); // (10 + 15) / 2
        assert!((stats.avg_reward - 0.85).abs() < 0.01); // (0.8 + 0.9) / 2
    }

    #[test]
    fn test_success_rate() {
        let mut cache = DomainStatisticsCache::new();

        cache.update_incremental("test", 60.0, 10, 0.8, true);
        cache.update_incremental("test", 60.0, 10, 0.8, true);
        cache.update_incremental("test", 60.0, 10, 0.3, false);

        let stats = cache.get("test").unwrap();
        assert_eq!(stats.success_rate(), 2.0 / 3.0);
    }

    #[test]
    fn test_reliability() {
        let mut stats = DomainStatistics::new("test".to_string());
        assert!(!stats.is_reliable());

        stats.episode_count = 5;
        assert!(stats.is_reliable());
    }
}
