//! Hybrid Storage Recovery and Resilience Tests
//!
//! Verifies that the hybrid storage system handles partial backend failures
//! and correctly reconciles data when one backend is ahead of another.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use do_memory_core::episode::{CleanupResult, EpisodeRetentionPolicy, PatternId};
#[cfg(any(feature = "turso", feature = "redb"))]
use do_memory_core::memory::SelfLearningMemory;
use do_memory_core::memory::attribution::{
    RecommendationFeedback, RecommendationSession, RecommendationStats,
};
use do_memory_core::{Episode, Error, Heuristic, Pattern, Result, StorageBackend};
#[cfg(any(feature = "turso", feature = "redb"))]
use do_memory_core::{MemoryConfig, TaskContext, TaskOutcome, TaskType};
#[cfg(feature = "redb")]
use do_memory_test_utils::in_memory_redb_storage;
#[cfg(feature = "turso")]
use do_memory_test_utils::temp_local_storage;
use std::sync::Arc;
use uuid::Uuid;

/// A storage backend that fails on every operation
struct FailingStorage;

#[async_trait]
impl StorageBackend for FailingStorage {
    async fn store_episode(&self, _episode: &Episode) -> Result<()> {
        Err(Error::Storage("Simulated failure".to_string()))
    }
    async fn get_episode(&self, _id: Uuid) -> Result<Option<Episode>> {
        Err(Error::Storage("Simulated failure".to_string()))
    }
    async fn delete_episode(&self, _id: Uuid) -> Result<()> {
        Err(Error::Storage("Simulated failure".to_string()))
    }
    async fn store_pattern(&self, _pattern: &Pattern) -> Result<()> {
        Err(Error::Storage("Simulated failure".to_string()))
    }
    async fn get_pattern(&self, _id: PatternId) -> Result<Option<Pattern>> {
        Err(Error::Storage("Simulated failure".to_string()))
    }
    async fn store_heuristic(&self, _heuristic: &Heuristic) -> Result<()> {
        Err(Error::Storage("Simulated failure".to_string()))
    }
    async fn get_heuristic(&self, _id: Uuid) -> Result<Option<Heuristic>> {
        Err(Error::Storage("Simulated failure".to_string()))
    }
    async fn query_episodes_since(
        &self,
        _since: DateTime<Utc>,
        _limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        Err(Error::Storage("Simulated failure".to_string()))
    }
    async fn query_episodes_by_metadata(
        &self,
        _key: &str,
        _value: &str,
        _limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        Err(Error::Storage("Simulated failure".to_string()))
    }
    async fn store_embedding(&self, _id: &str, _embedding: Vec<f32>) -> Result<()> {
        Err(Error::Storage("Simulated failure".to_string()))
    }
    async fn get_embedding(&self, _id: &str) -> Result<Option<Vec<f32>>> {
        Err(Error::Storage("Simulated failure".to_string()))
    }
    async fn delete_embedding(&self, _id: &str) -> Result<bool> {
        Err(Error::Storage("Simulated failure".to_string()))
    }
    async fn store_embeddings_batch(&self, _embeddings: Vec<(String, Vec<f32>)>) -> Result<()> {
        Err(Error::Storage("Simulated failure".to_string()))
    }
    async fn get_embeddings_batch(&self, _ids: &[String]) -> Result<Vec<Option<Vec<f32>>>> {
        Err(Error::Storage("Simulated failure".to_string()))
    }
    async fn store_recommendation_session(&self, _session: &RecommendationSession) -> Result<()> {
        Err(Error::Storage("Simulated failure".to_string()))
    }
    async fn get_recommendation_session(
        &self,
        _session_id: Uuid,
    ) -> Result<Option<RecommendationSession>> {
        Err(Error::Storage("Simulated failure".to_string()))
    }
    async fn get_recommendation_session_for_episode(
        &self,
        _episode_id: Uuid,
    ) -> Result<Option<RecommendationSession>> {
        Err(Error::Storage("Simulated failure".to_string()))
    }
    async fn store_recommendation_feedback(
        &self,
        _feedback: &RecommendationFeedback,
    ) -> Result<()> {
        Err(Error::Storage("Simulated failure".to_string()))
    }
    async fn get_recommendation_feedback(
        &self,
        _session_id: Uuid,
    ) -> Result<Option<RecommendationFeedback>> {
        Err(Error::Storage("Simulated failure".to_string()))
    }
    async fn get_recommendation_stats(&self) -> Result<RecommendationStats> {
        Err(Error::Storage("Simulated failure".to_string()))
    }
    async fn cleanup_episodes(&self, _policy: &EpisodeRetentionPolicy) -> Result<CleanupResult> {
        Err(Error::Storage("Simulated failure".to_string()))
    }
    async fn count_cleanup_candidates(&self, _policy: &EpisodeRetentionPolicy) -> Result<usize> {
        Err(Error::Storage("Simulated failure".to_string()))
    }
}

#[cfg(feature = "redb")]
#[tokio::test]
async fn test_recovery_turso_unavailable_on_read() {
    let (redb, _redb_dir) = in_memory_redb_storage().await;
    redb.initialize_schema().await.unwrap();
    let failing_turso = Arc::new(FailingStorage);

    let config = MemoryConfig {
        quality_threshold: 0.0,
        pattern_extraction_threshold: 0.0,
        ..MemoryConfig::default()
    };

    let memory = SelfLearningMemory::with_storage(config, failing_turso, Arc::new(redb));

    // Inject an episode directly into redb
    let episode = Episode::new(
        "Cached episode".to_string(),
        TaskContext::default(),
        TaskType::Testing,
    );
    let episode_id = episode.episode_id;

    let (_, cache) = memory.storage_backends();
    cache.unwrap().store_episode(&episode).await.unwrap();

    // Read from memory - should fall back to redb since Turso fails
    let retrieved = memory
        .get_episode(episode_id)
        .await
        .expect("Should return episode from cache");
    assert_eq!(retrieved.episode_id, episode_id);
    assert_eq!(retrieved.task_description, "Cached episode");
}

#[cfg(all(feature = "turso", feature = "redb"))]
#[tokio::test]
async fn test_reconciliation_stale_cache() {
    let (turso, _turso_dir) = temp_local_storage().await;
    turso.initialize_schema().await.unwrap();
    let (redb, _redb_dir) = in_memory_redb_storage().await;
    redb.initialize_schema().await.unwrap();

    let config = MemoryConfig {
        quality_threshold: 0.0,
        pattern_extraction_threshold: 0.0,
        ..MemoryConfig::default()
    };

    let memory = SelfLearningMemory::with_storage(config, Arc::new(turso), Arc::new(redb));

    // 1. Manually inject an episode only into Turso
    let episode = Episode::new(
        "Turso only episode".to_string(),
        TaskContext::default(),
        TaskType::Testing,
    );
    let episode_id = episode.episode_id;

    let (turso_backend, redb_backend) = memory.storage_backends();
    turso_backend
        .unwrap()
        .store_episode(&episode)
        .await
        .unwrap();

    // Verify it's NOT in redb yet
    assert!(
        redb_backend
            .as_ref()
            .unwrap()
            .get_episode(episode_id)
            .await
            .unwrap()
            .is_none()
    );

    // 2. Read through the memory orchestrator
    // This should trigger the reconciliation logic (fetch from Turso, then store in redb)
    let retrieved = memory
        .get_episode(episode_id)
        .await
        .expect("Should retrieve from Turso");
    assert_eq!(retrieved.episode_id, episode_id);

    // 3. Verify it is now in redb
    let cached = redb_backend.unwrap().get_episode(episode_id).await.unwrap();
    assert!(cached.is_some());
    assert_eq!(cached.unwrap().episode_id, episode_id);
}

#[cfg(feature = "redb")]
#[tokio::test]
async fn test_recovery_turso_failure_on_write_does_not_block() {
    let (redb, _redb_dir) = in_memory_redb_storage().await;
    redb.initialize_schema().await.unwrap();
    let failing_turso = Arc::new(FailingStorage);

    let config = MemoryConfig {
        quality_threshold: 0.0,
        pattern_extraction_threshold: 0.0,
        ..MemoryConfig::default()
    };

    let memory = SelfLearningMemory::with_storage(config, failing_turso, Arc::new(redb));

    let episode_id = memory
        .start_episode(
            "Failing Turso test".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Complete episode - should succeed even if Turso fails, as long as it's logged/warned
    // Currently SelfLearningMemory::complete_episode logs warnings for storage failures but continues.
    let outcome = TaskOutcome::Success {
        verdict: "ok".to_string(),
        artifacts: vec![],
    };
    let result = memory.complete_episode(episode_id, outcome).await;

    assert!(
        result.is_ok(),
        "complete_episode should be resilient to single backend failure"
    );

    // Verify it was at least stored in redb
    let (_, cache) = memory.storage_backends();
    let redb_ep = cache.unwrap().get_episode(episode_id).await.unwrap();
    assert!(redb_ep.is_some());
}
