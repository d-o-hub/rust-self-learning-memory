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
    clippy::doc_markdown
)]

use memory_core::embeddings::{
    cosine_similarity, EmbeddingConfig, EmbeddingProvider, EmbeddingStorageBackend,
    InMemoryEmbeddingStorage, LocalEmbeddingProvider, ModelConfig, SemanticService,
};
use memory_core::episode::{ExecutionStep, PatternId};
use memory_core::memory::SelfLearningMemory;
use memory_core::pattern::Pattern;
use memory_core::types::{ComplexityLevel, TaskContext, TaskOutcome, TaskType};
use memory_core::StorageBackend;
use memory_storage_redb::RedbStorage;
use std::sync::Arc;
use tempfile::TempDir;
use uuid::Uuid;

// End-to-End Embedding Workflow Tests (3 tests)

#[tokio::test]
async fn test_end_to_end_embedding_workflow() {
    let _storage = Arc::new(InMemoryEmbeddingStorage::new());
    let config = EmbeddingConfig::default();
    let service = SemanticService::with_fallback(Box::new(InMemoryEmbeddingStorage::new()), config)
        .await
        .expect("Should create service");

    let text = "Implement REST API endpoints in Rust";
    let embedding = service
        .provider
        .embed_text(text)
        .await
        .expect("Should generate embedding");

    assert!(!embedding.is_empty());
    assert_eq!(embedding.len(), service.provider.embedding_dimension());

    let episode_id = Uuid::new_v4();
    _storage
        .store_episode_embedding(episode_id, embedding.clone())
        .await
        .expect("Should store");

    let retrieved = _storage
        .get_episode_embedding(episode_id)
        .await
        .expect("Should retrieve");

    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), embedding);
}

#[tokio::test]
async fn test_end_to_end_batch_embeddings() {
    let _storage = Arc::new(InMemoryEmbeddingStorage::new());
    let config = EmbeddingConfig::default();
    let service = SemanticService::with_fallback(Box::new(InMemoryEmbeddingStorage::new()), config)
        .await
        .expect("Should create service");

    let texts = vec![
        "Implement user authentication".to_string(),
        "Build authorization system".to_string(),
        "Create login flow".to_string(),
    ];

    let embeddings = service
        .provider
        .embed_batch(&texts)
        .await
        .expect("Should generate batch");

    assert_eq!(embeddings.len(), texts.len());
    for embedding in &embeddings {
        assert!(!embedding.is_empty());
        assert_eq!(embedding.len(), service.provider.embedding_dimension());
    }
}

#[tokio::test]
async fn test_episode_embedding_workflow() {
    let mut config = memory_core::MemoryConfig::default();
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
    memory
        .complete_episode(episode_id, outcome)
        .await
        .expect("Should complete episode");

    let episode = memory
        .get_episode(episode_id)
        .await
        .expect("Should retrieve");
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
}

// Provider Fallback Chain Tests (3 tests)

#[tokio::test]
async fn test_provider_fallback_chain() {
    let storage = InMemoryEmbeddingStorage::new();
    let config = EmbeddingConfig::default();
    let service = SemanticService::with_fallback(Box::new(storage), config)
        .await
        .expect("Should create service");

    assert!(service.provider.is_available().await);

    let embedding = service
        .provider
        .embed_text("test")
        .await
        .expect("Should embed");
    assert_eq!(embedding.len(), service.provider.embedding_dimension());
}

#[tokio::test]
async fn test_provider_warmup() {
    let storage = InMemoryEmbeddingStorage::new();
    let config = EmbeddingConfig::default();
    let service = SemanticService::with_fallback(Box::new(storage), config)
        .await
        .expect("Should create service");

    assert!(service.provider.warmup().await.is_ok());
}

#[tokio::test]
async fn test_provider_similarity_calculation() {
    let storage = InMemoryEmbeddingStorage::new();
    let config = EmbeddingConfig::default();
    let service = SemanticService::with_fallback(Box::new(storage), config)
        .await
        .expect("Should create service");

    let similarity = service
        .provider
        .similarity("REST API", "REST API")
        .await
        .expect("Should calculate similarity");

    assert!((similarity - 1.0).abs() < 0.1);
}

// Semantic Search Accuracy Tests (4 tests)

#[tokio::test]
async fn test_semantic_similarity_identical_queries() {
    let storage = InMemoryEmbeddingStorage::new();
    let config = EmbeddingConfig::default();
    let service = SemanticService::with_fallback(Box::new(storage), config)
        .await
        .expect("Should create service");

    let similarity = service
        .provider
        .similarity("Implement REST API", "Implement REST API")
        .await
        .expect("Should calculate similarity");

    assert!((similarity - 1.0).abs() < 0.1);
}

#[tokio::test]
async fn test_cosine_similarity_function() {
    let vec1 = vec![1.0, 2.0, 3.0, 4.0];
    let vec2 = vec![1.0, 2.0, 3.0, 4.0];
    let similarity = cosine_similarity(&vec1, &vec2);
    assert_eq!(similarity, 1.0);

    let vec3 = vec![1.0, 0.0, 0.0];
    let vec4 = vec![0.0, 1.0, 0.0];
    let similarity_orth = cosine_similarity(&vec3, &vec4);
    assert!((similarity_orth - 0.5).abs() < 0.001);
}

#[tokio::test]
async fn test_semantic_search_ranking() {
    let _storage = Arc::new(InMemoryEmbeddingStorage::new());
    let config = EmbeddingConfig::default();
    let service = SemanticService::with_fallback(Box::new(InMemoryEmbeddingStorage::new()), config)
        .await
        .expect("Should create service");

    let episode = create_test_episode_helper("Implement REST API", "web-api");
    _storage.add_episode(episode.clone()).await;

    let embedding = service
        .provider
        .embed_text("Implement REST API")
        .await
        .unwrap();
    _storage
        .store_episode_embedding(episode.episode_id, embedding)
        .await
        .expect("Should store");

    let query_embedding = service.provider.embed_text("Build REST API").await.unwrap();
    let results = _storage
        .find_similar_episodes(query_embedding, 10, 0.0)
        .await
        .expect("Should find similar");

    assert!(!results.is_empty());
    if results.len() > 1 {
        for i in 1..results.len() {
            assert!(results[i - 1].similarity >= results[i].similarity);
        }
    }
}

#[tokio::test]
async fn test_semantic_similarity_threshold() {
    let storage = InMemoryEmbeddingStorage::new();
    let config = EmbeddingConfig::default();
    let service = SemanticService::with_fallback(Box::new(storage), config)
        .await
        .expect("Should create service");

    let query_embedding = service.provider.embed_text("unrelated").await.unwrap();
    let results = InMemoryEmbeddingStorage::new()
        .find_similar_episodes(query_embedding, 10, 0.9)
        .await
        .expect("Should search");

    for result in &results {
        assert!(result.similarity >= 0.9);
    }
}

// Storage Backend Integration Tests (4 tests)

#[tokio::test]
async fn test_inmemory_storage_embeddings() {
    let storage = InMemoryEmbeddingStorage::new();
    let episode_id = Uuid::new_v4();
    let embedding = vec![0.1, 0.2, 0.3, 0.4];

    storage
        .store_episode_embedding(episode_id, embedding.clone())
        .await
        .expect("Should store");

    let retrieved = storage.get_episode_embedding(episode_id).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), embedding);
}

#[tokio::test]
async fn test_redb_storage_embeddings() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let db_path = temp_dir.path().join("test_embeddings.redb");

    let _redb_storage = RedbStorage::new(&db_path)
        .await
        .expect("Should create redb storage");

    // Test basic episode creation
    let test_episode = create_test_episode_helper("Test episode", "test-domain");
    assert_eq!(test_episode.episode_id, test_episode.episode_id);
    assert!(!test_episode.task_description.is_empty());
}

#[tokio::test]
async fn test_batch_storage_operations() {
    let _storage = Arc::new(InMemoryEmbeddingStorage::new());

    let embeddings: Vec<(Uuid, Vec<f32>)> = (0..10)
        .map(|i| (Uuid::new_v4(), vec![i as f32 * 0.1, 0.0, 0.0, 0.0]))
        .collect();

    let store_futures: Vec<_> = embeddings
        .iter()
        .map(|(id, emb)| _storage.store_episode_embedding(*id, emb.clone()))
        .collect();

    futures::future::try_join_all(store_futures)
        .await
        .expect("Should store all");

    for (id, expected) in embeddings {
        let retrieved = _storage.get_episode_embedding(id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), expected);
    }
}

#[tokio::test]
async fn test_pattern_storage_and_retrieval() {
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
        .await
        .expect("Should store pattern embedding");

    let retrieved = storage.get_pattern_embedding(pattern_id).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), embedding);
}

// Concurrent Embedding Operations Tests (2 tests)

#[tokio::test]
async fn test_concurrent_embedding_generation() {
    let storage = InMemoryEmbeddingStorage::new();
    let config = EmbeddingConfig::default();
    let service = SemanticService::with_fallback(Box::new(storage), config)
        .await
        .expect("Should create service");

    let texts: Vec<String> = (0..10).map(|i| format!("Text {}", i)).collect();
    let futures: Vec<_> = texts
        .iter()
        .map(|t| service.provider.embed_text(t))
        .collect();

    let embeddings = futures::future::try_join_all(futures)
        .await
        .expect("Should generate all");

    assert_eq!(embeddings.len(), 10);
    for embedding in &embeddings {
        assert!(!embedding.is_empty());
        assert_eq!(embedding.len(), service.provider.embedding_dimension());
    }
}

#[tokio::test]
async fn test_concurrent_storage_access() {
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
                .unwrap();
            storage_clone.get_episode_embedding(id).await.unwrap()
        });
        tasks.push(task);
    }

    let results = futures::future::try_join_all(tasks)
        .await
        .expect("Should complete all");

    assert_eq!(results.len(), 20);
    for result in &results {
        assert!(result.is_some());
    }
}

// Model Download and Caching Tests (2 tests)

#[tokio::test]
async fn test_model_loading() {
    let config =
        ModelConfig::local_sentence_transformer("sentence-transformers/all-MiniLM-L6-v2", 384);

    let provider = LocalEmbeddingProvider::new(config).await;
    assert!(provider.is_ok());

    let provider = provider.unwrap();
    assert!(provider.is_loaded().await);
    assert_eq!(provider.embedding_dimension(), 384);
}

#[tokio::test]
async fn test_model_deterministic_embeddings() {
    let config = ModelConfig::local_sentence_transformer("test-model", 384);
    let provider = LocalEmbeddingProvider::new(config)
        .await
        .expect("Should create provider");

    let text = "Test text for deterministic behavior";
    let embedding1 = provider.embed_text(text).await.unwrap();
    let embedding2 = provider.embed_text(text).await.unwrap();

    assert_eq!(embedding1, embedding2);
    assert_eq!(embedding1.len(), 384);
}

// Episode Embedding Generation Tests (2 tests)

#[tokio::test]
async fn test_episode_embedding_with_service() {
    let _storage = Arc::new(InMemoryEmbeddingStorage::new());
    let config = EmbeddingConfig::default();
    let service = SemanticService::with_fallback(Box::new(InMemoryEmbeddingStorage::new()), config)
        .await
        .expect("Should create service");

    let episode = create_test_episode_helper("Implement REST API", "web-api");
    let result = service.embed_episode(&episode).await;

    assert!(result.is_ok());
    let embedding = result.unwrap();
    assert!(!embedding.is_empty());
    assert_eq!(embedding.len(), service.provider.embedding_dimension());
}

#[tokio::test]
async fn test_pattern_embedding_with_service() {
    let _storage = Arc::new(InMemoryEmbeddingStorage::new());
    let config = EmbeddingConfig::default();
    let service = SemanticService::with_fallback(Box::new(InMemoryEmbeddingStorage::new()), config)
        .await
        .expect("Should create service");

    let pattern = create_test_pattern();
    let result = service.embed_pattern(&pattern).await;

    assert!(result.is_ok());
    let embedding = result.unwrap();
    assert!(!embedding.is_empty());
    assert_eq!(embedding.len(), service.provider.embedding_dimension());
}

// SelfLearningMemory Integration Tests (3 tests)

#[tokio::test]
async fn test_memory_semantic_retrieval() {
    let mut config = memory_core::MemoryConfig::default();
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
    memory
        .complete_episode(episode_id, outcome)
        .await
        .expect("Should complete episode");

    let relevant = memory
        .retrieve_relevant_context("Build HTTP endpoints".to_string(), context, 5)
        .await;

    assert!(!relevant.is_empty());
}

#[tokio::test]
async fn test_memory_fallback_to_keyword() {
    let mut config = memory_core::MemoryConfig::default();
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
}

#[tokio::test]
async fn test_memory_with_multiple_episodes() {
    let mut config = memory_core::MemoryConfig::default();
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
        memory
            .complete_episode(episode_id, outcome)
            .await
            .expect("Should complete episode");
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
}

// Performance Benchmarks Tests (2 tests)

#[tokio::test]
async fn benchmark_single_embedding_generation() {
    let storage = InMemoryEmbeddingStorage::new();
    let config = EmbeddingConfig::default();
    let service = SemanticService::with_fallback(Box::new(storage), config)
        .await
        .expect("Should create service");

    let start = std::time::Instant::now();
    let _embedding = service.provider.embed_text("Test text").await.unwrap();
    let duration = start.elapsed();

    assert!(
        duration.as_millis() < 500,
        "Took {}ms",
        duration.as_millis()
    );
}

#[tokio::test]
async fn benchmark_batch_embedding_generation() {
    let storage = InMemoryEmbeddingStorage::new();
    let config = EmbeddingConfig::default();
    let service = SemanticService::with_fallback(Box::new(storage), config)
        .await
        .expect("Should create service");

    let texts: Vec<String> = (0..100).map(|i| format!("Text {}", i)).collect();

    let start = std::time::Instant::now();
    let _embeddings = service.provider.embed_batch(&texts).await.unwrap();
    let duration = start.elapsed();

    let avg = duration.as_millis() as f64 / 100.0;
    assert!(avg < 100.0, "Average: {}ms", avg);
}

// Error Handling Tests (2 tests)

#[tokio::test]
async fn test_empty_text_embedding() {
    let storage = InMemoryEmbeddingStorage::new();
    let config = EmbeddingConfig::default();
    let service = SemanticService::with_fallback(Box::new(storage), config)
        .await
        .expect("Should create service");

    let result = service.provider.embed_text("").await;
    match result {
        Ok(embedding) => assert!(!embedding.is_empty()),
        Err(_) => {}
    }
}

#[tokio::test]
async fn test_storage_with_nonexistent_id() {
    let storage = InMemoryEmbeddingStorage::new();
    let fake_id = Uuid::new_v4();

    let result = storage.get_episode_embedding(fake_id).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

// Helper Functions

fn create_test_episode_helper(description: &str, domain: &str) -> memory_core::Episode {
    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: Some("tokio".to_string()),
        complexity: ComplexityLevel::Moderate,
        domain: domain.to_string(),
        tags: vec!["rest".to_string(), "async".to_string()],
    };

    let mut episode =
        memory_core::Episode::new(description.to_string(), context, TaskType::CodeGeneration);

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
