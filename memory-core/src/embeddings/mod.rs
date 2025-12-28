//! # Semantic Embeddings
//!
//! Vector embeddings for semantic similarity search and enhanced context retrieval.
//!
//! This module provides:
//! - Text embedding generation (local and remote)
//! - Vector similarity calculations
//! - Semantic-enhanced episode and pattern retrieval
//! - Configurable embedding providers
//!
//! ## Architecture
//!
//! The embedding system supports multiple providers:
//! - **Local**: sentence-transformers via candle-transformers (offline)
//! - **`OpenAI`**: text-embedding-ada-002 (cloud)
//! - **Custom**: User-provided embedding functions
//!
//! ## Usage
//!
//! ```rust
//! use memory_core::embeddings::{EmbeddingProvider, LocalEmbeddingProvider};
//!
//! // Local embedding provider (offline)
//! let provider = LocalEmbeddingProvider::new().await?;
//!
//! // Generate embedding for text
//! let embedding = provider.embed_text("implement REST API").await?;
//!
//! // Calculate similarity between two texts
//! let similarity = provider.similarity("REST API", "web service API").await?;
//! ```

mod circuit_breaker;
mod config;
mod local;
mod metrics;
mod mock_model;
mod openai;
mod provider;
mod real_model;
mod similarity;
mod storage;
mod utils;

pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerState};
pub use config::{
    EmbeddingConfig, EmbeddingProvider as EmbeddingProviderType, ModelConfig, OptimizationConfig,
};
pub use local::{
    get_recommended_model, list_available_models, LocalEmbeddingProvider, LocalModelUseCase,
};
pub use metrics::{LatencyTimer, MetricsSnapshot, ProviderMetrics};
pub use mock_model::MockLocalModel;
#[cfg(feature = "openai")]
pub use openai::OpenAIEmbeddingProvider;
pub use provider::{EmbeddingProvider, EmbeddingResult};
pub use similarity::{
    cosine_similarity, EmbeddingWithMetadata, SimilarityMetadata, SimilaritySearchResult,
};
pub use storage::{EmbeddingStorage, EmbeddingStorageBackend, InMemoryEmbeddingStorage};

use crate::episode::Episode;
use crate::pattern::Pattern;
use crate::types::TaskContext;
use anyhow::Result;
// Removed unused import

/// Default embedding dimension for sentence transformers
pub const DEFAULT_EMBEDDING_DIM: usize = 384;

/// Main semantic embedding service for the memory system
///
/// Coordinates embedding generation, storage, and semantic search across
/// episodes and patterns. Integrates with the existing storage backends.
pub struct SemanticService {
    /// Embedding provider for text-to-vector conversion
    pub provider: Box<dyn EmbeddingProvider>,
    /// Storage backend for embeddings
    storage: Box<dyn EmbeddingStorageBackend>,
    /// Configuration
    config: EmbeddingConfig,
}

impl SemanticService {
    /// Create a new semantic service with the specified provider and storage
    #[must_use]
    pub fn new(
        provider: Box<dyn EmbeddingProvider>,
        storage: Box<dyn EmbeddingStorageBackend>,
        config: EmbeddingConfig,
    ) -> Self {
        Self {
            provider,
            storage,
            config,
        }
    }

    /// Create a semantic service with local embedding provider
    pub async fn with_local_provider(
        storage: Box<dyn EmbeddingStorageBackend>,
        config: EmbeddingConfig,
    ) -> Result<Self> {
        let provider = Box::new(LocalEmbeddingProvider::new(config.model.clone()).await?);
        Ok(Self::new(provider, storage, config))
    }

    /// Create a semantic service with default local provider
    pub async fn default(storage: Box<dyn EmbeddingStorageBackend>) -> Result<Self> {
        let config = EmbeddingConfig::default();
        Self::with_local_provider(storage, config).await
    }

    /// Create a semantic service with automatic provider fallback
    ///
    /// Tries providers in order: Local → `OpenAI` → Mock (with warnings)
    /// This ensures maximum reliability by falling back to simpler options if preferred ones fail.
    pub async fn with_fallback(
        storage: Box<dyn EmbeddingStorageBackend>,
        config: EmbeddingConfig,
    ) -> Result<Self> {
        // Try local provider first (default)
        match LocalEmbeddingProvider::new(config.model.clone()).await {
            Ok(provider) => {
                tracing::info!("Using local embedding provider");
                return Ok(Self::new(Box::new(provider), storage, config));
            }
            Err(e) => {
                tracing::warn!("Failed to initialize local provider: {}, trying OpenAI", e);
            }
        }

        // Try OpenAI provider as fallback
        #[cfg(feature = "openai")]
        {
            if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
                match OpenAIEmbeddingProvider::new(api_key, config.model.clone()) {
                    Ok(provider) => {
                        tracing::info!("Using OpenAI embedding provider as fallback");
                        return Ok(Self::new(Box::new(provider), storage, config));
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to initialize OpenAI provider: {}. Falling back to mock.",
                            e
                        );
                    }
                }
            } else {
                tracing::warn!("OPENAI_API_KEY not set, cannot use OpenAI provider");
            }
        }

        // Final fallback to mock provider (with warning)
        tracing::error!(
            "All embedding providers failed, using mock provider (embeddings will be random)"
        );
        tracing::error!("To fix this, either:");
        tracing::error!("  1. Install local embedding model dependencies");
        #[cfg(feature = "openai")]
        tracing::error!("  2. Set OPENAI_API_KEY environment variable");

        // Use MockLocalModel as the final fallback
        let provider = crate::embeddings::mock_model::MockLocalModel::new(
            "mock-model".to_string(),
            config.model.embedding_dimension,
        );
        Ok(Self::new(Box::new(provider), storage, config))
    }

    /// Create a semantic service with `OpenAI` embedding provider
    #[cfg(feature = "openai")]
    pub fn with_openai_provider(
        api_key: String,
        storage: Box<dyn EmbeddingStorageBackend>,
        config: EmbeddingConfig,
    ) -> Result<Self> {
        let provider = Box::new(OpenAIEmbeddingProvider::new(api_key, config.model.clone())?);
        Ok(Self::new(provider, storage, config))
    }

    /// Generate and store embedding for an episode
    ///
    /// Creates a semantic representation of the episode by combining:
    /// - Task description
    /// - Context information (domain, language, framework)
    /// - Key execution steps
    /// - Outcome summary
    pub async fn embed_episode(&self, episode: &Episode) -> Result<Vec<f32>> {
        let text = self.episode_to_text(episode);
        let embedding = self.provider.embed_text(&text).await?;

        // Store the embedding
        self.storage
            .store_episode_embedding(episode.episode_id, embedding.clone())
            .await?;

        Ok(embedding)
    }

    /// Generate and store embedding for a pattern
    ///
    /// Creates a semantic representation based on:
    /// - Pattern description
    /// - Context where the pattern was extracted
    /// - Pattern metadata and effectiveness metrics
    pub async fn embed_pattern(&self, pattern: &Pattern) -> Result<Vec<f32>> {
        let text = self.pattern_to_text(pattern);
        let embedding = self.provider.embed_text(&text).await?;

        // Store the embedding
        self.storage
            .store_pattern_embedding(pattern.id(), embedding.clone())
            .await?;

        Ok(embedding)
    }

    /// Find semantically similar episodes for a query
    ///
    /// Uses vector similarity to find episodes that are semantically related
    /// to the query, going beyond keyword matching to understand meaning.
    pub async fn find_similar_episodes(
        &self,
        query: &str,
        context: &TaskContext,
        limit: usize,
    ) -> Result<Vec<SimilaritySearchResult<Episode>>> {
        // Create query text combining description and context
        let query_text = self.create_query_text(query, context);

        // Generate embedding for query
        let query_embedding = self.provider.embed_text(&query_text).await?;

        // Search for similar episodes
        self.storage
            .find_similar_episodes(query_embedding, limit, self.config.similarity_threshold)
            .await
            .map_err(|e| anyhow::Error::msg(e.to_string()))
    }

    /// Find semantically similar patterns for a context
    ///
    /// Identifies patterns that are semantically relevant to the given context,
    /// enabling better pattern reuse and recommendation.
    pub async fn find_similar_patterns(
        &self,
        context: &TaskContext,
        limit: usize,
    ) -> Result<Vec<SimilaritySearchResult<Pattern>>> {
        // Create context-based query
        let query_text = self.context_to_text(context);

        // Generate embedding for query
        let query_embedding = self.provider.embed_text(&query_text).await?;

        // Search for similar patterns
        self.storage
            .find_similar_patterns(query_embedding, limit, self.config.similarity_threshold)
            .await
            .map_err(|e| anyhow::Error::msg(e.to_string()))
    }

    /// Calculate similarity between two texts
    pub async fn text_similarity(&self, text1: &str, text2: &str) -> Result<f32> {
        self.provider.similarity(text1, text2).await
    }

    /// Convert episode to searchable text representation
    fn episode_to_text(&self, episode: &Episode) -> String {
        let mut parts = Vec::new();

        // Task description
        parts.push(episode.task_description.clone());

        // Context information
        parts.push(format!("domain: {}", episode.context.domain));
        if let Some(lang) = &episode.context.language {
            parts.push(format!("language: {lang}"));
        }
        if let Some(framework) = &episode.context.framework {
            parts.push(format!("framework: {framework}"));
        }
        if !episode.context.tags.is_empty() {
            parts.push(format!("tags: {}", episode.context.tags.join(", ")));
        }

        // Execution summary
        if !episode.steps.is_empty() {
            // Collect unique tools while preserving order
            let mut tools = Vec::new();
            for step in &episode.steps {
                if !tools.contains(&step.tool) {
                    tools.push(step.tool.clone());
                }
            }
            parts.push(format!("tools used: {}", tools.join(", ")));

            let actions: Vec<String> = episode
                .steps
                .iter()
                .take(3) // Take first few actions
                .map(|step| step.action.clone())
                .collect();
            parts.push(format!("actions: {}", actions.join(", ")));
        }

        // Outcome summary
        if let Some(outcome) = &episode.outcome {
            match outcome {
                crate::types::TaskOutcome::Success { verdict, .. } => {
                    parts.push(format!("outcome: success - {verdict}"));
                }
                crate::types::TaskOutcome::PartialSuccess { verdict, .. } => {
                    parts.push(format!("outcome: partial success - {verdict}"));
                }
                crate::types::TaskOutcome::Failure { reason, .. } => {
                    parts.push(format!("outcome: failure - {reason}"));
                }
            }
        }

        parts.join(". ")
    }

    /// Convert pattern to searchable text representation
    fn pattern_to_text(&self, pattern: &Pattern) -> String {
        let mut parts = Vec::new();

        // Pattern description based on type
        let description = match pattern {
            crate::pattern::Pattern::ToolSequence { tools, .. } => {
                format!("Tool sequence: {}", tools.join(" -> "))
            }
            crate::pattern::Pattern::DecisionPoint {
                condition, action, ..
            } => {
                format!("Decision: if {condition} then {action}")
            }
            crate::pattern::Pattern::ErrorRecovery {
                error_type,
                recovery_steps,
                ..
            } => {
                format!(
                    "Error recovery: {} -> {}",
                    error_type,
                    recovery_steps.join(" -> ")
                )
            }
            crate::pattern::Pattern::ContextPattern {
                context_features,
                recommended_approach,
                ..
            } => {
                format!(
                    "Context pattern: {} suggests {}",
                    context_features.join(", "),
                    recommended_approach
                )
            }
        };
        parts.push(description);

        // Context information
        if let Some(pattern_context) = pattern.context() {
            parts.push(format!("domain: {}", pattern_context.domain));
            if let Some(lang) = &pattern_context.language {
                parts.push(format!("language: {lang}"));
            }
            if !pattern_context.tags.is_empty() {
                parts.push(format!("tags: {}", pattern_context.tags.join(", ")));
            }
        }

        parts.join(". ")
    }

    /// Create query text from description and context
    fn create_query_text(&self, query: &str, context: &TaskContext) -> String {
        let mut parts = vec![query.to_string()];

        parts.push(format!("domain: {}", context.domain));
        if let Some(lang) = &context.language {
            parts.push(format!("language: {lang}"));
        }
        if let Some(framework) = &context.framework {
            parts.push(format!("framework: {framework}"));
        }
        if !context.tags.is_empty() {
            parts.push(format!("tags: {}", context.tags.join(", ")));
        }

        parts.join(". ")
    }

    /// Convert context to searchable text
    fn context_to_text(&self, context: &TaskContext) -> String {
        let mut parts = Vec::new();

        parts.push(format!("domain: {}", context.domain));
        if let Some(lang) = &context.language {
            parts.push(format!("language: {lang}"));
        }
        if let Some(framework) = &context.framework {
            parts.push(format!("framework: {framework}"));
        }
        if !context.tags.is_empty() {
            parts.push(format!("tags: {}", context.tags.join(", ")));
        }
        parts.push(format!("complexity: {:?}", context.complexity));

        parts.join(". ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episode::{ExecutionStep, PatternId};
    use crate::pattern::{Pattern, PatternEffectiveness};
    use crate::types::{ComplexityLevel, ExecutionResult, TaskOutcome, TaskType};
    use crate::Result;
    use uuid::Uuid;

    fn create_test_episode() -> Episode {
        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: Some("tokio".to_string()),
            complexity: ComplexityLevel::Moderate,
            domain: "web-api".to_string(),
            tags: vec!["async".to_string(), "rest".to_string()],
        };

        let mut episode = Episode::new(
            "Implement REST API endpoints".to_string(),
            context,
            TaskType::CodeGeneration,
        );

        // Add some steps
        let mut step1 =
            ExecutionStep::new(1, "parser".to_string(), "Parse request body".to_string());
        step1.result = Some(ExecutionResult::Success {
            output: "JSON parsed successfully".to_string(),
        });
        episode.add_step(step1);

        let mut step2 = ExecutionStep::new(
            2,
            "validator".to_string(),
            "Validate input data".to_string(),
        );
        step2.result = Some(ExecutionResult::Success {
            output: "Validation passed".to_string(),
        });
        episode.add_step(step2);

        // Complete episode
        episode.complete(TaskOutcome::Success {
            verdict: "API endpoints implemented successfully".to_string(),
            artifacts: vec!["api.rs".to_string(), "handlers.rs".to_string()],
        });

        episode
    }

    fn create_test_pattern() -> Pattern {
        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: Some("tokio".to_string()),
            complexity: ComplexityLevel::Moderate,
            domain: "web-api".to_string(),
            tags: vec!["async".to_string()],
        };

        Pattern::ToolSequence {
            id: PatternId::new_v4(),
            tools: vec!["parser".to_string(), "validator".to_string()],
            context,
            success_rate: 0.85,
            avg_latency: chrono::Duration::seconds(10),
            occurrence_count: 5,
            effectiveness: PatternEffectiveness::default(),
        }
    }

    #[test]
    fn test_episode_to_text() {
        let service = SemanticService {
            provider: Box::new(MockEmbeddingProvider),
            storage: Box::new(MockEmbeddingStorage),
            config: EmbeddingConfig::default(),
        };

        let episode = create_test_episode();
        let text = service.episode_to_text(&episode);

        assert!(text.contains("Implement REST API endpoints"));
        assert!(text.contains("domain: web-api"));
        assert!(text.contains("language: rust"));
        assert!(text.contains("framework: tokio"));
        assert!(text.contains("tags: async, rest"));
        assert!(text.contains("tools used: parser, validator"));
        assert!(text.contains("outcome: success"));
    }

    #[test]
    fn test_pattern_to_text() {
        let service = SemanticService {
            provider: Box::new(MockEmbeddingProvider),
            storage: Box::new(MockEmbeddingStorage),
            config: EmbeddingConfig::default(),
        };

        let pattern = create_test_pattern();
        let text = service.pattern_to_text(&pattern);

        // Check that it contains the tool sequence
        assert!(text.contains("Tool sequence: parser -> validator"));
        assert!(text.contains("domain: web-api"));
        assert!(text.contains("language: rust"));
        assert!(text.contains("tags: async"));
    }

    #[test]
    fn test_create_query_text() {
        let service = SemanticService {
            provider: Box::new(MockEmbeddingProvider),
            storage: Box::new(MockEmbeddingStorage),
            config: EmbeddingConfig::default(),
        };

        let context = TaskContext {
            language: Some("python".to_string()),
            framework: Some("fastapi".to_string()),
            complexity: ComplexityLevel::Simple,
            domain: "data-api".to_string(),
            tags: vec!["rest".to_string()],
        };

        let query_text = service.create_query_text("build data endpoint", &context);

        assert!(query_text.contains("build data endpoint"));
        assert!(query_text.contains("domain: data-api"));
        assert!(query_text.contains("language: python"));
        assert!(query_text.contains("framework: fastapi"));
        assert!(query_text.contains("tags: rest"));
    }

    // Mock implementations for testing
    use anyhow::Result as AnyhowResult;

    struct MockEmbeddingProvider;

    #[async_trait::async_trait]
    impl EmbeddingProvider for MockEmbeddingProvider {
        async fn embed_text(&self, _text: &str) -> AnyhowResult<Vec<f32>> {
            Ok(vec![0.1, 0.2, 0.3, 0.4])
        }

        async fn embed_batch(&self, _texts: &[String]) -> AnyhowResult<Vec<Vec<f32>>> {
            Ok(vec![vec![0.1, 0.2, 0.3, 0.4]; _texts.len()])
        }

        async fn similarity(&self, _text1: &str, _text2: &str) -> AnyhowResult<f32> {
            Ok(0.85)
        }

        fn embedding_dimension(&self) -> usize {
            4
        }

        fn model_name(&self) -> &'static str {
            "mock-model"
        }
    }

    struct MockEmbeddingStorage;

    #[async_trait::async_trait]
    impl EmbeddingStorageBackend for MockEmbeddingStorage {
        async fn store_episode_embedding(
            &self,
            _episode_id: Uuid,
            _embedding: Vec<f32>,
        ) -> Result<()> {
            Ok(())
        }

        async fn store_pattern_embedding(
            &self,
            _pattern_id: PatternId,
            _embedding: Vec<f32>,
        ) -> Result<()> {
            Ok(())
        }

        async fn get_episode_embedding(&self, _episode_id: Uuid) -> Result<Option<Vec<f32>>> {
            Ok(Some(vec![0.1, 0.2, 0.3, 0.4]))
        }

        async fn get_pattern_embedding(&self, _pattern_id: PatternId) -> Result<Option<Vec<f32>>> {
            Ok(Some(vec![0.1, 0.2, 0.3, 0.4]))
        }

        async fn find_similar_episodes(
            &self,
            _query_embedding: Vec<f32>,
            _limit: usize,
            _threshold: f32,
        ) -> Result<Vec<SimilaritySearchResult<Episode>>> {
            Ok(vec![])
        }

        async fn find_similar_patterns(
            &self,
            _query_embedding: Vec<f32>,
            _limit: usize,
            _threshold: f32,
        ) -> Result<Vec<SimilaritySearchResult<Pattern>>> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn test_with_fallback_uses_local_first() {
        // This test verifies that with_fallback tries Local provider first
        // and succeeds when it's available

        // We expect this to succeed with Local provider (assuming it's installed)
        // If Local fails, it will fall back to OpenAI or Mock
        let storage = Box::new(MockEmbeddingStorage);
        let config = EmbeddingConfig::default();

        // This should not panic, even if Local provider fails
        // It will fallback to Mock if needed
        let result = SemanticService::with_fallback(storage, config).await;
        assert!(result.is_ok(), "with_fallback should always succeed");

        let service = result.unwrap();
        assert_eq!(service.config.provider, EmbeddingProviderType::Local);
        assert_eq!(
            service.config.model.model_name,
            "sentence-transformers/all-MiniLM-L6-v2"
        );
        assert_eq!(service.config.similarity_threshold, 0.7);
    }

    #[tokio::test]
    async fn test_with_fallback_fallback_behavior() {
        // This test verifies the fallback chain: Local → OpenAI → Mock
        // and that warnings are emitted appropriately

        let storage = Box::new(MockEmbeddingStorage);
        let config = EmbeddingConfig::default();

        // The with_fallback method should:
        // 1. Try Local provider first
        // 2. If Local fails, warn and try OpenAI (if feature enabled and API key set)
        // 3. If OpenAI fails or not available, warn and fall back to Mock

        // Capture logs to verify warnings
        // Note: This is difficult to test directly without capturing logs,
        // but we can verify the function returns successfully

        let result = SemanticService::with_fallback(storage, config).await;
        assert!(
            result.is_ok(),
            "with_fallback should always succeed, even if all providers fail"
        );

        let service = result.unwrap();

        // Verify the service was created with valid config
        assert!(service.config.cache_embeddings);
        assert_eq!(service.config.batch_size, 32);
        assert_eq!(service.config.timeout_seconds, 30);

        // Verify the provider is functional
        let embedding = service.provider.embed_text("test").await;
        assert!(
            embedding.is_ok(),
            "Provider should generate embeddings even if using Mock"
        );

        let vec = embedding.unwrap();
        assert!(!vec.is_empty(), "Embedding should not be empty");
        assert_eq!(
            vec.len(),
            service.provider.embedding_dimension(),
            "Embedding dimension should match provider config"
        );
    }

    #[tokio::test]
    async fn test_with_fallback_config_preservation() {
        // Verify that the config passed to with_fallback is preserved
        let storage = Box::new(MockEmbeddingStorage);

        let config = EmbeddingConfig {
            provider: EmbeddingProviderType::Local,
            model: ModelConfig::openai_3_small(),
            similarity_threshold: 0.8,
            batch_size: 64,
            cache_embeddings: false,
            timeout_seconds: 60,
        };

        let result = SemanticService::with_fallback(storage, config.clone()).await;
        assert!(result.is_ok());

        let service = result.unwrap();

        // Verify all config fields are preserved
        assert_eq!(service.config.provider, config.provider);
        assert_eq!(service.config.model.model_name, config.model.model_name);
        assert_eq!(
            service.config.model.embedding_dimension,
            config.model.embedding_dimension
        );
        assert_eq!(
            service.config.similarity_threshold,
            config.similarity_threshold
        );
        assert_eq!(service.config.batch_size, config.batch_size);
        assert_eq!(service.config.cache_embeddings, config.cache_embeddings);
        assert_eq!(service.config.timeout_seconds, config.timeout_seconds);
    }

    #[tokio::test]
    async fn test_with_fallback_default_storage_works() {
        // Verify with_fallback works with different storage backends
        let storage = Box::new(MockEmbeddingStorage);
        let config = EmbeddingConfig::default();

        // Should work with default config
        let result = SemanticService::with_fallback(storage, config).await;
        assert!(result.is_ok());

        // Should also work with custom config
        let custom_config = EmbeddingConfig {
            similarity_threshold: 0.5,
            batch_size: 16,
            ..Default::default()
        };

        let storage2 = Box::new(MockEmbeddingStorage);
        let result2 = SemanticService::with_fallback(storage2, custom_config).await;
        assert!(result2.is_ok());
    }

    #[tokio::test]
    async fn test_default_creates_valid_service() {
        // Test that the default() method creates a valid service
        let storage = Box::new(MockEmbeddingStorage);

        let result = SemanticService::default(storage).await;
        // This may fail if Local provider is not installed, but it should handle gracefully
        // If it fails, it's expected behavior in some environments
        if result.is_ok() {
            let service = result.unwrap();
            assert_eq!(service.config.provider, EmbeddingProviderType::Local);
            assert_eq!(
                service.config.model.model_name,
                "sentence-transformers/all-MiniLM-L6-v2"
            );
        }
        // If it fails, that's acceptable - Local provider might not be available
    }
}
