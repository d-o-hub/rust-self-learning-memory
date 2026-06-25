//! Pattern type definitions

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::episode::PatternId;
use crate::types::{OutcomeStats, TaskContext};

/// Merge two ToolSequence pattern statistics
pub(super) fn merge_tool_sequence(
    sr1: &mut f32,
    oc1: &mut usize,
    lat1: &mut Duration,
    sr2: f32,
    oc2: usize,
    lat2: &Duration,
) {
    let total_count = *oc1 + oc2;
    *sr1 = (*sr1 * *oc1 as f32 + sr2 * oc2 as f32) / total_count as f32;
    *lat1 = Duration::milliseconds(
        (lat1.num_milliseconds() * *oc1 as i64 + lat2.num_milliseconds() * oc2 as i64)
            / total_count as i64,
    );
    *oc1 = total_count;
}

/// Merge two DecisionPoint pattern statistics
pub(super) fn merge_decision_point(stats1: &mut OutcomeStats, stats2: &OutcomeStats) {
    let prev_total = stats1.total_count;
    stats1.success_count += stats2.success_count;
    stats1.failure_count += stats2.failure_count;
    stats1.total_count += stats2.total_count;
    stats1.avg_duration_secs = (stats1.avg_duration_secs * prev_total as f32
        + stats2.avg_duration_secs * stats2.total_count as f32)
        / stats1.total_count as f32;
}

/// Merge two ErrorRecovery pattern statistics
pub(super) fn merge_error_recovery(sr1: &mut f32, sr2: f32) {
    *sr1 = f32::midpoint(*sr1, sr2);
}

/// Merge two ContextPattern pattern statistics
pub(super) fn merge_context_pattern(ev1: &mut Vec<Uuid>, sr1: &mut f32, ev2: &[Uuid], sr2: f32) {
    let size1 = ev1.len();
    let size2 = ev2.len();
    ev1.extend_from_slice(ev2);
    *sr1 = (*sr1 * size1 as f32 + sr2 * size2 as f32) / (size1 + size2) as f32;
}

/// Tracks the real-world effectiveness of a pattern based on actual usage
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternEffectiveness {
    /// Number of times this pattern was retrieved in queries
    pub times_retrieved: usize,
    /// Number of times this pattern was explicitly applied
    pub times_applied: usize,
    /// Number of successful outcomes when applied
    pub success_when_applied: usize,
    /// Number of failed outcomes when applied
    pub failure_when_applied: usize,
    /// Average reward improvement when this pattern was used (can be negative)
    pub avg_reward_delta: f32,
    /// When this pattern was last used
    pub last_used: DateTime<Utc>,
    /// When this pattern was created
    pub created_at: DateTime<Utc>,
}

impl Default for PatternEffectiveness {
    fn default() -> Self {
        Self {
            times_retrieved: 0,
            times_applied: 0,
            success_when_applied: 0,
            failure_when_applied: 0,
            avg_reward_delta: 0.0,
            last_used: Utc::now(),
            created_at: Utc::now(),
        }
    }
}

impl PatternEffectiveness {
    /// Create a new effectiveness tracker
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate the success rate when applied (0.0 to 1.0)
    #[must_use]
    pub fn application_success_rate(&self) -> f32 {
        if self.times_applied == 0 {
            0.5 // Neutral for untested patterns
        } else {
            self.success_when_applied as f32 / self.times_applied as f32
        }
    }

    /// Calculate the usage rate (applied / retrieved)
    #[must_use]
    pub fn usage_rate(&self) -> f32 {
        if self.times_retrieved == 0 {
            0.0
        } else {
            self.times_applied as f32 / self.times_retrieved as f32
        }
    }

    /// Calculate an overall effectiveness score (0.0 to 1.0+)
    /// Combines success rate, usage rate, and reward delta
    #[must_use]
    pub fn effectiveness_score(&self) -> f32 {
        let success_rate = self.application_success_rate();
        let usage_weight = (self.times_applied as f32).ln().clamp(0.0, 3.0);
        let reward_factor = (1.0 + self.avg_reward_delta).max(0.0);

        // Score combines success rate, confidence from usage, and reward impact
        success_rate * (1.0 + usage_weight / 3.0) * reward_factor
    }

    /// Record that this pattern was retrieved
    pub fn record_retrieval(&mut self) {
        self.times_retrieved += 1;
    }

    /// Record that this pattern was applied with an outcome
    pub fn record_application(&mut self, success: bool, reward_delta: f32) {
        self.times_applied += 1;
        if success {
            self.success_when_applied += 1;
        } else {
            self.failure_when_applied += 1;
        }

        // Update moving average of reward delta
        let n = self.times_applied as f32;
        self.avg_reward_delta = ((n - 1.0) * self.avg_reward_delta + reward_delta) / n;

        self.last_used = Utc::now();
    }
}

/// Pattern types extracted from episodes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Pattern {
    /// Sequence of tools used successfully
    ToolSequence {
        id: PatternId,
        tools: Vec<String>,
        context: TaskContext,
        success_rate: f32,
        avg_latency: Duration,
        occurrence_count: usize,
        #[serde(default)]
        effectiveness: PatternEffectiveness,
    },
    /// Decision point with outcome statistics
    DecisionPoint {
        id: PatternId,
        condition: String,
        action: String,
        outcome_stats: OutcomeStats,
        context: TaskContext,
        #[serde(default)]
        effectiveness: PatternEffectiveness,
    },
    /// Error recovery pattern
    ErrorRecovery {
        id: PatternId,
        error_type: String,
        recovery_steps: Vec<String>,
        success_rate: f32,
        context: TaskContext,
        #[serde(default)]
        effectiveness: PatternEffectiveness,
    },
    /// Context-based pattern
    ContextPattern {
        id: PatternId,
        context_features: Vec<String>,
        recommended_approach: String,
        evidence: Vec<Uuid>, // Episode IDs
        success_rate: f32,
        #[serde(default)]
        effectiveness: PatternEffectiveness,
    },
}

impl Pattern {
    /// Get the unique ID of this pattern
    #[must_use]
    pub fn id(&self) -> PatternId {
        match self {
            Pattern::ToolSequence { id, .. } => *id,
            Pattern::DecisionPoint { id, .. } => *id,
            Pattern::ErrorRecovery { id, .. } => *id,
            Pattern::ContextPattern { id, .. } => *id,
        }
    }

    /// Get the success rate of this pattern
    #[must_use]
    pub fn success_rate(&self) -> f32 {
        match self {
            Pattern::ToolSequence { success_rate, .. } => *success_rate,
            Pattern::DecisionPoint { outcome_stats, .. } => outcome_stats.success_rate(),
            Pattern::ErrorRecovery { success_rate, .. } => *success_rate,
            Pattern::ContextPattern { success_rate, .. } => *success_rate,
        }
    }

    /// Get the context associated with this pattern
    #[must_use]
    pub fn context(&self) -> Option<&TaskContext> {
        match self {
            Pattern::ToolSequence { context, .. } => Some(context),
            Pattern::DecisionPoint { context, .. } => Some(context),
            Pattern::ErrorRecovery { context, .. } => Some(context),
            Pattern::ContextPattern { .. } => None,
        }
    }

    /// Get the sample size (number of occurrences) for this pattern
    #[must_use]
    pub fn sample_size(&self) -> usize {
        match self {
            Pattern::ToolSequence {
                occurrence_count, ..
            } => *occurrence_count,
            Pattern::DecisionPoint { outcome_stats, .. } => outcome_stats.total_count,
            Pattern::ErrorRecovery { .. } => {
                // For error recovery, we estimate from context complexity
                // This is a fallback; ideally we'd track occurrences
                1
            }
            Pattern::ContextPattern { evidence, .. } => evidence.len(),
        }
    }

    /// Get the effectiveness tracking data for this pattern
    #[must_use]
    pub fn effectiveness(&self) -> &PatternEffectiveness {
        match self {
            Pattern::ToolSequence { effectiveness, .. } => effectiveness,
            Pattern::DecisionPoint { effectiveness, .. } => effectiveness,
            Pattern::ErrorRecovery { effectiveness, .. } => effectiveness,
            Pattern::ContextPattern { effectiveness, .. } => effectiveness,
        }
    }

    /// Get mutable access to effectiveness tracking data
    #[must_use]
    pub fn effectiveness_mut(&mut self) -> &mut PatternEffectiveness {
        match self {
            Pattern::ToolSequence { effectiveness, .. } => effectiveness,
            Pattern::DecisionPoint { effectiveness, .. } => effectiveness,
            Pattern::ErrorRecovery { effectiveness, .. } => effectiveness,
            Pattern::ContextPattern { effectiveness, .. } => effectiveness,
        }
    }

    /// Record that this pattern was retrieved in a query
    pub fn record_retrieval(&mut self) {
        self.effectiveness_mut().record_retrieval();
    }

    /// Record that this pattern was applied with an outcome
    pub fn record_application(&mut self, success: bool, reward_delta: f32) {
        self.effectiveness_mut()
            .record_application(success, reward_delta);
    }
}
