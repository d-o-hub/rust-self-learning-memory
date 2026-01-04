//! Tests for redb storage backend.

use super::*;
use tempfile::tempdir;

async fn create_test_storage() -> Result<RedbStorage> {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.redb");
    RedbStorage::new(&db_path).await
}

#[tokio::test]
async fn test_storage_creation() {
    let storage = create_test_storage().await;
    assert!(storage.is_ok());
}

#[tokio::test]
async fn test_health_check() {
    let storage = create_test_storage().await.unwrap();
    let healthy = storage.health_check().await.unwrap();
    assert!(healthy);
}

#[tokio::test]
async fn test_statistics() {
    let storage = create_test_storage().await.unwrap();
    let stats = storage.get_statistics().await.unwrap();
    assert_eq!(stats.episode_count, 0);
    assert_eq!(stats.pattern_count, 0);
    assert_eq!(stats.heuristic_count, 0);
}

#[tokio::test]
async fn test_clear_all() {
    let storage = create_test_storage().await.unwrap();
    let result = storage.clear_all().await;
    assert!(result.is_ok());
}

// ========== Embedding Storage Tests ==========

#[tokio::test]
async fn test_store_and_get_embedding() {
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
async fn test_get_nonexistent_embedding() {
    let storage = create_test_storage().await.unwrap();

    let retrieved = storage.get_embedding("nonexistent").await.unwrap();
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_delete_embedding() {
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
async fn test_delete_nonexistent_embedding() {
    let storage = create_test_storage().await.unwrap();

    let deleted = storage.delete_embedding("nonexistent").await.unwrap();
    assert!(!deleted);
}

#[tokio::test]
async fn test_store_embeddings_batch() {
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
async fn test_get_embeddings_batch() {
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
async fn test_different_embedding_dimensions() {
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
async fn test_update_existing_embedding() {
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
async fn test_empty_embeddings_batch() {
    let storage = create_test_storage().await.unwrap();

    // Store empty batch
    storage.store_embeddings_batch(vec![]).await.unwrap();

    // Get empty batch
    let results = storage.get_embeddings_batch(&[]).await.unwrap();
    assert!(results.is_empty());
}

#[tokio::test]
async fn test_embedding_size_limit() {
    let storage = create_test_storage().await.unwrap();

    // Test that embeddings larger than MAX_EMBEDDING_SIZE (1MB) are rejected
    let large_embedding: Vec<f32> = vec![0.0_f32; 1_000_000]; // ~4MB

    let result = storage.store_embedding("too_large", large_embedding).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("exceeds maximum"));
}
