//! Multi-dimension routing tests for Turso storage
//!
//! Tests that embeddings are routed to the correct dimension-specific tables.

#![allow(clippy::expect_used)]

use anyhow::Result;
use memory_core::{Episode, StorageBackend};
use memory_storage_turso::TursoStorage;
use tempfile::TempDir;
use test_utils::multi_dimension::{MultiDimensionTestHarness, table_for_dimension};

/// Helper to create test storage with initialized schema
async fn create_test_storage() -> Result<(TursoStorage, TempDir)> {
    let dir = TempDir::new()?;
    let db_path = dir.path().join("test.db");

    let url = format!(
        "file://{}",
        db_path.to_str().expect("temp path should be valid UTF-8")
    );
    let storage = TursoStorage::new(&url, "").await?;
    storage.initialize_schema().await?;
    Ok((storage, dir))
}

#[tokio::test]
async fn test_384_dimension_routing() {
    // This test will verify that 384-dim embeddings go to embeddings_384 table
    let harness = MultiDimensionTestHarness::new()
        .await
        .expect("Failed to create harness");

    let (episode, _embedding) = harness
        .create_episode_with_embedding(384, 42)
        .await
        .expect("Failed to create episode with embedding");

    // Verify the embedding was stored in the correct table
    let in_correct_table = harness
        .verify_table_usage(episode.episode_id, 384)
        .await
        .expect("Failed to verify table usage");

    assert!(
        in_correct_table,
        "384-dim embedding should be in embeddings_384 table"
    );
}

#[tokio::test]
async fn test_1536_dimension_routing() {
    // This test will verify that 1536-dim embeddings go to embeddings_1536 table
    let harness = MultiDimensionTestHarness::new()
        .await
        .expect("Failed to create harness");

    let (episode, _embedding) = harness
        .create_episode_with_embedding(1536, 42)
        .await
        .expect("Failed to create episode with embedding");

    let in_correct_table = harness
        .verify_table_usage(episode.episode_id, 1536)
        .await
        .expect("Failed to verify table usage");

    assert!(
        in_correct_table,
        "1536-dim embedding should be in embeddings_1536 table"
    );
}

#[tokio::test]
async fn test_unsupported_dimension_routing() {
    // This test will verify that non-standard dimensions go to embeddings_other table
    let harness = MultiDimensionTestHarness::new()
        .await
        .expect("Failed to create harness");

    let (episode, _embedding) = harness
        .create_episode_with_embedding(512, 42)
        .await
        .expect("Failed to create episode with embedding");

    let in_correct_table = harness
        .verify_table_usage(episode.episode_id, 512)
        .await
        .expect("Failed to verify table usage");

    assert!(
        in_correct_table,
        "512-dim embedding should be in embeddings_other table"
    );
}

#[tokio::test]
async fn test_mixed_dimension_routing() {
    // This test will verify mixed dimensions are routed correctly
    let harness = MultiDimensionTestHarness::new()
        .await
        .expect("Failed to create harness");

    let dimensions = [384, 1024, 1536, 3072, 500];

    for (i, &dim) in dimensions.iter().enumerate() {
        let (episode, _) = harness
            .create_episode_with_embedding(dim, 42 + i as u64)
            .await
            .expect("Failed to create episode with embedding");

        let in_correct_table = harness
            .verify_table_usage(episode.episode_id, dim)
            .await
            .expect("Failed to verify table usage");

        assert!(
            in_correct_table,
            "{}-dim embedding should be in {} table",
            dim,
            table_for_dimension(dim)
        );
    }
}

#[tokio::test]
async fn test_backward_compatibility_embedding_retrieval() {
    // Verify existing embedding retrieval still works after routing changes
    let (storage, _dir) = create_test_storage()
        .await
        .expect("Failed to create storage");

    // Create and store episode with embedding (using current API)
    use memory_core::types::{ComplexityLevel, TaskContext, TaskType};

    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: None,
        complexity: ComplexityLevel::Simple,
        domain: "web-api".to_string(),
        tags: vec![],
    };

    let mut episode = Episode::new("Test episode".to_string(), context, TaskType::Testing);
    episode.complete(memory_core::types::TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });

    storage
        .store_episode(&episode)
        .await
        .expect("Failed to store episode");

    // Store embedding using current API
    let embedding = vec![0.1_f32; 384];
    storage
        .store_embedding(&episode.episode_id.to_string(), embedding.clone())
        .await
        .expect("Failed to store embedding");

    // Retrieve embedding using current API
    let retrieved = storage
        .get_embedding(&episode.episode_id.to_string())
        .await
        .expect("Failed to retrieve embedding");

    assert!(retrieved.is_some(), "Should retrieve embedding");
    let retrieved_embedding = retrieved.unwrap();
    assert_eq!(retrieved_embedding.len(), 384);
    assert_eq!(retrieved_embedding, embedding);
}

#[tokio::test]
async fn test_vector_search_across_dimensions() {
    // Verify vector similarity search works across dimension tables
    let harness = MultiDimensionTestHarness::new()
        .await
        .expect("Failed to create harness");

    // Create episodes with embeddings of different dimensions
    let dimensions = [384, 1536];

    for (i, &dim) in dimensions.iter().enumerate() {
        let (episode, embedding) = harness
            .create_episode_with_embedding(dim, 100 + i as u64)
            .await
            .expect("Failed to create episode with embedding");

        // Search for similar episodes (should find the one we just created)
        let results = harness
            .run_similarity_search(embedding.clone(), 5, 0.5)
            .await
            .expect("Failed to run similarity search");

        assert!(
            !results.is_empty(),
            "Should find similar episodes for {} dimension",
            dim
        );

        // The most similar should be the episode itself (if threshold allows)
        let found = results.iter().any(|(id, _)| *id == episode.episode_id);
        assert!(
            found,
            "Should find the episode we just created for {} dimension",
            dim
        );
    }
}
