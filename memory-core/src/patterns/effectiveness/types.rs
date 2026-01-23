//! Pattern usage tracking types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Tracks usage of a specific pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternUsage {
    /// Pattern ID
    pub pattern_id: Uuid,
    /// Number of times pattern was retrieved
    pub retrieval_count: usize,
    /// Number of times pattern was applied
    pub application_count: usize,
    /// Number of successful applications
    pub success_count: usize,
    /// Number of failed applications
    pub failure_count: usize,
    /// Last time pattern was retrieved
    pub last_retrieved: Option<DateTime<Utc>>,
    /// Last time pattern was applied
    pub last_applied: Option<DateTime<Utc>>,
    /// When tracking started
    pub created_at: DateTime<Utc>,
    /// Current effectiveness score (0.0 to 1.0)
    pub effectiveness_score: f32,
}

impl PatternUsage {
    /// Create new usage tracking for a pattern
    #[must_use]
    pub fn new(pattern_id: Uuid) -> Self {
        Self {
            pattern_id,
            retrieval_count: 0,
            application_count: 0,
            success_count: 0,
            failure_count: 0,
            last_retrieved: None,
            last_applied: None,
            created_at: Utc::now(),
            effectiveness_score: 0.5, // Start neutral
        }
    }

    /// Calculate success rate (0.0 to 1.0)
    #[must_use]
    pub fn success_rate(&self) -> f32 {
        if self.application_count == 0 {
            return 0.5; // Neutral if never applied
        }
        self.success_count as f32 / self.application_count as f32
    }

    /// Calculate application rate (how often retrieved patterns are applied)
    #[must_use]
    pub fn application_rate(&self) -> f32 {
        if self.retrieval_count == 0 {
            return 0.0;
        }
        self.application_count as f32 / self.retrieval_count as f32
    }

    /// Calculate recency factor (0.0 to 1.0, higher for recent usage)
    #[must_use]
    pub fn recency_factor(&self, now: DateTime<Utc>) -> f32 {
        let last_use = self
            .last_applied
            .or(self.last_retrieved)
            .unwrap_or(self.created_at);

        let days_since_use = (now - last_use).num_days().max(0) as f32;

        // Decay with half-life of 30 days
        let decay_rate = (days_since_use / 30.0).min(10.0);
        (-decay_rate * 0.693).exp() // e^(-decay_rate * ln(2))
    }

    /// Update effectiveness score based on usage patterns
    pub fn update_effectiveness_score(&mut self) {
        let now = Utc::now();

        // Weight factors:
        // - Success rate (40%): How well it works when applied
        // - Application rate (30%): How often it's actually used after retrieval
        // - Recency (20%): How recently it's been used
        // - Confidence (10%): Higher with more data

        let success_weight = 0.4;
        let application_weight = 0.3;
        let recency_weight = 0.2;
        let confidence_weight = 0.1;

        let success_score = self.success_rate();
        let application_score = self.application_rate();
        let recency_score = self.recency_factor(now);

        // Confidence increases with usage (max at 20 applications)
        let confidence_score = (self.application_count.min(20) as f32 / 20.0).min(1.0);

        self.effectiveness_score = (success_score * success_weight)
            + (application_score * application_weight)
            + (recency_score * recency_weight)
            + (confidence_score * confidence_weight);
    }

    /// Check if pattern should be kept (not decayed away)
    #[must_use]
    pub fn should_keep(&self, min_effectiveness: f32) -> bool {
        self.effectiveness_score >= min_effectiveness
    }
}

/// Statistics about pattern usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    /// Total retrievals
    pub retrieval_count: usize,
    /// Total applications
    pub application_count: usize,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f32,
    /// Application rate (0.0 to 1.0)
    pub application_rate: f32,
    /// Effectiveness score (0.0 to 1.0)
    pub effectiveness_score: f32,
    /// Days since last use
    pub days_since_last_use: i64,
    /// Total usage count (retrievals + applications)
    pub usage_count: usize,
}

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
    pub fn record_retrieval(&mut self, pattern_id: Uuid) {
        let usage = self
            .usage_map
            .entry(pattern_id)
            .or_insert_with(|| PatternUsage::new(pattern_id));

        usage.retrieval_count += 1;
        usage.last_retrieved = Some(Utc::now());
    }

    /// Record that a pattern was applied with a result
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
    fn test_pattern_usage_creation() {
        let pattern_id = Uuid::new_v4();
        let usage = PatternUsage::new(pattern_id);

        assert_eq!(usage.pattern_id, pattern_id);
        assert_eq!(usage.retrieval_count, 0);
        assert_eq!(usage.application_count, 0);
        assert_eq!(usage.success_count, 0);
        assert_eq!(usage.effectiveness_score, 0.5);
    }

    #[test]
    fn test_success_rate_calculation() {
        let mut usage = PatternUsage::new(Uuid::new_v4());

        // No applications yet
        assert_eq!(usage.success_rate(), 0.5);

        // Add some successes
        usage.application_count = 10;
        usage.success_count = 8;
        assert_eq!(usage.success_rate(), 0.8);

        // All failures
        usage.success_count = 0;
        assert_eq!(usage.success_rate(), 0.0);
    }

    #[test]
    fn test_application_rate() {
        let mut usage = PatternUsage::new(Uuid::new_v4());

        usage.retrieval_count = 10;
        usage.application_count = 5;
        assert_eq!(usage.application_rate(), 0.5);

        // Never retrieved
        usage.retrieval_count = 0;
        assert_eq!(usage.application_rate(), 0.0);
    }

    #[test]
    fn test_recency_factor() {
        let mut usage = PatternUsage::new(Uuid::new_v4());
        let now = Utc::now();

        // Just created
        let recency = usage.recency_factor(now);
        assert!(recency > 0.95); // Very recent

        // Simulate old usage (30 days ago)
        usage.last_applied = Some(now - Duration::days(30));
        let recency = usage.recency_factor(now);
        assert!(recency < 0.6 && recency > 0.4); // Half-life

        // Very old usage (90 days ago)
        usage.last_applied = Some(now - Duration::days(90));
        let recency = usage.recency_factor(now);
        assert!(recency < 0.2); // Low recency
    }

    #[test]
    fn test_effectiveness_score_update() {
        let mut usage = PatternUsage::new(Uuid::new_v4());

        // Simulate good usage pattern
        usage.retrieval_count = 10;
        usage.application_count = 8;
        usage.success_count = 7;
        usage.last_applied = Some(Utc::now());

        usage.update_effectiveness_score();

        // Should have high effectiveness
        assert!(usage.effectiveness_score > 0.6);

        // Simulate poor usage pattern
        usage.application_count = 1;
        usage.success_count = 0;
        usage.last_applied = Some(Utc::now() - Duration::days(60));

        usage.update_effectiveness_score();

        // Should have low effectiveness
        assert!(usage.effectiveness_score < 0.4);
    }

    #[test]
    fn test_should_keep_pattern() {
        let mut usage = PatternUsage::new(Uuid::new_v4());

        // High effectiveness
        usage.effectiveness_score = 0.8;
        assert!(usage.should_keep(0.3));

        // Low effectiveness
        usage.effectiveness_score = 0.2;
        assert!(!usage.should_keep(0.3));

        // Edge case
        usage.effectiveness_score = 0.3;
        assert!(usage.should_keep(0.3));
    }
}
