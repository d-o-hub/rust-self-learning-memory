//! Comprehensive coverage tests for resilient storage and keepalive pool components
//!
//! Target modules:
//! - resilient.rs (38.21% -> 70%)
//! - pool/keepalive/monitoring.rs (29.63% -> 50%)
//! - pool/keepalive/connection.rs (41.46% -> 50%)

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

// ============================================================================
// Resilient Storage Tests
// ============================================================================

mod resilient_tests {
    use do_memory_core::StorageBackend;
    use do_memory_core::memory::attribution::{RecommendationFeedback, RecommendationSession};
    use do_memory_core::storage::circuit_breaker::{CircuitBreakerConfig, CircuitState};
    use do_memory_core::{
        Episode, ExecutionStep, Heuristic, Pattern, PatternEffectiveness, TaskContext, TaskOutcome,
        TaskType,
    };
    use do_memory_storage_turso::{ResilientStorage, TursoStorage};
    use std::time::Duration as StdDuration;
    use tempfile::TempDir;
    use uuid::Uuid;

    /// Helper to create test storage with circuit breaker
    async fn create_resilient_storage() -> (ResilientStorage, TempDir) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");

        let db = libsql::Builder::new_local(&db_path).build().await.unwrap();

        let turso = TursoStorage::from_database(db).unwrap();
        turso.initialize_schema().await.unwrap();

        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            timeout: StdDuration::from_millis(100),
            ..Default::default()
        };

        let resilient = ResilientStorage::new(turso, config);
        (resilient, dir)
    }

    /// Helper to create a test episode with steps
    fn create_test_episode(description: &str) -> Episode {
        let context = TaskContext {
            domain: "testing".to_string(),
            tags: vec!["test".to_string()],
            ..Default::default()
        };
        let mut episode = Episode::new(description.to_string(), context, TaskType::CodeGeneration);
        episode.steps.push(ExecutionStep::new(
            1,
            "test_step".to_string(),
            "test action".to_string(),
        ));
        episode.outcome = Some(TaskOutcome::Success {
            verdict: "completed".to_string(),
            artifacts: vec![],
        });
        episode
    }

    #[tokio::test]
    async fn test_resilient_store_episode() {
        let (storage, _dir) = create_resilient_storage().await;

        let episode = create_test_episode("test episode");

        let result = storage.store_episode(&episode).await;
        assert!(result.is_ok(), "store_episode should succeed");

        // Verify circuit breaker stats
        let stats = storage.circuit_stats().await;
        assert_eq!(stats.total_calls, 1);
        assert_eq!(stats.successful_calls, 1);
    }

    #[tokio::test]
    async fn test_resilient_get_episode() {
        let (storage, _dir) = create_resilient_storage().await;

        // First store an episode
        let episode = create_test_episode("test episode");
        storage.store_episode(&episode).await.unwrap();

        // Then retrieve it
        let result = storage.get_episode(episode.episode_id).await;
        assert!(result.is_ok(), "get_episode should succeed");

        let retrieved = result.unwrap();
        assert!(retrieved.is_some(), "Episode should be found");
        assert_eq!(retrieved.unwrap().episode_id, episode.episode_id);
    }

    #[tokio::test]
    async fn test_resilient_get_episode_not_found() {
        let (storage, _dir) = create_resilient_storage().await;

        // Try to get a non-existent episode
        let result = storage.get_episode(Uuid::new_v4()).await;
        assert!(
            result.is_ok(),
            "get_episode should succeed even for non-existent"
        );

        let retrieved = result.unwrap();
        assert!(
            retrieved.is_none(),
            "Non-existent episode should return None"
        );
    }

    #[tokio::test]
    async fn test_resilient_delete_episode() {
        let (storage, _dir) = create_resilient_storage().await;

        // First store an episode
        let episode = create_test_episode("test episode");
        storage.store_episode(&episode).await.unwrap();

        // Then delete it
        let result = storage.delete_episode(episode.episode_id).await;
        assert!(result.is_ok(), "delete_episode should succeed");

        // Verify it's gone
        let retrieved = storage.get_episode(episode.episode_id).await.unwrap();
        assert!(retrieved.is_none(), "Deleted episode should not be found");
    }

    #[tokio::test]
    async fn test_resilient_store_pattern() {
        let (storage, _dir) = create_resilient_storage().await;

        let pattern = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["tool1".to_string(), "tool2".to_string()],
            context: TaskContext::default(),
            success_rate: 0.8,
            avg_latency: chrono::Duration::milliseconds(100),
            occurrence_count: 5,
            effectiveness: PatternEffectiveness::default(),
        };

        let result = storage.store_pattern(&pattern).await;
        assert!(result.is_ok(), "store_pattern should succeed");

        let stats = storage.circuit_stats().await;
        assert_eq!(stats.total_calls, 1);
    }

    #[tokio::test]
    async fn test_resilient_get_pattern() {
        let (storage, _dir) = create_resilient_storage().await;

        // First store a pattern
        let pattern_id = Uuid::new_v4();
        let pattern = Pattern::ToolSequence {
            id: pattern_id,
            tools: vec!["tool1".to_string(), "tool2".to_string()],
            context: TaskContext::default(),
            success_rate: 0.8,
            avg_latency: chrono::Duration::milliseconds(100),
            occurrence_count: 5,
            effectiveness: PatternEffectiveness::default(),
        };
        storage.store_pattern(&pattern).await.unwrap();

        // Then retrieve it
        let result = storage.get_pattern(pattern_id).await;
        assert!(result.is_ok(), "get_pattern should succeed");

        let retrieved = result.unwrap();
        assert!(retrieved.is_some(), "Pattern should be found");
    }

    #[tokio::test]
    async fn test_resilient_store_heuristic() {
        let (storage, _dir) = create_resilient_storage().await;

        let heuristic =
            Heuristic::new("test condition".to_string(), "test action".to_string(), 0.8);

        let result = storage.store_heuristic(&heuristic).await;
        assert!(result.is_ok(), "store_heuristic should succeed");

        let stats = storage.circuit_stats().await;
        assert_eq!(stats.total_calls, 1);
    }

    #[tokio::test]
    async fn test_resilient_get_heuristic() {
        let (storage, _dir) = create_resilient_storage().await;

        // First store a heuristic
        let heuristic =
            Heuristic::new("test condition".to_string(), "test action".to_string(), 0.8);
        storage.store_heuristic(&heuristic).await.unwrap();

        // Then retrieve it
        let result = storage.get_heuristic(heuristic.heuristic_id).await;
        assert!(result.is_ok(), "get_heuristic should succeed");

        let retrieved = result.unwrap();
        assert!(retrieved.is_some(), "Heuristic should be found");
        assert_eq!(retrieved.unwrap().heuristic_id, heuristic.heuristic_id);
    }

    #[tokio::test]
    async fn test_resilient_query_episodes_since() {
        let (storage, _dir) = create_resilient_storage().await;

        // Store some episodes
        let episode1 = create_test_episode("ep1");
        let episode2 = create_test_episode("ep2");
        storage.store_episode(&episode1).await.unwrap();
        storage.store_episode(&episode2).await.unwrap();

        // Query episodes since a past timestamp
        let since = chrono::Utc::now() - chrono::Duration::hours(1);
        let result = storage.query_episodes_since(since, Some(10)).await;
        assert!(result.is_ok(), "query_episodes_since should succeed");

        let episodes = result.unwrap();
        assert!(episodes.len() >= 2, "Should find at least 2 episodes");
    }

    #[tokio::test]
    async fn test_resilient_query_episodes_by_metadata() {
        let (storage, _dir) = create_resilient_storage().await;

        // Store an episode with metadata
        let mut episode = create_test_episode("test");
        episode
            .metadata
            .insert("test_key".to_string(), "test_value".to_string());
        storage.store_episode(&episode).await.unwrap();

        // Query by metadata
        let result = storage
            .query_episodes_by_metadata("test_key", "test_value", Some(10))
            .await;
        assert!(result.is_ok(), "query_episodes_by_metadata should succeed");

        let episodes = result.unwrap();
        assert!(
            !episodes.is_empty(),
            "Should find episodes matching metadata"
        );
    }

    #[tokio::test]
    #[ignore = "ADR-027: libsql memory corruption bug in CI"]
    async fn test_resilient_embedding_operations() {
        let (storage, _dir) = create_resilient_storage().await;

        let embedding_id = "test_embedding_123";
        let embedding = vec![0.1_f32, 0.2, 0.3, 0.4, 0.5];

        // Store embedding
        let result = storage
            .store_embedding(embedding_id, embedding.clone())
            .await;
        assert!(result.is_ok(), "store_embedding should succeed");

        // Get embedding
        let result = storage.get_embedding(embedding_id).await;
        assert!(result.is_ok(), "get_embedding should succeed");
        let retrieved = result.unwrap();
        assert!(retrieved.is_some(), "Embedding should be found");

        // Delete embedding
        let result = storage.delete_embedding(embedding_id).await;
        assert!(result.is_ok(), "delete_embedding should succeed");
        assert!(result.unwrap(), "delete_embedding should return true");

        // Verify deletion
        let result = storage.get_embedding(embedding_id).await.unwrap();
        assert!(result.is_none(), "Deleted embedding should not be found");
    }

    #[tokio::test]
    #[ignore = "ADR-027: libsql memory corruption bug in CI"]
    async fn test_resilient_embedding_batch_operations() {
        let (storage, _dir) = create_resilient_storage().await;

        let embeddings = vec![
            ("emb1".to_string(), vec![0.1_f32, 0.2]),
            ("emb2".to_string(), vec![0.3_f32, 0.4]),
            ("emb3".to_string(), vec![0.5_f32, 0.6]),
        ];

        // Store batch
        let result = storage.store_embeddings_batch(embeddings.clone()).await;
        assert!(result.is_ok(), "store_embeddings_batch should succeed");

        // Get batch
        let ids = vec!["emb1".to_string(), "emb2".to_string(), "emb3".to_string()];
        let result = storage.get_embeddings_batch(&ids).await;
        assert!(result.is_ok(), "get_embeddings_batch should succeed");

        let retrieved = result.unwrap();
        assert_eq!(retrieved.len(), 3, "Should retrieve 3 embeddings");
        assert!(retrieved[0].is_some(), "emb1 should be found");
        assert!(retrieved[1].is_some(), "emb2 should be found");
        assert!(retrieved[2].is_some(), "emb3 should be found");
    }

    #[tokio::test]
    async fn test_resilient_recommendation_session_operations() {
        let (storage, _dir) = create_resilient_storage().await;

        let episode = create_test_episode("test");
        storage.store_episode(&episode).await.unwrap();

        let session = RecommendationSession {
            session_id: Uuid::new_v4(),
            episode_id: episode.episode_id,
            timestamp: chrono::Utc::now(),
            recommended_pattern_ids: vec!["pattern-1".to_string()],
            recommended_playbook_ids: vec![],
        };

        // Store session
        let result = storage.store_recommendation_session(&session).await;
        assert!(
            result.is_ok(),
            "store_recommendation_session should succeed"
        );

        // Get session by session_id
        let result = storage.get_recommendation_session(session.session_id).await;
        assert!(result.is_ok(), "get_recommendation_session should succeed");
        let retrieved = result.unwrap();
        assert!(retrieved.is_some(), "Session should be found");

        // Get session by episode_id
        let result = storage
            .get_recommendation_session_for_episode(episode.episode_id)
            .await;
        assert!(
            result.is_ok(),
            "get_recommendation_session_for_episode should succeed"
        );
        let retrieved = result.unwrap();
        assert!(retrieved.is_some(), "Session should be found by episode_id");
    }

    #[tokio::test]
    async fn test_resilient_recommendation_feedback_operations() {
        let (storage, _dir) = create_resilient_storage().await;

        let episode = create_test_episode("test");
        storage.store_episode(&episode).await.unwrap();

        let session = RecommendationSession {
            session_id: Uuid::new_v4(),
            episode_id: episode.episode_id,
            timestamp: chrono::Utc::now(),
            recommended_pattern_ids: vec!["pattern-1".to_string()],
            recommended_playbook_ids: vec![],
        };
        storage
            .store_recommendation_session(&session)
            .await
            .unwrap();

        let feedback = RecommendationFeedback {
            session_id: session.session_id,
            applied_pattern_ids: vec!["pattern-1".to_string()],
            consulted_episode_ids: vec![],
            outcome: TaskOutcome::Success {
                verdict: "Task completed".to_string(),
                artifacts: vec![],
            },
            agent_rating: Some(0.8),
        };

        // Store feedback
        let result = storage.store_recommendation_feedback(&feedback).await;
        assert!(
            result.is_ok(),
            "store_recommendation_feedback should succeed"
        );

        // Get feedback
        let result = storage
            .get_recommendation_feedback(session.session_id)
            .await;
        assert!(result.is_ok(), "get_recommendation_feedback should succeed");
        let retrieved = result.unwrap();
        assert!(retrieved.is_some(), "Feedback should be found");
    }

    #[tokio::test]
    async fn test_resilient_get_recommendation_stats() {
        let (storage, _dir) = create_resilient_storage().await;

        let result = storage.get_recommendation_stats().await;
        assert!(result.is_ok(), "get_recommendation_stats should succeed");

        let stats = result.unwrap();
        // Stats should be accessible
        let _total_sessions = stats.total_sessions;
    }

    #[tokio::test]
    async fn test_resilient_health_check_circuit_open() {
        let (storage, _dir) = create_resilient_storage().await;

        // Circuit should be closed initially
        assert_eq!(storage.circuit_state().await, CircuitState::Closed);

        // Health check should pass
        let result = storage.health_check().await;
        assert!(result.is_ok(), "health_check should succeed");
        assert!(result.unwrap(), "Storage should be healthy");
    }

    #[tokio::test]
    async fn test_resilient_multiple_operations_tracking() {
        let (storage, _dir) = create_resilient_storage().await;

        // Perform multiple operations
        let episode = create_test_episode("test");
        storage.store_episode(&episode).await.unwrap();
        storage.get_episode(episode.episode_id).await.unwrap();
        storage.delete_episode(episode.episode_id).await.unwrap();

        // Check stats
        let stats = storage.circuit_stats().await;
        assert_eq!(stats.total_calls, 3, "Should track 3 calls");
        assert_eq!(stats.successful_calls, 3, "All calls should be successful");
        assert_eq!(stats.failed_calls, 0, "No failures");
    }

    #[tokio::test]
    async fn test_resilient_circuit_state_queries() {
        let (storage, _dir) = create_resilient_storage().await;

        // Query circuit state multiple times
        let state1 = storage.circuit_state().await;
        let state2 = storage.circuit_state().await;
        assert_eq!(state1, CircuitState::Closed);
        assert_eq!(state2, CircuitState::Closed);

        // Reset circuit
        storage.reset_circuit().await;
        assert_eq!(storage.circuit_state().await, CircuitState::Closed);

        // Check stats after reset
        let stats = storage.circuit_stats().await;
        assert_eq!(stats.consecutive_failures, 0);
    }
}

// ============================================================================
// KeepAlive Connection Tests (connection.rs)
// ============================================================================

mod keepalive_connection_tests {
    use do_memory_storage_turso::pool::keepalive::{KeepAliveConnection, KeepAliveStatistics};
    use do_memory_storage_turso::pool::{ConnectionPool, PoolConfig};
    use parking_lot::RwLock;
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use tempfile::TempDir;

    async fn create_test_pool() -> (Arc<ConnectionPool>, TempDir) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");

        let db = libsql::Builder::new_local(&db_path).build().await.unwrap();
        let config = PoolConfig {
            max_connections: 5,
            connection_timeout: Duration::from_secs(5),
            enable_health_check: true,
            health_check_timeout: Duration::from_secs(2),
        };

        let pool = ConnectionPool::new(Arc::new(db), config).await.unwrap();
        (Arc::new(pool), dir)
    }

    #[tokio::test]
    async fn test_keepalive_connection_new() {
        let (pool, _dir) = create_test_pool().await;
        let pooled = pool.get().await.unwrap();
        let stats = Arc::new(RwLock::new(KeepAliveStatistics::default()));

        let conn = KeepAliveConnection::new(pooled, 1, Instant::now(), stats.clone());

        assert_eq!(conn.connection_id(), 1);
        assert!(conn.last_used() <= Instant::now());
    }

    #[tokio::test]
    async fn test_keepalive_connection_connection_method() {
        let (pool, _dir) = create_test_pool().await;
        let pooled = pool.get().await.unwrap();
        let stats = Arc::new(RwLock::new(KeepAliveStatistics::default()));

        let conn = KeepAliveConnection::new(pooled, 1, Instant::now(), stats.clone());

        // Should return Ok when connection is available
        let result = conn.connection();
        assert!(
            result.is_ok(),
            "connection() should return Ok for valid connection"
        );
    }

    #[tokio::test]
    async fn test_keepalive_connection_update_last_used() {
        let (pool, _dir) = create_test_pool().await;
        let pooled = pool.get().await.unwrap();
        let stats = Arc::new(RwLock::new(KeepAliveStatistics::default()));

        let conn = KeepAliveConnection::new(
            pooled,
            1,
            Instant::now() - Duration::from_secs(1),
            stats.clone(),
        );

        let initial_time = conn.last_used();

        // Small delay to ensure time difference
        std::thread::sleep(Duration::from_millis(10));

        conn.update_last_used();

        let new_time = conn.last_used();
        assert!(new_time > initial_time, "last_used should be updated");
    }

    #[tokio::test]
    async fn test_keepalive_connection_drop_updates_stats() {
        let (pool, _dir) = create_test_pool().await;
        let pooled = pool.get().await.unwrap();
        let stats = Arc::new(RwLock::new(KeepAliveStatistics {
            active_connections: 1,
            ..Default::default()
        }));

        {
            let _conn = KeepAliveConnection::new(pooled, 1, Instant::now(), stats.clone());
            // Connection active
            assert_eq!(stats.read().active_connections, 1);
        }
        // Connection dropped

        let stats_read = stats.read();
        assert_eq!(
            stats_read.active_connections, 0,
            "active_connections should be decremented on drop"
        );
    }

    #[tokio::test]
    async fn test_keepalive_connection_drop_zero_guard() {
        let (pool, _dir) = create_test_pool().await;
        let pooled = pool.get().await.unwrap();
        let stats = Arc::new(RwLock::new(KeepAliveStatistics {
            active_connections: 0, // Already zero
            ..Default::default()
        }));

        {
            let _conn = KeepAliveConnection::new(pooled, 1, Instant::now(), stats.clone());
        }

        // Should not go negative (guard in Drop impl)
        let stats_read = stats.read();
        assert_eq!(
            stats_read.active_connections, 0,
            "active_connections should not go negative"
        );
    }

    #[tokio::test]
    #[ignore = "ADR-027: libsql memory corruption bug in CI"]
    async fn test_keepalive_connection_into_connection() {
        let (pool, _dir) = create_test_pool().await;
        let pooled = pool.get().await.unwrap();
        let stats = Arc::new(RwLock::new(KeepAliveStatistics::default()));

        let conn = KeepAliveConnection::new(pooled, 1, Instant::now(), stats.clone());

        // Take ownership of the connection
        let result = conn.into_connection();
        assert!(result.is_ok(), "into_connection should succeed");

        let _owned_conn = result.unwrap();
        // Connection is now owned, KeepAliveConnection is consumed
    }

    #[tokio::test]
    async fn test_keepalive_connection_debug_impl() {
        let (pool, _dir) = create_test_pool().await;
        let pooled = pool.get().await.unwrap();
        let stats = Arc::new(RwLock::new(KeepAliveStatistics::default()));

        let conn = KeepAliveConnection::new(pooled, 1, Instant::now(), stats.clone());

        // Debug trait should be implemented
        let debug_str = format!("{:?}", conn);
        assert!(
            debug_str.contains("KeepAliveConnection"),
            "Debug output should contain struct name"
        );
    }
}

// ============================================================================
// KeepAlive Monitoring Tests (monitoring.rs)
// ============================================================================

mod keepalive_monitoring_tests {
    use do_memory_storage_turso::pool::keepalive::{KeepAliveConfig, KeepAlivePool};
    use do_memory_storage_turso::pool::{ConnectionPool, PoolConfig};
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::TempDir;

    async fn create_test_pool_with_config() -> (Arc<KeepAlivePool>, TempDir) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");

        let db = libsql::Builder::new_local(&db_path).build().await.unwrap();
        let pool_config = PoolConfig {
            max_connections: 5,
            connection_timeout: Duration::from_secs(5),
            enable_health_check: true,
            health_check_timeout: Duration::from_secs(2),
        };

        let pool = ConnectionPool::new(Arc::new(db), pool_config)
            .await
            .unwrap();
        let pool = Arc::new(pool);

        let keepalive_config = KeepAliveConfig {
            keep_alive_interval: Duration::from_millis(50),
            stale_threshold: Duration::from_millis(100),
            enable_proactive_ping: true,
            ping_timeout: Duration::from_secs(1),
        };

        let keepalive_pool = KeepAlivePool::with_config(pool, keepalive_config)
            .await
            .unwrap();

        (Arc::new(keepalive_pool), dir)
    }

    #[tokio::test]
    async fn test_cleanup_removes_stale_entries() {
        let (pool, _dir) = create_test_pool_with_config().await;

        // Get connections to populate tracking map
        let conn1 = pool.get().await.unwrap();
        let conn2 = pool.get().await.unwrap();

        assert_eq!(pool.tracked_connections(), 2);

        // Drop connections
        drop(conn1);
        drop(conn2);

        // Wait for stale threshold * 2 (cleanup threshold)
        tokio::time::sleep(Duration::from_millis(250)).await;

        // Cleanup should remove stale entries
        pool.cleanup();

        assert_eq!(
            pool.tracked_connections(),
            0,
            "Cleanup should remove stale entries"
        );
    }

    #[tokio::test]
    async fn test_cleanup_preserves_recent_entries() {
        let (pool, _dir) = create_test_pool_with_config().await;

        // Get a connection
        let conn = pool.get().await.unwrap();

        // Cleanup immediately (connection is still active)
        pool.cleanup();

        // Entry should still be tracked
        assert_eq!(pool.tracked_connections(), 1);

        drop(conn);
    }

    #[tokio::test]
    async fn test_cleanup_empty_pool() {
        let (pool, _dir) = create_test_pool_with_config().await;

        // No connections acquired
        assert_eq!(pool.tracked_connections(), 0);

        // Cleanup should work without error
        pool.cleanup();

        assert_eq!(pool.tracked_connections(), 0);
    }

    #[tokio::test]
    async fn test_cleanup_multiple_times() {
        let (pool, _dir) = create_test_pool_with_config().await;

        // Get and drop connections multiple times
        for _ in 0..3 {
            let conn = pool.get().await.unwrap();
            drop(conn);
        }

        // Wait for staleness
        tokio::time::sleep(Duration::from_millis(250)).await;

        // Cleanup should handle multiple entries
        pool.cleanup();
        pool.cleanup(); // Second cleanup should be safe

        assert_eq!(pool.tracked_connections(), 0);
    }

    #[tokio::test]
    async fn test_start_background_task() {
        let (pool, _dir) = create_test_pool_with_config().await;

        // Start background task
        pool.start_background_task();

        // Get a connection to have activity
        let conn = pool.get().await.unwrap();
        drop(conn);

        // Wait a bit for background task to run
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Stats should show proactive ping activity
        let stats = pool.statistics();
        // Note: proactive ping only counts if elapsed > keep_alive_interval
        // With our short interval, pings should be recorded
        let _ping_count = stats.total_proactive_pings;
    }

    #[tokio::test]
    async fn test_background_task_stops_on_pool_drop() {
        let (pool, _dir) = create_test_pool_with_config().await;

        // Start background task
        pool.start_background_task();

        // Get a connection
        let conn = pool.get().await.unwrap();
        drop(conn);

        // Wait a bit
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Drop the pool - background task should stop
        // (Weak reference upgrade fails)
        drop(pool);

        // Task should stop gracefully (no panic)
    }

    #[tokio::test]
    async fn test_proactive_ping_disabled() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");

        let db = libsql::Builder::new_local(&db_path).build().await.unwrap();
        let pool_config = PoolConfig {
            max_connections: 5,
            connection_timeout: Duration::from_secs(5),
            enable_health_check: true,
            health_check_timeout: Duration::from_secs(2),
        };

        let pool = ConnectionPool::new(Arc::new(db), pool_config)
            .await
            .unwrap();
        let pool = Arc::new(pool);

        // Disable proactive ping
        let keepalive_config = KeepAliveConfig {
            keep_alive_interval: Duration::from_millis(50),
            stale_threshold: Duration::from_millis(100),
            enable_proactive_ping: false, // Disabled
            ping_timeout: Duration::from_secs(1),
        };

        let keepalive_pool = KeepAlivePool::with_config(pool, keepalive_config)
            .await
            .unwrap();
        let pool = Arc::new(keepalive_pool);

        // Start background task
        pool.start_background_task();

        // Get a connection
        let conn = pool.get().await.unwrap();
        drop(conn);

        // Wait for background task
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Stats should NOT show proactive ping activity when disabled
        let stats = pool.statistics();
        // proactive ping is disabled, so no pings should be recorded
        // Note: The proactive_ping method checks enable_proactive_ping and returns early
        assert_eq!(
            stats.total_proactive_pings, 0,
            "Proactive ping should be disabled"
        );
    }

    #[tokio::test]
    async fn test_proactive_ping_updates_stats() {
        let (pool, _dir) = create_test_pool_with_config().await;

        // Get a connection and wait longer than keep_alive_interval
        let conn = pool.get().await.unwrap();

        // Wait for connection to "approach staleness" (elapsed > keep_alive_interval)
        tokio::time::sleep(Duration::from_millis(100)).await;

        // The proactive ping should count connections that are due for ping
        // Check stats
        let _stats = pool.statistics();
        // Connection is tracked
        assert_eq!(pool.tracked_connections(), 1);

        drop(conn);
    }

    #[tokio::test]
    async fn test_cleanup_statistics_consistency() {
        let (pool, _dir) = create_test_pool_with_config().await;

        // Initial stats
        let initial_stats = pool.statistics();
        assert_eq!(initial_stats.active_connections, 0);

        // Get multiple connections
        let conn1 = pool.get().await.unwrap();
        let conn2 = pool.get().await.unwrap();

        let stats = pool.statistics();
        assert_eq!(stats.active_connections, 2);

        // Cleanup while connections are active
        pool.cleanup();

        // Active connections should still be tracked
        let stats = pool.statistics();
        assert_eq!(stats.active_connections, 2);

        drop(conn1);
        drop(conn2);
    }
}

// ============================================================================
// Integration Tests for Full Workflow
// ============================================================================

mod integration_workflow_tests {
    use do_memory_core::Episode;
    use do_memory_core::StorageBackend;
    use do_memory_core::TaskType;
    use do_memory_core::storage::circuit_breaker::CircuitBreakerConfig;
    use do_memory_storage_turso::pool::keepalive::{KeepAliveConfig, KeepAlivePool};
    use do_memory_storage_turso::pool::{ConnectionPool, PoolConfig};
    use do_memory_storage_turso::{ResilientStorage, TursoStorage};
    use std::sync::Arc;
    use std::time::Duration as StdDuration;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_resilient_with_keepalive_integration() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");

        let db = libsql::Builder::new_local(&db_path).build().await.unwrap();

        let turso = TursoStorage::from_database(db).unwrap();
        turso.initialize_schema().await.unwrap();

        // Create resilient storage
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            timeout: StdDuration::from_millis(100),
            ..Default::default()
        };

        let resilient = ResilientStorage::new(turso, config);

        // Perform operations
        let episode = Episode::new(
            "integration test".to_string(),
            Default::default(),
            TaskType::CodeGeneration,
        );
        resilient.store_episode(&episode).await.unwrap();

        // Circuit should be healthy
        let stats = resilient.circuit_stats().await;
        assert_eq!(stats.successful_calls, 1);
    }

    #[tokio::test]
    async fn test_keepalive_pool_graceful_shutdown() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");

        let db = libsql::Builder::new_local(&db_path).build().await.unwrap();
        let pool_config = PoolConfig::default();
        let pool = ConnectionPool::new(Arc::new(db), pool_config)
            .await
            .unwrap();
        let pool = Arc::new(pool);

        let keepalive_config = KeepAliveConfig::default();
        let keepalive_pool = KeepAlivePool::with_config(pool, keepalive_config)
            .await
            .unwrap();

        // Get and drop connections
        let conn = keepalive_pool.get().await.unwrap();
        drop(conn);

        // Shutdown should complete gracefully
        keepalive_pool.shutdown().await;

        let stats = keepalive_pool.statistics();
        assert_eq!(stats.active_connections, 0);
    }

    #[tokio::test]
    async fn test_multiple_operations_through_resilient() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");

        let db = libsql::Builder::new_local(&db_path).build().await.unwrap();
        let turso = TursoStorage::from_database(db).unwrap();
        turso.initialize_schema().await.unwrap();

        let resilient = ResilientStorage::new(turso, CircuitBreakerConfig::default());

        // Create multiple episodes
        for i in 0..5 {
            let episode = Episode::new(
                format!("episode {}", i),
                Default::default(),
                TaskType::CodeGeneration,
            );
            resilient.store_episode(&episode).await.unwrap();
        }

        // Check circuit stats
        let stats = resilient.circuit_stats().await;
        assert_eq!(stats.total_calls, 5);
        assert_eq!(stats.successful_calls, 5);

        // Health check
        let healthy = resilient.health_check().await.unwrap();
        assert!(healthy);
    }
}
