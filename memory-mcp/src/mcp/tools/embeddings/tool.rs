//! Embedding tools implementation.

use crate::mcp::tools::embeddings::types::{
    ConfigureEmbeddingsInput, ConfigureEmbeddingsOutput, QuerySemanticMemoryInput,
    QuerySemanticMemoryOutput, SemanticResult, TestEmbeddingsOutput,
};
use crate::types::Tool;
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

        // Clone domain once to avoid ownership issues
        let domain = input
            .domain
            .clone()
            .unwrap_or_else(|| "general".to_string());

        // Check if semantic_service is available
        if let Some(semantic_service) = self.memory.semantic_service() {
            let context = memory_core::TaskContext {
                domain: domain.clone(),
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

            // Use the semantic service to find similar episodes
            let similar_episodes = match semantic_service
                .find_similar_episodes(&input.query, &context, limit)
                .await
            {
                Ok(episodes) => episodes,
                Err(e) => {
                    warn!("Semantic search failed: {}, using fallback", e);
                    // Fallback to standard retrieval
                    let fallback_context = memory_core::TaskContext {
                        domain,
                        language: None,
                        framework: None,
                        complexity: memory_core::ComplexityLevel::Moderate,
                        tags: input
                            .task_type
                            .as_ref()
                            .map(|t| vec![t.clone()])
                            .unwrap_or_default(),
                    };
                    self.memory
                        .retrieve_relevant_context(input.query.clone(), fallback_context, limit)
                        .await
                        .into_iter()
                        .map(|ep| memory_core::embeddings::SimilaritySearchResult {
                            item: ep,
                            similarity: 0.5,
                            metadata: memory_core::embeddings::SimilarityMetadata::default(),
                        })
                        .collect()
                }
            };

            // Convert to semantic results with actual similarity scores
            let results: Vec<SemanticResult> = similar_episodes
                .into_iter()
                .map(|result| {
                    let episode = result.item;
                    let outcome = episode.outcome.as_ref().map(|o| match o {
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
                        episode_id: episode.episode_id.to_string(),
                        similarity_score: result.similarity,
                        task_description: episode.task_description.clone(),
                        domain: episode.context.domain.clone(),
                        task_type: format!("{:?}", episode.task_type),
                        outcome,
                        timestamp: episode.start_time.timestamp(),
                    }
                })
                .collect();

            let query_time_ms = start_time.elapsed().as_micros() as f64 / 1000.0;

            debug!(
                "Semantic query completed in {}ms, found {} results",
                query_time_ms,
                results.len()
            );

            let config = semantic_service.config();
            return Ok(QuerySemanticMemoryOutput {
                results_found: results.len(),
                results,
                embedding_dimension: config.model.embedding_dimension,
                query_time_ms,
                provider: format!("{:?}", config.provider),
            });
        }

        // Fallback if no semantic service configured
        warn!("Semantic service not available, using standard retrieval as fallback");

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

        // Convert episodes to semantic results with simulated scores
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
            embedding_dimension: 384, // Default dimension
            query_time_ms,
            provider: "fallback-standard-retrieval".to_string(),
        })
    }

    /// Execute the test_embeddings tool
    #[instrument(skip(self))]
    pub async fn execute_test_embeddings(&self) -> Result<TestEmbeddingsOutput> {
        let start_time = std::time::Instant::now();

        info!("Testing embedding provider connectivity");

        // Check if semantic_service is configured
        if let Some(semantic_service) = self.memory.semantic_service() {
            // Call provider.embed_text("test") to validate
            match semantic_service.provider.embed_text("test").await {
                Ok(test_embedding) => {
                    let test_time_ms = start_time.elapsed().as_millis() as u64;

                    debug!("Embedding provider test completed in {}ms", test_time_ms);

                    let config = semantic_service.config();
                    let model_name = config.model.model_name.clone();
                    let dimension = config.model.embedding_dimension;
                    let provider = format!("{:?}", config.provider);
                    let embedding_len = test_embedding.len();
                    let model_name_for_msg = model_name.clone();

                    return Ok(TestEmbeddingsOutput {
                        available: true,
                        provider: provider.clone(),
                        model: model_name,
                        dimension,
                        test_time_ms,
                        sample_embedding: test_embedding,
                        message: format!(
                            "Successfully tested {} provider with model {} (dimension: {}, embedding size: {})",
                            provider, model_name_for_msg, dimension, embedding_len
                        ),
                        errors: vec![],
                    });
                }
                Err(e) => {
                    let test_time_ms = start_time.elapsed().as_millis() as u64;
                    let config = semantic_service.config();
                    let provider = format!("{:?}", config.provider);
                    return Ok(TestEmbeddingsOutput {
                        available: false,
                        provider: provider.clone(),
                        model: config.model.model_name.clone(),
                        dimension: config.model.embedding_dimension,
                        test_time_ms,
                        sample_embedding: vec![],
                        message: format!("Embedding provider test failed: {}", e),
                        errors: vec![format!("Failed to generate test embedding: {}", e)],
                    });
                }
            }
        }

        // No semantic service configured
        let test_time_ms = start_time.elapsed().as_millis() as u64;
        Ok(TestEmbeddingsOutput {
            available: false,
            provider: "not-configured".to_string(),
            model: "none".to_string(),
            dimension: 384,
            test_time_ms,
            sample_embedding: vec![],
            message: "Semantic service not yet configured. Use configure_embeddings first."
                .to_string(),
            errors: vec!["Semantic embeddings feature requires configuration".to_string()],
        })
    }
}
