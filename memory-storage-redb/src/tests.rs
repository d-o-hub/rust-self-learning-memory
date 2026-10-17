//! Tests for redb storage backend.

use super::*;
use do_memory_core::StorageBackend;
use do_memory_core::embeddings::EmbeddingStorageBackend;
use do_memory_core::{Episode, TaskContext, TaskType};
use tempfile::tempdir;
use uuid::Uuid;

async fn create_test_storage() -> Result<RedbStorage> {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.redb");
    RedbStorage::new(&db_path).await
}

#[tokio::test]
pub async fn test_storage_creation() {
    let storage = create_test_storage().await;
    assert!(storage.is_ok());
}

#[tokio::test]
pub async fn test_health_check() {
    let storage = create_test_storage().await.unwrap();
    let healthy = storage.health_check().await.unwrap();
    assert!(healthy);
}

#[tokio::test]
pub async fn test_statistics() {
    let storage = create_test_storage().await.unwrap();
    let stats = storage.get_statistics().await.unwrap();
    assert_eq!(stats.episode_count, 0);
    assert_eq!(stats.pattern_count, 0);
    assert_eq!(stats.heuristic_count, 0);
}

#[tokio::test]
pub async fn test_clear_all() {
    let storage = create_test_storage().await.unwrap();
    let result = storage.clear_all().await;
    assert!(result.is_ok());
}

#[tokio::test]
pub async fn test_store_episodes_batch_empty_and_non_empty() {
    let storage = create_test_storage().await.unwrap();

    // 1. Empty batch
    let empty_res = storage.store_episodes_batch(&[]).await;
    assert!(empty_res.is_ok());

    // 2. Multiple episodes batch
    let ep1 = Episode::new(
        "Batch task 1".to_string(),
        TaskContext::default(),
        TaskType::Testing,
    );
    let ep2 = Episode::new(
        "Batch task 2".to_string(),
        TaskContext::default(),
        TaskType::Testing,
    );

    let batch = vec![ep1.clone(), ep2.clone()];
    storage.store_episodes_batch(&batch).await.unwrap();

    // Verify both retrieved
    let ret1 = storage.get_episode(ep1.episode_id).await.unwrap();
    assert!(ret1.is_some());
    assert_eq!(ret1.unwrap().task_description, "Batch task 1");

    let ret2 = storage.get_episode(ep2.episode_id).await.unwrap();
    assert!(ret2.is_some());
    assert_eq!(ret2.unwrap().task_description, "Batch task 2");
}

/// The cache derives `modified_at` from the allocated revision sequence, so a
/// batch must allocate a strictly increasing revision per episode. Sharing one
/// revision across a batch collapses every entry onto the same timestamp and
/// the keyset read then orders by `episode_id`, silently losing write order.
#[tokio::test]
pub async fn test_store_episodes_batch_preserves_revision_order() {
    let storage = create_test_storage().await.unwrap();
    let since = chrono::Utc::now() - chrono::Duration::hours(1);

    let episodes: Vec<Episode> = (0..50)
        .map(|i| {
            Episode::new(
                format!("Ordered task {i}"),
                TaskContext::default(),
                TaskType::Testing,
            )
        })
        .collect();
    storage.store_episodes_batch(&episodes).await.unwrap();

    let rows = storage
        .query_episodes_modified_since(since, None, Some(episodes.len()))
        .await
        .unwrap();
    assert_eq!(rows.len(), episodes.len(), "no episode may be dropped");

    let stamps: std::collections::HashSet<i64> =
        rows.iter().map(|(_, at)| at.timestamp_millis()).collect();
    assert_eq!(
        stamps.len(),
        rows.len(),
        "each episode must receive its own revision sequence"
    );

    let read_ids: Vec<Uuid> = rows.iter().map(|(episode, _)| episode.episode_id).collect();
    let written_ids: Vec<Uuid> = episodes.iter().map(|episode| episode.episode_id).collect();
    assert_eq!(
        read_ids, written_ids,
        "a batch write must preserve write order in the cache"
    );
}

// ========== Embedding Storage Tests ==========

#[tokio::test]
pub async fn test_store_and_get_embedding() {
    let storage = create_test_storage().await.unwrap();

    let id = "test_embedding_1";
    let embedding = vec![0.1_f32, 0.2, 0.3, 0.4];

    // Store embedding
    storage
        .store_embedding(id, embedding.clone())
        .await
        .unwrap();

    // Retrieve embedding
    let retrieved = storage.get_embedding(id).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), embedding);
}

#[tokio::test]
pub async fn test_get_nonexistent_embedding() {
    let storage = create_test_storage().await.unwrap();

    let retrieved = storage.get_embedding("nonexistent").await.unwrap();
    assert!(retrieved.is_none());
}

#[tokio::test]
pub async fn test_delete_embedding() {
    let storage = create_test_storage().await.unwrap();

    let id = "test_embedding_delete";
    let embedding = vec![0.1_f32, 0.2, 0.3];

    // Store embedding
    storage
        .store_embedding(id, embedding.clone())
        .await
        .unwrap();

    // Verify it exists
    let retrieved = storage.get_embedding(id).await.unwrap();
    assert!(retrieved.is_some());

    // Delete embedding
    let deleted = storage.delete_embedding(id).await.unwrap();
    assert!(deleted);

    // Verify it's gone
    let retrieved = storage.get_embedding(id).await.unwrap();
    assert!(retrieved.is_none());
}

#[tokio::test]
pub async fn test_delete_nonexistent_embedding() {
    let storage = create_test_storage().await.unwrap();

    let deleted = storage.delete_embedding("nonexistent").await.unwrap();
    assert!(!deleted);
}

#[tokio::test]
pub async fn test_store_embeddings_batch() {
    let storage = create_test_storage().await.unwrap();

    let embeddings = vec![
        ("batch_1".to_string(), vec![0.1_f32, 0.2, 0.3]),
        ("batch_2".to_string(), vec![0.4_f32, 0.5, 0.6]),
        ("batch_3".to_string(), vec![0.7_f32, 0.8, 0.9]),
    ];

    // Store embeddings in batch
    storage
        .store_embeddings_batch(embeddings.clone())
        .await
        .unwrap();

    // Verify all embeddings were stored
    for (id, expected_embedding) in &embeddings {
        let retrieved = storage.get_embedding(id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), *expected_embedding);
    }
}

#[tokio::test]
pub async fn test_get_embeddings_batch() {
    let storage = create_test_storage().await.unwrap();

    let embeddings = vec![
        ("get_batch_1".to_string(), vec![0.1_f32, 0.2]),
        ("get_batch_2".to_string(), vec![0.3_f32, 0.4]),
        ("get_batch_3".to_string(), vec![0.5_f32, 0.6]),
    ];

    // Store embeddings
    storage
        .store_embeddings_batch(embeddings.clone())
        .await
        .unwrap();

    // Get embeddings in batch
    let ids = vec![
        "get_batch_1".to_string(),
        "get_batch_2".to_string(),
        "get_batch_3".to_string(),
        "nonexistent".to_string(),
    ];

    let results = storage.get_embeddings_batch(&ids).await.unwrap();

    // Verify results
    assert_eq!(results.len(), 4);

    assert!(results[0].is_some());
    assert_eq!(results[0].as_ref().unwrap(), &embeddings[0].1);

    assert!(results[1].is_some());
    assert_eq!(results[1].as_ref().unwrap(), &embeddings[1].1);

    assert!(results[2].is_some());
    assert_eq!(results[2].as_ref().unwrap(), &embeddings[2].1);

    assert!(results[3].is_none()); // Nonexistent embedding
}

#[tokio::test]
pub async fn test_different_embedding_dimensions() {
    let storage = create_test_storage().await.unwrap();

    // Test different dimensions (384, 1024, 1536)
    let dim_384: Vec<f32> = (0..384).map(|i| i as f32 / 384.0).collect();
    let dim_1024: Vec<f32> = (0..1024).map(|i| i as f32 / 1024.0).collect();
    let dim_1536: Vec<f32> = (0..1536).map(|i| i as f32 / 1536.0).collect();

    // Store different dimensions
    storage.store_embedding("dim_384", dim_384).await.unwrap();

    storage.store_embedding("dim_1024", dim_1024).await.unwrap();

    storage.store_embedding("dim_1536", dim_1536).await.unwrap();

    // Retrieve and verify dimensions
    let retrieved_384 = storage.get_embedding("dim_384").await.unwrap();
    assert!(retrieved_384.is_some());
    assert_eq!(retrieved_384.unwrap().len(), 384);

    let retrieved_1024 = storage.get_embedding("dim_1024").await.unwrap();
    assert!(retrieved_1024.is_some());
    assert_eq!(retrieved_1024.unwrap().len(), 1024);

    let retrieved_1536 = storage.get_embedding("dim_1536").await.unwrap();
    assert!(retrieved_1536.is_some());
    assert_eq!(retrieved_1536.unwrap().len(), 1536);
}

#[tokio::test]
pub async fn test_update_existing_embedding() {
    let storage = create_test_storage().await.unwrap();

    let id = "update_test";
    let embedding_v1 = vec![0.1_f32, 0.2, 0.3];
    let embedding_v2 = vec![0.9_f32, 0.8, 0.7];

    // Store initial embedding
    storage
        .store_embedding(id, embedding_v1.clone())
        .await
        .unwrap();

    // Verify initial embedding
    let retrieved = storage.get_embedding(id).await.unwrap();
    assert_eq!(retrieved.unwrap(), embedding_v1);

    // Update embedding
    storage
        .store_embedding(id, embedding_v2.clone())
        .await
        .unwrap();

    // Verify updated embedding
    let retrieved = storage.get_embedding(id).await.unwrap();
    assert_eq!(retrieved.unwrap(), embedding_v2);
}

#[tokio::test]
pub async fn test_empty_embeddings_batch() {
    let storage = create_test_storage().await.unwrap();

    // Store empty batch
    storage.store_embeddings_batch(vec![]).await.unwrap();

    // Get empty batch
    let results = storage.get_embeddings_batch(&[]).await.unwrap();
    assert_eq!(results.len(), 0);
}

#[tokio::test]
pub async fn test_embedding_size_limit() {
    let storage = create_test_storage().await.unwrap();

    // Test that embeddings larger than MAX_EMBEDDING_SIZE (1MB) are rejected
    let large_embedding: Vec<f32> = vec![0.0_f32; 1_000_000]; // ~4MB

    let result = storage.store_embedding("too_large", large_embedding).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("exceeds maximum"));
}

#[tokio::test]
pub async fn test_find_similar_episodes_extended() {
    let storage = create_test_storage().await.unwrap();
    let episode_id_1 = Uuid::new_v4();
    let episode_id_2 = Uuid::new_v4();
    let episode_id_3 = Uuid::new_v4();

    // 1. Arrange: Create episodes and embeddings
    let mut ep1 = Episode::new(
        "episode 1".to_string(),
        TaskContext::default(),
        TaskType::Other,
    );
    ep1.episode_id = episode_id_1;

    let mut ep2 = Episode::new(
        "episode 2".to_string(),
        TaskContext::default(),
        TaskType::Other,
    );
    ep2.episode_id = episode_id_2;

    // ep3 will be "missing" (embedding exists, but no episode)

    storage.store_episode(&ep1).await.unwrap();
    storage.store_episode(&ep2).await.unwrap();

    // Store embeddings with known similarities
    storage
        .store_episode_embedding(episode_id_1, vec![1.0, 0.0])
        .await
        .unwrap();
    storage
        .store_episode_embedding(episode_id_2, vec![-1.0, 0.0])
        .await
        .unwrap();
    storage
        .store_episode_embedding(episode_id_3, vec![0.5, 0.0])
        .await
        .unwrap();

    let query = vec![1.0, 0.0];

    // 2. Act: Search with threshold pruning
    let results = storage
        .find_similar_episodes(query.clone(), 10, 0.8)
        .await
        .unwrap();

    // 3. Assert: ep1 should be found, ep3 skipped (missing), ep2 skipped (threshold)
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].item.episode_id, episode_id_1);
    assert!((results[0].similarity - 1.0).abs() < f32::EPSILON);

    // 4. Act: Search with exact limit
    let results = storage.find_similar_episodes(query, 1, 0.0).await.unwrap();
    assert_eq!(results.len(), 1);
}

#[tokio::test]
pub async fn test_deterministic_tie_breaking() {
    let storage = create_test_storage().await.unwrap();

    // Create two episodes with SAME similarity
    let id_a = Uuid::parse_str("00000000-0000-0000-0000-00000000000a").unwrap();
    let id_b = Uuid::parse_str("00000000-0000-0000-0000-00000000000b").unwrap();

    let mut ep_a = Episode::new("A".to_string(), TaskContext::default(), TaskType::Other);
    ep_a.episode_id = id_a;

    let mut ep_b = Episode::new("B".to_string(), TaskContext::default(), TaskType::Other);
    ep_b.episode_id = id_b;

    storage.store_episode(&ep_a).await.unwrap();
    storage.store_episode(&ep_b).await.unwrap();

    // Same embedding -> same similarity
    let vec = vec![1.0, 0.0];
    storage
        .store_episode_embedding(id_a, vec.clone())
        .await
        .unwrap();
    storage
        .store_episode_embedding(id_b, vec.clone())
        .await
        .unwrap();

    // Search with limit 2
    let results = storage.find_similar_episodes(vec, 2, 0.0).await.unwrap();

    assert_eq!(results.len(), 2);
    // Should be ordered by ID: A then B
    assert_eq!(results[0].item.episode_id, id_a);
    assert_eq!(results[1].item.episode_id, id_b);
}

#[tokio::test]
pub async fn test_find_similar_episodes_exact_limit() {
    let storage = create_test_storage().await.unwrap();

    // Create 5 episodes
    for i in 0..5 {
        let id = Uuid::new_v4();
        let mut ep = Episode::new(format!("ep {}", i), TaskContext::default(), TaskType::Other);
        ep.episode_id = id;
        storage.store_episode(&ep).await.unwrap();
        storage
            .store_episode_embedding(id, vec![i as f32, 0.0])
            .await
            .unwrap();
    }

    // Search with limit 3 (less than available)
    let results = storage
        .find_similar_episodes(vec![10.0, 0.0], 3, 0.0)
        .await
        .unwrap();
    assert_eq!(results.len(), 3);

    // Search with limit 5 (equal to available)
    let results = storage
        .find_similar_episodes(vec![10.0, 0.0], 5, 0.0)
        .await
        .unwrap();
    assert_eq!(results.len(), 5);

    // Search with limit 10 (more than available)
    let results = storage
        .find_similar_episodes(vec![10.0, 0.0], 10, 0.0)
        .await
        .unwrap();
    assert_eq!(results.len(), 5);
}

#[tokio::test]
pub async fn test_weighted_relationships_parity() {
    let storage = create_test_storage().await.unwrap();
    let ep1 = Uuid::new_v4();
    let ep2 = Uuid::new_v4();

    let mut metadata = do_memory_core::episode::RelationshipMetadata::new();
    metadata.weight = Some(0.7);

    let rel = do_memory_core::episode::EpisodeRelationship::new(
        ep1,
        ep2,
        do_memory_core::episode::RelationshipType::RelatedTo,
        metadata,
    );

    storage.store_relationship(&rel).await.unwrap();

    let retrieved = storage
        .get_relationships(ep1, do_memory_core::episode::Direction::Outgoing)
        .await
        .unwrap();
    assert_eq!(retrieved.len(), 1);
    assert_eq!(retrieved[0].metadata.weight, Some(0.7));
}

#[tokio::test]
pub async fn test_episode_pattern_relationships_parity() {
    let storage = create_test_storage().await.unwrap();
    let ep_id = Uuid::new_v4();
    let pt_id = Uuid::new_v4();

    let mut metadata = do_memory_core::episode::RelationshipMetadata::new();
    metadata.weight = Some(0.42);

    let rel = do_memory_core::episode::EpisodePatternRelationship::new(
        ep_id,
        pt_id,
        do_memory_core::episode::RelationshipType::References,
        metadata,
    );

    storage
        .store_episode_pattern_relationship(&rel)
        .await
        .unwrap();

    let retrieved = storage
        .get_episode_pattern_relationships(ep_id)
        .await
        .unwrap();
    assert_eq!(retrieved.len(), 1);
    assert_eq!(retrieved[0].pattern_id, pt_id);
    assert_eq!(retrieved[0].metadata.weight, Some(0.42));
}

#[tokio::test]
pub async fn test_get_weighted_neighbors_parity() {
    let storage = create_test_storage().await.unwrap();
    let ep1 = Uuid::new_v4();
    let ep2 = Uuid::new_v4();
    let pt1 = Uuid::new_v4();

    // 1. Episode-Episode
    let mut meta1 = do_memory_core::episode::RelationshipMetadata::new();
    meta1.weight = Some(0.6);
    let rel1 = do_memory_core::episode::EpisodeRelationship::new(
        ep1,
        ep2,
        do_memory_core::episode::RelationshipType::RelatedTo,
        meta1,
    );
    storage.store_relationship(&rel1).await.unwrap();

    // 2. Episode-Pattern
    let mut meta2 = do_memory_core::episode::RelationshipMetadata::new();
    meta2.weight = Some(0.3);
    let rel2 = do_memory_core::episode::EpisodePatternRelationship::new(
        ep1,
        pt1,
        do_memory_core::episode::RelationshipType::References,
        meta2,
    );
    storage
        .store_episode_pattern_relationship(&rel2)
        .await
        .unwrap();

    let neighbors = storage.get_weighted_neighbors(ep1).await.unwrap();
    assert_eq!(neighbors.len(), 2);

    let ep2_neighbor = neighbors
        .iter()
        .find(|(id, _, is_pt)| *id == ep2 && !*is_pt)
        .unwrap();
    let pt1_neighbor = neighbors
        .iter()
        .find(|(id, _, is_pt)| *id == pt1 && *is_pt)
        .unwrap();

    assert_eq!(ep2_neighbor.1, 0.6);
    assert_eq!(pt1_neighbor.1, 0.3);
}
