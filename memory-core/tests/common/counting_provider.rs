//! A deterministic embedding provider that counts how often it is asked to
//! embed text.
//!
//! Reused by the embedding-activation and pattern-search integration tests to
//! prove that *every* core path (completion, retrieval, pattern search) reaches
//! the runtime-activated provider instead of the construction-time
//! `semantic_service` field (issue #1074).

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::Result;
use async_trait::async_trait;
use do_memory_core::embeddings::{
    EmbeddingConfig, EmbeddingHealth, EmbeddingProvider, InMemoryEmbeddingStorage, MockLocalModel,
    SemanticService,
};

/// Wraps [`MockLocalModel`] and records every embedding request.
pub struct CountingProvider {
    inner: MockLocalModel,
    embed_calls: Arc<AtomicUsize>,
}

impl CountingProvider {
    /// Build a provider plus the shared counter it increments.
    pub fn new(model: &str, dimension: usize) -> (Self, Arc<AtomicUsize>) {
        let embed_calls = Arc::new(AtomicUsize::new(0));
        (
            Self {
                inner: MockLocalModel::new(model.to_string(), dimension),
                embed_calls: Arc::clone(&embed_calls),
            },
            embed_calls,
        )
    }
}

#[async_trait]
impl EmbeddingProvider for CountingProvider {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        self.embed_calls.fetch_add(1, Ordering::SeqCst);
        self.inner.embed_text(text).await
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        self.embed_calls.fetch_add(texts.len(), Ordering::SeqCst);
        self.inner.embed_batch(texts).await
    }

    async fn similarity(&self, text1: &str, text2: &str) -> Result<f32> {
        self.inner.similarity(text1, text2).await
    }

    fn embedding_dimension(&self) -> usize {
        self.inner.embedding_dimension()
    }

    fn model_name(&self) -> &str {
        self.inner.model_name()
    }

    async fn is_available(&self) -> bool {
        self.inner.is_available().await
    }

    async fn health(&self) -> EmbeddingHealth {
        self.inner.health().await
    }

    async fn warmup(&self) -> Result<()> {
        self.inner.warmup().await
    }

    fn metadata(&self) -> serde_json::Value {
        self.inner.metadata()
    }
}

/// Build a [`SemanticService`] backed by a [`CountingProvider`] and return the
/// shared call counter.
pub fn counting_service(model: &str) -> (Arc<SemanticService>, Arc<AtomicUsize>) {
    let (provider, calls) = CountingProvider::new(model, 4);
    let service = SemanticService::new(
        Box::new(provider),
        Box::new(InMemoryEmbeddingStorage::new()),
        EmbeddingConfig::default(),
    );
    (Arc::new(service), calls)
}
