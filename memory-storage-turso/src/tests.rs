//! # Turso Storage Tests
//!
//! Integration tests for Turso storage backend.

use super::*;
use do_memory_core::StorageBackend;
use std::sync::Arc;
use tempfile::TempDir;

/// Helper function to create a 384-dimensional test embedding
/// Required because the embeddings table uses F32_BLOB(384) fixed dimension
fn create_test_embedding_384() -> Vec<f32> {
    // Create a 384-dimensional embedding with normalized values
    let mut embedding = Vec::with_capacity(384);
    for i in 0..384 {
        embedding.push(0.01_f32 * (i as f32 % 100.0 + 1.0));
    }
    embedding
}

/// Helper function to create a 384-dimensional embedding with specific seed value
fn create_test_embedding_384_with_seed(seed: f32) -> Vec<f32> {
    let mut embedding = Vec::with_capacity(384);
    for i in 0..384 {
        embedding.push(seed + 0.001_f32 * (i as f32));
    }
    embedding
}

async fn create_test_storage() -> Result<(TursoStorage, TempDir)> {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test.db");

    // Use Builder::new_local for file-based databases
    let db = libsql::Builder::new_local(&db_path)
        .build()
        .await
        .map_err(|e| Error::Storage(format!("Failed to create test database: {}", e)))?;

    let storage = TursoStorage {
        db: Arc::new(db),
        pool: None,
        #[cfg(feature = "keepalive-pool")]
        keepalive_pool: None,
        adaptive_pool: None,
        caching_pool: None,
        prepared_cache: Arc::new(crate::PreparedStatementCache::with_config(
            crate::PreparedCacheConfig::default(),
        )),
        config: TursoConfig::default(),
        #[cfg(feature = "compression")]
        compression_stats: Arc::new(std::sync::Mutex::new(
            crate::CompressionStatistics::default(),
        )),
        #[cfg(feature = "adaptive-ttl")]
        episode_cache: None,
    };

    storage.initialize_schema().await?;
    Ok((storage, dir))
}

#[tokio::test]
async fn test_storage_creation() {
    let result = create_test_storage().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_health_check() {
    let (storage, _dir) = create_test_storage().await.unwrap();
    let healthy = storage.health_check().await.unwrap();
    assert!(healthy);
}

#[tokio::test]
async fn test_statistics() {
    let (storage, _dir) = create_test_storage().await.unwrap();
    let stats = storage.get_statistics().await.unwrap();
    assert_eq!(stats.episode_count, 0);
    assert_eq!(stats.pattern_count, 0);
    assert_eq!(stats.heuristic_count, 0);
}

// ========== Embedding Storage Tests ==========

#[tokio::test]
async fn test_store_and_get_embedding() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let id = "test_embedding_1";
    let embedding = create_test_embedding_384(); // Use 384-dim embedding for F32_BLOB(384) column

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
    let (storage, _dir) = create_test_storage().await.unwrap();

    let retrieved = storage.get_embedding("nonexistent").await.unwrap();
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_delete_embedding() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let id = "test_embedding_delete";
    let embedding = create_test_embedding_384(); // Use 384-dim embedding

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
    let (storage, _dir) = create_test_storage().await.unwrap();

    let deleted = storage.delete_embedding("nonexistent").await.unwrap();
    assert!(!deleted);
}

#[tokio::test]
async fn test_store_embeddings_batch() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    // Use 384-dimensional embeddings for F32_BLOB(384) column
    let embeddings = vec![
        (
            "batch_1".to_string(),
            create_test_embedding_384_with_seed(0.1),
        ),
        (
            "batch_2".to_string(),
            create_test_embedding_384_with_seed(0.2),
        ),
        (
            "batch_3".to_string(),
            create_test_embedding_384_with_seed(0.3),
        ),
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
    let (storage, _dir) = create_test_storage().await.unwrap();

    // Use 384-dimensional embeddings for F32_BLOB(384) column
    let embeddings = vec![
        (
            "get_batch_1".to_string(),
            create_test_embedding_384_with_seed(0.1),
        ),
        (
            "get_batch_2".to_string(),
            create_test_embedding_384_with_seed(0.2),
        ),
        (
            "get_batch_3".to_string(),
            create_test_embedding_384_with_seed(0.3),
        ),
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

/// Test different embedding dimensions (requires turso_multi_dimension feature)
/// Without this feature, only 384-dimension embeddings are supported via F32_BLOB(384)
#[tokio::test]
#[cfg(feature = "turso_multi_dimension")]
async fn test_different_embedding_dimensions() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    // Test different dimensions (384, 1024, 1536)
    let dim_384: Vec<f32> = (0..384).map(|i| i as f32 / 384.0).collect();
    let dim_1024: Vec<f32> = (0..1024).map(|i| i as f32 / 1024.0).collect();
    let dim_1536: Vec<f32> = (0..1536).map(|i| i as f32 / 1536.0).collect();

    // Store different dimensions
    storage
        .store_embedding("dim_384", dim_384.clone())
        .await
        .unwrap();

    storage
        .store_embedding("dim_1024", dim_1024.clone())
        .await
        .unwrap();

    storage
        .store_embedding("dim_1536", dim_1536.clone())
        .await
        .unwrap();

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
    let (storage, _dir) = create_test_storage().await.unwrap();

    let id = "update_test";
    let embedding_v1 = create_test_embedding_384_with_seed(0.1); // Use 384-dim embeddings
    let embedding_v2 = create_test_embedding_384_with_seed(0.9);

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
    let (storage, _dir) = create_test_storage().await.unwrap();

    // Store empty batch
    storage.store_embeddings_batch(vec![]).await.unwrap();

    // Get empty batch
    let results = storage.get_embeddings_batch(&[]).await.unwrap();
    assert!(results.is_empty());
}

// ========== Compression Integration Tests ==========

#[cfg(feature = "compression")]
mod compression_tests {
    use super::*;
    use do_memory_core::StorageBackend;

    /// Test that large episodes are compressed and retrieved correctly
    #[tokio::test]
    async fn test_large_episode_compression() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        // Create a large episode with many steps
        let mut steps = Vec::new();
        for i in 0..100 {
            steps.push(do_memory_core::episode::ExecutionStep {
                step_number: i,
                tool: format!("tool_{}", i % 10),
                action: format!("action_{}", i),
                parameters: serde_json::json!({
                    "param": format!("value_{}", i),
                    "data": "x".repeat(100) // Add some repeatable data
                }),
                result: Some(do_memory_core::types::ExecutionResult::Success {
                    output: format!("output_{}", i),
                }),
                latency_ms: i as u64,
                timestamp: chrono::Utc::now(),
                tokens_used: None,
                metadata: std::collections::HashMap::new(),
            });
        }

        let episode = do_memory_core::Episode {
            episode_id: uuid::Uuid::new_v4(),
            task_type: do_memory_core::TaskType::CodeGeneration,
            task_description: "Test large episode compression".to_string(),
            context: do_memory_core::TaskContext {
                domain: "test".to_string(),
                language: Some("rust".to_string()),
                framework: None,
                complexity: do_memory_core::types::ComplexityLevel::Complex,
                tags: vec!["compression".to_string()],
            },
            steps,
            outcome: None,
            reward: None,
            reflection: None,
            patterns: vec![],
            heuristics: vec![],
            applied_patterns: vec![],
            salient_features: None,
            tags: vec![],
            checkpoints: vec![],
            start_time: chrono::Utc::now(),
            end_time: None,
            metadata: std::collections::HashMap::new(),
        };

        // Store episode
        storage.store_episode(&episode).await.unwrap();

        // Retrieve episode
        let retrieved = storage.get_episode(episode.episode_id).await.unwrap();
        assert!(retrieved.is_some());

        let retrieved_episode = retrieved.unwrap();
        assert_eq!(retrieved_episode.steps.len(), 100);
        assert_eq!(retrieved_episode.task_description, episode.task_description);
    }

    /// Test that compression is skipped for small episodes
    #[tokio::test]
    async fn test_small_episode_no_compression() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        // Create a small episode
        let episode = do_memory_core::Episode {
            episode_id: uuid::Uuid::new_v4(),
            task_type: do_memory_core::TaskType::Analysis,
            task_description: "Test small episode without compression".to_string(),
            context: do_memory_core::TaskContext {
                domain: "test".to_string(),
                language: Some("rust".to_string()),
                framework: None,
                complexity: do_memory_core::types::ComplexityLevel::Simple,
                tags: vec![],
            },
            steps: vec![],
            outcome: None,
            reward: None,
            reflection: None,
            patterns: vec![],
            heuristics: vec![],
            applied_patterns: vec![],
            salient_features: None,
            tags: vec![],
            checkpoints: vec![],
            start_time: chrono::Utc::now(),
            end_time: None,
            metadata: std::collections::HashMap::new(),
        };

        // Store and retrieve
        storage.store_episode(&episode).await.unwrap();
        let retrieved = storage.get_episode(episode.episode_id).await.unwrap();

        assert!(retrieved.is_some());
        assert_eq!(
            retrieved.unwrap().task_description,
            episode.task_description
        );
    }

    /// Test embedding compression
    #[tokio::test]
    async fn test_embedding_compression() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        // Create a 384-dimensional embedding (required for F32_BLOB(384) column)
        let embedding: Vec<f32> = (0..384).map(|i| (i as f32 / 384.0).sin()).collect();

        // Store embedding
        storage
            .store_embedding("test_compressed_embedding", embedding.clone())
            .await
            .unwrap();

        // Retrieve embedding
        let retrieved = storage
            .get_embedding("test_compressed_embedding")
            .await
            .unwrap();
        assert!(retrieved.is_some());

        let retrieved_embedding = retrieved.unwrap();
        assert_eq!(retrieved_embedding.len(), 384); // Match 384-dim input

        // Verify values match (with some tolerance for float precision)
        for (original, retrieved) in embedding.iter().zip(retrieved_embedding.iter()) {
            assert!((original - retrieved).abs() < 1e-5);
        }
    }

    /// Test compression statistics
    #[cfg(feature = "compression")]
    #[tokio::test]
    async fn test_compression_statistics() {
        use crate::CompressionStatistics;

        let mut stats = CompressionStatistics::new();

        // Record some fake compression operations
        stats.record_compression(1000, 400, 50);
        stats.record_compression(2000, 800, 100);
        stats.record_skipped();
        stats.record_decompression(75);

        // Verify statistics
        assert_eq!(stats.total_original_bytes, 3000);
        assert_eq!(stats.total_compressed_bytes, 1200);
        assert_eq!(stats.compression_count, 2);
        assert_eq!(stats.skipped_count, 1);
        assert_eq!(stats.compression_time_us, 150);
        assert_eq!(stats.decompression_time_us, 75);

        // Verify compression ratio (1200/3000 = 0.4)
        let ratio = stats.compression_ratio();
        assert!(ratio > 0.35 && ratio < 0.45);

        // Verify bandwidth savings (60%)
        let savings = stats.bandwidth_savings_percent();
        assert!(savings > 55.0 && savings < 65.0);
    }
}
