//! Comprehensive tests for recommendations and similarity search modules
//!
//! Target coverage: 50% for each module:
//! - storage/recommendations.rs (315 LOC)
//! - storage/search/episodes.rs (154 LOC)
//! - storage/search/patterns.rs (137 LOC)

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use chrono::Utc;
use do_memory_core::{
    Episode, Pattern,
    embeddings::EmbeddingStorageBackend,
    memory::attribution::{RecommendationFeedback, RecommendationSession},
    pattern::PatternEffectiveness,
    types::{TaskContext, TaskOutcome, TaskType},
};
use do_memory_storage_turso::TursoStorage;
use libsql::Builder;
use tempfile::TempDir;
use uuid::Uuid;

// ============================================================================
// Test Helpers
// ============================================================================

async fn create_test_storage() -> (TursoStorage, TempDir) {
    let dir = TempDir::new().expect("create temp dir");
    let db_path = dir.path().join("test.db");

    let db = Builder::new_local(&db_path)
        .build()
        .await
        .expect("create local db");

    let storage = TursoStorage::from_database(db).expect("turso from db");
    storage.initialize_schema().await.expect("init schema");

    (storage, dir)
}

fn create_test_episode(description: &str) -> Episode {
    let context = TaskContext::default();
    let mut episode = Episode::new(description.to_string(), context, TaskType::Testing);
    episode.complete(TaskOutcome::Success {
        verdict: "Test completed".to_string(),
        artifacts: vec![],
    });
    episode
}

fn create_test_pattern(description: &str) -> Pattern {
    use chrono::Duration;
    Pattern::ToolSequence {
        id: Uuid::new_v4(),
        tools: vec![description.to_string()],
        context: TaskContext::default(),
        success_rate: 1.0,
        avg_latency: Duration::milliseconds(100),
        occurrence_count: 1,
        effectiveness: PatternEffectiveness::default(),
    }
}

fn create_test_recommendation_session(episode_id: Uuid) -> RecommendationSession {
    RecommendationSession {
        session_id: Uuid::new_v4(),
        episode_id,
        timestamp: Utc::now(),
        recommended_pattern_ids: vec!["pattern-1".to_string(), "pattern-2".to_string()],
        recommended_playbook_ids: vec![Uuid::new_v4()],
    }
}

fn create_test_feedback(session_id: Uuid) -> RecommendationFeedback {
    RecommendationFeedback {
        session_id,
        applied_pattern_ids: vec!["pattern-1".to_string()],
        consulted_episode_ids: vec![Uuid::new_v4()],
        outcome: TaskOutcome::Success {
            verdict: "Task completed successfully".to_string(),
            artifacts: vec![],
        },
        agent_rating: Some(0.9),
    }
}

fn create_test_embedding(dimension: usize, value: f32) -> Vec<f32> {
    vec![value; dimension]
}

// ============================================================================
// Recommendations Tests
// ============================================================================

mod recommendations_tests {
    use super::*;

    #[tokio::test]
    async fn test_store_and_retrieve_recommendation_session() {
        // Arrange
        let (storage, _dir) = create_test_storage().await;
        let episode = create_test_episode("Test task");
        storage
            .store_episode(&episode)
            .await
            .expect("store episode");

        let session = create_test_recommendation_session(episode.episode_id);
        let session_id = session.session_id;

        // Act
        storage
            .store_recommendation_session(&session)
            .await
            .expect("store session");

        let retrieved = storage
            .get_recommendation_session(session_id)
            .await
            .expect("get session");

        // Assert
        assert!(retrieved.is_some(), "Session should be retrieved");
        let retrieved_session = retrieved.expect("session exists");
        assert_eq!(retrieved_session.session_id, session_id);
        assert_eq!(
            retrieved_session.recommended_pattern_ids.len(),
            session.recommended_pattern_ids.len()
        );
        assert_eq!(
            retrieved_session.recommended_playbook_ids.len(),
            session.recommended_playbook_ids.len()
        );
    }

    #[tokio::test]
    async fn test_get_recommendation_session_not_found() {
        // Arrange
        let (storage, _dir) = create_test_storage().await;
        let non_existent_id = Uuid::new_v4();

        // Act
        let retrieved = storage
            .get_recommendation_session(non_existent_id)
            .await
            .expect("get session");

        // Assert
        assert!(retrieved.is_none(), "Session should not exist");
    }

    #[tokio::test]
    async fn test_get_recommendation_session_for_episode() {
        // Arrange
        let (storage, _dir) = create_test_storage().await;
        let episode = create_test_episode("Test task");
        storage
            .store_episode(&episode)
            .await
            .expect("store episode");

        let session = create_test_recommendation_session(episode.episode_id);
        storage
            .store_recommendation_session(&session)
            .await
            .expect("store session");

        // Act
        let retrieved = storage
            .get_recommendation_session_for_episode(episode.episode_id)
            .await
            .expect("get session for episode");

        // Assert
        assert!(retrieved.is_some(), "Session should be found for episode");
        let retrieved_session = retrieved.expect("session exists");
        assert_eq!(retrieved_session.episode_id, episode.episode_id);
    }

    #[tokio::test]
    async fn test_get_recommendation_session_for_episode_not_found() {
        // Arrange
        let (storage, _dir) = create_test_storage().await;
        let non_existent_episode = Uuid::new_v4();

        // Act
        let retrieved = storage
            .get_recommendation_session_for_episode(non_existent_episode)
            .await
            .expect("get session for episode");

        // Assert
        assert!(
            retrieved.is_none(),
            "Session should not be found for non-existent episode"
        );
    }

    #[tokio::test]
    async fn test_store_and_retrieve_recommendation_feedback() {
        // Arrange
        let (storage, _dir) = create_test_storage().await;
        let episode = create_test_episode("Test task");
        storage
            .store_episode(&episode)
            .await
            .expect("store episode");

        let session = create_test_recommendation_session(episode.episode_id);
        storage
            .store_recommendation_session(&session)
            .await
            .expect("store session");

        let feedback = create_test_feedback(session.session_id);

        // Act
        storage
            .store_recommendation_feedback(&feedback)
            .await
            .expect("store feedback");

        let retrieved = storage
            .get_recommendation_feedback(session.session_id)
            .await
            .expect("get feedback");

        // Assert
        assert!(retrieved.is_some(), "Feedback should be retrieved");
        let retrieved_feedback = retrieved.expect("feedback exists");
        assert_eq!(retrieved_feedback.session_id, session.session_id);
        assert_eq!(retrieved_feedback.agent_rating, Some(0.9));
    }

    #[tokio::test]
    async fn test_get_recommendation_feedback_not_found() {
        // Arrange
        let (storage, _dir) = create_test_storage().await;
        let non_existent_session = Uuid::new_v4();

        // Act
        let retrieved = storage
            .get_recommendation_feedback(non_existent_session)
            .await
            .expect("get feedback");

        // Assert
        assert!(retrieved.is_none(), "Feedback should not exist");
    }

    #[tokio::test]
    async fn test_get_recommendation_stats_empty() {
        // Arrange
        let (storage, _dir) = create_test_storage().await;

        // Act
        let stats = storage.get_recommendation_stats().await.expect("get stats");

        // Assert
        assert_eq!(stats.total_sessions, 0);
        assert_eq!(stats.total_feedback, 0);
        assert_eq!(stats.patterns_applied, 0);
        assert_eq!(stats.patterns_ignored, 0);
        assert_eq!(stats.successful_applications, 0);
        assert_eq!(stats.adoption_rate, 0.0);
        assert_eq!(stats.success_after_adoption_rate, 0.0);
        assert!(stats.avg_agent_rating.is_none());
    }

    #[tokio::test]
    async fn test_get_recommendation_stats_with_data() {
        // Arrange
        let (storage, _dir) = create_test_storage().await;

        // Create episode and session
        let episode = create_test_episode("Test task");
        storage
            .store_episode(&episode)
            .await
            .expect("store episode");

        let session = RecommendationSession {
            session_id: Uuid::new_v4(),
            episode_id: episode.episode_id,
            timestamp: Utc::now(),
            recommended_pattern_ids: vec!["p1".to_string(), "p2".to_string(), "p3".to_string()],
            recommended_playbook_ids: vec![],
        };
        storage
            .store_recommendation_session(&session)
            .await
            .expect("store session");

        // Create feedback with partial adoption
        let feedback = RecommendationFeedback {
            session_id: session.session_id,
            applied_pattern_ids: vec!["p1".to_string(), "p2".to_string()], // Applied 2 of 3
            consulted_episode_ids: vec![],
            outcome: TaskOutcome::Success {
                verdict: "Success".to_string(),
                artifacts: vec![],
            },
            agent_rating: Some(0.8),
        };
        storage
            .store_recommendation_feedback(&feedback)
            .await
            .expect("store feedback");

        // Act
        let stats = storage.get_recommendation_stats().await.expect("get stats");

        // Assert
        assert_eq!(stats.total_sessions, 1);
        assert_eq!(stats.total_feedback, 1);
        assert_eq!(stats.patterns_applied, 2);
        assert_eq!(stats.patterns_ignored, 1); // 3 recommended - 2 applied = 1 ignored
        assert_eq!(stats.successful_applications, 2);
        assert!(
            (stats.adoption_rate - 0.666).abs() < 0.01,
            "Adoption rate should be ~0.666"
        );
        assert_eq!(stats.success_after_adoption_rate, 1.0);
        assert_eq!(stats.avg_agent_rating, Some(0.8));
    }

    #[tokio::test]
    async fn test_recommendation_stats_partial_success_outcome() {
        // Arrange
        let (storage, _dir) = create_test_storage().await;

        let episode = create_test_episode("Test task");
        storage
            .store_episode(&episode)
            .await
            .expect("store episode");

        let session = RecommendationSession {
            session_id: Uuid::new_v4(),
            episode_id: episode.episode_id,
            timestamp: Utc::now(),
            recommended_pattern_ids: vec!["p1".to_string()],
            recommended_playbook_ids: vec![],
        };
        storage
            .store_recommendation_session(&session)
            .await
            .expect("store session");

        // PartialSuccess outcome should still count as successful application
        let feedback = RecommendationFeedback {
            session_id: session.session_id,
            applied_pattern_ids: vec!["p1".to_string()],
            consulted_episode_ids: vec![],
            outcome: TaskOutcome::PartialSuccess {
                verdict: "Partial".to_string(),
                completed: vec!["task1".to_string()],
                failed: vec!["task2".to_string()],
            },
            agent_rating: Some(0.5),
        };
        storage
            .store_recommendation_feedback(&feedback)
            .await
            .expect("store feedback");

        // Act
        let stats = storage.get_recommendation_stats().await.expect("get stats");

        // Assert - PartialSuccess counts as successful application
        assert_eq!(stats.successful_applications, 1);
        assert_eq!(stats.success_after_adoption_rate, 1.0);
    }

    #[tokio::test]
    async fn test_recommendation_stats_failure_outcome() {
        // Arrange
        let (storage, _dir) = create_test_storage().await;

        let episode = create_test_episode("Test task");
        storage
            .store_episode(&episode)
            .await
            .expect("store episode");

        let session = RecommendationSession {
            session_id: Uuid::new_v4(),
            episode_id: episode.episode_id,
            timestamp: Utc::now(),
            recommended_pattern_ids: vec!["p1".to_string()],
            recommended_playbook_ids: vec![],
        };
        storage
            .store_recommendation_session(&session)
            .await
            .expect("store session");

        // Failure outcome should not count as successful application
        let feedback = RecommendationFeedback {
            session_id: session.session_id,
            applied_pattern_ids: vec!["p1".to_string()],
            consulted_episode_ids: vec![],
            outcome: TaskOutcome::Failure {
                reason: "Failed".to_string(),
                error_details: None,
            },
            agent_rating: Some(0.1),
        };
        storage
            .store_recommendation_feedback(&feedback)
            .await
            .expect("store feedback");

        // Act
        let stats = storage.get_recommendation_stats().await.expect("get stats");

        // Assert - Failure does not count as successful application
        assert_eq!(stats.patterns_applied, 1);
        assert_eq!(stats.successful_applications, 0);
        assert_eq!(stats.success_after_adoption_rate, 0.0);
    }

    #[tokio::test]
    async fn test_recommendation_stats_multiple_sessions() {
        // Arrange
        let (storage, _dir) = create_test_storage().await;

        // Create multiple sessions and feedback
        for i in 0..3 {
            let episode = create_test_episode(&format!("Task {}", i));
            storage
                .store_episode(&episode)
                .await
                .expect("store episode");

            let session = RecommendationSession {
                session_id: Uuid::new_v4(),
                episode_id: episode.episode_id,
                timestamp: Utc::now(),
                recommended_pattern_ids: vec![format!("p{}", i)],
                recommended_playbook_ids: vec![],
            };
            storage
                .store_recommendation_session(&session)
                .await
                .expect("store session");

            let feedback = RecommendationFeedback {
                session_id: session.session_id,
                applied_pattern_ids: vec![format!("p{}", i)],
                consulted_episode_ids: vec![],
                outcome: TaskOutcome::Success {
                    verdict: "Success".to_string(),
                    artifacts: vec![],
                },
                agent_rating: Some(0.9),
            };
            storage
                .store_recommendation_feedback(&feedback)
                .await
                .expect("store feedback");
        }

        // Act
        let stats = storage.get_recommendation_stats().await.expect("get stats");

        // Assert
        assert_eq!(stats.total_sessions, 3);
        assert_eq!(stats.total_feedback, 3);
        assert_eq!(stats.patterns_applied, 3);
    }

    #[tokio::test]
    async fn test_recommendation_session_update() {
        // Arrange
        let (storage, _dir) = create_test_storage().await;
        let episode = create_test_episode("Test task");
        storage
            .store_episode(&episode)
            .await
            .expect("store episode");

        let session = RecommendationSession {
            session_id: Uuid::new_v4(),
            episode_id: episode.episode_id,
            timestamp: Utc::now(),
            recommended_pattern_ids: vec!["original".to_string()],
            recommended_playbook_ids: vec![],
        };
        storage
            .store_recommendation_session(&session)
            .await
            .expect("store session");

        // Update session with same session_id
        let updated_session = RecommendationSession {
            session_id: session.session_id,
            episode_id: episode.episode_id,
            timestamp: Utc::now(),
            recommended_pattern_ids: vec!["updated".to_string(), "new".to_string()],
            recommended_playbook_ids: vec![],
        };

        // Act
        storage
            .store_recommendation_session(&updated_session)
            .await
            .expect("update session");

        let retrieved = storage
            .get_recommendation_session(session.session_id)
            .await
            .expect("get session")
            .expect("session exists");

        // Assert - Updated data should be retrieved
        assert_eq!(retrieved.recommended_pattern_ids.len(), 2);
        assert!(
            retrieved
                .recommended_pattern_ids
                .contains(&"updated".to_string())
        );
    }

    #[tokio::test]
    async fn test_recommendation_feedback_update() {
        // Arrange
        let (storage, _dir) = create_test_storage().await;
        let episode = create_test_episode("Test task");
        storage
            .store_episode(&episode)
            .await
            .expect("store episode");

        let session = create_test_recommendation_session(episode.episode_id);
        storage
            .store_recommendation_session(&session)
            .await
            .expect("store session");

        let feedback = RecommendationFeedback {
            session_id: session.session_id,
            applied_pattern_ids: vec!["original".to_string()],
            consulted_episode_ids: vec![],
            outcome: TaskOutcome::Success {
                verdict: "Success".to_string(),
                artifacts: vec![],
            },
            agent_rating: Some(0.5),
        };
        storage
            .store_recommendation_feedback(&feedback)
            .await
            .expect("store feedback");

        // Update feedback
        let updated_feedback = RecommendationFeedback {
            session_id: session.session_id,
            applied_pattern_ids: vec!["updated".to_string()],
            consulted_episode_ids: vec![],
            outcome: TaskOutcome::Success {
                verdict: "Success updated".to_string(),
                artifacts: vec![],
            },
            agent_rating: Some(0.9),
        };

        // Act
        storage
            .store_recommendation_feedback(&updated_feedback)
            .await
            .expect("update feedback");

        let retrieved = storage
            .get_recommendation_feedback(session.session_id)
            .await
            .expect("get feedback")
            .expect("feedback exists");

        // Assert
        assert_eq!(retrieved.agent_rating, Some(0.9));
        assert_eq!(retrieved.applied_pattern_ids.len(), 1);
        assert_eq!(retrieved.applied_pattern_ids[0], "updated");
    }
}

// ============================================================================
// Episode Similarity Search Tests
// ============================================================================

mod episode_search_tests {
    use super::*;

    // NOTE: All tests in this module are ignored due to ADR-027 (libsql memory corruption bug in CI)
    // These tests use TursoStorage initialization which triggers the bug

    #[tokio::test]
    #[ignore = "ADR-027: libsql memory corruption bug in CI"]
    async fn test_find_similar_episodes_empty_database() {
        // Arrange
        let (storage, _dir) = create_test_storage().await;
        let query_embedding = create_test_embedding(384, 0.5);

        // Act
        let results = storage
            .find_similar_episodes(query_embedding, 10, 0.5)
            .await
            .expect("find similar episodes");

        // Assert
        assert!(
            results.is_empty(),
            "Should return empty results for empty database"
        );
    }

    #[tokio::test]
    #[ignore = "ADR-027: libsql memory corruption bug in CI"]
    async fn test_find_similar_episodes_with_embeddings() {
        // Arrange
        let (storage, _dir) = create_test_storage().await;

        // Create and store episode with embedding
        let episode = create_test_episode("Test episode");
        storage
            .store_episode(&episode)
            .await
            .expect("store episode");

        let embedding = create_test_embedding(384, 0.8);
        storage
            .store_episode_embedding(episode.episode_id, embedding.clone())
            .await
            .expect("store embedding");

        // Act - Query with similar embedding
        let query_embedding = create_test_embedding(384, 0.9);
        let results = storage
            .find_similar_episodes(query_embedding, 10, 0.5)
            .await
            .expect("find similar episodes");

        // Assert
        assert!(!results.is_empty(), "Should find similar episodes");
        assert_eq!(results[0].item.episode_id, episode.episode_id);
        assert!(
            results[0].similarity > 0.5,
            "Similarity should exceed threshold"
        );
    }

    #[tokio::test]
    #[ignore = "ADR-027: libsql memory corruption bug in CI"]
    async fn test_find_similar_episodes_threshold_filtering() {
        // Arrange
        let (storage, _dir) = create_test_storage().await;

        let episode1 = create_test_episode("Episode 1");
        let episode2 = create_test_episode("Episode 2");
        storage
            .store_episode(&episode1)
            .await
            .expect("store episode1");
        storage
            .store_episode(&episode2)
            .await
            .expect("store episode2");

        // Very similar embedding for episode1
        let embedding1 = create_test_embedding(384, 1.0);
        // Very different embedding for episode2
        let embedding2 = create_test_embedding(384, 0.0);

        storage
            .store_episode_embedding(episode1.episode_id, embedding1)
            .await
            .expect("store embedding1");
        storage
            .store_episode_embedding(episode2.episode_id, embedding2)
            .await
            .expect("store embedding2");

        // Act - Query similar to embedding1 with high threshold
        let query_embedding = create_test_embedding(384, 0.95);
        let results = storage
            .find_similar_episodes(query_embedding, 10, 0.9)
            .await
            .expect("find similar episodes");

        // Assert - Only episode1 should match high threshold
        assert!(!results.is_empty(), "Should find at least one episode");
        for result in &results {
            assert!(
                result.similarity >= 0.9,
                "All results should have similarity >= 0.9"
            );
        }
    }

    #[tokio::test]
    #[ignore = "ADR-027: libsql memory corruption bug in CI"]
    async fn test_find_similar_episodes_limit() {
        // Arrange
        let (storage, _dir) = create_test_storage().await;

        // Create 5 episodes with similar embeddings
        for i in 0..5 {
            let episode = create_test_episode(&format!("Episode {}", i));
            storage
                .store_episode(&episode)
                .await
                .expect("store episode");

            let embedding = create_test_embedding(384, 0.8 + i as f32 * 0.01);
            storage
                .store_episode_embedding(episode.episode_id, embedding)
                .await
                .expect("store embedding");
        }

        // Act - Limit to 3 results
        let query_embedding = create_test_embedding(384, 0.85);
        let results = storage
            .find_similar_episodes(query_embedding, 3, 0.5)
            .await
            .expect("find similar episodes");

        // Assert
        assert_eq!(results.len(), 3, "Should respect limit parameter");
    }

    #[tokio::test]
    #[ignore = "ADR-027: libsql memory corruption bug in CI"]
    async fn test_find_similar_episodes_sorted_by_similarity() {
        // Arrange
        let (storage, _dir) = create_test_storage().await;

        let episode_high = create_test_episode("High similarity");
        let episode_low = create_test_episode("Low similarity");
        storage
            .store_episode(&episode_high)
            .await
            .expect("store episode_high");
        storage
            .store_episode(&episode_low)
            .await
            .expect("store episode_low");

        // High similarity embedding
        let embedding_high = create_test_embedding(384, 1.0);
        // Low similarity embedding
        let embedding_low = create_test_embedding(384, 0.5);

        storage
            .store_episode_embedding(episode_high.episode_id, embedding_high)
            .await
            .expect("store embedding_high");
        storage
            .store_episode_embedding(episode_low.episode_id, embedding_low)
            .await
            .expect("store embedding_low");

        // Act
        let query_embedding = create_test_embedding(384, 0.95);
        let results = storage
            .find_similar_episodes(query_embedding, 10, 0.3)
            .await
            .expect("find similar episodes");

        // Assert - Results should be sorted by similarity (descending)
        assert!(results.len() >= 2, "Should find at least 2 episodes");
        for i in 1..results.len() {
            assert!(
                results[i - 1].similarity >= results[i].similarity,
                "Results should be sorted by similarity descending"
            );
        }
    }

    #[tokio::test]
    #[ignore = "ADR-027: libsql memory corruption bug in CI"]
    async fn test_find_similar_episodes_metadata_correct() {
        // Arrange
        let (storage, _dir) = create_test_storage().await;

        let episode = create_test_episode("Test episode");
        storage
            .store_episode(&episode)
            .await
            .expect("store episode");

        let embedding = create_test_embedding(384, 0.8);
        storage
            .store_episode_embedding(episode.episode_id, embedding.clone())
            .await
            .expect("store embedding");

        // Act
        let query_embedding = create_test_embedding(384, 0.8);
        let results = storage
            .find_similar_episodes(query_embedding, 10, 0.5)
            .await
            .expect("find similar episodes");

        // Assert - Metadata should contain dimension info
        assert!(!results.is_empty(), "Should find episodes");
        let metadata = &results[0].metadata;
        assert_eq!(metadata.embedding_model, "turso");
        // Context should contain dimension info
        let context_obj = &metadata.context;
        assert!(
            context_obj.get("dimension").is_some(),
            "Metadata should contain dimension"
        );
    }
}

// ============================================================================
// Pattern Similarity Search Tests
// ============================================================================

mod pattern_search_tests {
    use super::*;

    #[tokio::test]
    async fn test_find_similar_patterns_empty_database() {
        // Arrange
        let (storage, _dir) = create_test_storage().await;
        let query_embedding = create_test_embedding(384, 0.5);

        // Act
        let results = storage
            .find_similar_patterns(query_embedding, 10, 0.5)
            .await
            .expect("find similar patterns");

        // Assert
        assert!(
            results.is_empty(),
            "Should return empty results for empty database"
        );
    }

    #[tokio::test]
    async fn test_find_similar_patterns_with_embeddings() {
        // Arrange
        let (storage, _dir) = create_test_storage().await;

        // Create and store pattern with embedding
        let pattern = create_test_pattern("Test pattern");
        storage
            .store_pattern(&pattern)
            .await
            .expect("store pattern");

        let embedding = create_test_embedding(384, 0.8);
        storage
            .store_pattern_embedding(pattern.id(), embedding.clone())
            .await
            .expect("store embedding");

        // Act - Query with similar embedding
        let query_embedding = create_test_embedding(384, 0.9);
        let results = storage
            .find_similar_patterns(query_embedding, 10, 0.5)
            .await
            .expect("find similar patterns");

        // Assert
        assert!(!results.is_empty(), "Should find similar patterns");
        assert_eq!(results[0].item.id(), pattern.id());
        assert!(
            results[0].similarity > 0.5,
            "Similarity should exceed threshold"
        );
    }

    #[tokio::test]
    async fn test_find_similar_patterns_threshold_filtering() {
        // Arrange
        let (storage, _dir) = create_test_storage().await;

        let pattern1 = create_test_pattern("Pattern 1");
        let pattern2 = create_test_pattern("Pattern 2");
        storage
            .store_pattern(&pattern1)
            .await
            .expect("store pattern1");
        storage
            .store_pattern(&pattern2)
            .await
            .expect("store pattern2");

        // Very similar embedding for pattern1
        let embedding1 = create_test_embedding(384, 1.0);
        // Very different embedding for pattern2
        let embedding2 = create_test_embedding(384, 0.0);

        storage
            .store_pattern_embedding(pattern1.id(), embedding1)
            .await
            .expect("store embedding1");
        storage
            .store_pattern_embedding(pattern2.id(), embedding2)
            .await
            .expect("store embedding2");

        // Act - Query similar to embedding1 with high threshold
        let query_embedding = create_test_embedding(384, 0.95);
        let results = storage
            .find_similar_patterns(query_embedding, 10, 0.9)
            .await
            .expect("find similar patterns");

        // Assert - Only pattern1 should match high threshold
        for result in &results {
            assert!(
                result.similarity >= 0.9,
                "All results should have similarity >= 0.9"
            );
        }
    }

    #[tokio::test]
    async fn test_find_similar_patterns_limit() {
        // Arrange
        let (storage, _dir) = create_test_storage().await;

        // Create 5 patterns with similar embeddings
        for i in 0..5 {
            let pattern = create_test_pattern(&format!("Pattern {}", i));
            storage
                .store_pattern(&pattern)
                .await
                .expect("store pattern");

            let embedding = create_test_embedding(384, 0.8 + i as f32 * 0.01);
            storage
                .store_pattern_embedding(pattern.id(), embedding)
                .await
                .expect("store embedding");
        }

        // Act - Limit to 3 results
        let query_embedding = create_test_embedding(384, 0.85);
        let results = storage
            .find_similar_patterns(query_embedding, 3, 0.5)
            .await
            .expect("find similar patterns");

        // Assert
        assert_eq!(results.len(), 3, "Should respect limit parameter");
    }

    #[tokio::test]
    async fn test_find_similar_patterns_sorted_by_similarity() {
        // Arrange
        let (storage, _dir) = create_test_storage().await;

        let pattern_high = create_test_pattern("High similarity");
        let pattern_low = create_test_pattern("Low similarity");
        storage
            .store_pattern(&pattern_high)
            .await
            .expect("store pattern_high");
        storage
            .store_pattern(&pattern_low)
            .await
            .expect("store pattern_low");

        // High similarity embedding
        let embedding_high = create_test_embedding(384, 1.0);
        // Low similarity embedding
        let embedding_low = create_test_embedding(384, 0.5);

        storage
            .store_pattern_embedding(pattern_high.id(), embedding_high)
            .await
            .expect("store embedding_high");
        storage
            .store_pattern_embedding(pattern_low.id(), embedding_low)
            .await
            .expect("store embedding_low");

        // Act
        let query_embedding = create_test_embedding(384, 0.95);
        let results = storage
            .find_similar_patterns(query_embedding, 10, 0.3)
            .await
            .expect("find similar patterns");

        // Assert - Results should be sorted by similarity (descending)
        assert!(results.len() >= 2, "Should find at least 2 patterns");
        for i in 1..results.len() {
            assert!(
                results[i - 1].similarity >= results[i].similarity,
                "Results should be sorted by similarity descending"
            );
        }
    }

    #[tokio::test]
    async fn test_find_similar_patterns_metadata_correct() {
        // Arrange
        let (storage, _dir) = create_test_storage().await;

        let pattern = create_test_pattern("Test pattern");
        storage
            .store_pattern(&pattern)
            .await
            .expect("store pattern");

        let embedding = create_test_embedding(384, 0.8);
        storage
            .store_pattern_embedding(pattern.id(), embedding.clone())
            .await
            .expect("store embedding");

        // Act
        let query_embedding = create_test_embedding(384, 0.8);
        let results = storage
            .find_similar_patterns(query_embedding, 10, 0.5)
            .await
            .expect("find similar patterns");

        // Assert - Metadata should contain dimension info
        assert!(!results.is_empty(), "Should find patterns");
        let metadata = &results[0].metadata;
        assert_eq!(metadata.embedding_model, "turso");
        let context_obj = &metadata.context;
        assert!(
            context_obj.get("dimension").is_some(),
            "Metadata should contain dimension"
        );
    }
}

// ============================================================================
// Cosine Similarity Helper Tests
// ============================================================================

mod similarity_helper_tests {
    use do_memory_core::embeddings::cosine_similarity;

    #[test]
    fn test_cosine_similarity_identical_vectors() {
        // Arrange
        let vec1 = vec![1.0, 0.0, 0.0];
        let vec2 = vec![1.0, 0.0, 0.0];

        // Act
        let similarity = cosine_similarity(&vec1, &vec2);

        // Assert - Identical vectors should have similarity of 1.0
        assert!((similarity - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_cosine_similarity_opposite_vectors() {
        // Arrange
        let vec1 = vec![1.0, 0.0, 0.0];
        let vec2 = vec![-1.0, 0.0, 0.0];

        // Act
        let similarity = cosine_similarity(&vec1, &vec2);

        // Assert - Opposite vectors should have similarity of 0.0 after normalization from [-1,1] to [0,1]
        assert!((similarity - 0.0).abs() < 0.0001);
    }

    #[test]
    fn test_cosine_similarity_orthogonal_vectors() {
        // Arrange
        let vec1 = vec![1.0, 0.0, 0.0];
        let vec2 = vec![0.0, 1.0, 0.0];

        // Act
        let similarity = cosine_similarity(&vec1, &vec2);

        // Assert - Orthogonal vectors should have similarity of 0.5 after normalization from [-1,1] to [0,1]
        assert!((similarity - 0.5).abs() < 0.0001);
    }

    #[test]
    fn test_cosine_similarity_high_similarity() {
        // Arrange
        let vec1 = vec![0.9, 0.9, 0.9];
        let vec2 = vec![0.8, 0.8, 0.8];

        // Act
        let similarity = cosine_similarity(&vec1, &vec2);

        // Assert - Should be close to 1.0
        assert!(similarity > 0.99);
    }

    #[test]
    fn test_cosine_similarity_low_similarity() {
        // Arrange
        let vec1 = vec![1.0, 0.0, 0.0, 0.0];
        let vec2 = vec![0.0, 0.0, 0.0, 1.0];

        // Act
        let similarity = cosine_similarity(&vec1, &vec2);

        // Assert - Orthogonal vectors: 0.0 → 0.5 after normalization [-1,1]→[0,1]
        assert!((similarity - 0.5).abs() < 0.0001);
    }

    #[test]
    fn test_cosine_similarity_384_dimension_vectors() {
        // Arrange
        let vec1 = vec![0.5; 384];
        let vec2 = vec![0.6; 384];

        // Act
        let similarity = cosine_similarity(&vec1, &vec2);

        // Assert - Same direction vectors should have high similarity
        assert!(similarity > 0.99);
    }
}

// ============================================================================
// RecommendationStats Type Tests
// ============================================================================

mod recommendation_stats_tests {
    use do_memory_core::memory::attribution::RecommendationStats;

    #[test]
    fn test_recommendation_stats_default() {
        let stats = RecommendationStats::default();

        assert_eq!(stats.total_sessions, 0);
        assert_eq!(stats.total_feedback, 0);
        assert_eq!(stats.patterns_applied, 0);
        assert_eq!(stats.patterns_ignored, 0);
        assert_eq!(stats.successful_applications, 0);
        assert_eq!(stats.adoption_rate, 0.0);
        assert_eq!(stats.success_after_adoption_rate, 0.0);
        assert!(stats.avg_agent_rating.is_none());
    }

    #[test]
    fn test_recommendation_stats_serialization() {
        let stats = RecommendationStats {
            total_sessions: 10,
            total_feedback: 8,
            patterns_applied: 15,
            patterns_ignored: 5,
            successful_applications: 12,
            adoption_rate: 0.75,
            success_after_adoption_rate: 0.8,
            avg_agent_rating: Some(0.85),
        };

        // Serialize and deserialize
        let json = serde_json::to_string(&stats).expect("serialize");
        let deserialized: RecommendationStats = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.total_sessions, 10);
        assert_eq!(deserialized.avg_agent_rating, Some(0.85));
    }
}

// ============================================================================
// Integration-style Tests
// ============================================================================

mod integration_tests {
    use super::*;

    // NOTE: Tests in this module are ignored due to ADR-027 (libsql memory corruption bug in CI)

    #[tokio::test]
    #[ignore = "ADR-027: libsql memory corruption bug in CI"]
    async fn test_full_recommendation_workflow() {
        // Arrange
        let (storage, _dir) = create_test_storage().await;

        // Create episode
        let episode = create_test_episode("Recommendation workflow test");
        storage
            .store_episode(&episode)
            .await
            .expect("store episode");

        // Create session with recommendations
        let session = RecommendationSession {
            session_id: Uuid::new_v4(),
            episode_id: episode.episode_id,
            timestamp: Utc::now(),
            recommended_pattern_ids: vec!["pattern-a".to_string(), "pattern-b".to_string()],
            recommended_playbook_ids: vec![],
        };
        storage
            .store_recommendation_session(&session)
            .await
            .expect("store session");

        // Create feedback
        let feedback = RecommendationFeedback {
            session_id: session.session_id,
            applied_pattern_ids: vec!["pattern-a".to_string()],
            consulted_episode_ids: vec![episode.episode_id],
            outcome: TaskOutcome::Success {
                verdict: "Workflow completed".to_string(),
                artifacts: vec!["output.txt".to_string()],
            },
            agent_rating: Some(1.0),
        };
        storage
            .store_recommendation_feedback(&feedback)
            .await
            .expect("store feedback");

        // Act - Get stats
        let stats = storage.get_recommendation_stats().await.expect("get stats");

        // Assert
        assert_eq!(stats.total_sessions, 1);
        assert_eq!(stats.total_feedback, 1);
        assert_eq!(stats.patterns_applied, 1);
        assert_eq!(stats.patterns_ignored, 1);
        assert_eq!(stats.successful_applications, 1);
        assert_eq!(stats.success_after_adoption_rate, 1.0);
    }

    #[tokio::test]
    #[ignore = "ADR-027: libsql memory corruption bug in CI"]
    async fn test_search_with_recommendation_data() {
        // Arrange
        let (storage, _dir) = create_test_storage().await;

        // Create episode and pattern with embeddings
        let episode = create_test_episode("Search integration test");
        storage
            .store_episode(&episode)
            .await
            .expect("store episode");

        let pattern = create_test_pattern("Search pattern");
        storage
            .store_pattern(&pattern)
            .await
            .expect("store pattern");

        let episode_embedding = create_test_embedding(384, 0.8);
        let pattern_embedding = create_test_embedding(384, 0.9);

        storage
            .store_episode_embedding(episode.episode_id, episode_embedding.clone())
            .await
            .expect("store episode embedding");
        storage
            .store_pattern_embedding(pattern.id(), pattern_embedding.clone())
            .await
            .expect("store pattern embedding");

        // Create recommendation session for the episode
        let session = RecommendationSession {
            session_id: Uuid::new_v4(),
            episode_id: episode.episode_id,
            timestamp: Utc::now(),
            recommended_pattern_ids: vec![pattern.id().to_string()],
            recommended_playbook_ids: vec![],
        };
        storage
            .store_recommendation_session(&session)
            .await
            .expect("store session");

        // Act - Search for similar episodes and patterns
        let episode_results = storage
            .find_similar_episodes(create_test_embedding(384, 0.85), 5, 0.7)
            .await
            .expect("find episodes");

        let pattern_results = storage
            .find_similar_patterns(create_test_embedding(384, 0.95), 5, 0.7)
            .await
            .expect("find patterns");

        // Assert
        assert!(!episode_results.is_empty(), "Should find episodes");
        assert!(!pattern_results.is_empty(), "Should find patterns");

        // Verify recommendation session links episode to pattern
        let retrieved_session = storage
            .get_recommendation_session_for_episode(episode.episode_id)
            .await
            .expect("get session")
            .expect("session exists");

        assert!(
            retrieved_session
                .recommended_pattern_ids
                .contains(&pattern.id().to_string())
        );
    }
}
