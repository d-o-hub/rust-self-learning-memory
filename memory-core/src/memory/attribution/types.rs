//! Recommendation Attribution Types
//!
//! Types for tracking recommendation sessions and feedback, enabling
//! the system to learn which recommendations actually help agents.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::TaskOutcome;

/// Records a recommendation session when patterns/playbooks are suggested to an agent.
///
/// This struct captures what was recommended at a point in time, enabling
/// later correlation with what the agent actually used and whether it helped.
///
/// # Example
///
/// ```
/// use do_memory_core::memory::attribution::RecommendationSession;
/// use uuid::Uuid;
///
/// let session = RecommendationSession {
///     session_id: Uuid::new_v4(),
///     episode_id: Uuid::new_v4(),
///     timestamp: chrono::Utc::now(),
///     recommended_pattern_ids: vec!["pattern-1".to_string(), "pattern-2".to_string()],
///     recommended_playbook_ids: vec![Uuid::new_v4()],
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecommendationSession {
    /// Unique identifier for this recommendation session
    pub session_id: Uuid,
    /// The episode for which recommendations were made
    pub episode_id: Uuid,
    /// When the recommendations were made
    pub timestamp: DateTime<Utc>,
    /// Pattern IDs that were recommended (string format for flexibility)
    pub recommended_pattern_ids: Vec<String>,
    /// Playbook IDs that were recommended
    pub recommended_playbook_ids: Vec<Uuid>,
}

/// Records agent feedback about which recommendations were used and outcomes.
///
/// This closes the feedback loop, enabling the system to learn which
/// recommendations are actually helpful. Agents should record feedback
/// after completing or abandoning a task.
///
/// # Example
///
/// ```
/// use do_memory_core::memory::attribution::RecommendationFeedback;
/// use do_memory_core::TaskOutcome;
/// use uuid::Uuid;
///
/// let feedback = RecommendationFeedback {
///     session_id: Uuid::new_v4(),
///     applied_pattern_ids: vec!["pattern-1".to_string()],
///     consulted_episode_ids: vec![Uuid::new_v4()],
///     outcome: TaskOutcome::Success {
///         verdict: "Task completed successfully".to_string(),
///         artifacts: vec!["output.txt".to_string()],
///     },
///     agent_rating: Some(0.9),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecommendationFeedback {
    /// The session this feedback relates to
    pub session_id: Uuid,
    /// Pattern IDs that were actually applied by the agent
    pub applied_pattern_ids: Vec<String>,
    /// Episode IDs that were consulted (from retrieval results)
    pub consulted_episode_ids: Vec<Uuid>,
    /// The final outcome of the task
    pub outcome: TaskOutcome,
    /// Optional agent rating of recommendation quality (0.0-1.0)
    pub agent_rating: Option<f32>,
}

/// Statistics about recommendation effectiveness.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecommendationStats {
    /// Total recommendation sessions recorded
    pub total_sessions: usize,
    /// Total feedback records received
    pub total_feedback: usize,
    /// Patterns recommended and applied
    pub patterns_applied: usize,
    /// Patterns recommended but not applied
    pub patterns_ignored: usize,
    /// Sessions with successful outcomes where patterns were applied
    pub successful_applications: usize,
    /// Overall adoption rate (applied / recommended)
    pub adoption_rate: f32,
    /// Success rate after adoption (successful / applied)
    pub success_after_adoption_rate: f32,
    /// Average agent rating for recommendations
    pub avg_agent_rating: Option<f32>,
}

impl Default for RecommendationStats {
    fn default() -> Self {
        Self {
            total_sessions: 0,
            total_feedback: 0,
            patterns_applied: 0,
            patterns_ignored: 0,
            successful_applications: 0,
            adoption_rate: 0.0,
            success_after_adoption_rate: 0.0,
            avg_agent_rating: None,
        }
    }
}

/// Combined session with its feedback (if any).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionWithFeedback {
    /// The recommendation session
    pub session: RecommendationSession,
    /// Associated feedback, if provided
    pub feedback: Option<RecommendationFeedback>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recommendation_session_creation() {
        let session = RecommendationSession {
            session_id: Uuid::new_v4(),
            episode_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            recommended_pattern_ids: vec!["p1".to_string()],
            recommended_playbook_ids: vec![Uuid::new_v4()],
        };

        assert!(!session.recommended_pattern_ids.is_empty());
    }

    #[test]
    fn test_recommendation_feedback_creation() {
        let feedback = RecommendationFeedback {
            session_id: Uuid::new_v4(),
            applied_pattern_ids: vec!["p1".to_string()],
            consulted_episode_ids: vec![Uuid::new_v4()],
            outcome: TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            },
            agent_rating: Some(0.85),
        };

        assert_eq!(feedback.agent_rating, Some(0.85));
    }

    #[test]
    fn test_recommendation_stats_default() {
        let stats = RecommendationStats::default();
        assert_eq!(stats.total_sessions, 0);
        assert_eq!(stats.adoption_rate, 0.0);
    }
}
