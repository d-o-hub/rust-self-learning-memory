//! Feedback acceptance rules for recommendation attribution (ADR-080 §4).
//!
//! These are the in-memory integrity checks. Resolving a session from durable
//! storage before they run is the memory layer's responsibility — see
//! `SelfLearningMemory::record_recommendation_feedback` (ADR-081 §1).

use tracing::{info, instrument};

use super::RecommendationTracker;
use crate::error::Result;
use crate::memory::attribution::types::RecommendationFeedback;

impl RecommendationTracker {
    /// Record feedback for a recommendation session.
    ///
    /// Call this when an agent provides feedback about which recommendations
    /// were used and the outcome.
    ///
    /// # Arguments
    ///
    /// * `feedback` - The feedback to record
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the session exists and feedback is valid,
    /// or an error if the session is not found or applied IDs were not recommended.
    ///
    /// # Integrity (ADR-080 §4)
    ///
    /// - Unknown session IDs are rejected.
    /// - `applied_pattern_ids` must be a subset of the recommended IDs in the session.
    /// - Replacement feedback for an existing session is idempotent.
    ///
    /// This resolves the session from memory only. Callers that need
    /// restart-safe behavior must resolve from storage first (ADR-081 §1).
    #[instrument(skip(self, feedback), fields(session_id = %feedback.session_id))]
    pub async fn record_feedback(&self, feedback: RecommendationFeedback) -> Result<()> {
        let session_id = feedback.session_id;

        // ADR-080 §4: Resolve session before accepting feedback.
        let session = {
            let sessions = self.sessions.read().await;
            sessions.get(&session_id).cloned()
        };

        let Some(session) = session else {
            return Err(crate::error::Error::InvalidInput(format!(
                "Feedback references unknown session {session_id}; resolve the session before submitting feedback"
            )));
        };

        // ADR-080 §4: Reject applied pattern IDs not in the recommended set.
        let recommended_set: std::collections::HashSet<&str> = session
            .recommended_pattern_ids
            .iter()
            .map(String::as_str)
            .collect();

        for applied in &feedback.applied_pattern_ids {
            if !recommended_set.contains(applied.as_str()) {
                return Err(crate::error::Error::InvalidInput(format!(
                    "Applied pattern ID '{applied}' was not recommended in session {session_id}"
                )));
            }
        }

        // ADR-080 §4: Replacement feedback is idempotent (overwrite).
        {
            let mut feedback_map = self.feedback.write().await;
            feedback_map.insert(session_id, feedback);
        }

        info!(
            session_id = %session_id,
            "Recorded recommendation feedback"
        );

        Ok(())
    }

    /// Populate the in-memory feedback cache from durable storage (ADR-081 §1).
    ///
    /// This bypasses the [`record_feedback`](Self::record_feedback) integrity checks
    /// deliberately: stored feedback already passed them when it was first accepted,
    /// and the session it references may not be in this process's session map.
    /// Use [`record_feedback`](Self::record_feedback) for caller-submitted feedback.
    #[instrument(skip(self, feedback), fields(session_id = %feedback.session_id))]
    pub(crate) async fn hydrate_feedback(&self, feedback: RecommendationFeedback) {
        let mut feedback_map = self.feedback.write().await;
        feedback_map.insert(feedback.session_id, feedback);
    }
}
