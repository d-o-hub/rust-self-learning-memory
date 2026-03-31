//! Recommendation Feedback Tool Implementation
//!
//! MCP tool for recording feedback about recommendation effectiveness.

use super::types::{
    RecommendationStatsOutput, RecordRecommendationFeedbackInput,
    RecordRecommendationFeedbackOutput, RecordRecommendationSessionInput,
    RecordRecommendationSessionOutput,
};
use anyhow::{Result, anyhow};
use do_memory_core::SelfLearningMemory;
use do_memory_core::memory::attribution::{RecommendationFeedback, RecommendationSession};
use std::sync::Arc;
use tracing::{info, instrument};
use uuid::Uuid;

/// Tools for recording and querying recommendation feedback.
#[derive(Clone)]
pub struct RecommendationFeedbackTools {
    memory: Arc<SelfLearningMemory>,
}

impl RecommendationFeedbackTools {
    /// Create a new instance.
    pub fn new(memory: Arc<SelfLearningMemory>) -> Self {
        Self { memory }
    }

    /// Record a recommendation session.
    ///
    /// Call this when the system recommends patterns or playbooks to an agent.
    /// This creates a session that can later be correlated with feedback.
    #[instrument(skip(self, input), fields(episode_id = %input.episode_id))]
    pub async fn record_session(
        &self,
        input: RecordRecommendationSessionInput,
    ) -> Result<RecordRecommendationSessionOutput> {
        info!(
            "Recording recommendation session for episode: {}",
            input.episode_id
        );

        // Parse episode ID
        let episode_id =
            Uuid::parse_str(&input.episode_id).map_err(|e| anyhow!("Invalid episode ID: {}", e))?;

        // Create session
        let session_id = Uuid::new_v4();
        let session = RecommendationSession {
            session_id,
            episode_id,
            timestamp: chrono::Utc::now(),
            recommended_pattern_ids: input.recommended_pattern_ids.clone(),
            recommended_playbook_ids: input
                .recommended_playbook_ids
                .iter()
                .filter_map(|id| Uuid::parse_str(id).ok())
                .collect(),
        };

        // Record session
        self.memory.record_recommendation_session(session).await;

        let patterns_count = input.recommended_pattern_ids.len();
        let playbooks_count = input.recommended_playbook_ids.len();

        info!(
            session_id = %session_id,
            patterns = patterns_count,
            playbooks = playbooks_count,
            "Recorded recommendation session"
        );

        Ok(RecordRecommendationSessionOutput {
            success: true,
            session_id: session_id.to_string(),
            episode_id: input.episode_id,
            patterns_recommended: patterns_count,
            playbooks_recommended: playbooks_count,
            message: format!(
                "Recorded recommendation session with {} patterns and {} playbooks",
                patterns_count, playbooks_count
            ),
        })
    }

    /// Record feedback about a recommendation session.
    ///
    /// Call this after an agent completes or abandons a task to indicate
    /// which recommendations were used and the outcome.
    #[instrument(skip(self, input), fields(session_id = %input.session_id))]
    pub async fn record_feedback(
        &self,
        input: RecordRecommendationFeedbackInput,
    ) -> Result<RecordRecommendationFeedbackOutput> {
        info!(
            "Recording recommendation feedback for session: {}",
            input.session_id
        );

        // Parse session ID
        let session_id =
            Uuid::parse_str(&input.session_id).map_err(|e| anyhow!("Invalid session ID: {}", e))?;

        // Convert consulted episode IDs
        let consulted_episode_ids: Vec<Uuid> = input
            .consulted_episode_ids
            .iter()
            .filter_map(|id| Uuid::parse_str(id).ok())
            .collect();

        // Create feedback
        let feedback = RecommendationFeedback {
            session_id,
            applied_pattern_ids: input.applied_pattern_ids.clone(),
            consulted_episode_ids,
            outcome: input.outcome.to_task_outcome(),
            agent_rating: input.agent_rating,
        };

        // Record feedback
        self.memory.record_recommendation_feedback(feedback).await?;

        let patterns_applied = input.applied_pattern_ids.len();
        let episodes_consulted = input.consulted_episode_ids.len();

        info!(
            session_id = %session_id,
            patterns_applied = patterns_applied,
            episodes_consulted = episodes_consulted,
            "Recorded recommendation feedback"
        );

        Ok(RecordRecommendationFeedbackOutput {
            success: true,
            session_id: input.session_id,
            patterns_applied,
            episodes_consulted,
            message: format!(
                "Recorded feedback: {} patterns applied, {} episodes consulted",
                patterns_applied, episodes_consulted
            ),
        })
    }

    /// Get recommendation statistics.
    #[instrument(skip(self))]
    pub async fn get_stats(&self) -> Result<RecommendationStatsOutput> {
        info!("Getting recommendation statistics");

        let stats = self.memory.get_recommendation_stats().await;

        Ok(RecommendationStatsOutput {
            success: true,
            total_sessions: stats.total_sessions,
            total_feedback: stats.total_feedback,
            patterns_applied: stats.patterns_applied,
            patterns_ignored: stats.patterns_ignored,
            adoption_rate: stats.adoption_rate,
            success_after_adoption_rate: stats.success_after_adoption_rate,
            avg_agent_rating: stats.avg_agent_rating,
            message: format!(
                "Adoption rate: {:.1}%, Success after adoption: {:.1}%",
                stats.adoption_rate * 100.0,
                stats.success_after_adoption_rate * 100.0
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::tools::recommendation_feedback::TaskOutcomeJson;

    #[tokio::test]
    async fn test_record_session() {
        let memory = Arc::new(SelfLearningMemory::new());
        let tools = RecommendationFeedbackTools::new(memory);

        let input = RecordRecommendationSessionInput {
            episode_id: Uuid::new_v4().to_string(),
            recommended_pattern_ids: vec!["p1".to_string()],
            recommended_playbook_ids: vec![],
        };

        let output = tools.record_session(input).await.unwrap();
        assert!(output.success);
        assert_eq!(output.patterns_recommended, 1);
    }

    #[tokio::test]
    async fn test_record_feedback() {
        let memory = Arc::new(SelfLearningMemory::new());
        let tools = RecommendationFeedbackTools::new(memory);

        // First record a session
        let session_input = RecordRecommendationSessionInput {
            episode_id: Uuid::new_v4().to_string(),
            recommended_pattern_ids: vec!["p1".to_string()],
            recommended_playbook_ids: vec![],
        };
        let session_output = tools.record_session(session_input).await.unwrap();

        // Then record feedback
        let feedback_input = RecordRecommendationFeedbackInput {
            session_id: session_output.session_id,
            applied_pattern_ids: vec!["p1".to_string()],
            consulted_episode_ids: vec![],
            outcome: TaskOutcomeJson::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            },
            agent_rating: Some(0.9),
        };

        let output = tools.record_feedback(feedback_input).await.unwrap();
        assert!(output.success);
        assert_eq!(output.patterns_applied, 1);
    }
}
