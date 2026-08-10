//! Private persistence helper methods for `SelfLearningMemory`.
//!
//! These methods bridge the recommendation attribution system with durable
//! storage (Turso) and cache storage (redb), providing fallback chain logic
//! for loading sessions, feedback, and statistics.

use crate::memory::attribution::PersistenceReceipt;
use crate::memory::attribution::{
    RecommendationFeedback, RecommendationSession, RecommendationStats,
};

use tracing::warn;

use super::SelfLearningMemory;

impl SelfLearningMemory {
    pub(crate) async fn persist_recommendation_session(&self, session: &RecommendationSession) {
        if let Some(storage) = &self.turso_storage {
            if let Err(err) = storage.store_recommendation_session(session).await {
                warn!(
                    session_id = %session.session_id,
                    episode_id = %session.episode_id,
                    error = %err,
                    "Failed to persist recommendation session to durable storage"
                );
            }
        }

        if let Some(cache) = &self.cache_storage {
            if let Err(err) = cache.store_recommendation_session(session).await {
                warn!(
                    session_id = %session.session_id,
                    episode_id = %session.episode_id,
                    error = %err,
                    "Failed to persist recommendation session to cache storage"
                );
            }
        }
    }

    pub(crate) async fn persist_recommendation_feedback(&self, feedback: &RecommendationFeedback) {
        if let Some(storage) = &self.turso_storage {
            if let Err(err) = storage.store_recommendation_feedback(feedback).await {
                warn!(
                    session_id = %feedback.session_id,
                    error = %err,
                    "Failed to persist recommendation feedback to durable storage"
                );
            }
        }

        if let Some(cache) = &self.cache_storage {
            if let Err(err) = cache.store_recommendation_feedback(feedback).await {
                warn!(
                    session_id = %feedback.session_id,
                    error = %err,
                    "Failed to persist recommendation feedback to cache storage"
                );
            }
        }
    }

    pub(crate) async fn fetch_session_for_episode_from_storage(
        &self,
        episode_id: uuid::Uuid,
    ) -> Option<RecommendationSession> {
        if let Some(storage) = &self.turso_storage {
            match storage
                .get_recommendation_session_for_episode(episode_id)
                .await
            {
                Ok(Some(session)) => {
                    self.recommendation_tracker
                        .record_session(session.clone())
                        .await;
                    return Some(session);
                }
                Ok(None) => {}
                Err(err) => warn!(
                    episode_id = %episode_id,
                    error = %err,
                    "Failed to load recommendation session from durable storage"
                ),
            }
        }

        if let Some(cache) = &self.cache_storage {
            match cache
                .get_recommendation_session_for_episode(episode_id)
                .await
            {
                Ok(Some(session)) => {
                    self.recommendation_tracker
                        .record_session(session.clone())
                        .await;
                    return Some(session);
                }
                Ok(None) => {}
                Err(err) => warn!(
                    episode_id = %episode_id,
                    error = %err,
                    "Failed to load recommendation session from cache storage"
                ),
            }
        }

        None
    }

    /// Resolve a recommendation session by ID from durable storage (ADR-081 §1).
    ///
    /// Checks Turso then redb, hydrating the in-memory tracker on a hit so that
    /// subsequent lookups in this process are served locally. Returns `None` only
    /// when no configured backend holds the session.
    ///
    /// This is what makes feedback restart-safe: a session persisted before a
    /// restart is resolvable by a cold tracker.
    pub(crate) async fn fetch_session_by_id_from_storage(
        &self,
        session_id: uuid::Uuid,
    ) -> Option<RecommendationSession> {
        if let Some(storage) = &self.turso_storage {
            match storage.get_recommendation_session(session_id).await {
                Ok(Some(session)) => {
                    self.recommendation_tracker
                        .record_session(session.clone())
                        .await;
                    return Some(session);
                }
                Ok(None) => {}
                Err(err) => warn!(
                    session_id = %session_id,
                    error = %err,
                    "Failed to load recommendation session from durable storage"
                ),
            }
        }

        if let Some(cache) = &self.cache_storage {
            match cache.get_recommendation_session(session_id).await {
                Ok(Some(session)) => {
                    self.recommendation_tracker
                        .record_session(session.clone())
                        .await;
                    return Some(session);
                }
                Ok(None) => {}
                Err(err) => warn!(
                    session_id = %session_id,
                    error = %err,
                    "Failed to load recommendation session from cache storage"
                ),
            }
        }

        None
    }

    pub(crate) async fn fetch_feedback_from_storage(
        &self,
        session_id: uuid::Uuid,
    ) -> Option<RecommendationFeedback> {
        if let Some(storage) = &self.turso_storage {
            match storage.get_recommendation_feedback(session_id).await {
                Ok(Some(feedback)) => {
                    self.recommendation_tracker
                        .hydrate_feedback(feedback.clone())
                        .await;
                    return Some(feedback);
                }
                Ok(None) => {}
                Err(err) => warn!(
                    session_id = %session_id,
                    error = %err,
                    "Failed to load recommendation feedback from durable storage"
                ),
            }
        }

        if let Some(cache) = &self.cache_storage {
            match cache.get_recommendation_feedback(session_id).await {
                Ok(Some(feedback)) => {
                    self.recommendation_tracker
                        .hydrate_feedback(feedback.clone())
                        .await;
                    return Some(feedback);
                }
                Ok(None) => {}
                Err(err) => warn!(
                    session_id = %session_id,
                    error = %err,
                    "Failed to load recommendation feedback from cache storage"
                ),
            }
        }

        None
    }

    pub(crate) async fn fetch_recommendation_stats_from_storage(
        &self,
    ) -> Option<RecommendationStats> {
        if let Some(storage) = &self.turso_storage {
            match storage.get_recommendation_stats().await {
                Ok(stats) => return Some(stats),
                Err(err) => warn!(
                    error = %err,
                    "Failed to load recommendation stats from durable storage"
                ),
            }
        }

        if let Some(cache) = &self.cache_storage {
            match cache.get_recommendation_stats().await {
                Ok(stats) => return Some(stats),
                Err(err) => warn!(
                    error = %err,
                    "Failed to load recommendation stats from cache storage"
                ),
            }
        }

        None
    }

    /// Persist a recommendation session and return a truthful `PersistenceReceipt` (ADR-080 §3).
    ///
    /// Unlike `persist_recommendation_session`, this method tracks which
    /// backends advertise recommendation-attribution capability (ADR-081 §2),
    /// whether each write succeeded, and returns a machine-stable state that
    /// callers can use to determine whether feedback submitted after restart
    /// will find the session. Receipts count advertised-capable backends only:
    /// a configured backend that does not advertise capability is never
    /// counted as durable, so the receipt can never claim a write the backend
    /// cannot honor.
    pub(crate) async fn persist_session_checked(
        &self,
        session: &RecommendationSession,
    ) -> PersistenceReceipt {
        let session_id = session.session_id;
        let episode_id = session.episode_id;

        let turso_capable = self
            .turso_storage
            .as_ref()
            .is_some_and(|s| s.supports_recommendation_attribution());
        let redb_capable = self
            .cache_storage
            .as_ref()
            .is_some_and(|c| c.supports_recommendation_attribution());

        if !turso_capable && !redb_capable {
            return PersistenceReceipt::MemoryOnly {
                session_id,
                episode_id,
            };
        }

        let mut failed: Vec<String> = Vec::new();
        let mut succeeded: usize = 0;

        if turso_capable {
            if let Some(storage) = &self.turso_storage {
                match storage.store_recommendation_session(session).await {
                    Ok(()) => succeeded += 1,
                    Err(err) => {
                        warn!(
                            session_id = %session_id,
                            error = %err,
                            "Failed to persist session to Turso"
                        );
                        failed.push("turso".to_string());
                    }
                }
            }
        }

        if redb_capable {
            if let Some(cache) = &self.cache_storage {
                match cache.store_recommendation_session(session).await {
                    Ok(()) => succeeded += 1,
                    Err(err) => {
                        warn!(
                            session_id = %session_id,
                            error = %err,
                            "Failed to persist session to redb"
                        );
                        failed.push("redb".to_string());
                    }
                }
            }
        }

        let capable = usize::from(turso_capable) + usize::from(redb_capable);

        if succeeded == 0 {
            PersistenceReceipt::PersistenceFailed {
                session_id,
                episode_id,
                failed_backends: failed,
            }
        } else if succeeded < capable {
            PersistenceReceipt::PartiallyPersisted {
                session_id,
                episode_id,
                failed_backends: failed,
            }
        } else {
            PersistenceReceipt::Persisted {
                session_id,
                episode_id,
            }
        }
    }
}

impl SelfLearningMemory {
    /// Save the ANN index snapshot to the configured path.
    pub fn save_ann_snapshot(&self) -> crate::Result<()> {
        if let (Some(retriever), Some(path)) =
            (&self.semantic_retriever, &self.config.ann_index_path)
        {
            let index = retriever.vector_index.read();
            index.save(path)?;
        }
        Ok(())
    }
}
