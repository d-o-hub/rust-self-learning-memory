//! Feedback-to-ranking index integration for `SelfLearningMemory` (ADR-082).
//!
//! Lazy-loads the derived `RankingIndex` from in-process tracker data plus the
//! durable history of capable storage backends, and refreshes it when feedback
//! is recorded. The index is a deterministic reduction of (in-process tracker ∪
//! capability-gated durable history), converging to a pure function of durable
//! history after a cold restart; a failed or partial refresh degrades safely:
//! errors are logged and the derived state is rebuilt (rollback-safe) on the
//! next refresh.

use std::collections::HashMap;
use std::sync::atomic::Ordering;

use tracing::warn;

use super::SelfLearningMemory;
use crate::StorageBackend;
use crate::memory::attribution::{RankingIndex, RecommendationFeedback, RecommendationSession};

impl SelfLearningMemory {
    /// Ensure `ranking_index` has been loaded from durable history at least once.
    pub(crate) async fn ensure_ranking_index_loaded(&self) {
        if self.ranking_loaded.load(Ordering::Acquire) {
            return;
        }
        self.refresh_ranking_index().await;
        self.ranking_loaded.store(true, Ordering::Release);
    }

    /// Rebuild `ranking_index` from the in-process tracker and every capable
    /// durable backend's recommendation history.
    ///
    /// Merge order: capable durable backends are loaded first, then the
    /// in-process tracker overwrites them (a `HashMap` insert is last-write-
    /// wins). The tracker is updated *before* persistence, so it is never older
    /// than a durable row for the same session; preferring it guarantees
    /// "latest feedback wins" even when persisting the newest record fails and
    /// a stale durable row remains. After a cold restart the tracker is empty,
    /// so the index is a pure function of capability-gated durable history.
    pub(crate) async fn refresh_ranking_index(&self) {
        let mut sessions: HashMap<uuid::Uuid, RecommendationSession> = HashMap::new();
        let mut feedback: HashMap<uuid::Uuid, RecommendationFeedback> = HashMap::new();

        // Durable capable backends (Turso then cache/redb); errors → warn! and
        // continue (derived state is rollback-safe and rebuilt on refresh).
        if let Some(t) = &self.turso_storage {
            merge_backend_ranking_history(t.as_ref(), &mut sessions, &mut feedback).await;
        }
        if let Some(c) = &self.cache_storage {
            merge_backend_ranking_history(c.as_ref(), &mut sessions, &mut feedback).await;
        }

        // In-process tracker last (authoritative — see doc comment).
        for s in self.recommendation_tracker.get_all_sessions().await {
            sessions.insert(s.session_id, s);
        }
        for f in self.recommendation_tracker.get_all_feedback().await {
            feedback.insert(f.session_id, f);
        }

        let sess: Vec<_> = sessions.into_values().collect();
        let fb: Vec<_> = feedback.into_values().collect();
        let index = RankingIndex::from_history(&sess, &fb);
        *self.ranking_index.write().await = index;
    }
}

/// Best-effort merge of one durable backend's recommendation history into the
/// in-process maps. Non-capable backends and list failures contribute nothing
/// (failures are logged), so the derived index stays deterministic.
async fn merge_backend_ranking_history(
    backend: &dyn StorageBackend,
    sessions: &mut HashMap<uuid::Uuid, RecommendationSession>,
    feedback: &mut HashMap<uuid::Uuid, RecommendationFeedback>,
) {
    if !backend.supports_ranking_adaptation() {
        return;
    }
    match backend.list_recommendation_sessions().await {
        Ok(vs) => {
            sessions.extend(vs.into_iter().map(|s| (s.session_id, s)));
        }
        Err(e) => warn!(error = %e, "ranking: sessions list failed"),
    }
    match backend.list_recommendation_feedback().await {
        Ok(vs) => {
            feedback.extend(vs.into_iter().map(|f| (f.session_id, f)));
        }
        Err(e) => warn!(error = %e, "ranking: feedback list failed"),
    }
}
