use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::episode::PatternId;
use crate::types::{Evidence, OutcomeStats, TaskContext};

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

    /// Check if this pattern is relevant to a given context
    pub fn is_relevant_to(&self, query_context: &TaskContext) -> bool {
        if let Some(pattern_context) = self.context() {
            // Match on domain
            if pattern_context.domain == query_context.domain {
                return true;
            }

            // Match on language
            if pattern_context.language == query_context.language
                && pattern_context.language.is_some()
            {
                return true;
            }

            // Match on tags
            let common_tags: Vec<_> = pattern_context
                .tags
                .iter()
                .filter(|t| query_context.tags.contains(t))
                .collect();

            if !common_tags.is_empty() {
                return true;
            }
        }

        false
    }
}

/// Heuristic rule learned from patterns
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Heuristic {
    /// Unique heuristic ID
    pub heuristic_id: Uuid,
    /// Condition to check (as natural language or code)
    pub condition: String,
    /// Recommended action
    pub action: String,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f32,
    /// Evidence supporting this heuristic
    pub evidence: Evidence,
    /// When created
    pub created_at: DateTime<Utc>,
    /// Last updated
    pub updated_at: DateTime<Utc>,
}

impl Heuristic {
    /// Create a new heuristic
    pub fn new(condition: String, action: String, confidence: f32) -> Self {
        let now = Utc::now();
        Self {
            heuristic_id: Uuid::new_v4(),
            condition,
            action,
            confidence,
            evidence: Evidence {
                episode_ids: Vec::new(),
                success_rate: 0.0,
                sample_size: 0,
            },
            created_at: now,
            updated_at: now,
        }
    }

    /// Update the heuristic with new evidence
    pub fn update_evidence(&mut self, episode_id: Uuid, success: bool) {
        self.evidence.episode_ids.push(episode_id);
        self.evidence.sample_size += 1;

        // Recalculate success rate
        let successes = if success {
            (self.evidence.success_rate * (self.evidence.sample_size - 1) as f32) + 1.0
        } else {
            self.evidence.success_rate * (self.evidence.sample_size - 1) as f32
        };

        self.evidence.success_rate = successes / self.evidence.sample_size as f32;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ComplexityLevel;

    #[test]
    fn test_pattern_id() {
        let pattern = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["tool1".to_string(), "tool2".to_string()],
            context: TaskContext::default(),
            success_rate: 0.9,
            avg_latency: Duration::milliseconds(100),
            occurrence_count: 5,
        };

        assert!(pattern.success_rate() > 0.8);
        assert!(pattern.context().is_some());
    }

    #[test]
    fn test_pattern_relevance() {
        let pattern_context = TaskContext {
            language: Some("rust".to_string()),
            framework: None,
            complexity: ComplexityLevel::Moderate,
            domain: "web-api".to_string(),
            tags: vec!["async".to_string()],
        };

        let pattern = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec![],
            context: pattern_context.clone(),
            success_rate: 0.9,
            avg_latency: Duration::milliseconds(100),
            occurrence_count: 1,
        };

        // Should match on domain
        let query_context = TaskContext {
            domain: "web-api".to_string(),
            ..Default::default()
        };
        assert!(pattern.is_relevant_to(&query_context));

        // Should match on language
        let query_context2 = TaskContext {
            language: Some("rust".to_string()),
            domain: "cli".to_string(),
            ..Default::default()
        };
        assert!(pattern.is_relevant_to(&query_context2));

        // Should not match
        let query_context3 = TaskContext {
            language: Some("python".to_string()),
            domain: "data-science".to_string(),
            ..Default::default()
        };
        assert!(!pattern.is_relevant_to(&query_context3));
    }

    #[test]
    fn test_heuristic_evidence_update() {
        let mut heuristic = Heuristic::new(
            "When refactoring async code".to_string(),
            "Use tokio::spawn for CPU-intensive tasks".to_string(),
            0.7,
        );

        assert_eq!(heuristic.evidence.sample_size, 0);

        // Add successful evidence
        heuristic.update_evidence(Uuid::new_v4(), true);
        assert_eq!(heuristic.evidence.sample_size, 1);
        assert_eq!(heuristic.evidence.success_rate, 1.0);

        // Add failed evidence
        heuristic.update_evidence(Uuid::new_v4(), false);
        assert_eq!(heuristic.evidence.sample_size, 2);
        assert_eq!(heuristic.evidence.success_rate, 0.5);

        // Add more successful evidence
        heuristic.update_evidence(Uuid::new_v4(), true);
        assert_eq!(heuristic.evidence.sample_size, 3);
        assert!((heuristic.evidence.success_rate - 0.666).abs() < 0.01);
    }
}
