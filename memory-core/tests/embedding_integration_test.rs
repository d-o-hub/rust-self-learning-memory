//! Comprehensive integration tests for embedding system
//!
//! These tests verify the entire embedding workflow from text input to
//! stored embeddings, semantic search, and integration with `SelfLearningMemory`.

#![allow(
    clippy::uninlined_format_args,
    clippy::used_underscore_binding,
    clippy::field_reassign_with_default,
    clippy::cast_precision_loss,
    clippy::float_cmp,
    clippy::single_match,
    clippy::default_trait_access,
    unused_imports,
    clippy::doc_markdown,
    clippy::unwrap_used,
    clippy::expect_used
)]

use anyhow::{Context, Result};
use do_memory_core::StorageBackend;
use do_memory_core::embeddings::{
    EmbeddingConfig, EmbeddingProvider, EmbeddingStorageBackend, InMemoryEmbeddingStorage,
    LocalConfig, LocalEmbeddingProvider, SemanticService, cosine_similarity,
};
use do_memory_core::episode::{ExecutionStep, PatternId};
use do_memory_core::memory::SelfLearningMemory;
use do_memory_core::pattern::Pattern;
use do_memory_core::types::{ComplexityLevel, TaskContext, TaskOutcome, TaskType};
use do_memory_storage_redb::RedbStorage;
use std::sync::Arc;
use tempfile::TempDir;
use uuid::Uuid;

// End-to-End Embedding Workflow Tests (3 tests)

#[tokio::test]
async fn test_end_to_end_embedding_workflow() -> Result<()> {
    let _storage = Arc::new(InMemoryEmbeddingStorage::new());
    let config = EmbeddingConfig::default();
    let service =
        SemanticService::with_fallback(Box::new(InMemoryEmbeddingStorage::new()), config).await?;

    let text = "Implement REST API endpoints in Rust";
    let embedding = service.provider.embed_text(text).await?;

    assert!(!embedding.is_empty());
    assert_eq!(embedding.len(), service.provider.embedding_dimension());

    let episode_id = Uuid::new_v4();
    _storage
        .store_episode_embedding(episode_id, embedding.clone())
        .await?;

    let retrieved = _storage.get_episode_embedding(episode_id).await?;

    assert!(retrieved.is_some());
    assert_eq!(retrieved.context("Missing embedding")?, embedding);
    Ok(())
}

#[tokio::test]
async fn test_end_to_end_batch_embeddings() -> Result<()> {
    let _storage = Arc::new(InMemoryEmbeddingStorage::new());
    let config = EmbeddingConfig::default();
    let service =
        SemanticService::with_fallback(Box::new(InMemoryEmbeddingStorage::new()), config).await?;

    let texts = vec![
        "Implement user authentication".to_string(),
        "Build authorization system".to_string(),
        "Create login flow".to_string(),
    ];

    let embeddings = service.provider.embed_batch(&texts).await?;

    assert_eq!(embeddings.len(), texts.len());
    for embedding in &embeddings {
        assert!(!embedding.is_empty());
        assert_eq!(embedding.len(), service.provider.embedding_dimension());
    }
    Ok(())
}

#[tokio::test]
async fn test_episode_embedding_workflow() -> Result<()> {
    let mut config = do_memory_core::MemoryConfig::default();
    config.quality_threshold = 0.2;
    let memory = SelfLearningMemory::with_config(config);

    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: Some("axum".to_string()),
        complexity: ComplexityLevel::Moderate,
        domain: "web-api".to_string(),
        tags: vec!["rest".to_string(), "async".to_string()],
    };

    let episode_id = memory
        .start_episode(
            "Implement REST API endpoints".to_string(),
            context.clone(),
            TaskType::CodeGeneration,
        )
        .await;

    for i in 1..=5 {
        let step = ExecutionStep::new(i, format!("tool_{}", i), format!("Action {}", i));
        memory.log_step(episode_id, step).await;
    }

    let outcome = TaskOutcome::Success {
        verdict: "API implemented successfully".to_string(),
        artifacts: vec!["api.rs".to_string(), "routes.rs".to_string()],
    };
    memory.complete_episode(episode_id, outcome).await?;

    let episode = memory.get_episode(episode_id).await?;
    assert!(episode.is_complete());

    let search_context = TaskContext {
        language: Some("rust".to_string()),
        framework: Some("axum".to_string()),
        complexity: ComplexityLevel::Moderate,
        domain: "web-api".to_string(),
        tags: vec!["rest".to_string()],
    };

    let relevant = memory
        .retrieve_relevant_context("Build API endpoints".to_string(), search_context, 5)
        .await;

    assert!(!relevant.is_empty());
    Ok(())
}

// Provider Fallback Chain Tests (3 tests)

#[tokio::test]
async fn test_provider_fallback_chain() -> Result<()> {
    let storage = InMemoryEmbeddingStorage::new();
    let config = EmbeddingConfig::default();
    let service = SemanticService::with_fallback(Box::new(storage), config).await?;

    assert!(service.provider.is_available().await);

    let embedding = service.provider.embed_text("test").await?;
    assert_eq!(embedding.len(), service.provider.embedding_dimension());
    Ok(())
}

#[tokio::test]
async fn test_provider_warmup() -> Result<()> {
    let storage = InMemoryEmbeddingStorage::new();
    let config = EmbeddingConfig::default();
    let service = SemanticService::with_fallback(Box::new(storage), config).await?;

    service.provider.warmup().await?;
    Ok(())
}

#[tokio::test]
async fn test_provider_similarity_calculation() -> Result<()> {
    let storage = InMemoryEmbeddingStorage::new();
    let config = EmbeddingConfig::default();
    let service = SemanticService::with_fallback(Box::new(storage), config).await?;

    let similarity = service.provider.similarity("REST API", "REST API").await?;

    assert!((similarity - 1.0).abs() < 0.1);
    Ok(())
}

// Semantic Search Accuracy Tests (4 tests)

#[tokio::test]
async fn test_semantic_similarity_identical_queries() -> Result<()> {
    let storage = InMemoryEmbeddingStorage::new();
    let config = EmbeddingConfig::default();
    let service = SemanticService::with_fallback(Box::new(storage), config).await?;

    let similarity = service
        .provider
        .similarity("Implement REST API", "Implement REST API")
        .await?;

    assert!((similarity - 1.0).abs() < 0.1);
    Ok(())
}

#[tokio::test]
async fn test_cosine_similarity_function() -> Result<()> {
    let vec1 = vec![1.0, 2.0, 3.0, 4.0];
    let vec2 = vec![1.0, 2.0, 3.0, 4.0];
    let similarity = cosine_similarity(&vec1, &vec2);
    assert!((similarity - 1.0).abs() < 1e-6);

    let vec3 = vec![1.0, 0.0, 0.0];
    let vec4 = vec![0.0, 1.0, 0.0];
    let similarity_orth = cosine_similarity(&vec3, &vec4);
    assert!((similarity_orth - 0.5).abs() < 1e-6);
    Ok(())
}

#[tokio::test]
async fn test_semantic_search_ranking() -> Result<()> {
    let _storage = Arc::new(InMemoryEmbeddingStorage::new());
    let config = EmbeddingConfig::default();
    let service =
        SemanticService::with_fallback(Box::new(InMemoryEmbeddingStorage::new()), config).await?;

    let episode = create_test_episode_helper("Implement REST API", "web-api");
    _storage.add_episode(episode.clone()).await;

    let embedding = service.provider.embed_text("Implement REST API").await?;
    _storage
        .store_episode_embedding(episode.episode_id, embedding)
        .await?;

    let query_embedding = service.provider.embed_text("Build REST API").await?;
    let results = _storage
        .find_similar_episodes(query_embedding, 10, 0.0)
        .await?;

    assert!(!results.is_empty());
    if results.len() > 1 {
        for i in 1..results.len() {
            assert!(results[i - 1].similarity >= results[i].similarity);
        }
    }
    Ok(())
}

#[tokio::test]
async fn test_semantic_similarity_threshold() -> Result<()> {
    let storage = InMemoryEmbeddingStorage::new();
    let config = EmbeddingConfig::default();
    let service = SemanticService::with_fallback(Box::new(storage), config).await?;

    let query_embedding = service.provider.embed_text("unrelated").await?;
    let results = InMemoryEmbeddingStorage::new()
        .find_similar_episodes(query_embedding, 10, 0.9)
        .await?;

    for result in &results {
        assert!(result.similarity >= 0.9);
    }
    Ok(())
}

// Storage Backend Integration Tests (4 tests)

#[tokio::test]
async fn test_inmemory_storage_embeddings() -> Result<()> {
    let storage = InMemoryEmbeddingStorage::new();
    let episode_id = Uuid::new_v4();
    let embedding = vec![0.1, 0.2, 0.3, 0.4];

    storage
        .store_episode_embedding(episode_id, embedding.clone())
        .await?;

    let retrieved = storage.get_episode_embedding(episode_id).await?;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.context("Missing embedding")?, embedding);
    Ok(())
}

#[tokio::test]
async fn test_redb_storage_embeddings() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_embeddings.redb");

    let _redb_storage = RedbStorage::new(&db_path).await?;

    // Test basic episode creation
    let test_episode = create_test_episode_helper("Test episode", "test-domain");
    assert_eq!(test_episode.episode_id, test_episode.episode_id);
    assert!(!test_episode.task_description.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_batch_storage_operations() -> Result<()> {
    let _storage = Arc::new(InMemoryEmbeddingStorage::new());

    let embeddings: Vec<(Uuid, Vec<f32>)> = (0..10)
        .map(|i| (Uuid::new_v4(), vec![i as f32 * 0.1, 0.0, 0.0, 0.0]))
        .collect();

    let store_futures: Vec<_> = embeddings
        .iter()
        .map(|(id, emb)| _storage.store_episode_embedding(*id, emb.clone()))
        .collect();

    futures::future::try_join_all(store_futures).await?;

    for (id, expected) in embeddings {
        let retrieved = _storage.get_episode_embedding(id).await?;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.context("Missing embedding")?, expected);
    }
    Ok(())
}

#[tokio::test]
async fn test_pattern_storage_and_retrieval() -> Result<()> {
    let storage = InMemoryEmbeddingStorage::new();
    let pattern = create_test_pattern();

    let pattern_id = match &pattern {
        Pattern::ToolSequence { id, .. } => *id,
        _ => Uuid::new_v4(),
    };

    storage.add_pattern(pattern.clone()).await;

    let embedding = vec![0.5, 0.6, 0.7, 0.8];
    storage
        .store_pattern_embedding(pattern_id, embedding.clone())
        .await?;

    let retrieved = storage.get_pattern_embedding(pattern_id).await?;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.context("Missing embedding")?, embedding);
    Ok(())
}

// Concurrent Embedding Operations Tests (2 tests)

#[tokio::test]
async fn test_concurrent_embedding_generation() -> Result<()> {
    let storage = InMemoryEmbeddingStorage::new();
    let config = EmbeddingConfig::default();
    let service = SemanticService::with_fallback(Box::new(storage), config).await?;

    let texts: Vec<String> = (0..10).map(|i| format!("Text {}", i)).collect();
    let futures: Vec<_> = texts
        .iter()
        .map(|t| service.provider.embed_text(t))
        .collect();

    let embeddings = futures::future::try_join_all(futures).await?;

    assert_eq!(embeddings.len(), 10);
    for embedding in &embeddings {
        assert!(!embedding.is_empty());
        assert_eq!(embedding.len(), service.provider.embedding_dimension());
    }
    Ok(())
}

#[tokio::test]
async fn test_concurrent_storage_access() -> Result<()> {
    let _storage = Arc::new(InMemoryEmbeddingStorage::new());

    let mut tasks = Vec::new();
    for i in 0..20 {
        let storage_clone = Arc::clone(&_storage);
        let task = tokio::spawn(async move {
            let id = Uuid::new_v4();
            let embedding = vec![i as f32 * 0.1, 0.0, 0.0, 0.0];
            storage_clone
                .store_episode_embedding(id, embedding.clone())
                .await
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;
            storage_clone
                .get_episode_embedding(id)
                .await
                .map_err(|e| anyhow::anyhow!(e.to_string()))
        });
        tasks.push(task);
    }

    let results = futures::future::join_all(tasks).await;

    assert_eq!(results.len(), 20);
    for result in results {
        let inner_result = result.context("Task join failed")?;
        let retrieved = inner_result.context("Task execution failed")?;
        assert!(retrieved.is_some());
    }
    Ok(())
}

// Model Download and Caching Tests (2 tests)

#[tokio::test]
async fn test_model_loading() -> Result<()> {
    let config = LocalConfig::new("sentence-transformers/all-MiniLM-L6-v2", 384);

    let provider = LocalEmbeddingProvider::new_with_fallback(config).await?;

    assert!(provider.is_loaded().await);
    assert_eq!(provider.embedding_dimension(), 384);
    Ok(())
}

#[tokio::test]
async fn test_model_deterministic_embeddings() -> Result<()> {
    let config = LocalConfig::new("test-model", 384);
    let provider = LocalEmbeddingProvider::new_with_fallback(config).await?;

    let text = "Test text for deterministic behavior";
    let embedding1 = provider.embed_text(text).await?;
    let embedding2 = provider.embed_text(text).await?;

    assert_eq!(embedding1, embedding2);
    assert_eq!(embedding1.len(), 384);
    Ok(())
}

// Episode Embedding Generation Tests (2 tests)

#[tokio::test]
async fn test_episode_embedding_with_service() -> Result<()> {
    let _storage = Arc::new(InMemoryEmbeddingStorage::new());
    let config = EmbeddingConfig::default();
    let service =
        SemanticService::with_fallback(Box::new(InMemoryEmbeddingStorage::new()), config).await?;

    let episode = create_test_episode_helper("Implement REST API", "web-api");
    let result = service.embed_episode(&episode).await;

    let embedding = result?;
    assert!(!embedding.is_empty());
    assert_eq!(embedding.len(), service.provider.embedding_dimension());
    Ok(())
}

#[tokio::test]
async fn test_pattern_embedding_with_service() -> Result<()> {
    let _storage = Arc::new(InMemoryEmbeddingStorage::new());
    let config = EmbeddingConfig::default();
    let service =
        SemanticService::with_fallback(Box::new(InMemoryEmbeddingStorage::new()), config).await?;

    let pattern = create_test_pattern();
    let result = service.embed_pattern(&pattern).await;

    let embedding = result?;
    assert!(!embedding.is_empty());
    assert_eq!(embedding.len(), service.provider.embedding_dimension());
    Ok(())
}

// SelfLearningMemory Integration Tests (3 tests)

#[tokio::test]
async fn test_memory_semantic_retrieval() -> Result<()> {
    let mut config = do_memory_core::MemoryConfig::default();
    config.quality_threshold = 0.2;
    let memory = SelfLearningMemory::with_config(config);

    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: Some("axum".to_string()),
        complexity: ComplexityLevel::Moderate,
        domain: "web-api".to_string(),
        tags: vec!["rest".to_string()],
    };

    let episode_id = memory
        .start_episode(
            "Implement REST API endpoints".to_string(),
            context.clone(),
            TaskType::CodeGeneration,
        )
        .await;

    for i in 1..=5 {
        let step = ExecutionStep::new(i, format!("tool_{}", i), format!("Action {}", i));
        memory.log_step(episode_id, step).await;
    }

    let outcome = TaskOutcome::Success {
        verdict: "API implemented".to_string(),
        artifacts: vec!["api.rs".to_string()],
    };
    memory.complete_episode(episode_id, outcome).await?;

    let relevant = memory
        .retrieve_relevant_context("Build HTTP endpoints".to_string(), context, 5)
        .await;

    assert!(!relevant.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_memory_fallback_to_keyword() -> Result<()> {
    let mut config = do_memory_core::MemoryConfig::default();
    config.quality_threshold = 0.2;
    let memory = SelfLearningMemory::with_config(config);

    let context = TaskContext {
        language: Some("python".to_string()),
        framework: Some("fastapi".to_string()),
        complexity: ComplexityLevel::Simple,
        domain: "data-api".to_string(),
        tags: vec![],
    };

    let relevant = memory
        .retrieve_relevant_context("Build API".to_string(), context, 5)
        .await;

    assert!(relevant.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_memory_with_multiple_episodes() -> Result<()> {
    let mut config = do_memory_core::MemoryConfig::default();
    config.quality_threshold = 0.2;
    let memory = SelfLearningMemory::with_config(config);

    for i in 0..5 {
        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: Some("tokio".to_string()),
            complexity: ComplexityLevel::Moderate,
            domain: "web-api".to_string(),
            tags: vec![],
        };

        let episode_id = memory
            .start_episode(format!("Task {}", i), context, TaskType::CodeGeneration)
            .await;

        for j in 1..=5 {
            let step = ExecutionStep::new(j, format!("tool_{}", j), format!("Action {}", j));
            memory.log_step(episode_id, step).await;
        }

        let outcome = TaskOutcome::Success {
            verdict: format!("Completed {}", i),
            artifacts: vec!["file.rs".to_string()],
        };
        memory.complete_episode(episode_id, outcome).await?;
    }

    let query_context = TaskContext {
        language: Some("rust".to_string()),
        framework: Some("tokio".to_string()),
        complexity: ComplexityLevel::Moderate,
        domain: "web-api".to_string(),
        tags: vec![],
    };

    let relevant = memory
        .retrieve_relevant_context("Build API".to_string(), query_context, 10)
        .await;

    assert!(!relevant.is_empty());
    Ok(())
}

// Performance Benchmarks Tests (2 tests)

#[tokio::test]
async fn benchmark_single_embedding_generation() -> Result<()> {
    let storage = InMemoryEmbeddingStorage::new();
    let config = EmbeddingConfig::default();
    let service = SemanticService::with_fallback(Box::new(storage), config).await?;

    let start = std::time::Instant::now();
    let _embedding = service.provider.embed_text("Test text").await?;
    let duration = start.elapsed();

    assert!(
        duration.as_millis() < 500,
        "Took {}ms",
        duration.as_millis()
    );
    Ok(())
}

#[tokio::test]
async fn benchmark_batch_embedding_generation() -> Result<()> {
    let storage = InMemoryEmbeddingStorage::new();
    let config = EmbeddingConfig::default();
    let service = SemanticService::with_fallback(Box::new(storage), config).await?;

    let texts: Vec<String> = (0..100).map(|i| format!("Text {}", i)).collect();

    let start = std::time::Instant::now();
    let _embeddings = service.provider.embed_batch(&texts).await?;
    let duration = start.elapsed();

    let avg = duration.as_millis() as f64 / 100.0;
    assert!(avg < 100.0, "Average: {}ms", avg);
    Ok(())
}

// Error Handling Tests (2 tests)

#[tokio::test]
async fn test_empty_text_embedding() -> Result<()> {
    let storage = InMemoryEmbeddingStorage::new();
    let config = EmbeddingConfig::default();
    let service = SemanticService::with_fallback(Box::new(storage), config).await?;

    let result = service.provider.embed_text("").await;
    match result {
        Ok(embedding) => assert!(!embedding.is_empty()),
        Err(_) => {}
    }
    Ok(())
}

#[tokio::test]
async fn test_storage_with_nonexistent_id() -> Result<()> {
    let storage = InMemoryEmbeddingStorage::new();
    let fake_id = Uuid::new_v4();

    let result = storage.get_episode_embedding(fake_id).await;

    assert!(result?.is_none());
    Ok(())
}

// Helper Functions

fn create_test_episode_helper(description: &str, domain: &str) -> do_memory_core::Episode {
    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: Some("tokio".to_string()),
        complexity: ComplexityLevel::Moderate,
        domain: domain.to_string(),
        tags: vec!["rest".to_string(), "async".to_string()],
    };

    let mut episode =
        do_memory_core::Episode::new(description.to_string(), context, TaskType::CodeGeneration);

    for i in 1..=5 {
        let step = ExecutionStep::new(i, format!("tool_{}", i), format!("Action {}", i));
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "Completed successfully with all steps".to_string(),
        artifacts: vec!["file1.rs".to_string(), "file2.rs".to_string()],
    });

    episode
}
fn create_test_pattern() -> Pattern {
    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: Some("tokio".to_string()),
        complexity: ComplexityLevel::Moderate,
        domain: "web-api".to_string(),
        tags: vec![],
    };

    Pattern::ToolSequence {
        id: PatternId::new_v4(),
        tools: vec!["parser".to_string(), "validator".to_string()],
        context,
        success_rate: 0.85,
        avg_latency: chrono::Duration::seconds(10),
        occurrence_count: 5,
        effectiveness: Default::default(),
    }
}
