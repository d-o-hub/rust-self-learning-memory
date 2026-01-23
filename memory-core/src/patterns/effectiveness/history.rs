//! History tracking for pattern effectiveness
//!
//! Tracks usage patterns over time, including retrieval counts,
//! application counts, success rates, and recency factors.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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
