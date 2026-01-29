//! Integration tests for context-aware embeddings
//!
//! Tests the full workflow of training and using task-specific embedding adapters.

use memory_core::embeddings::{EmbeddingProvider, LocalEmbeddingProvider};
use memory_core::episode::Episode;
use memory_core::spatiotemporal::embeddings::{ContextAwareEmbeddings, ContrastivePair};
use memory_core::types::{ComplexityLevel, TaskContext, TaskOutcome, TaskType};
use std::sync::Arc;

fn create_test_episode(task_type: TaskType, description: &str, domain: &str) -> Episode {
    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: None,
        complexity: ComplexityLevel::Moderate,
        domain: domain.to_string(),
        tags: vec![],
    };

    let mut episode = Episode::new(description.to_string(), context, task_type);
    episode.complete(TaskOutcome::Success {
        verdict: "Success".to_string(),
        artifacts: vec![],
    });
    episode
}

#[tokio::test]
async fn test_context_aware_embeddings_integration() {
    // Create base provider (using mock in this test)
    let config = memory_core::embeddings::EmbeddingConfig::default();
    let local_config = if let memory_core::embeddings::ProviderConfig::Local(cfg) = &config.provider
    {
        cfg.clone()
    } else {
        eprintln!("Skipping test - default config is not local");
        return;
    };
    let base = if let Ok(provider) = LocalEmbeddingProvider::new(local_config).await {
        Arc::new(provider)
    } else {
        // Skip test if model not available
        eprintln!("Skipping test - embedding model not available");
        return;
    };

    let mut embeddings = ContextAwareEmbeddings::new(base.clone());

    // Verify no adapters initially
    assert_eq!(embeddings.adapter_count(), 0);
    assert!(!embeddings.has_adapter(TaskType::CodeGeneration));

    // Train adapter for CodeGeneration tasks
    let coding_pairs = vec![
        ContrastivePair {
            anchor: create_test_episode(TaskType::CodeGeneration, "implement API", "web"),
            positive: create_test_episode(TaskType::CodeGeneration, "build REST service", "web"),
            negative: create_test_episode(TaskType::Debugging, "fix crash", "web"),
        },
        ContrastivePair {
            anchor: create_test_episode(TaskType::CodeGeneration, "create database", "backend"),
            positive: create_test_episode(TaskType::CodeGeneration, "add schema", "backend"),
            negative: create_test_episode(TaskType::Testing, "write tests", "backend"),
        },
    ];

    let result = embeddings
        .train_adapter(TaskType::CodeGeneration, &coding_pairs)
        .await;
    assert!(result.is_ok());

    // Verify adapter created
    assert_eq!(embeddings.adapter_count(), 1);
    assert!(embeddings.has_adapter(TaskType::CodeGeneration));

    let adapter = embeddings.get_adapter(TaskType::CodeGeneration).unwrap();
    assert_eq!(adapter.trained_on_count, 2);

    // Test embedding generation with and without adapter
    let text = "implement authentication";

    let base_embedding = base.embed_text(text).await.unwrap();
    let adapted_embedding = embeddings
        .get_adapted_embedding(text, Some(TaskType::CodeGeneration))
        .await
        .unwrap();

    // Both should have same dimension
    assert_eq!(base_embedding.len(), adapted_embedding.len());

    // Test fallback for task type without adapter
    let debugging_embedding = embeddings
        .get_adapted_embedding(text, Some(TaskType::Debugging))
        .await
        .unwrap();

    // Should equal base embedding (no adapter trained)
    assert_eq!(debugging_embedding, base_embedding);
}

#[tokio::test]
async fn test_multiple_task_adapters() {
    let config = memory_core::embeddings::EmbeddingConfig::default();
    let local_config = if let memory_core::embeddings::ProviderConfig::Local(cfg) = &config.provider
    {
        cfg.clone()
    } else {
        eprintln!("Skipping test - default config is not local");
        return;
    };
    let base = if let Ok(provider) = LocalEmbeddingProvider::new(local_config).await {
        Arc::new(provider)
    } else {
        eprintln!("Skipping test - embedding model not available");
        return;
    };

    let mut embeddings = ContextAwareEmbeddings::new(base);

    // Train adapters for different task types
    let task_types = vec![
        TaskType::CodeGeneration,
        TaskType::Debugging,
        TaskType::Refactoring,
    ];

    for task_type in task_types {
        let pairs = vec![ContrastivePair {
            anchor: create_test_episode(task_type, "task 1", "domain"),
            positive: create_test_episode(task_type, "task 2", "domain"),
            negative: create_test_episode(TaskType::Analysis, "analyze", "domain"),
        }];

        embeddings.train_adapter(task_type, &pairs).await.unwrap();
    }

    // Verify all adapters created
    assert_eq!(embeddings.adapter_count(), 3);
    assert!(embeddings.has_adapter(TaskType::CodeGeneration));
    assert!(embeddings.has_adapter(TaskType::Debugging));
    assert!(embeddings.has_adapter(TaskType::Refactoring));
    assert!(!embeddings.has_adapter(TaskType::Testing));
}

#[tokio::test]
async fn test_empty_training_pairs_error() {
    let _config = memory_core::embeddings::EmbeddingConfig::default();
    // Use mock provider for this async test
    let mock = memory_core::embeddings::MockLocalModel::new("mock".to_string(), 128);
    let mut embeddings = ContextAwareEmbeddings::new(Arc::new(mock));

    let result = embeddings
        .train_adapter(TaskType::CodeGeneration, &[])
        .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("empty"));
}

#[tokio::test]
async fn test_backward_compatibility_no_adapters() {
    let config = memory_core::embeddings::EmbeddingConfig::default();
    let local_config = if let memory_core::embeddings::ProviderConfig::Local(cfg) = &config.provider
    {
        cfg.clone()
    } else {
        eprintln!("Skipping test - default config is not local");
        return;
    };
    let base = if let Ok(provider) = LocalEmbeddingProvider::new(local_config).await {
        Arc::new(provider)
    } else {
        eprintln!("Skipping test - embedding model not available");
        return;
    };

    let embeddings = ContextAwareEmbeddings::new(base.clone());

    let text = "test task";

    // All should return base embedding
    let none_result = embeddings.get_adapted_embedding(text, None).await.unwrap();
    let some_result = embeddings
        .get_adapted_embedding(text, Some(TaskType::CodeGeneration))
        .await
        .unwrap();
    let base_result = base.embed_text(text).await.unwrap();

    assert_eq!(none_result, base_result);
    assert_eq!(some_result, base_result);
}
