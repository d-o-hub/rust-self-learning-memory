//! Feedback-to-ranking index integration for `SelfLearningMemory` (ADR-082).
//!
//! Lazy-loads the derived `RankingIndex` from in-process tracker data plus the
//! durable history of capable storage backends, and refreshes it when feedback
//! is recorded. A single accepted replacement is applied incrementally (only the
//! changed session's contribution is recomputed); the canonical full rebuild is
//! used for the cold start, as the fallback when the incremental path cannot
//! prove it is equivalent, and whenever an index mutation races a rebuild's
//! history scan. The index is a deterministic reduction of (in-process tracker ∪
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

    /// Take an owned snapshot of the derived ranking index (read-then-release).
    ///
    /// The guard is confined to this call, so it never crosses an `.await` and
    /// cannot starve a concurrent [`refresh_ranking_index`](Self::refresh_ranking_index)
    /// writer (AGENTS.md: no locks held across `.await`). Callers that need
    /// learned weights around an await point must use this instead of holding
    /// `ranking_index.read()` themselves. Only the learned weights are copied
    /// (`RankingIndex::read_view`), so the snapshot is proportional to the
    /// pattern count — not the session history — and cheap relative to the
    /// provider/storage work a recommendation call does afterwards.
    pub(crate) async fn ranking_index_snapshot(&self) -> RankingIndex {
        // The read guard is dropped at the end of this statement, before the
        // temporary is returned to the caller.
        self.ranking_index.read().await.read_view()
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
    ///
    /// History is gathered without holding `ranking_index` (AGENTS.md: no lock
    /// across `.await`), so an incremental
    /// [`Self::apply_recommendation_feedback`] update can land during the scan.
    /// The rebuild therefore only overwrites the index while its mutation
    /// counter is unchanged, re-gathering while it moves: a stale snapshot never
    /// clobbers a concurrent contribution.
    pub(crate) async fn refresh_ranking_index(&self) {
        // Bounded retry: every attempt re-reads the tracker (the authority for
        // concurrent feedback), so convergence is immediate once writers settle.
        const MAX_ATTEMPTS: usize = 8;
        for _ in 0..MAX_ATTEMPTS {
            let base = self.ranking_index.read().await.revision();
            let (sessions, feedback) = self.collect_ranking_history().await;
            let rebuilt = RankingIndex::from_history(&sessions, &feedback);
            let mut slot = self.ranking_index.write().await;
            if slot.revision() == base {
                slot.replace_with(rebuilt);
                return;
            }
        }

        warn!("ranking: index mutated during rebuild; writing last gathered snapshot");
        let (sessions, feedback) = self.collect_ranking_history().await;
        let rebuilt = RankingIndex::from_history(&sessions, &feedback);
        self.ranking_index.write().await.replace_with(rebuilt);
    }

    /// Incrementally apply one accepted feedback record to `ranking_index`.
    ///
    /// Recomputes only the changed session's contribution (ADR-082), so refresh
    /// cost is independent of total history size. Falls back to the canonical
    /// [`refresh_ranking_index`](Self::refresh_ranking_index) rebuild when the
    /// session cannot be resolved (orphan path) or the index reports an
    /// inconsistent prior contribution.
    pub(crate) async fn apply_recommendation_feedback(&self, feedback: &RecommendationFeedback) {
        // Also loads the index on first use, which already includes `feedback`
        // (the tracker is written before this call), so the incremental update
        // below is then an exact replacement.
        self.ensure_ranking_index_loaded().await;

        // The tracker is authoritative and rejects unknown sessions before we
        // get here, so a resolvable session is the normal case.
        let session_known = self
            .recommendation_tracker
            .get_session(feedback.session_id)
            .await
            .is_some();

        if session_known {
            let mut index = self.ranking_index.write().await;
            if index.apply_feedback(feedback) {
                return;
            }
        }

        warn!(
            session_id = %feedback.session_id,
            "ranking: incremental update unavailable; rebuilding index"
        );
        self.refresh_ranking_index().await;
    }

    /// Gather the merged (capable durable backends ∪ in-process tracker)
    /// recommendation history for a rebuild.
    async fn collect_ranking_history(
        &self,
    ) -> (Vec<RecommendationSession>, Vec<RecommendationFeedback>) {
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

        (
            sessions.into_values().collect(),
            feedback.into_values().collect(),
        )
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
