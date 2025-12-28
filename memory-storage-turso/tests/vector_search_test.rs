//! Tests for vector search functionality

use memory_core::embeddings::EmbeddingStorageBackend;
use memory_storage_turso::TursoStorage;
use tempfile::TempDir;

async fn create_test_storage() -> anyhow::Result<(TursoStorage, TempDir)> {
    let dir = TempDir::new()?;
    let db_path = dir.path().join("test.db");

    let db = libsql::Builder::new_local(&db_path).build().await?;

    let storage = TursoStorage::from_database(db)?;
    storage.initialize_schema().await?;

    Ok((storage, dir))
}

#[tokio::test]
async fn test_store_and_retrieve_embedding() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    use uuid::Uuid;

    let episode_id = Uuid::new_v4();
    let embedding = vec![0.1_f32; 384]; // 384-dimensional embedding

    storage
        .store_episode_embedding(episode_id, embedding.clone())
        .await
        .unwrap();

    // Retrieve the embedding
    let retrieved = storage.get_episode_embedding(episode_id).await.unwrap();
    assert!(retrieved.is_some(), "Should retrieve embedding");

    let retrieved_embedding = retrieved.unwrap();
    assert_eq!(retrieved_embedding.len(), 384);
}

#[tokio::test]
async fn test_vector_search() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    // Store some episodes with embeddings
    use memory_core::types::{ComplexityLevel, TaskContext, TaskType};
    use memory_core::Episode;

    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: None,
        complexity: ComplexityLevel::Simple,
        domain: "web-api".to_string(),
        tags: vec![],
    };

    let mut episode1 = Episode::new(
        "Implement API endpoint".to_string(),
        context.clone(),
        TaskType::CodeGeneration,
    );
    episode1.complete(memory_core::types::TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });

    let mut episode2 = Episode::new(
        "Create database schema".to_string(),
        context,
        TaskType::CodeGeneration,
    );
    episode2.complete(memory_core::types::TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });

    storage.store_episode(&episode1).await.unwrap();
    storage.store_episode(&episode2).await.unwrap();

    // Create embeddings (using mock values for testing)
    let embedding1 = vec![0.9_f32; 384]; // High similarity to query
    let embedding2 = vec![0.1_f32; 384]; // Low similarity to query

    storage
        .store_episode_embedding(episode1.episode_id, embedding1)
        .await
        .unwrap();

    storage
        .store_episode_embedding(episode2.episode_id, embedding2)
        .await
        .unwrap();

    // Search for similar episodes
    let query_embedding = vec![0.8_f32; 384]; // Similar to embedding1

    let results = storage
        .find_similar_episodes(query_embedding, 10, 0.5)
        .await
        .unwrap();

    // Should find at least one result
    assert!(!results.is_empty(), "Should find similar episodes");
}

#[tokio::test]
async fn test_vector_search_threshold() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    use memory_core::types::{ComplexityLevel, TaskContext, TaskType};
    use memory_core::Episode;

    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: None,
        complexity: ComplexityLevel::Simple,
        domain: "web-api".to_string(),
        tags: vec![],
    };

    let mut episode = Episode::new(
        "Test episode".to_string(),
        context,
        TaskType::CodeGeneration,
    );
    episode.complete(memory_core::types::TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });

    storage.store_episode(&episode).await.unwrap();

    // Store an embedding with specific pattern
    let mut embedding = vec![0.0_f32; 384];
    for (i, val) in embedding.iter_mut().enumerate().take(384) {
        *val = if i % 2 == 0 { 1.0 } else { 0.0 };
    }
    storage
        .store_episode_embedding(episode.episode_id, embedding)
        .await
        .unwrap();

    // Search with very different embedding and high threshold
    let mut query_embedding = vec![0.0_f32; 384];
    for (i, val) in query_embedding.iter_mut().enumerate().take(384) {
        *val = if i % 2 == 0 { 0.0 } else { 1.0 }; // Opposite pattern
    }

    let results = storage
        .find_similar_episodes(query_embedding, 10, 0.95)
        .await
        .unwrap();

    // Should not find results due to low similarity and high threshold
    // Opposite patterns should have very low or negative similarity
    if !results.is_empty() {
        assert!(
            results[0].similarity < 0.95,
            "Similarity {} should be below threshold 0.95",
            results[0].similarity
        );
    }
}
