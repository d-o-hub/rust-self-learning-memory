//! E2E test verifying data persistence across system restarts

use memory_core::types::{MemoryConfig, TaskContext, TaskType};
use memory_core::{SelfLearningMemory};
use memory_storage_redb::RedbStorage;
use std::sync::Arc;
use tempfile::tempdir;
use uuid::Uuid;

#[tokio::test]
async fn test_persistence_across_restart() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("memory.redb");
    let cache_path = dir.path().join("cache.redb");

    let episode_id;
    let session_id = Uuid::new_v4();

    // PHASE 1: Initialize, store data, then drop
    {
        let turso = Arc::new(RedbStorage::new(&db_path).await.unwrap());
        let cache = Arc::new(RedbStorage::new(&cache_path).await.unwrap());
        let memory = SelfLearningMemory::with_storage(
            MemoryConfig::default(),
            turso,
            cache
        );

        episode_id = memory.start_episode(
            "Persistent task".to_string(),
            TaskContext::default(),
            TaskType::Testing
        ).await;

        // Add a checkpoint
        memory_core::memory::checkpoint::checkpoint_episode(&memory, episode_id, "Save point".to_string()).await.unwrap();

        // Record a recommendation session
        let session = memory_core::memory::attribution::RecommendationSession {
            session_id,
            episode_id,
            timestamp: chrono::Utc::now(),
            recommended_pattern_ids: vec!["p1".to_string()],
            recommended_playbook_ids: vec![],
        };
        memory.record_recommendation_session(session).await;
    }

    // PHASE 2: Re-initialize from same storage and verify
    {
        let turso = Arc::new(RedbStorage::new(&db_path).await.unwrap());
        let cache = Arc::new(RedbStorage::new(&cache_path).await.unwrap());
        let memory = SelfLearningMemory::with_storage(
            MemoryConfig::default(),
            turso,
            cache
        );

        // Verify episode exists and has checkpoint
        let episode = memory.get_episode(episode_id).await.unwrap();
        assert_eq!(episode.task_description, "Persistent task");
        assert_eq!(episode.checkpoints.len(), 1);
        assert_eq!(episode.checkpoints[0].reason, "Save point");

        // Verify recommendation session exists in storage
        // Since we can't easily access the internal storage backend from SelfLearningMemory
        // without downcasting if it was Arc<dyn StorageBackend>, but here we have the RedbStorage directly.
        // Wait, memory.turso_storage() returns &Arc<dyn StorageBackend>.

        let storage = memory.turso_storage().unwrap();
        let retrieved_session = storage.get_recommendation_session(session_id).await.unwrap().unwrap();
        assert_eq!(retrieved_session.session_id, session_id);
    }
}
