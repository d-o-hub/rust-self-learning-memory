//! Public API methods for `SelfLearningMemory`
//!
//! This module contains all public methods that users interact with,
//! organized by functionality area.

use super::SelfLearningMemory;
use crate::error::Result;
use crate::memory::attribution::{
    PersistenceReceipt, RecommendationFeedback, RecommendationSession, RecommendationStats,
};
use crate::monitoring::AgentMetrics;
use crate::procedural::ProceduralMemory;
use std::time::Duration;

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

    /// Record a recommendation session and return a truthful `PersistenceReceipt` (ADR-080 §3).
    ///
    /// Tracks the session in the in-memory tracker and persists it through the
    /// capability-aware receipt path (ADR-081 §2). The receipt distinguishes
    /// durable, partial, memory-only, and failed persistence so callers can
    /// determine whether feedback submitted after restart will find the
    /// session. Configured backends that do not advertise capability are
    /// skipped; raw backend errors are logged, never embedded in the receipt.
    pub async fn record_recommendation_session_checked(
        &self,
        session: RecommendationSession,
    ) -> PersistenceReceipt {
        self.recommendation_tracker
            .record_session(session.clone())
            .await;
        self.persist_session_checked(&session).await
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
    /// Returns `Ok(())` on success. Returns an error if the session cannot be
    /// resolved from either the in-memory tracker or any configured storage
    /// backend, or if an applied pattern ID was not recommended in that session.
    ///
    /// A session that was durably persisted before a restart is resolvable here,
    /// so feedback remains accepted across restarts (ADR-081 §1).
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
        // ADR-081 §1: resolve the session through memory → storage before the
        // tracker's unknown-session rejection can fire. Without this, feedback for
        // a durably persisted session fails after a restart, when the in-memory
        // tracker is cold.
        if self
            .recommendation_tracker
            .get_session(feedback.session_id)
            .await
            .is_none()
        {
            self.fetch_session_by_id_from_storage(feedback.session_id)
                .await;
        }

        self.recommendation_tracker
            .record_feedback(feedback.clone())
            .await?;
        self.persist_recommendation_feedback(&feedback).await;
        self.refresh_ranking_index().await;
        Ok(())
    }

    /// Record feedback and return a truthful `PersistenceReceipt` (ADR-080 §3).
    ///
    /// Performs the same memory → Turso → redb session resolution and ADR-080 §4
    /// integrity validation as the legacy [`record_recommendation_feedback`](Self::record_recommendation_feedback)
    /// (unknown sessions and non-recommended applied IDs are rejected exactly
    /// as before), then persists the feedback through the capability-aware
    /// receipt path. The receipt's episode ID is the resolved session's,
    /// obtained after hydration and integrity validation.
    pub async fn record_recommendation_feedback_checked(
        &self,
        feedback: RecommendationFeedback,
    ) -> Result<PersistenceReceipt> {
        // ADR-081 §1: resolve the session through memory → storage before the
        // tracker's unknown-session rejection can fire. Without this, feedback
        // for a durably persisted session fails after a restart, when the
        // in-memory tracker is cold.
        let resolved = match self
            .recommendation_tracker
            .get_session(feedback.session_id)
            .await
        {
            Some(session) => Some(session),
            None => {
                self.fetch_session_by_id_from_storage(feedback.session_id)
                    .await
            }
        };

        // ADR-080 §4: integrity checks (unknown session / non-recommended IDs).
        self.recommendation_tracker
            .record_feedback(feedback.clone())
            .await?;

        let episode_id = resolved
            .map(|session| session.episode_id)
            .unwrap_or_else(uuid::Uuid::nil);

        let receipt = self.persist_feedback_checked(&feedback, episode_id).await;
        self.refresh_ranking_index().await;
        Ok(receipt)
    }

    /// Validate that an episode exists before an attributed recommendation
    /// creates a session (ADR-080 §1).
    ///
    /// A missing episode is mapped to `Error::InvalidInput` naming the
    /// operation and UUID so callers can distinguish "bad attribution target"
    /// from storage failures; any non-`NotFound` error is propagated unchanged.
    pub(crate) async fn validate_attributed_episode(
        &self,
        episode_id: uuid::Uuid,
        operation: &str,
    ) -> Result<()> {
        match self.get_episode(episode_id).await {
            Ok(_) => Ok(()),
            Err(crate::error::Error::NotFound(_)) => Err(crate::error::Error::InvalidInput(
                format!("{operation}: episode {episode_id} does not exist"),
            )),
            Err(err) => Err(err),
        }
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

// ============================================================================
// Procedural Memory API
// ============================================================================

impl SelfLearningMemory {
    /// Store a procedural memory
    pub async fn store_procedural_memory(&self, procedural: ProceduralMemory) -> Result<()> {
        // Store in durable storage
        if let Some(storage) = &self.turso_storage {
            storage.store_procedural(&procedural).await?;
        }

        // Store in cache
        if let Some(storage) = &self.cache_storage {
            let _ = storage.store_procedural(&procedural).await;
        }

        // Always store in fallback for in-memory access
        let mut procedural_fallback = self.procedural_fallback.write().await;
        procedural_fallback.insert(procedural.id, procedural.clone());

        Ok(())
    }

    /// Retrieve a procedural memory by ID
    pub async fn get_procedural_memory(&self, id: uuid::Uuid) -> Result<Option<ProceduralMemory>> {
        // Try cache first
        if let Some(storage) = &self.cache_storage {
            if let Ok(Some(procedural)) = storage.get_procedural(id).await {
                return Ok(Some(procedural));
            }
        }

        // Try durable storage
        if let Some(storage) = &self.turso_storage {
            match storage.get_procedural(id).await {
                Ok(Some(procedural)) => {
                    // Update cache
                    if let Some(cache) = &self.cache_storage {
                        let _ = cache.store_procedural(&procedural).await;
                    }
                    return Ok(Some(procedural));
                }
                Ok(None) => {}
                Err(e) => return Err(e),
            }
        }

        // Fallback to in-memory
        let procedural_fallback = self.procedural_fallback.read().await;
        Ok(procedural_fallback.get(&id).cloned())
    }

    /// Delete a procedural memory by ID
    pub async fn delete_procedural_memory(&self, id: uuid::Uuid) -> Result<()> {
        // Delete from durable storage
        if let Some(storage) = &self.turso_storage {
            storage.delete_procedural(id).await?;
        }

        // Delete from cache
        if let Some(storage) = &self.cache_storage {
            let _ = storage.delete_procedural(id).await;
        }

        // Delete from fallback
        let mut procedural_fallback = self.procedural_fallback.write().await;
        procedural_fallback.remove(&id);

        Ok(())
    }

    /// Query procedural memories
    pub async fn query_procedural_memory(
        &self,
        limit: Option<usize>,
    ) -> Result<Vec<ProceduralMemory>> {
        // Try durable storage first (source of truth)
        if let Some(storage) = &self.turso_storage {
            return storage.query_procedural(limit).await;
        }

        // Try cache storage if no durable storage available
        if let Some(storage) = &self.cache_storage {
            if let Ok(results) = storage.query_procedural(limit).await {
                return Ok(results);
            }
        }

        // Fallback to in-memory
        let procedural = self.procedural_fallback.read().await;
        let mut results: Vec<ProceduralMemory> = procedural.values().cloned().collect();
        results.sort_by_key(|b| std::cmp::Reverse(b.updated_at));
        if let Some(l) = limit {
            results.truncate(l);
        }
        Ok(results)
    }
}
