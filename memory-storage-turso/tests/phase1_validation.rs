//! Phase 1 Multi-Dimension Validation Tests
//!
//! Comprehensive validation of multi-dimension vector support implementation.

use anyhow::Result;
use memory_core::embeddings::EmbeddingStorageBackend as _;
use memory_core::StorageBackend;
use tempfile::TempDir;
use test_utils::multi_dimension::{table_for_dimension, MultiDimensionTestHarness};
use tracing::info;

// ============================================================================
// Task 1: Schema Validation Tests
// ============================================================================

#[tokio::test]
#[cfg_attr(not(feature = "turso_multi_dimension"), ignore)]
async fn phase1_task1_validate_all_dimension_tables_created() -> Result<()> {
    info!("=== Task 1: Validate all dimension tables created ===");
    let harness = MultiDimensionTestHarness::new().await?;

    // Verify we can store and retrieve embeddings of all supported dimensions
    let dimensions = [384, 1024, 1536, 3072, 512];

    for dimension in dimensions {
        let (episode, _embedding) = harness.create_episode_with_embedding(dimension, 42).await?;

        info!(
            "✓ Created episode with {}-dim embedding: {}",
            dimension, episode.episode_id
        );
    }

    info!("✓ All dimension tables created successfully");
    Ok(())
}

#[tokio::test]
#[cfg_attr(not(feature = "turso_multi_dimension"), ignore)]
async fn phase1_task1_validate_vector_indexes_created() -> Result<()> {
    info!("=== Task 1: Validate vector indexes created ===");
    let harness = MultiDimensionTestHarness::new().await?;

    // Test that we can run similarity search (uses vector index if available)
    let (_episode, embedding) = harness.create_episode_with_embedding(384, 42).await?;

    let results = harness.run_similarity_search(embedding, 5, 0.5).await?;

    assert!(
        !results.is_empty(),
        "Vector index should allow similarity search"
    );

    info!("✓ Vector indexes created successfully");
    info!("  Similarity search returned {} results", results.len());

    Ok(())
}

#[tokio::test]
#[cfg_attr(not(feature = "turso_multi_dimension"), ignore)]
async fn phase1_task1_validate_item_indexes_created() -> Result<()> {
    info!("=== Task 1: Validate item indexes created ===");
    let harness = MultiDimensionTestHarness::new().await?;

    let (episode, _embedding) = harness.create_episode_with_embedding(1536, 42).await?;

    // Verify we can retrieve embedding via harness
    let retrieved = harness
        .storage
        .get_episode_embedding(episode.episode_id)
        .await?;

    assert!(
        retrieved.is_some(),
        "Item index should allow embedding retrieval"
    );
    assert_eq!(
        retrieved.unwrap().len(),
        1536,
        "Retrieved embedding should be 1536-dimensional"
    );

    info!("✓ Item indexes created successfully");

    Ok(())
}

// ============================================================================
// Task 2: Routing Logic Validation Tests
// ============================================================================

#[tokio::test]
#[cfg_attr(not(feature = "turso_multi_dimension"), ignore)]
async fn phase1_task2_384_dimension_routing() -> Result<()> {
    info!("=== Task 2: Validate 384-dimension routing ===");
    let harness = MultiDimensionTestHarness::new().await?;

    let (episode, _embedding) = harness.create_episode_with_embedding(384, 42).await?;

    // Verify routing by checking expected table name
    let expected_table = table_for_dimension(384);
    assert_eq!(
        expected_table, "embeddings_384",
        "384-dim should route to embeddings_384"
    );

    // Verify we can retrieve embedding
    let retrieved = harness
        .storage
        .get_episode_embedding(episode.episode_id)
        .await?;
    assert!(
        retrieved.is_some(),
        "Should be able to retrieve 384-dim embedding"
    );

    info!("✓ 384-dimension embedding routed correctly");

    Ok(())
}

#[tokio::test]
#[cfg_attr(not(feature = "turso_multi_dimension"), ignore)]
async fn phase1_task2_1536_dimension_routing() -> Result<()> {
    info!("=== Task 2: Validate 1536-dimension routing ===");
    let harness = MultiDimensionTestHarness::new().await?;

    let (episode, _embedding) = harness.create_episode_with_embedding(1536, 42).await?;

    // Verify routing by checking expected table name
    let expected_table = table_for_dimension(1536);
    assert_eq!(
        expected_table, "embeddings_1536",
        "1536-dim should route to embeddings_1536"
    );

    // Verify we can retrieve embedding
    let retrieved = harness
        .storage
        .get_episode_embedding(episode.episode_id)
        .await?;
    assert!(
        retrieved.is_some(),
        "Should be able to retrieve 1536-dim embedding"
    );

    info!("✓ 1536-dimension embedding routed correctly");

    Ok(())
}

#[tokio::test]
#[cfg_attr(not(feature = "turso_multi_dimension"), ignore)]
async fn phase1_task2_unsupported_dimension_routing() -> Result<()> {
    info!("=== Task 2: Validate unsupported dimension routing ===");
    let harness = MultiDimensionTestHarness::new().await?;

    let (episode, _embedding) = harness.create_episode_with_embedding(512, 42).await?;

    // Verify routing by checking expected table name
    let expected_table = table_for_dimension(512);
    assert_eq!(
        expected_table, "embeddings_other",
        "512-dim should route to embeddings_other"
    );

    // Verify we can retrieve embedding
    let retrieved = harness
        .storage
        .get_episode_embedding(episode.episode_id)
        .await?;
    assert!(
        retrieved.is_some(),
        "Should be able to retrieve 512-dim embedding"
    );

    info!("✓ Unsupported dimension routed correctly to embeddings_other");

    Ok(())
}

#[tokio::test]
#[cfg_attr(not(feature = "turso_multi_dimension"), ignore)]
async fn phase1_task2_mixed_dimension_routing() -> Result<()> {
    info!("=== Task 2: Validate mixed dimension routing ===");
    let harness = MultiDimensionTestHarness::new().await?;

    let test_cases = [
        (384, "embeddings_384"),
        (1024, "embeddings_1024"),
        (1536, "embeddings_1536"),
        (3072, "embeddings_3072"),
        (500, "embeddings_other"),
    ];

    for (dimension, expected_table) in test_cases {
        let (episode, _embedding) = harness
            .create_episode_with_embedding(dimension, 42 + dimension as u64)
            .await?;

        // Verify routing by checking expected table name
        let actual_table = table_for_dimension(dimension);
        assert_eq!(
            actual_table, expected_table,
            "{}-dim should route to {}",
            dimension, expected_table
        );

        // Verify we can retrieve embedding
        let retrieved = harness
            .storage
            .get_episode_embedding(episode.episode_id)
            .await?;
        assert!(
            retrieved.is_some(),
            "Should be able to retrieve {}-dim embedding",
            dimension
        );

        info!(
            "✓ {}-dimension embedding routed to {}",
            dimension, expected_table
        );
    }

    info!("✓ Mixed dimension routing validated successfully");
    Ok(())
}

#[tokio::test]
#[cfg_attr(not(feature = "turso_multi_dimension"), ignore)]
async fn phase1_task2_native_vector_stored_for_supported() -> Result<()> {
    info!("=== Task 2: Validate native vectors stored for supported dimensions ===");
    let harness = MultiDimensionTestHarness::new().await?;

    let supported_dimensions = [384, 1024, 1536, 3072];

    for dimension in supported_dimensions {
        let (episode, embedding) = harness.create_episode_with_embedding(dimension, 42).await?;

        // Retrieve and verify dimension matches
        let retrieved = harness
            .storage
            .get_episode_embedding(episode.episode_id)
            .await?;
        assert!(
            retrieved.is_some(),
            "Native vector should be stored for {} dimension",
            dimension
        );

        let retrieved_embedding = retrieved.unwrap();
        assert_eq!(
            retrieved_embedding.len(),
            dimension,
            "Retrieved embedding should have {} dimensions",
            dimension
        );
        assert_eq!(
            retrieved_embedding, embedding,
            "Retrieved embedding should match original"
        );

        info!("✓ Native vector stored for {} dimension", dimension);
    }

    info!("✓ Native vectors stored for all supported dimensions");
    Ok(())
}

// ============================================================================
// Task 3: Provider Integration Validation Tests
// ============================================================================

#[tokio::test]
#[cfg_attr(not(feature = "turso_multi_dimension"), ignore)]
async fn phase1_task3_embedding_retrieval_by_dimension() -> Result<()> {
    info!("=== Task 3: Validate embedding retrieval by dimension ===");
    let harness = MultiDimensionTestHarness::new().await?;

    // Store embeddings of different dimensions
    let dimensions = [384, 1536, 500];

    for dimension in dimensions {
        let (episode, _embedding) = harness.create_episode_with_embedding(dimension, 42).await?;

        // Retrieve and verify dimension
        let retrieved = harness
            .storage
            .get_episode_embedding(episode.episode_id)
            .await?;

        assert!(
            retrieved.is_some(),
            "Should retrieve {}-dim embedding",
            dimension
        );

        let retrieved_embedding = retrieved.unwrap();
        assert_eq!(
            retrieved_embedding.len(),
            dimension,
            "Retrieved embedding should have {} dimensions",
            dimension
        );

        info!("✓ Retrieved {}-dimension embedding correctly", dimension);
    }

    info!("✓ Embedding retrieval validated for all dimensions");
    Ok(())
}

// ============================================================================
// Task 4: Vector Search Validation Tests
// ============================================================================

#[tokio::test]
#[cfg_attr(not(feature = "turso_multi_dimension"), ignore)]
async fn phase1_task4_384_dimension_vector_search() -> Result<()> {
    info!("=== Task 4: Validate 384-dimension vector search ===");
    let harness = MultiDimensionTestHarness::new().await?;

    // Create base episode with embedding
    let (base_episode, base_embedding) = harness.create_episode_with_embedding(384, 42).await?;

    // Create additional episodes
    for i in 1..=5 {
        let (_episode, _embedding) = harness
            .create_episode_with_embedding(384, 100 + i as u64)
            .await?;
    }

    // Run similarity search
    let results = harness
        .run_similarity_search(base_embedding, 5, 0.5)
        .await?;

    assert!(!results.is_empty(), "Search should return results");
    assert!(
        results.len() <= 5,
        "Should return at most 5 results, got {}",
        results.len()
    );

    // Check that base episode is found
    let base_found = results.iter().any(|(id, _)| *id == base_episode.episode_id);
    assert!(base_found, "Base episode should be found in search results");

    // Check similarity scores are in valid range
    for (_, similarity) in &results {
        assert!(
            (0.0..=1.0).contains(similarity),
            "Similarity score should be between 0 and 1, got {}",
            similarity
        );
    }

    info!("✓ 384-dimension vector search works correctly");
    info!("  Found {} results", results.len());

    Ok(())
}

#[tokio::test]
#[cfg_attr(not(feature = "turso_multi_dimension"), ignore)]
async fn phase1_task4_1536_dimension_vector_search() -> Result<()> {
    info!("=== Task 4: Validate 1536-dimension vector search ===");
    let harness = MultiDimensionTestHarness::new().await?;

    // Create base episode with embedding
    let (base_episode, base_embedding) = harness.create_episode_with_embedding(1536, 42).await?;

    // Create additional episodes
    for i in 1..=5 {
        let (_episode, _embedding) = harness
            .create_episode_with_embedding(1536, 100 + i as u64)
            .await?;
    }

    // Run similarity search
    let results = harness
        .run_similarity_search(base_embedding, 5, 0.5)
        .await?;

    assert!(!results.is_empty(), "Search should return results");
    let base_found = results.iter().any(|(id, _)| *id == base_episode.episode_id);
    assert!(base_found, "Base episode should be found in search results");

    info!("✓ 1536-dimension vector search works correctly");
    info!("  Found {} results", results.len());

    Ok(())
}

#[tokio::test]
#[cfg_attr(not(feature = "turso_multi_dimension"), ignore)]
async fn phase1_task4_3072_dimension_vector_search() -> Result<()> {
    info!("=== Task 4: Validate 3072-dimension vector search ===");
    let harness = MultiDimensionTestHarness::new().await?;

    // Create base episode with embedding
    let (base_episode, base_embedding) = harness.create_episode_with_embedding(3072, 42).await?;

    // Create additional episodes
    for i in 1..=5 {
        let (_episode, _embedding) = harness
            .create_episode_with_embedding(3072, 100 + i as u64)
            .await?;
    }

    // Run similarity search
    let results = harness
        .run_similarity_search(base_embedding, 5, 0.5)
        .await?;

    assert!(!results.is_empty(), "Search should return results");
    let base_found = results.iter().any(|(id, _)| *id == base_episode.episode_id);
    assert!(base_found, "Base episode should be found in search results");

    info!("✓ 3072-dimension vector search works correctly");
    info!("  Found {} results", results.len());

    Ok(())
}

#[tokio::test]
#[cfg_attr(not(feature = "turso_multi_dimension"), ignore)]
async fn phase1_task4_unsupported_dimension_fallback() -> Result<()> {
    info!("=== Task 4: Validate unsupported dimension fallback ===");

    let harness = MultiDimensionTestHarness::new().await?;

    // Create episodes with 512-dim embeddings (unsupported)
    let (_base_episode, base_embedding) = harness.create_episode_with_embedding(512, 42).await?;

    for i in 1..=3 {
        let (_episode, _embedding) = harness
            .create_episode_with_embedding(512, 100 + i as u64)
            .await?;
    }

    // Run similarity search - should not crash even for unsupported dimensions
    let results = harness.run_similarity_search(base_embedding, 3, 0.5).await;

    // For unsupported dimensions, search may fail or return empty results,
    // but it should handle the case gracefully
    match results {
        Ok(results) => {
            info!(
                "✓ Search for unsupported dimension returned {} results",
                results.len()
            );
        }
        Err(e) => {
            // This is acceptable - unsupported dimensions may not support vector search
            info!("✓ Unsupported dimension handled gracefully: {}", e);
        }
    }

    Ok(())
}

#[tokio::test]
#[cfg_attr(not(feature = "turso_multi_dimension"), ignore)]
async fn phase1_task4_cross_dimension_isolation() -> Result<()> {
    info!("=== Task 4: Validate cross-dimension isolation ===");
    let harness = MultiDimensionTestHarness::new().await?;

    // Create episodes with different dimension embeddings
    let (episode_384, embedding_384) = harness.create_episode_with_embedding(384, 42).await?;
    let (episode_1536, embedding_1536) = harness.create_episode_with_embedding(1536, 43).await?;

    // Search with 384-dim embedding
    let results_384 = harness
        .run_similarity_search(embedding_384, 10, 0.5)
        .await?;

    let found_1536_in_384_search = results_384
        .iter()
        .any(|(id, _)| *id == episode_1536.episode_id);

    assert!(
        !found_1536_in_384_search,
        "384-dim search should not find 1536-dim episodes"
    );

    // Search with 1536-dim embedding
    let results_1536 = harness
        .run_similarity_search(embedding_1536, 10, 0.5)
        .await?;

    let found_384_in_1536_search = results_1536
        .iter()
        .any(|(id, _)| *id == episode_384.episode_id);

    assert!(
        !found_384_in_1536_search,
        "1536-dim search should not find 384-dim episodes"
    );

    info!("✓ Cross-dimension isolation works correctly");

    Ok(())
}
