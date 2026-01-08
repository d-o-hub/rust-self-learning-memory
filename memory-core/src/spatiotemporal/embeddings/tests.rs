//! Tests for spatiotemporal embeddings module.

use super::*;
use crate::embeddings::EmbeddingProvider;
use crate::types::{ComplexityLevel, TaskContext, TaskOutcome};
use async_trait::async_trait;

// Mock embedding provider for testing
struct MockEmbeddingProvider {
    dimension: usize,
}

impl MockEmbeddingProvider {
    fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

#[async_trait]
impl EmbeddingProvider for MockEmbeddingProvider {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let mut embedding = vec![0.0; self.dimension];
        let text_len = text.len() as f32;
        for (i, val) in embedding.iter_mut().enumerate() {
            *val = (i as f32 + text_len) / 100.0;
        }
        Ok(embedding)
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();
        for text in texts {
            embeddings.push(self.embed_text(text).await?);
        }
        Ok(embeddings)
    }

    async fn similarity(&self, text1: &str, text2: &str) -> Result<f32> {
        let emb1 = self.embed_text(text1).await?;
        let emb2 = self.embed_text(text2).await?;
        Ok(cosine_similarity(&emb1, &emb2))
    }

    fn embedding_dimension(&self) -> usize {
        self.dimension
    }

    fn model_name(&self) -> &'static str {
        "mock-embeddings"
    }
}

fn create_test_episode(task_type: TaskType, name: &str) -> Episode {
    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: Some("tokio".to_string()),
        complexity: ComplexityLevel::Moderate,
        domain: "test".to_string(),
        tags: vec!["test".to_string()],
    };

    let mut episode = Episode::new(name.to_string(), context, task_type);
    episode.complete(TaskOutcome::Success {
        verdict: "Test completed".to_string(),
        artifacts: vec![],
    });
    episode
}

#[test]
fn test_task_adapter_identity_initialization() {
    let adapter = TaskAdapter::new_identity(TaskType::CodeGeneration, 384);

    assert_eq!(adapter.task_type, TaskType::CodeGeneration);
    assert_eq!(adapter.trained_on_count, 0);
    assert_eq!(adapter.adaptation_matrix.len(), 384);
    assert_eq!(adapter.adaptation_matrix[0].len(), 384);
}

#[test]
fn test_task_adapter_apply_identity() {
    let adapter = TaskAdapter::new_identity(TaskType::CodeGeneration, 4);
    let input = vec![1.0, 2.0, 3.0, 4.0];

    let output = adapter.apply(&input);

    assert_eq!(input, output);
}

#[test]
fn test_task_adapter_apply_custom() {
    let adapter = TaskAdapter {
        task_type: TaskType::CodeGeneration,
        adaptation_matrix: vec![vec![2.0, 0.0], vec![0.0, 3.0]],
        trained_on_count: 1,
    };
    let input = vec![1.0, 2.0];

    let output = adapter.apply(&input);

    assert_eq!(output, vec![2.0, 6.0]);
}

#[test]
fn test_context_aware_embeddings_creation() {
    let provider = Arc::new(MockEmbeddingProvider::new(384));
    let embeddings = ContextAwareEmbeddings::new(provider);

    assert_eq!(embeddings.adapter_count(), 0);
}

#[tokio::test]
async fn test_get_embedding_without_adapter() {
    let provider = Arc::new(MockEmbeddingProvider::new(384));
    let embeddings = ContextAwareEmbeddings::new(provider);

    let result = embeddings.get_embedding("test query").await;

    assert!(result.is_ok());
    let embedding = result.unwrap();
    assert_eq!(embedding.len(), 384);
}

#[tokio::test]
async fn test_get_adapted_embedding_without_adapter() {
    let provider = Arc::new(MockEmbeddingProvider::new(384));
    let embeddings = ContextAwareEmbeddings::new(provider);

    let result = embeddings
        .get_adapted_embedding("test query", Some(TaskType::CodeGeneration))
        .await;

    assert!(result.is_ok());
    let embedding = result.unwrap();
    assert_eq!(embedding.len(), 384);
}

#[tokio::test]
async fn test_get_adapted_embedding_with_adapter() {
    let provider = Arc::new(MockEmbeddingProvider::new(384));
    let mut embeddings = ContextAwareEmbeddings::new(provider);

    // Create training data
    let pairs = vec![
        ContrastivePair {
            anchor: create_test_episode(TaskType::CodeGeneration, "code 1"),
            positive: create_test_episode(TaskType::CodeGeneration, "code 2"),
            negative: create_test_episode(TaskType::Debugging, "debug 1"),
        },
        ContrastivePair {
            anchor: create_test_episode(TaskType::CodeGeneration, "code 3"),
            positive: create_test_episode(TaskType::CodeGeneration, "code 4"),
            negative: create_test_episode(TaskType::Debugging, "debug 2"),
        },
    ];

    // Train adapter
    embeddings
        .train_adapter(TaskType::CodeGeneration, &pairs)
        .await
        .unwrap();

    // Get adapted embedding
    let result = embeddings
        .get_adapted_embedding("write tests", Some(TaskType::CodeGeneration))
        .await;

    assert!(result.is_ok());
    let embedding = result.unwrap();
    assert_eq!(embedding.len(), 384);
}

#[tokio::test]
async fn test_train_adapter() {
    let provider = Arc::new(MockEmbeddingProvider::new(256));
    let mut embeddings = ContextAwareEmbeddings::new(provider);

    let pairs = vec![ContrastivePair {
        anchor: create_test_episode(TaskType::CodeGeneration, "code 1"),
        positive: create_test_episode(TaskType::CodeGeneration, "code 2"),
        negative: create_test_episode(TaskType::Debugging, "debug 1"),
    }];

    let result = embeddings
        .train_adapter(TaskType::CodeGeneration, &pairs)
        .await;

    assert!(result.is_ok());
    assert!(embeddings.has_adapter(TaskType::CodeGeneration));
    assert_eq!(embeddings.adapter_count(), 1);
}

#[tokio::test]
async fn test_train_adapter_multiple_task_types() {
    let provider = Arc::new(MockEmbeddingProvider::new(256));
    let mut embeddings = ContextAwareEmbeddings::new(provider);

    let coding_pairs = vec![ContrastivePair {
        anchor: create_test_episode(TaskType::CodeGeneration, "code 1"),
        positive: create_test_episode(TaskType::CodeGeneration, "code 2"),
        negative: create_test_episode(TaskType::Debugging, "debug 1"),
    }];

    let debug_pairs = vec![ContrastivePair {
        anchor: create_test_episode(TaskType::Debugging, "debug 1"),
        positive: create_test_episode(TaskType::Debugging, "debug 2"),
        negative: create_test_episode(TaskType::CodeGeneration, "code 3"),
    }];

    embeddings
        .train_adapter(TaskType::CodeGeneration, &coding_pairs)
        .await
        .unwrap();
    embeddings
        .train_adapter(TaskType::Debugging, &debug_pairs)
        .await
        .unwrap();

    assert_eq!(embeddings.adapter_count(), 2);
    assert!(embeddings.has_adapter(TaskType::CodeGeneration));
    assert!(embeddings.has_adapter(TaskType::Debugging));
    assert!(!embeddings.has_adapter(TaskType::Testing));
}

#[test]
fn test_contrastive_pair_structure() {
    let anchor = create_test_episode(TaskType::CodeGeneration, "anchor");
    let positive = create_test_episode(TaskType::CodeGeneration, "positive");
    let negative = create_test_episode(TaskType::Debugging, "negative");

    let pair = ContrastivePair {
        anchor: anchor.clone(),
        positive: positive.clone(),
        negative: negative.clone(),
    };

    assert_eq!(pair.anchor.task_type, anchor.task_type);
    assert_eq!(pair.positive.task_type, positive.task_type);
    assert_eq!(pair.negative.task_type, negative.task_type);
}

#[tokio::test]
async fn test_embedding_dimension_consistency() {
    let provider = Arc::new(MockEmbeddingProvider::new(256));
    let mut embeddings = ContextAwareEmbeddings::new(provider);

    let pairs = vec![ContrastivePair {
        anchor: create_test_episode(TaskType::Analysis, "analyze 1"),
        positive: create_test_episode(TaskType::Analysis, "analyze 2"),
        negative: create_test_episode(TaskType::Testing, "test 1"),
    }];

    embeddings
        .train_adapter(TaskType::Analysis, &pairs)
        .await
        .unwrap();

    let embedding = embeddings
        .get_adapted_embedding("analyze data", Some(TaskType::Analysis))
        .await
        .unwrap();

    assert_eq!(embedding.len(), 256);
}
