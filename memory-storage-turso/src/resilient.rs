//! # Resilient Storage with Circuit Breaker
//!
//! Wraps TursoStorage with circuit breaker protection for production resilience.
//!
//! - Protects against cascading failures with circuit breaker pattern
//! - Tracks failure statistics and recovery

use async_trait::async_trait;
use do_memory_core::memory::attribution::{
    RecommendationFeedback, RecommendationSession, RecommendationStats,
};
use do_memory_core::storage::circuit_breaker::{
    CircuitBreaker, CircuitBreakerConfig, CircuitState,
};
use do_memory_core::{Episode, Heuristic, Pattern, Result, StorageBackend};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::TursoStorage;

/// Resilient storage wrapper with circuit breaker protection
///
/// Wraps TursoStorage operations with circuit breaker pattern to provide:
/// - Fast failure when service is down
/// - Automatic recovery attempts
/// - Failure statistics and monitoring
pub struct ResilientStorage {
    /// Underlying Turso storage
    pub(crate) storage: Arc<TursoStorage>,
    /// Circuit breaker for resilience
    circuit_breaker: Arc<CircuitBreaker>,
}

impl ResilientStorage {
    /// Create a new resilient storage wrapper
    ///
    /// # Arguments
    ///
    /// * `storage` - Turso storage backend to wrap
    /// * `config` - Circuit breaker configuration
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use do_memory_storage_turso::{TursoStorage, ResilientStorage};
    /// # use do_memory_core::storage::circuit_breaker::CircuitBreakerConfig;
    /// # async fn example() -> anyhow::Result<()> {
    /// let turso = TursoStorage::new("libsql://localhost:8080", "token").await?;
    ///
    /// let config = CircuitBreakerConfig {
    ///     failure_threshold: 5,
    ///     timeout: std::time::Duration::from_secs(30),
    ///     ..Default::default()
    /// };
    ///
    /// let resilient = ResilientStorage::new(turso, config);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(storage: TursoStorage, config: CircuitBreakerConfig) -> Self {
        info!("Creating resilient storage with circuit breaker protection");

        Self {
            storage: Arc::new(storage),
            circuit_breaker: Arc::new(CircuitBreaker::new(config)),
        }
    }

    /// Get the current circuit breaker state
    ///
    /// Useful for monitoring and health checks.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use do_memory_storage_turso::ResilientStorage;
    /// # use do_memory_core::storage::circuit_breaker::CircuitState;
    /// # async fn example(storage: ResilientStorage) {
    /// let state = storage.circuit_state().await;
    /// match state {
    ///     CircuitState::Closed => println!("Circuit is healthy"),
    ///     CircuitState::Open => println!("Circuit is open - service down"),
    ///     CircuitState::HalfOpen => println!("Circuit is testing recovery"),
    /// }
    /// # }
    /// ```
    pub async fn circuit_state(&self) -> CircuitState {
        self.circuit_breaker.state().await
    }

    /// Get circuit breaker statistics
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use do_memory_storage_turso::ResilientStorage;
    /// # async fn example(storage: ResilientStorage) {
    /// let stats = storage.circuit_stats().await;
    /// println!("Total calls: {}", stats.total_calls);
    /// println!("Failures: {}", stats.failed_calls);
    /// println!("Circuit opened {} times", stats.circuit_opened_count);
    /// # }
    /// ```
    pub async fn circuit_stats(
        &self,
    ) -> do_memory_core::storage::circuit_breaker::CircuitBreakerStats {
        self.circuit_breaker.stats().await
    }

    /// Reset the circuit breaker
    ///
    /// Useful for manual intervention or testing.
    pub async fn reset_circuit(&self) {
        self.circuit_breaker.reset().await;
    }

    /// Health check with circuit breaker awareness
    ///
    /// Returns true if both the storage is healthy AND the circuit is closed.
    pub async fn health_check(&self) -> Result<bool> {
        let circuit_state = self.circuit_state().await;

        if circuit_state != CircuitState::Closed {
            warn!("Health check: circuit breaker is {:?}", circuit_state);
            return Ok(false);
        }

        self.circuit_call(|s| async move { s.health_check().await })
            .await
    }

    /// Helper to reduce circuit-breaker boilerplate in StorageBackend methods
    pub(crate) async fn circuit_call<F, Fut, T>(&self, op: F) -> Result<T>
    where
        F: FnOnce(Arc<TursoStorage>) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<T>> + Send,
        T: Send + 'static,
    {
        let storage = Arc::clone(&self.storage);
        self.circuit_breaker
            .call(move || {
                let storage = Arc::clone(&storage);
                op(storage)
            })
            .await
    }
}

#[async_trait]
impl StorageBackend for ResilientStorage {
    async fn store_episode(&self, episode: &Episode) -> Result<()> {
        let episode = episode.clone();
        self.circuit_call(move |s| async move { s.store_episode(&episode).await })
            .await
    }

    async fn get_episode(&self, id: Uuid) -> Result<Option<Episode>> {
        self.circuit_call(move |s| async move { s.get_episode(id).await })
            .await
    }

    async fn delete_episode(&self, id: Uuid) -> Result<()> {
        self.circuit_call(move |s| async move { s.delete_episode(id).await })
            .await
    }

    async fn store_pattern(&self, pattern: &Pattern) -> Result<()> {
        let pattern = pattern.clone();
        self.circuit_call(move |s| async move { s.store_pattern(&pattern).await })
            .await
    }

    async fn get_pattern(&self, id: do_memory_core::episode::PatternId) -> Result<Option<Pattern>> {
        self.circuit_call(move |s| async move { s.get_pattern(id).await })
            .await
    }

    async fn get_all_patterns(&self) -> Result<Vec<Pattern>> {
        self.circuit_call(move |s| async move { s.get_all_patterns().await })
            .await
    }

    async fn delete_pattern(&self, id: do_memory_core::episode::PatternId) -> Result<()> {
        self.circuit_call(move |s| async move { s.delete_pattern(id).await })
            .await
    }
    async fn store_heuristic(&self, heuristic: &Heuristic) -> Result<()> {
        let heuristic = heuristic.clone();
        self.circuit_call(move |s| async move { s.store_heuristic(&heuristic).await })
            .await
    }
    async fn get_heuristic(&self, id: Uuid) -> Result<Option<Heuristic>> {
        self.circuit_call(move |s| async move { s.get_heuristic(id).await })
            .await
    }

    async fn query_episodes_since(
        &self,
        since: chrono::DateTime<chrono::Utc>,
        limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        self.circuit_call(move |s| async move { s.query_episodes_since(since, limit).await })
            .await
    }

    async fn query_episodes_by_metadata(
        &self,
        key: &str,
        value: &str,
        limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        let key = key.to_string();
        let value = value.to_string();
        self.circuit_call(move |s| {
            let key = key;
            let value = value;
            async move { s.query_episodes_by_metadata(&key, &value, limit).await }
        })
        .await
    }

    async fn store_embedding(&self, id: &str, embedding: Vec<f32>) -> Result<()> {
        let id = id.to_string();
        self.circuit_call(move |s| async move { s.store_embedding(&id, embedding).await })
            .await
    }

    async fn get_embedding(&self, id: &str) -> Result<Option<Vec<f32>>> {
        let id = id.to_string();
        self.circuit_call(move |s| async move { s.get_embedding(&id).await })
            .await
    }

    async fn delete_embedding(&self, id: &str) -> Result<bool> {
        let id = id.to_string();
        self.circuit_call(move |s| async move { s.delete_embedding(&id).await })
            .await
    }

    async fn store_embeddings_batch(&self, embeddings: Vec<(String, Vec<f32>)>) -> Result<()> {
        self.circuit_call(move |s| async move { s.store_embeddings_batch(embeddings).await })
            .await
    }

    async fn get_embeddings_batch(&self, ids: &[String]) -> Result<Vec<Option<Vec<f32>>>> {
        let ids = ids.to_vec();
        self.circuit_call(move |s| async move { s.get_embeddings_batch(&ids).await })
            .await
    }

    async fn list_embedding_ids(&self) -> Result<Vec<String>> {
        self.circuit_call(move |s| async move { s.list_embedding_ids().await })
            .await
    }

    async fn store_recommendation_session(&self, session: &RecommendationSession) -> Result<()> {
        let session = session.clone();
        self.circuit_call(move |s| async move { s.store_recommendation_session(&session).await })
            .await
    }

    async fn get_recommendation_session(
        &self,
        session_id: Uuid,
    ) -> Result<Option<RecommendationSession>> {
        self.circuit_call(move |s| async move { s.get_recommendation_session(session_id).await })
            .await
    }

    async fn get_recommendation_session_for_episode(
        &self,
        episode_id: Uuid,
    ) -> Result<Option<RecommendationSession>> {
        self.circuit_call(move |s| async move {
            s.get_recommendation_session_for_episode(episode_id).await
        })
        .await
    }

    async fn store_recommendation_feedback(&self, feedback: &RecommendationFeedback) -> Result<()> {
        let feedback = feedback.clone();
        self.circuit_call(move |s| async move { s.store_recommendation_feedback(&feedback).await })
            .await
    }

    async fn get_recommendation_feedback(
        &self,
        session_id: Uuid,
    ) -> Result<Option<RecommendationFeedback>> {
        self.circuit_call(move |s| async move { s.get_recommendation_feedback(session_id).await })
            .await
    }

    async fn get_recommendation_stats(&self) -> Result<RecommendationStats> {
        self.circuit_call(move |s| async move { s.get_recommendation_stats().await })
            .await
    }

    async fn list_recommendation_sessions(&self) -> Result<Vec<RecommendationSession>> {
        self.circuit_call(move |s| async move { s.list_recommendation_sessions().await })
            .await
    }

    async fn list_recommendation_feedback(&self) -> Result<Vec<RecommendationFeedback>> {
        self.circuit_call(move |s| async move { s.list_recommendation_feedback().await })
            .await
    }

    async fn store_episode_summary(
        &self,
        summary: &do_memory_core::semantic::EpisodeSummary,
    ) -> Result<()> {
        let summary = summary.clone();
        self.circuit_call(
            move |s| async move { s.store_summary(summary.episode_id, &summary).await },
        )
        .await
    }

    async fn get_episode_summary(
        &self,
        episode_id: Uuid,
    ) -> Result<Option<do_memory_core::semantic::EpisodeSummary>> {
        self.circuit_call(move |s| async move { s.get_summary(episode_id).await })
            .await
    }

    async fn cleanup_episodes(
        &self,
        policy: &do_memory_core::episode::EpisodeRetentionPolicy,
    ) -> Result<do_memory_core::episode::CleanupResult> {
        let policy = policy.clone();
        self.circuit_call(move |s| async move { s.cleanup_episodes(&policy).await })
            .await
    }

    async fn count_cleanup_candidates(
        &self,
        policy: &do_memory_core::episode::EpisodeRetentionPolicy,
    ) -> Result<usize> {
        let policy = policy.clone();
        self.circuit_call(move |s| async move { s.count_cleanup_candidates(&policy).await })
            .await
    }
}

#[cfg(test)]
#[path = "resilient_tests.rs"]
mod tests;
