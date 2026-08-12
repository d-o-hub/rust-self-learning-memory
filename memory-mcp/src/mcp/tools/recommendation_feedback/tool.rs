//! Recommendation Feedback Tool Implementation
//!
//! MCP tool for recording feedback about recommendation effectiveness.

use super::types::{
    RecommendationStatsOutput, RecordRecommendationFeedbackInput,
    RecordRecommendationFeedbackOutput, RecordRecommendationSessionInput,
    RecordRecommendationSessionOutput,
};
use crate::constants;
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
        mut input: RecordRecommendationSessionInput,
    ) -> Result<RecordRecommendationSessionOutput> {
        // Clamp array lengths (CWE-770)
        input
            .recommended_pattern_ids
            .truncate(constants::MAX_RECOMMENDED_IDS);
        input
            .recommended_playbook_ids
            .truncate(constants::MAX_RECOMMENDED_IDS);
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
        let receipt = self
            .memory
            .record_recommendation_session_checked(session)
            .await;

        let patterns_count = input.recommended_pattern_ids.len();
        let playbooks_count = input.recommended_playbook_ids.len();

        info!(
            session_id = %session_id,
            patterns = patterns_count,
            playbooks = playbooks_count,
            receipt_state = receipt_state_label(&receipt),
            "Recorded recommendation session"
        );

        Ok(RecordRecommendationSessionOutput {
            success: receipt.is_durable(),
            session_id: session_id.to_string(),
            episode_id: input.episode_id,
            patterns_recommended: patterns_count,
            playbooks_recommended: playbooks_count,
            message: format!(
                "Recorded recommendation session with {} patterns and {} playbooks (receipt state: {})",
                patterns_count,
                playbooks_count,
                receipt_state_label(&receipt)
            ),
            receipt,
        })
    }

    /// Record feedback about a recommendation session.
    ///
    /// Call this after an agent completes or abandons a task to indicate
    /// which recommendations were used and the outcome.
    #[instrument(skip(self, input), fields(session_id = %input.session_id))]
    pub async fn record_feedback(
        &self,
        mut input: RecordRecommendationFeedbackInput,
    ) -> Result<RecordRecommendationFeedbackOutput> {
        // Clamp array lengths and rating (CWE-770)
        input
            .applied_pattern_ids
            .truncate(constants::MAX_RECOMMENDED_IDS);
        input
            .consulted_episode_ids
            .truncate(constants::MAX_RECOMMENDED_IDS);
        if let Some(rating) = &mut input.agent_rating {
            *rating = rating.clamp(constants::MIN_AGENT_RATING, constants::MAX_AGENT_RATING);
        }

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
        let receipt = self
            .memory
            .record_recommendation_feedback_checked(feedback)
            .await?;

        let patterns_applied = input.applied_pattern_ids.len();
        let episodes_consulted = input.consulted_episode_ids.len();

        info!(
            session_id = %session_id,
            patterns_applied = patterns_applied,
            episodes_consulted = episodes_consulted,
            receipt_state = receipt_state_label(&receipt),
            "Recorded recommendation feedback"
        );

        Ok(RecordRecommendationFeedbackOutput {
            success: receipt.is_durable(),
            session_id: input.session_id,
            patterns_applied,
            episodes_consulted,
            message: format!(
                "Recorded feedback: {} patterns applied, {} episodes consulted (receipt state: {})",
                patterns_applied,
                episodes_consulted,
                receipt_state_label(&receipt)
            ),
            receipt,
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

/// Stable receipt state label matching the `PersistenceReceipt` JSON
/// discriminants (ADR-080 §3).
fn receipt_state_label(receipt: &do_memory_core::PersistenceReceipt) -> &'static str {
    match receipt {
        do_memory_core::PersistenceReceipt::Persisted { .. } => "persisted",
        do_memory_core::PersistenceReceipt::PartiallyPersisted { .. } => "partially_persisted",
        do_memory_core::PersistenceReceipt::MemoryOnly { .. } => "memory_only",
        do_memory_core::PersistenceReceipt::PersistenceFailed { .. } => "persistence_failed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::tools::recommendation_feedback::TaskOutcomeJson;
    use do_memory_core::MemoryConfig;

    #[tokio::test]
    async fn test_record_session_memory_only_receipt() {
        // No configured backends: the checked API returns a MemoryOnly receipt,
        // so success must be false (ADR-080 §3).
        let memory = Arc::new(SelfLearningMemory::new());
        let tools = RecommendationFeedbackTools::new(memory);

        let input = RecordRecommendationSessionInput {
            episode_id: Uuid::new_v4().to_string(),
            recommended_pattern_ids: vec!["p1".to_string()],
            recommended_playbook_ids: vec![],
        };

        let output = tools.record_session(input).await.unwrap();
        assert!(
            !output.success,
            "MemoryOnly receipt must not report success"
        );
        assert_eq!(output.success, output.receipt.is_durable());
        assert!(matches!(
            output.receipt,
            do_memory_core::PersistenceReceipt::MemoryOnly { .. }
        ));
        assert!(output.message.contains("memory_only"));
        assert_eq!(output.patterns_recommended, 1);
    }

    #[tokio::test]
    async fn test_record_session_persisted_with_redb() {
        // A capable (redb) backend yields a durable Persisted receipt.
        let temp_dir = tempfile::TempDir::new().unwrap();
        let redb_path = temp_dir.path().join("test_memory.redb");
        let redb: Arc<dyn do_memory_core::StorageBackend> = Arc::new(
            do_memory_storage_redb::RedbStorage::new(&redb_path)
                .await
                .unwrap(),
        );
        let memory = Arc::new(SelfLearningMemory::with_storage(
            MemoryConfig::default(),
            redb.clone(),
            redb,
        ));
        let tools = RecommendationFeedbackTools::new(memory);

        let input = RecordRecommendationSessionInput {
            episode_id: Uuid::new_v4().to_string(),
            recommended_pattern_ids: vec!["p1".to_string()],
            recommended_playbook_ids: vec![],
        };

        let output = tools.record_session(input).await.unwrap();
        assert!(output.success, "Persisted receipt must report success");
        assert_eq!(output.success, output.receipt.is_durable());
        assert!(matches!(
            output.receipt,
            do_memory_core::PersistenceReceipt::Persisted { .. }
        ));
        assert!(output.message.contains("persisted"));
        assert_eq!(output.patterns_recommended, 1);
    }

    #[tokio::test]
    async fn test_record_feedback_memory_only_receipt() {
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
        assert!(
            !output.success,
            "MemoryOnly receipt must not report success"
        );
        assert_eq!(output.success, output.receipt.is_durable());
        assert!(matches!(
            output.receipt,
            do_memory_core::PersistenceReceipt::MemoryOnly { .. }
        ));
        assert!(output.message.contains("memory_only"));
        assert_eq!(output.patterns_applied, 1);
    }

    #[tokio::test]
    async fn test_record_feedback_persisted_with_redb() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let redb_path = temp_dir.path().join("test_memory.redb");
        let redb: Arc<dyn do_memory_core::StorageBackend> = Arc::new(
            do_memory_storage_redb::RedbStorage::new(&redb_path)
                .await
                .unwrap(),
        );
        let memory = Arc::new(SelfLearningMemory::with_storage(
            MemoryConfig::default(),
            redb.clone(),
            redb,
        ));
        let tools = RecommendationFeedbackTools::new(memory);

        let session_input = RecordRecommendationSessionInput {
            episode_id: Uuid::new_v4().to_string(),
            recommended_pattern_ids: vec!["p1".to_string()],
            recommended_playbook_ids: vec![],
        };
        let session_output = tools.record_session(session_input).await.unwrap();
        assert!(session_output.success);

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
        assert!(output.success, "Persisted feedback must report success");
        assert_eq!(output.success, output.receipt.is_durable());
        assert!(matches!(
            output.receipt,
            do_memory_core::PersistenceReceipt::Persisted { .. }
        ));
        assert!(output.message.contains("persisted"));
        assert_eq!(output.patterns_applied, 1);
    }

    #[tokio::test]
    async fn test_record_session_truncates_large_arrays() {
        let memory = Arc::new(SelfLearningMemory::new());
        let tools = RecommendationFeedbackTools::new(memory);

        // Create arrays larger than MAX_RECOMMENDED_IDS
        let many_patterns: Vec<String> = (0..constants::MAX_RECOMMENDED_IDS + 50)
            .map(|i| format!("pattern-{}", i))
            .collect();
        let many_playbooks: Vec<String> = (0..constants::MAX_RECOMMENDED_IDS + 50)
            .map(|i| format!("playbook-{}", i))
            .collect();

        let input = RecordRecommendationSessionInput {
            episode_id: Uuid::new_v4().to_string(),
            recommended_pattern_ids: many_patterns,
            recommended_playbook_ids: many_playbooks,
        };

        let output = tools.record_session(input).await.unwrap();
        // MemoryOnly receipt: success must mirror the receipt, not the truncation.
        assert_eq!(output.success, output.receipt.is_durable());
        // Should have been truncated to MAX_RECOMMENDED_IDS
        assert_eq!(output.patterns_recommended, constants::MAX_RECOMMENDED_IDS);
        assert_eq!(output.playbooks_recommended, constants::MAX_RECOMMENDED_IDS);
    }

    #[tokio::test]
    async fn test_record_feedback_clamps_rating() {
        let memory = Arc::new(SelfLearningMemory::new());
        let tools = RecommendationFeedbackTools::new(memory);

        // First record a session. It must recommend every ID the feedback below
        // applies, or the ADR-080 §4 subset rule rejects the feedback before the
        // clamping this test actually exercises can be observed.
        let session_input = RecordRecommendationSessionInput {
            episode_id: Uuid::new_v4().to_string(),
            recommended_pattern_ids: (0..constants::MAX_RECOMMENDED_IDS + 10)
                .map(|i| format!("p{}", i))
                .collect(),
            recommended_playbook_ids: vec![],
        };
        let session_output = tools.record_session(session_input).await.unwrap();

        // Record feedback with out-of-range rating
        let feedback_input = RecordRecommendationFeedbackInput {
            session_id: session_output.session_id,
            applied_pattern_ids: (0..constants::MAX_RECOMMENDED_IDS + 10)
                .map(|i| format!("p{}", i))
                .collect(),
            consulted_episode_ids: (0..constants::MAX_RECOMMENDED_IDS + 10)
                .map(|i| format!("e{}", i))
                .collect(),
            outcome: TaskOutcomeJson::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            },
            agent_rating: Some(5.0), // Way above 1.0
        };

        let output = tools.record_feedback(feedback_input).await.unwrap();
        assert_eq!(output.success, output.receipt.is_durable());
        // Arrays should have been truncated
        assert_eq!(output.patterns_applied, constants::MAX_RECOMMENDED_IDS);
        assert_eq!(output.episodes_consulted, constants::MAX_RECOMMENDED_IDS);
    }
}
