//! Attributed playbook retrieval (ADR-080 §1–3).
//!
//! `AttributedPlaybookRequest` and `retrieve_playbooks_attributed` form the
//! attribution surface for playbook recommendations: a validated episode, a
//! fallible generation step, and a truthful `PersistenceReceipt`.

use crate::memory::attribution::{AttributedPlaybookResult, RecommendationSession};
use crate::memory::playbook::RecommendedPlaybook;
use crate::types::TaskContext;
use crate::types::TaskType;
use tracing::instrument;
use uuid::Uuid;

use super::super::SelfLearningMemory;

/// Request for attributed playbook retrieval (ADR-080 §1–3).
///
/// The `episode_id` must be non-nil and must reference an existing episode;
/// it is validated before any generation work so a nonexistent episode never
/// creates a recommendation session.
#[derive(Debug, Clone)]
pub struct AttributedPlaybookRequest {
    /// The episode the playbook recommendation targets.
    pub episode_id: Uuid,
    /// Description of the task to perform.
    pub task_description: String,
    /// Domain of the task (e.g., "web-api", "testing").
    pub domain: String,
    /// Type of task (CodeGeneration, Debugging, etc.).
    pub task_type: TaskType,
    /// Additional task context.
    pub context: TaskContext,
    /// Maximum number of playbooks to return.
    pub max_playbooks: usize,
    /// Maximum steps per playbook.
    pub max_steps_per_playbook: usize,
}

impl SelfLearningMemory {
    /// Retrieve actionable playbooks and create an attributed recommendation session (ADR-080 §1–3).
    ///
    /// Requires a valid, non-nil `episode_id` that exists. The episode is
    /// validated before any generation work, so a nonexistent episode never
    /// creates a session; a generation failure is returned as an error without
    /// recording a session; a successful empty generation still records an
    /// empty attributed session. Returns playbooks together with a session and
    /// a `PersistenceReceipt` describing durability state.
    #[instrument(skip(self, request))]
    pub async fn retrieve_playbooks_attributed(
        &self,
        request: AttributedPlaybookRequest,
    ) -> crate::error::Result<AttributedPlaybookResult<RecommendedPlaybook>> {
        if request.episode_id.is_nil() {
            return Err(crate::error::Error::InvalidInput(
                "Attributed playbook retrieval requires a non-nil episode ID".to_string(),
            ));
        }

        // ADR-080 §1: a nonexistent episode must never create a session, so the
        // episode is validated before any recommendation generation.
        self.validate_attributed_episode(request.episode_id, "retrieve_playbooks_attributed")
            .await?;

        let playbooks = self
            .try_retrieve_playbooks(
                &request.task_description,
                &request.domain,
                request.task_type,
                request.context.clone(),
                request.max_playbooks,
                request.max_steps_per_playbook,
            )
            .await?;

        let mut recommended_pattern_ids = Vec::new();
        let mut recommended_playbook_ids = Vec::new();

        for pb in &playbooks {
            recommended_playbook_ids.push(pb.playbook_id);
            for pid in &pb.supporting_pattern_ids {
                let pid_str = pid.to_string();
                if !recommended_pattern_ids.contains(&pid_str) {
                    recommended_pattern_ids.push(pid_str);
                }
            }
        }

        let session = RecommendationSession {
            session_id: Uuid::new_v4(),
            episode_id: request.episode_id,
            timestamp: chrono::Utc::now(),
            recommended_pattern_ids,
            recommended_playbook_ids,
        };

        // ADR-080 §3: a successful empty generation still creates an empty session.
        let receipt = self
            .record_recommendation_session_checked(session.clone())
            .await;

        Ok(AttributedPlaybookResult {
            playbooks,
            session,
            receipt,
        })
    }
}
