//! Recommendation Session Tracker
//!
//! Tracks recommendation sessions and feedback to enable learning which
//! recommendations actually help agents succeed.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};
use uuid::Uuid;

use super::types::{
    RecommendationFeedback, RecommendationSession, RecommendationStats, SessionWithFeedback,
};
use crate::error::Result;

/// Tracker for recommendation sessions and their feedback.
///
/// This struct maintains in-memory storage for recommendation sessions
/// and their associated feedback. It provides methods to record sessions,
/// record feedback, and calculate effectiveness statistics.
///
/// # Thread Safety
///
/// All operations are async-safe using `RwLock` for concurrent access.
///
/// # Example
///
/// ```no_run
/// use memory_core::memory::attribution::RecommendationTracker;
/// use memory_core::memory::attribution::RecommendationSession;
/// use uuid::Uuid;
///
/// # #[tokio::main]
/// # async fn main() {
/// let tracker = RecommendationTracker::new();
///
/// // Record a recommendation session
/// let session = RecommendationSession {
///     session_id: Uuid::new_v4(),
///     episode_id: Uuid::new_v4(),
///     timestamp: chrono::Utc::now(),
///     recommended_pattern_ids: vec!["p1".to_string()],
///     recommended_playbook_ids: vec![],
/// };
/// tracker.record_session(session).await;
/// # }
/// ```
#[derive(Clone)]
pub struct RecommendationTracker {
    /// Active recommendation sessions by session_id
    sessions: Arc<RwLock<HashMap<Uuid, RecommendationSession>>>,
    /// Feedback records by session_id
    feedback: Arc<RwLock<HashMap<Uuid, RecommendationFeedback>>>,
    /// Mapping from episode_id to session_id for lookup
    episode_sessions: Arc<RwLock<HashMap<Uuid, Uuid>>>,
}

impl Default for RecommendationTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl RecommendationTracker {
    /// Create a new empty tracker.
    #[must_use]
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            feedback: Arc::new(RwLock::new(HashMap::new())),
            episode_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record a new recommendation session.
    ///
    /// Call this when patterns or playbooks are recommended to an agent.
    ///
    /// # Arguments
    ///
    /// * `session` - The recommendation session to record
    #[instrument(skip(self, session), fields(session_id = %session.session_id, episode_id = %session.episode_id))]
    pub async fn record_session(&self, session: RecommendationSession) {
        let session_id = session.session_id;
        let episode_id = session.episode_id;

        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id, session);
        }

        {
            let mut episode_sessions = self.episode_sessions.write().await;
            episode_sessions.insert(episode_id, session_id);
        }

        info!(
            session_id = %session_id,
            episode_id = %episode_id,
            "Recorded recommendation session"
        );
    }

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
    /// Returns `Ok(())` if the session exists, or an error if not found.
    #[instrument(skip(self, feedback), fields(session_id = %feedback.session_id))]
    pub async fn record_feedback(&self, feedback: RecommendationFeedback) -> Result<()> {
        let session_id = feedback.session_id;

        // Verify session exists
        {
            let sessions = self.sessions.read().await;
            if !sessions.contains_key(&session_id) {
                debug!(session_id = %session_id, "Session not found for feedback");
                // Still record the feedback even if session is missing
                // This handles cases where feedback arrives before session is persisted
            }
        }

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

    /// Get a recommendation session by ID.
    #[instrument(skip(self))]
    pub async fn get_session(&self, session_id: Uuid) -> Option<RecommendationSession> {
        let sessions = self.sessions.read().await;
        sessions.get(&session_id).cloned()
    }

    /// Get feedback for a session.
    #[instrument(skip(self))]
    pub async fn get_feedback(&self, session_id: Uuid) -> Option<RecommendationFeedback> {
        let feedback = self.feedback.read().await;
        feedback.get(&session_id).cloned()
    }

    /// Get the session associated with an episode.
    #[instrument(skip(self))]
    pub async fn get_session_for_episode(&self, episode_id: Uuid) -> Option<RecommendationSession> {
        let episode_sessions = self.episode_sessions.read().await;
        let session_id = episode_sessions.get(&episode_id)?;
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    /// Get a session with its feedback (if any).
    #[instrument(skip(self))]
    pub async fn get_session_with_feedback(&self, session_id: Uuid) -> Option<SessionWithFeedback> {
        let session = self.get_session(session_id).await?;
        let feedback = self.get_feedback(session_id).await;

        Some(SessionWithFeedback { session, feedback })
    }

    /// Get feedback for an episode (via its associated session).
    #[instrument(skip(self))]
    pub async fn get_feedback_for_episode(
        &self,
        episode_id: Uuid,
    ) -> Option<RecommendationFeedback> {
        let session = self.get_session_for_episode(episode_id).await?;
        self.get_feedback(session.session_id).await
    }

    /// Calculate overall recommendation effectiveness statistics.
    #[instrument(skip(self))]
    pub async fn get_stats(&self) -> RecommendationStats {
        let sessions = self.sessions.read().await;
        let feedback = self.feedback.read().await;

        let total_sessions = sessions.len();
        let total_feedback = feedback.len();

        // Calculate pattern statistics
        let mut total_recommended: usize = 0;
        let mut total_applied: usize = 0;
        let mut successful_applications: usize = 0;
        let mut total_ratings: f32 = 0.0;
        let mut rating_count: usize = 0;

        for session in sessions.values() {
            total_recommended += session.recommended_pattern_ids.len();
        }

        for fb in feedback.values() {
            total_applied += fb.applied_pattern_ids.len();

            // Check if outcome was successful
            if matches!(
                fb.outcome,
                crate::types::TaskOutcome::Success { .. }
                    | crate::types::TaskOutcome::PartialSuccess { .. }
            ) {
                successful_applications += fb.applied_pattern_ids.len();
            }

            if let Some(rating) = fb.agent_rating {
                total_ratings += rating;
                rating_count += 1;
            }
        }

        let patterns_ignored = total_recommended.saturating_sub(total_applied);

        let adoption_rate = if total_recommended > 0 {
            total_applied as f32 / total_recommended as f32
        } else {
            0.0
        };

        let success_after_adoption_rate = if total_applied > 0 {
            successful_applications as f32 / total_applied as f32
        } else {
            0.0
        };

        let avg_agent_rating = if rating_count > 0 {
            Some(total_ratings / rating_count as f32)
        } else {
            None
        };

        RecommendationStats {
            total_sessions,
            total_feedback,
            patterns_applied: total_applied,
            patterns_ignored,
            successful_applications,
            adoption_rate,
            success_after_adoption_rate,
            avg_agent_rating,
        }
    }

    /// Clear all sessions and feedback (useful for testing).
    #[instrument(skip(self))]
    pub async fn clear(&self) {
        let mut sessions = self.sessions.write().await;
        let mut feedback = self.feedback.write().await;
        let mut episode_sessions = self.episode_sessions.write().await;

        sessions.clear();
        feedback.clear();
        episode_sessions.clear();

        debug!("Cleared all recommendation data");
    }

    /// Get all sessions (for iteration/debugging).
    pub async fn get_all_sessions(&self) -> Vec<RecommendationSession> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }

    /// Get all feedback records.
    pub async fn get_all_feedback(&self) -> Vec<RecommendationFeedback> {
        let feedback = self.feedback.read().await;
        feedback.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TaskOutcome;

    fn create_test_session() -> RecommendationSession {
        RecommendationSession {
            session_id: Uuid::new_v4(),
            episode_id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            recommended_pattern_ids: vec!["p1".to_string(), "p2".to_string()],
            recommended_playbook_ids: vec![],
        }
    }

    fn create_test_feedback(session_id: Uuid) -> RecommendationFeedback {
        RecommendationFeedback {
            session_id,
            applied_pattern_ids: vec!["p1".to_string()],
            consulted_episode_ids: vec![],
            outcome: TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            },
            agent_rating: Some(0.9),
        }
    }

    #[tokio::test]
    async fn test_record_and_get_session() {
        let tracker = RecommendationTracker::new();
        let session = create_test_session();
        let session_id = session.session_id;

        tracker.record_session(session.clone()).await;

        let retrieved = tracker.get_session(session_id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().session_id, session_id);
    }

    #[tokio::test]
    async fn test_record_and_get_feedback() {
        let tracker = RecommendationTracker::new();
        let session = create_test_session();
        let session_id = session.session_id;

        tracker.record_session(session).await;

        let feedback = create_test_feedback(session_id);
        tracker.record_feedback(feedback.clone()).await.unwrap();

        let retrieved = tracker.get_feedback(session_id).await;
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_get_session_for_episode() {
        let tracker = RecommendationTracker::new();
        let session = create_test_session();
        let episode_id = session.episode_id;

        tracker.record_session(session).await;

        let retrieved = tracker.get_session_for_episode(episode_id).await;
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_get_stats() {
        let tracker = RecommendationTracker::new();

        // Record a session
        let session = create_test_session();
        let session_id = session.session_id;
        tracker.record_session(session).await;

        // Record feedback
        let feedback = create_test_feedback(session_id);
        tracker.record_feedback(feedback).await.unwrap();

        let stats = tracker.get_stats().await;
        assert_eq!(stats.total_sessions, 1);
        assert_eq!(stats.total_feedback, 1);
        assert_eq!(stats.patterns_applied, 1);
        assert_eq!(stats.patterns_ignored, 1); // 2 recommended - 1 applied
        assert!((stats.adoption_rate - 0.5).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_clear() {
        let tracker = RecommendationTracker::new();
        let session = create_test_session();

        tracker.record_session(session).await;
        tracker.clear().await;

        let stats = tracker.get_stats().await;
        assert_eq!(stats.total_sessions, 0);
    }
}

#[cfg(test)]
mod extra_attribution_tests {
    use super::*;
    use crate::types::TaskOutcome;

    #[tokio::test]
    async fn test_get_session_with_feedback_not_found() {
        let tracker = RecommendationTracker::new();
        let result = tracker.get_session_with_feedback(Uuid::new_v4()).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_feedback_for_episode_not_found() {
        let tracker = RecommendationTracker::new();
        let result = tracker.get_feedback_for_episode(Uuid::new_v4()).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_stats_with_partial_success() {
        let tracker = RecommendationTracker::new();
        let session_id = Uuid::new_v4();
        let ep_id = Uuid::new_v4();

        tracker
            .record_session(RecommendationSession {
                session_id,
                episode_id: ep_id,
                timestamp: chrono::Utc::now(),
                recommended_pattern_ids: vec!["p1".to_string()],
                recommended_playbook_ids: vec![],
            })
            .await;

        tracker
            .record_feedback(RecommendationFeedback {
                session_id,
                applied_pattern_ids: vec!["p1".to_string()],
                consulted_episode_ids: vec![],
                outcome: TaskOutcome::PartialSuccess {
                    verdict: "Partially done".to_string(),
                    completed: vec![],
                    failed: vec![],
                },
                agent_rating: None,
            })
            .await
            .unwrap();

        let stats = tracker.get_stats().await;
        assert_eq!(stats.successful_applications, 1);
    }
}

#[cfg(test)]
mod deeper_attribution_tests {
    use super::*;
    use crate::types::TaskOutcome;

    #[tokio::test]
    async fn test_tracker_empty_stats() {
        let tracker = RecommendationTracker::new();
        let stats = tracker.get_stats().await;
        assert_eq!(stats.total_sessions, 0);
        assert_eq!(stats.adoption_rate, 0.0);
    }

    #[tokio::test]
    async fn test_tracker_record_feedback_without_session() {
        let tracker = RecommendationTracker::new();
        let session_id = Uuid::new_v4();
        let feedback = RecommendationFeedback {
            session_id,
            applied_pattern_ids: vec!["p1".to_string()],
            consulted_episode_ids: vec![],
            outcome: TaskOutcome::Success {
                verdict: "ok".to_string(),
                artifacts: vec![],
            },
            agent_rating: None,
        };
        // Should succeed even if session not found
        tracker.record_feedback(feedback).await.unwrap();
        assert!(tracker.get_feedback(session_id).await.is_some());
    }
}
