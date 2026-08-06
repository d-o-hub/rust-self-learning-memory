//! Recommendation Session Tracker
//!
//! Tracks recommendation sessions and feedback to enable learning which
//! recommendations actually help agents succeed.
//!
//! Split across submodules to stay within the 500-LOC file gate:
//! - [`integrity`] — feedback acceptance rules (ADR-080 §4)
//! - [`stats`] — aggregate effectiveness statistics
//! - `tests` — unit coverage for all three

mod integrity;
mod stats;
#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};
use uuid::Uuid;

use super::types::{RecommendationFeedback, RecommendationSession, SessionWithFeedback};

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
/// use do_memory_core::memory::attribution::RecommendationTracker;
/// use do_memory_core::memory::attribution::RecommendationSession;
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
    /// Mapping from episode_id to list of session_ids (ADR-080 §4: multiple sessions per episode).
    episode_sessions: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>,
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

        // ADR-080 §4: multiple exposures for one episode create distinct sessions.
        // Indexing the same session twice (e.g. re-hydrating it from storage after a
        // cache miss) must not grow the list, or latest-lookup ordering degrades.
        {
            let mut episode_sessions = self.episode_sessions.write().await;
            let ids = episode_sessions.entry(episode_id).or_default();
            if !ids.contains(&session_id) {
                ids.push(session_id);
            }
        }

        info!(
            session_id = %session_id,
            episode_id = %episode_id,
            "Recorded recommendation session"
        );
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

    /// Get the latest session associated with an episode (ADR-080 §4: deterministic latest lookup).
    ///
    /// When an episode has multiple recommendation sessions, the one with the
    /// most recent timestamp is returned. Tie-breaking is by session_id (UUID
    /// byte order) for determinism beyond second-resolution timestamps.
    #[instrument(skip(self))]
    pub async fn get_session_for_episode(&self, episode_id: Uuid) -> Option<RecommendationSession> {
        let session_ids = {
            let episode_sessions = self.episode_sessions.read().await;
            episode_sessions.get(&episode_id).cloned()?
        };
        let sessions = self.sessions.read().await;
        session_ids
            .iter()
            .filter_map(|id| sessions.get(id))
            .max_by(|a, b| {
                a.timestamp
                    .cmp(&b.timestamp)
                    .then_with(|| a.session_id.as_bytes().cmp(b.session_id.as_bytes()))
            })
            .cloned()
    }

    /// Get all sessions associated with an episode (oldest first).
    #[instrument(skip(self))]
    pub async fn get_all_sessions_for_episode(
        &self,
        episode_id: Uuid,
    ) -> Vec<RecommendationSession> {
        let session_ids = {
            let episode_sessions = self.episode_sessions.read().await;
            match episode_sessions.get(&episode_id) {
                Some(ids) => ids.clone(),
                None => return vec![],
            }
        };
        let sessions = self.sessions.read().await;
        let mut result: Vec<RecommendationSession> = session_ids
            .iter()
            .filter_map(|id| sessions.get(id).cloned())
            .collect();
        result.sort_by(|a, b| {
            a.timestamp
                .cmp(&b.timestamp)
                .then_with(|| a.session_id.as_bytes().cmp(b.session_id.as_bytes()))
        });
        result
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
