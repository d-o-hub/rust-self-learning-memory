//! Pattern type definitions

use chrono::Duration;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::episode::PatternId;
use crate::types::{OutcomeStats, TaskContext};

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
    },
    /// Decision point with outcome statistics
    DecisionPoint {
        id: PatternId,
        condition: String,
        action: String,
        outcome_stats: OutcomeStats,
        context: TaskContext,
    },
    /// Error recovery pattern
    ErrorRecovery {
        id: PatternId,
        error_type: String,
        recovery_steps: Vec<String>,
        success_rate: f32,
        context: TaskContext,
    },
    /// Context-based pattern
    ContextPattern {
        id: PatternId,
        context_features: Vec<String>,
        recommended_approach: String,
        evidence: Vec<Uuid>, // Episode IDs
        success_rate: f32,
    },
}

impl Pattern {
    /// Get the unique ID of this pattern
    pub fn id(&self) -> PatternId {
        match self {
            Pattern::ToolSequence { id, .. } => *id,
            Pattern::DecisionPoint { id, .. } => *id,
            Pattern::ErrorRecovery { id, .. } => *id,
            Pattern::ContextPattern { id, .. } => *id,
        }
    }

    /// Get the success rate of this pattern
    pub fn success_rate(&self) -> f32 {
        match self {
            Pattern::ToolSequence { success_rate, .. } => *success_rate,
            Pattern::DecisionPoint { outcome_stats, .. } => outcome_stats.success_rate(),
            Pattern::ErrorRecovery { success_rate, .. } => *success_rate,
            Pattern::ContextPattern { success_rate, .. } => *success_rate,
        }
    }

    /// Get the context associated with this pattern
    pub fn context(&self) -> Option<&TaskContext> {
        match self {
            Pattern::ToolSequence { context, .. } => Some(context),
            Pattern::DecisionPoint { context, .. } => Some(context),
            Pattern::ErrorRecovery { context, .. } => Some(context),
            Pattern::ContextPattern { .. } => None,
        }
    }

    /// Get the sample size (number of occurrences) for this pattern
    pub(super) fn sample_size(&self) -> usize {
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
}
