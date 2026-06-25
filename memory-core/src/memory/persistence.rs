//! Private persistence helper methods for `SelfLearningMemory`.
//!
//! These methods bridge the recommendation attribution system with durable
//! storage (Turso) and cache storage (redb), providing fallback chain logic
//! for loading sessions, feedback, and statistics.

use crate::memory::attribution::{
    RecommendationFeedback, RecommendationSession, RecommendationStats,
};
use tracing::warn;

use super::SelfLearningMemory;

impl SelfLearningMemory {
    pub(crate) async fn persist_recommendation_session(&self, session: &RecommendationSession) {
        if let Some(storage) = &self.turso_storage
            && let Err(err) = storage.store_recommendation_session(session).await
        {
            warn!(
                session_id = %session.session_id,
                episode_id = %session.episode_id,
                error = %err,
                "Failed to persist recommendation session to durable storage"
            );
        }

        if let Some(cache) = &self.cache_storage
            && let Err(err) = cache.store_recommendation_session(session).await
        {
            warn!(
                session_id = %session.session_id,
                episode_id = %session.episode_id,
                error = %err,
                "Failed to persist recommendation session to cache storage"
            );
        }
    }

    pub(crate) async fn persist_recommendation_feedback(&self, feedback: &RecommendationFeedback) {
        if let Some(storage) = &self.turso_storage
            && let Err(err) = storage.store_recommendation_feedback(feedback).await
        {
            warn!(
                session_id = %feedback.session_id,
                error = %err,
                "Failed to persist recommendation feedback to durable storage"
            );
        }

        if let Some(cache) = &self.cache_storage
            && let Err(err) = cache.store_recommendation_feedback(feedback).await
        {
            warn!(
                session_id = %feedback.session_id,
                error = %err,
                "Failed to persist recommendation feedback to cache storage"
            );
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

    pub(crate) async fn fetch_feedback_from_storage(
        &self,
        session_id: uuid::Uuid,
    ) -> Option<RecommendationFeedback> {
        if let Some(storage) = &self.turso_storage {
            match storage.get_recommendation_feedback(session_id).await {
                Ok(Some(feedback)) => {
                    if let Err(err) = self
                        .recommendation_tracker
                        .record_feedback(feedback.clone())
                        .await
                    {
                        warn!(
                            session_id = %session_id,
                            error = %err,
                            "Failed to cache recommendation feedback after durable load"
                        );
                    }
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
                    if let Err(err) = self
                        .recommendation_tracker
                        .record_feedback(feedback.clone())
                        .await
                    {
                        warn!(
                            session_id = %session_id,
                            error = %err,
                            "Failed to cache recommendation feedback after cache load"
                        );
                    }
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
}
