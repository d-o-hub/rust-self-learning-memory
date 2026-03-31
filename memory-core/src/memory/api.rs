//! Public API methods for `SelfLearningMemory`
//!
//! This module contains all public methods that users interact with,
//! organized by functionality area.

use crate::error::Result;
use crate::memory::attribution::{
    RecommendationFeedback, RecommendationSession, RecommendationStats,
};
use crate::monitoring::AgentMetrics;
use std::time::Duration;
use tracing::warn;

use super::SelfLearningMemory;

// ============================================================================
// Monitoring and Statistics
// ============================================================================

impl SelfLearningMemory {
    /// Get statistics about the memory system
    pub async fn get_stats(&self) -> (usize, usize, usize) {
        super::monitoring::get_stats(&self.episodes_fallback, &self.patterns_fallback).await
    }

    /// Record an agent execution for monitoring
    pub async fn record_agent_execution(
        &self,
        agent_name: &str,
        success: bool,
        duration: Duration,
    ) -> Result<()> {
        super::monitoring::record_agent_execution(
            &self.agent_monitor,
            agent_name,
            success,
            duration,
        )
        .await
    }

    /// Record detailed agent execution information
    pub async fn record_agent_execution_detailed(
        &self,
        agent_name: &str,
        success: bool,
        duration: Duration,
        task_description: Option<String>,
        error_message: Option<String>,
    ) -> Result<()> {
        super::monitoring::record_agent_execution_detailed(
            &self.agent_monitor,
            agent_name,
            success,
            duration,
            task_description,
            error_message,
        )
        .await
    }

    /// Get performance metrics for a specific agent
    pub async fn get_agent_metrics(&self, agent_name: &str) -> Option<AgentMetrics> {
        super::monitoring::get_agent_metrics(&self.agent_monitor, agent_name).await
    }

    /// Get metrics for all tracked agents
    pub async fn get_all_agent_metrics(&self) -> std::collections::HashMap<String, AgentMetrics> {
        super::monitoring::get_all_agent_metrics(&self.agent_monitor).await
    }

    /// Get monitoring system summary statistics
    pub async fn get_monitoring_summary(&self) -> crate::monitoring::MonitoringSummary {
        super::monitoring::get_monitoring_summary(&self.agent_monitor).await
    }

    /// Get query cache metrics (v0.1.12)
    #[must_use]
    pub fn get_cache_metrics(&self) -> crate::retrieval::CacheMetrics {
        super::monitoring::get_cache_metrics(&self.query_cache)
    }

    /// Clear query cache metrics (v0.1.12)
    pub fn clear_cache_metrics(&self) {
        super::monitoring::clear_cache_metrics(&self.query_cache);
    }

    /// Clear all cached query results (v0.1.12)
    pub fn clear_cache(&self) {
        super::monitoring::clear_cache(&self.query_cache);
    }
}

impl SelfLearningMemory {
    async fn persist_recommendation_session(&self, session: &RecommendationSession) {
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

    async fn persist_recommendation_feedback(&self, feedback: &RecommendationFeedback) {
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

    async fn fetch_session_for_episode_from_storage(
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

    async fn fetch_feedback_from_storage(
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

    async fn fetch_recommendation_stats_from_storage(&self) -> Option<RecommendationStats> {
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

// ============================================================================
// Recommendation Attribution (ADR-044 Feature 2)
// ============================================================================

impl SelfLearningMemory {
    /// Record a recommendation session when patterns/playbooks are suggested.
    ///
    /// Call this when the system recommends patterns or playbooks to an agent.
    /// This creates a record that can later be correlated with feedback.
    ///
    /// # Arguments
    ///
    /// * `session` - The recommendation session to record
    ///
    /// # Example
    ///
    /// ```no_run
    /// use do_memory_core::SelfLearningMemory;
    /// use do_memory_core::memory::attribution::RecommendationSession;
    /// use uuid::Uuid;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let memory = SelfLearningMemory::new();
    ///
    /// let session = RecommendationSession {
    ///     session_id: Uuid::new_v4(),
    ///     episode_id: Uuid::new_v4(),
    ///     timestamp: chrono::Utc::now(),
    ///     recommended_pattern_ids: vec!["pattern-123".to_string()],
    ///     recommended_playbook_ids: vec![],
    /// };
    ///
    /// memory.record_recommendation_session(session).await;
    /// # }
    /// ```
    pub async fn record_recommendation_session(&self, session: RecommendationSession) {
        self.recommendation_tracker
            .record_session(session.clone())
            .await;
        self.persist_recommendation_session(&session).await;
    }

    /// Record feedback about a recommendation session.
    ///
    /// Call this after an agent completes or abandons a task to indicate
    /// which recommendations were used and the outcome.
    ///
    /// # Arguments
    ///
    /// * `feedback` - The feedback to record
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the session doesn't exist.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use do_memory_core::SelfLearningMemory;
    /// use do_memory_core::memory::attribution::RecommendationFeedback;
    /// use do_memory_core::TaskOutcome;
    /// use uuid::Uuid;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let memory = SelfLearningMemory::new();
    ///
    /// let feedback = RecommendationFeedback {
    ///     session_id: Uuid::new_v4(),
    ///     applied_pattern_ids: vec!["pattern-123".to_string()],
    ///     consulted_episode_ids: vec![],
    ///     outcome: TaskOutcome::Success {
    ///         verdict: "Done".to_string(),
    ///         artifacts: vec![],
    ///     },
    ///     agent_rating: Some(0.9),
    /// };
    ///
    /// memory.record_recommendation_feedback(feedback).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn record_recommendation_feedback(
        &self,
        feedback: RecommendationFeedback,
    ) -> Result<()> {
        self.recommendation_tracker
            .record_feedback(feedback.clone())
            .await?;
        self.persist_recommendation_feedback(&feedback).await;
        Ok(())
    }

    /// Get the recommendation session for an episode.
    ///
    /// Returns the session that was recorded when recommendations were made
    /// for the specified episode.
    pub async fn get_recommendation_session_for_episode(
        &self,
        episode_id: uuid::Uuid,
    ) -> Option<RecommendationSession> {
        if let Some(session) = self
            .recommendation_tracker
            .get_session_for_episode(episode_id)
            .await
        {
            return Some(session);
        }

        self.fetch_session_for_episode_from_storage(episode_id)
            .await
    }

    /// Get feedback for a recommendation session.
    pub async fn get_recommendation_feedback(
        &self,
        session_id: uuid::Uuid,
    ) -> Option<RecommendationFeedback> {
        if let Some(feedback) = self.recommendation_tracker.get_feedback(session_id).await {
            return Some(feedback);
        }

        self.fetch_feedback_from_storage(session_id).await
    }

    /// Get overall recommendation effectiveness statistics.
    ///
    /// Returns aggregate metrics about recommendation adoption rates,
    /// success rates, and agent ratings.
    pub async fn get_recommendation_stats(&self) -> RecommendationStats {
        if let Some(stats) = self.fetch_recommendation_stats_from_storage().await {
            return stats;
        }

        self.recommendation_tracker.get_stats().await
    }
}
