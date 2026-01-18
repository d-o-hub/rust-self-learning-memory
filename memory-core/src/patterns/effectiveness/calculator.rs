//! Effectiveness calculation logic
//!
//! Core calculation for pattern effectiveness tracking including
//! scoring algorithms and decay mechanisms.

use super::history::{PatternUsage, UsageStats};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tracing::{debug, instrument};
use uuid::Uuid;

/// Overall system effectiveness statistics
#[derive(Debug, Clone, Default)]
pub struct OverallStats {
    /// Total number of patterns being tracked
    pub total_patterns: usize,
    /// Number of patterns used in last 30 days
    pub active_patterns: usize,
    /// Total retrievals across all patterns
    pub total_retrievals: usize,
    /// Total applications across all patterns
    pub total_applications: usize,
    /// Overall success rate
    pub overall_success_rate: f32,
    /// Average effectiveness score
    pub avg_effectiveness: f32,
}

/// Tracks effectiveness of all patterns
pub struct EffectivenessTracker {
    /// Usage data by pattern ID
    usage_map: HashMap<Uuid, PatternUsage>,
    /// Minimum effectiveness threshold for keeping patterns
    min_effectiveness: f32,
    /// How often to decay old patterns (in days)
    decay_interval_days: i64,
    /// Last time decay was performed
    last_decay: DateTime<Utc>,
}

impl Default for EffectivenessTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectivenessTracker {
    /// Create a new effectiveness tracker
    #[must_use]
    pub fn new() -> Self {
        Self {
            usage_map: HashMap::new(),
            min_effectiveness: 0.3, // Keep patterns above 30% effectiveness
            decay_interval_days: 7, // Weekly decay
            last_decay: Utc::now(),
        }
    }

    /// Create tracker with custom settings
    #[must_use]
    pub fn with_config(min_effectiveness: f32, decay_interval_days: i64) -> Self {
        Self {
            usage_map: HashMap::new(),
            min_effectiveness,
            decay_interval_days,
            last_decay: Utc::now(),
        }
    }

    /// Record that a pattern was retrieved
    #[instrument(skip(self))]
    pub fn record_retrieval(&mut self, pattern_id: Uuid) {
        let usage = self
            .usage_map
            .entry(pattern_id)
            .or_insert_with(|| PatternUsage::new(pattern_id));

        usage.retrieval_count += 1;
        usage.last_retrieved = Some(Utc::now());

        debug!(
            pattern_id = %pattern_id,
            retrieval_count = usage.retrieval_count,
            "Recorded pattern retrieval"
        );
    }

    /// Record that a pattern was applied with a result
    #[instrument(skip(self))]
    pub fn record_application(&mut self, pattern_id: Uuid, successful: bool) {
        let usage = self
            .usage_map
            .entry(pattern_id)
            .or_insert_with(|| PatternUsage::new(pattern_id));

        usage.application_count += 1;
        usage.last_applied = Some(Utc::now());

        if successful {
            usage.success_count += 1;
        } else {
            usage.failure_count += 1;
        }

        // Update effectiveness score after each application
        usage.update_effectiveness_score();

        debug!(
            pattern_id = %pattern_id,
            application_count = usage.application_count,
            successful = successful,
            effectiveness_score = usage.effectiveness_score,
            "Recorded pattern application"
        );
    }

    /// Get usage statistics for a pattern
    #[must_use]
    pub fn get_stats(&self, pattern_id: Uuid) -> Option<UsageStats> {
        self.usage_map.get(&pattern_id).map(|usage| {
            let now = Utc::now();
            let last_use = usage
                .last_applied
                .or(usage.last_retrieved)
                .unwrap_or(usage.created_at);
            let days_since = (now - last_use).num_days();

            UsageStats {
                retrieval_count: usage.retrieval_count,
                application_count: usage.application_count,
                success_rate: usage.success_rate(),
                application_rate: usage.application_rate(),
                effectiveness_score: usage.effectiveness_score,
                days_since_last_use: days_since,
                usage_count: usage.retrieval_count + usage.application_count,
            }
        })
    }

    /// Get all patterns sorted by effectiveness (best first)
    #[must_use]
    pub fn get_ranked_patterns(&self) -> Vec<(Uuid, f32)> {
        let mut patterns: Vec<_> = self
            .usage_map
            .iter()
            .map(|(id, usage)| (*id, usage.effectiveness_score))
            .collect();

        patterns.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        patterns
    }

    /// Get patterns that should be decayed (low effectiveness)
    #[must_use]
    pub fn get_patterns_to_decay(&self) -> Vec<Uuid> {
        self.usage_map
            .iter()
            .filter(|(_, usage)| !usage.should_keep(self.min_effectiveness))
            .map(|(id, _)| *id)
            .collect()
    }

    /// Perform periodic decay of ineffective patterns
    #[instrument(skip(self))]
    pub fn decay_old_patterns(&mut self) -> Vec<Uuid> {
        let now = Utc::now();

        // Check if it's time to decay
        if (now - self.last_decay).num_days() < self.decay_interval_days {
            return Vec::new();
        }

        // Update effectiveness scores for all patterns
        for usage in self.usage_map.values_mut() {
            usage.update_effectiveness_score();
        }

        // Find patterns to remove
        let to_remove = self.get_patterns_to_decay();

        // Remove them
        for pattern_id in &to_remove {
            self.usage_map.remove(pattern_id);
        }

        self.last_decay = now;

        debug!(
            removed_count = to_remove.len(),
            remaining_count = self.usage_map.len(),
            "Decayed old patterns"
        );

        to_remove
    }

    /// Get total number of tracked patterns
    #[must_use]
    pub fn pattern_count(&self) -> usize {
        self.usage_map.len()
    }

    /// Get overall system effectiveness statistics
    #[must_use]
    pub fn overall_stats(&self) -> OverallStats {
        let total_patterns = self.usage_map.len();
        if total_patterns == 0 {
            return OverallStats::default();
        }

        let total_retrievals: usize = self.usage_map.values().map(|u| u.retrieval_count).sum();
        let total_applications: usize = self.usage_map.values().map(|u| u.application_count).sum();
        let total_successes: usize = self.usage_map.values().map(|u| u.success_count).sum();

        let avg_effectiveness: f32 = self
            .usage_map
            .values()
            .map(|u| u.effectiveness_score)
            .sum::<f32>()
            / total_patterns as f32;

        let active_patterns = self
            .usage_map
            .values()
            .filter(|u| {
                u.last_applied
                    .is_some_and(|t| (Utc::now() - t).num_days() < 30)
            })
            .count();

        OverallStats {
            total_patterns,
            active_patterns,
            total_retrievals,
            total_applications,
            overall_success_rate: if total_applications > 0 {
                total_successes as f32 / total_applications as f32
            } else {
                0.0
            },
            avg_effectiveness,
        }
    }

    /// Clear all tracking data
    pub fn clear(&mut self) {
        self.usage_map.clear();
        self.last_decay = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_record_retrieval() {
        let mut tracker = EffectivenessTracker::new();
        let pattern_id = Uuid::new_v4();

        tracker.record_retrieval(pattern_id);
        tracker.record_retrieval(pattern_id);

        let stats = tracker.get_stats(pattern_id).unwrap();
        assert_eq!(stats.retrieval_count, 2);
        assert_eq!(stats.application_count, 0);
    }

    #[test]
    fn test_record_application() {
        let mut tracker = EffectivenessTracker::new();
        let pattern_id = Uuid::new_v4();

        tracker.record_application(pattern_id, true);
        tracker.record_application(pattern_id, true);
        tracker.record_application(pattern_id, false);

        let stats = tracker.get_stats(pattern_id).unwrap();
        assert_eq!(stats.application_count, 3);
        assert_eq!(stats.success_rate, 2.0 / 3.0);
    }

    #[test]
    fn test_get_ranked_patterns() {
        let mut tracker = EffectivenessTracker::new();

        let p1 = Uuid::new_v4();
        let p2 = Uuid::new_v4();
        let p3 = Uuid::new_v4();

        // Make p1 most effective
        for _ in 0..5 {
            tracker.record_application(p1, true);
        }

        // Make p2 moderately effective
        tracker.record_application(p2, true);
        tracker.record_application(p2, false);

        // Make p3 least effective
        tracker.record_application(p3, false);

        let ranked = tracker.get_ranked_patterns();

        // p1 should be first (most effective)
        assert_eq!(ranked[0].0, p1);
        assert!(ranked[0].1 > ranked[1].1);
    }

    #[test]
    fn test_decay_old_patterns() {
        let mut tracker = EffectivenessTracker::with_config(0.3, 0); // Decay immediately

        let good_pattern = Uuid::new_v4();
        let bad_pattern = Uuid::new_v4();

        // Good pattern: high success rate
        for _ in 0..5 {
            tracker.record_application(good_pattern, true);
        }

        // Bad pattern: low success rate
        tracker.record_application(bad_pattern, false);

        // Force decay
        tracker.last_decay = Utc::now() - Duration::days(1);

        let decayed = tracker.decay_old_patterns();

        // Bad pattern should be decayed
        assert!(decayed.contains(&bad_pattern));
        assert!(!decayed.contains(&good_pattern));

        // Good pattern should still be tracked
        assert!(tracker.get_stats(good_pattern).is_some());
        assert!(tracker.get_stats(bad_pattern).is_none());
    }

    #[test]
    fn test_overall_stats() {
        let mut tracker = EffectivenessTracker::new();

        let p1 = Uuid::new_v4();
        let p2 = Uuid::new_v4();

        tracker.record_retrieval(p1);
        tracker.record_application(p1, true);
        tracker.record_application(p1, true);

        tracker.record_retrieval(p2);
        tracker.record_application(p2, false);

        let stats = tracker.overall_stats();

        assert_eq!(stats.total_patterns, 2);
        assert_eq!(stats.total_retrievals, 2);
        assert_eq!(stats.total_applications, 3);
        assert_eq!(stats.overall_success_rate, 2.0 / 3.0);
        assert!(stats.avg_effectiveness > 0.0);
    }

    #[test]
    fn test_clear_tracker() {
        let mut tracker = EffectivenessTracker::new();

        tracker.record_application(Uuid::new_v4(), true);
        tracker.record_application(Uuid::new_v4(), false);

        assert_eq!(tracker.pattern_count(), 2);

        tracker.clear();

        assert_eq!(tracker.pattern_count(), 0);
    }
}
