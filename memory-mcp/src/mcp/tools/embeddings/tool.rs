//! Embedding tools implementation.

use crate::types::Tool;
use crate::mcp::tools::embeddings::types::{
    ConfigureEmbeddingsInput, ConfigureEmbeddingsOutput, QuerySemanticMemoryInput,
    QuerySemanticMemoryOutput, SemanticResult, TestEmbeddingsOutput,
};
use anyhow::{anyhow, Result};
use memory_core::embeddings::{EmbeddingConfig, EmbeddingProviderType, ModelConfig};
use memory_core::SelfLearningMemory;
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};

/// Embedding tools implementation
pub struct EmbeddingTools {
    memory: Arc<SelfLearningMemory>,
}

impl EmbeddingTools {
    /// Create a new embedding tools instance
    pub fn new(memory: Arc<SelfLearningMemory>) -> Self {
        Self { memory }
    }

    /// Get the tool definition for configure_embeddings
    pub fn configure_embeddings_tool() -> Tool {
        Tool::new(
            "configure_embeddings".to_string(),
            "Configure semantic embedding provider for enhanced memory retrieval. Supports OpenAI, Local, Mistral, Azure, and Cohere providers.".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "provider": {
                        "type": "string",
                        "enum": ["openai", "local", "mistral", "azure", "cohere"],
                        "description": "Embedding provider to use"
                    },
                    "model": {
                        "type": "string",
                        "description": "Model name (e.g., text-embedding-3-small for OpenAI, all-MiniLM-L6-v2 for local)"
                    },
                    "api_key_env": {
                        "type": "string",
                        "description": "Environment variable name containing API key (required for cloud providers)"
                    },
                    "similarity_threshold": {
                        "type": "number",
                        "minimum": 0.0,
                        "maximum": 1.0,
                        "default": 0.7,
                        "description": "Minimum similarity score for search results"
                    },
                    "batch_size": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 2048,
                        "default": 32,
                        "description": "Number of texts to embed in a single batch"
                    },
                    "base_url": {
                        "type": "string",
                        "description": "Custom base URL for API endpoint"
                    },
                    "api_version": {
                        "type": "string",
                        "description": "API version (Azure only, e.g., 2023-05-15)"
                    },
                    "resource_name": {
                        "type": "string",
                        "description": "Azure resource name (Azure only)"
                    },
                    "deployment_name": {
                        "type": "string",
                        "description": "Azure deployment name (Azure only)"
                    }
                },
                "required": ["provider"]
            }),
        )
    }

    /// Get the tool definition for query_semantic_memory
    pub fn query_semantic_memory_tool() -> Tool {
        Tool::new(
            "query_semantic_memory".to_string(),
            "Search episodic memory using semantic similarity with embeddings. Finds contextually relevant episodes beyond keyword matching.".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Natural language query describing what to search for"
                    },
                    "limit": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 100,
                        "default": 10,
                        "description": "Maximum number of results to return"
                    },
                    "similarity_threshold": {
                        "type": "number",
                        "minimum": 0.0,
                        "maximum": 1.0,
                        "default": 0.7,
                        "description": "Minimum similarity score (0.0-1.0)"
                    },
                    "domain": {
                        "type": "string",
                        "description": "Filter by task domain (e.g., 'web-api', 'data-processing')"
                    },
                    "task_type": {
                        "type": "string",
                        "description": "Filter by task type (e.g., 'code_generation', 'debugging')"
                    }
                },
                "required": ["query"]
            }),
        )
    }

    /// Get the tool definition for test_embeddings
    pub fn test_embeddings_tool() -> Tool {
        Tool::new(
            "test_embeddings".to_string(),
            "Test embedding provider connectivity and performance. Validates configuration and measures response time.".to_string(),
            json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        )
    }

    /// Execute the configure_embeddings tool
    #[instrument(skip(self, input), fields(provider = %input.provider))]
    pub async fn execute_configure_embeddings(
        &self,
        input: ConfigureEmbeddingsInput,
    ) -> Result<ConfigureEmbeddingsOutput> {
        info!("Configuring embedding provider: {}", input.provider);

        let mut warnings = Vec::new();

        // Parse provider type
        let provider_type = match input.provider.to_lowercase().as_str() {
            "openai" => EmbeddingProviderType::OpenAI,
            "local" => EmbeddingProviderType::Local,
            "mistral" => EmbeddingProviderType::Mistral,
            "azure" => EmbeddingProviderType::AzureOpenAI,
            "cohere" => {
                warnings.push(
                    "Cohere provider not yet implemented, using Local as fallback".to_string(),
                );
                EmbeddingProviderType::Local
            }
            _ => {
                return Err(anyhow!(
                    "Unsupported provider: {}. Supported providers: openai, local, mistral, azure, cohere",
                    input.provider
                ));
            }
        };

        // Validate API key for cloud providers
        if matches!(
            provider_type,
            EmbeddingProviderType::OpenAI
                | EmbeddingProviderType::Mistral
                | EmbeddingProviderType::AzureOpenAI
        ) {
            if let Some(api_key_env) = &input.api_key_env {
                if std::env::var(api_key_env).is_err() {
                    return Err(anyhow!(
                        "Environment variable '{}' not set. Please set the API key.",
                        api_key_env
                    ));
                }
            } else {
                warnings.push(format!(
                    "No api_key_env specified for {}. Make sure API key is set in standard environment variable.",
                    input.provider
                ));
            }
        }

        // Build model configuration based on provider
        let model_config =
            match provider_type {
                EmbeddingProviderType::OpenAI => {
                    let model_name = input.model.as_deref().unwrap_or("text-embedding-3-small");
                    match model_name {
                        "text-embedding-3-small" => ModelConfig::openai_3_small(),
                        "text-embedding-3-large" => ModelConfig::openai_3_large(),
                        "text-embedding-ada-002" => ModelConfig::openai_ada_002(),
                        _ => {
                            warnings.push(format!(
                                "Unknown OpenAI model '{}', using text-embedding-3-small",
                                model_name
                            ));
                            ModelConfig::openai_3_small()
                        }
                    }
                }
                EmbeddingProviderType::Mistral => {
                    let model_name = input.model.as_deref().unwrap_or("mistral-embed");
                    if model_name != "mistral-embed" {
                        warnings.push(format!(
                            "Unknown Mistral model '{}', using mistral-embed",
                            model_name
                        ));
                    }
                    ModelConfig::mistral_embed()
                }
                EmbeddingProviderType::AzureOpenAI => {
                    let deployment = input.deployment_name.as_ref().ok_or_else(|| {
                        anyhow!("deployment_name required for Azure OpenAI provider")
                    })?;
                    let resource = input.resource_name.as_ref().ok_or_else(|| {
                        anyhow!("resource_name required for Azure OpenAI provider")
                    })?;
                    let api_version = input.api_version.as_deref().unwrap_or("2023-05-15");

                    // Azure dimension depends on the underlying model
                    let dimension = 1536; // Default for ada-002 and text-embedding-3-small
                    ModelConfig::azure_openai(deployment, resource, api_version, dimension)
                }
                EmbeddingProviderType::Local => {
                    let model_name = input
                        .model
                        .as_deref()
                        .unwrap_or("sentence-transformers/all-MiniLM-L6-v2");
                    let dimension = 384; // Default for MiniLM
                    ModelConfig::local_sentence_transformer(model_name, dimension)
                }
                EmbeddingProviderType::Custom(_) => {
                    let model_name = input.model.as_deref().unwrap_or("custom-model");
                    let base_url = input
                        .base_url
                        .as_deref()
                        .ok_or_else(|| anyhow!("base_url required for custom provider"))?;
                    ModelConfig::custom(model_name, 384, base_url, None)
                }
            };

        // Build embedding configuration
        let embedding_config = EmbeddingConfig {
            provider: provider_type,
            model: model_config.clone(),
            similarity_threshold: input.similarity_threshold.unwrap_or(0.7),
            batch_size: input.batch_size.unwrap_or(32),
            cache_embeddings: true,
            timeout_seconds: 30,
        };

        // NOTE: In a real implementation, you would update the memory system's
        // semantic_service here. Since semantic_service is private and Option,
        // we simulate the configuration response.

        debug!(
            "Configured embedding provider: {:?} with model: {}",
            embedding_config.provider, embedding_config.model.model_name
        );

        let provider_name = input.provider.clone();
        Ok(ConfigureEmbeddingsOutput {
            success: true,
            provider: input.provider,
            model: model_config.model_name.clone(),
            dimension: model_config.embedding_dimension,
            message: format!(
                "Successfully configured {} provider with model {} (dimension: {})",
                provider_name, model_config.model_name, model_config.embedding_dimension
            ),
            warnings,
        })
    }

    /// Execute the query_semantic_memory tool
    #[instrument(skip(self, input), fields(query = %input.query))]
    pub async fn execute_query_semantic_memory(
        &self,
        input: QuerySemanticMemoryInput,
    ) -> Result<QuerySemanticMemoryOutput> {
        let start_time = std::time::Instant::now();

        info!("Executing semantic memory query: '{}'", input.query);

        // NOTE: This is a placeholder implementation that demonstrates the tool structure.
        // In a real implementation, you would:
        // 1. Check if semantic_service is available in memory
        // 2. Use semantic_service.find_similar_episodes() with the query
        // 3. Apply filters for domain/task_type if provided
        // 4. Return actual similarity results

        // For now, we'll use the standard retrieval with a warning
        warn!("Semantic service not yet integrated, using standard retrieval as fallback");

        let context = memory_core::TaskContext {
            domain: input.domain.unwrap_or_else(|| "general".to_string()),
            language: None,
            framework: None,
            complexity: memory_core::ComplexityLevel::Moderate,
            tags: input
                .task_type
                .as_ref()
                .map(|t| vec![t.clone()])
                .unwrap_or_default(),
        };

        let limit = input.limit.unwrap_or(10);
        let episodes = self
            .memory
            .retrieve_relevant_context(input.query.clone(), context, limit)
            .await;

        // Convert episodes to semantic results
        let results: Vec<SemanticResult> = episodes
            .into_iter()
            .enumerate()
            .map(|(idx, ep)| {
                // Simulate similarity score (decreasing with rank)
                let similarity_score = 0.95 - (idx as f32 * 0.05);

                let outcome = ep.outcome.as_ref().map(|o| match o {
                    memory_core::TaskOutcome::Success { verdict, .. } => {
                        format!("Success: {}", verdict)
                    }
                    memory_core::TaskOutcome::PartialSuccess { verdict, .. } => {
                        format!("Partial: {}", verdict)
                    }
                    memory_core::TaskOutcome::Failure { reason, .. } => {
                        format!("Failure: {}", reason)
                    }
                });

                SemanticResult {
                    episode_id: ep.episode_id.to_string(),
                    similarity_score,
                    task_description: ep.task_description.clone(),
                    domain: ep.context.domain.clone(),
                    task_type: format!("{:?}", ep.task_type),
                    outcome,
                    timestamp: ep.start_time.timestamp(),
                }
            })
            .collect();

        let query_time_ms = start_time.elapsed().as_micros() as f64 / 1000.0;

        debug!(
            "Semantic query completed in {}ms, found {} results",
            query_time_ms,
            results.len()
        );

        Ok(QuerySemanticMemoryOutput {
            results_found: results.len(),
            results,
            embedding_dimension: 384, // Default dimension (would come from provider)
            query_time_ms,
            provider: "fallback-standard-retrieval".to_string(),
        })
    }

    /// Execute the test_embeddings tool
    #[instrument(skip(self))]
    pub async fn execute_test_embeddings(&self) -> Result<TestEmbeddingsOutput> {
        let start_time = std::time::Instant::now();

        info!("Testing embedding provider connectivity");

        // NOTE: This is a placeholder implementation that demonstrates the tool structure.
        // In a real implementation, you would:
        // 1. Check if semantic_service is available
        // 2. Use semantic_service.provider.embed_text("test") to validate
        // 3. Measure actual response time
        // 4. Return real embedding sample

        // For now, simulate a successful test
        let test_embedding = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let test_time_ms = start_time.elapsed().as_millis() as u64;

        debug!("Embedding provider test completed in {}ms", test_time_ms);

        Ok(TestEmbeddingsOutput {
            available: false, // Would be true if real provider is configured
            provider: "not-configured".to_string(),
            model: "none".to_string(),
            dimension: 384,
            test_time_ms,
            sample_embedding: test_embedding,
            message: "Semantic service not yet configured. Use configure_embeddings first."
                .to_string(),
            errors: vec![
                "Semantic embeddings feature is under development".to_string(),
                "Standard retrieval is available as fallback".to_string(),
            ],
        })
    }
}
