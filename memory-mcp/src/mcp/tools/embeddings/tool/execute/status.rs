//! Test embeddings and provider status tool implementations.

use super::super::definitions::EmbeddingTools;
use crate::mcp::tools::embeddings::types::{
    EmbeddingProviderStatusInput, EmbeddingProviderStatusOutput, ProviderTestResult,
    TestEmbeddingsOutput,
};
use anyhow::Result;
use tracing::{debug, info, instrument};

impl EmbeddingTools {
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
                    let model_name = config.provider.model_name();
                    let dimension = config.provider.effective_dimension();
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
                        model: config.provider.model_name(),
                        dimension: config.provider.effective_dimension(),
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

    /// Execute the embedding_provider_status tool
    #[instrument(skip(self))]
    pub async fn execute_embedding_provider_status(
        &self,
        input: EmbeddingProviderStatusInput,
    ) -> Result<EmbeddingProviderStatusOutput> {
        info!("Getting embedding provider status");

        let mut warnings = Vec::new();

        // Check if semantic_service is configured
        if let Some(semantic_service) = self.memory.semantic_service() {
            let config = semantic_service.config();
            let model_name = config.provider.model_name();
            let dimension = config.provider.effective_dimension();
            let provider = format!("{:?}", config.provider);
            let similarity_threshold = config.similarity_threshold;
            let batch_size = config.batch_size;
            let cache_enabled = config.cache_embeddings;

            // Get provider metadata
            let metadata = semantic_service.provider.metadata();

            // Perform connectivity test if requested
            let test_result = if input.test_connectivity {
                let start_time = std::time::Instant::now();
                match semantic_service.provider.embed_text("test").await {
                    Ok(embedding) => {
                        let duration_ms = start_time.elapsed().as_millis() as u64;
                        // Get first 5 values as sample
                        let sample_embedding: Vec<f32> = embedding.into_iter().take(5).collect();

                        Some(ProviderTestResult {
                            success: true,
                            duration_ms,
                            sample_embedding,
                            error: None,
                        })
                    }
                    Err(e) => {
                        let duration_ms = start_time.elapsed().as_millis() as u64;
                        warnings.push(format!("Connectivity test failed: {}", e));
                        Some(ProviderTestResult {
                            success: false,
                            duration_ms,
                            sample_embedding: vec![],
                            error: Some(e.to_string()),
                        })
                    }
                }
            } else {
                None
            };

            // Check for potential configuration issues
            if similarity_threshold < 0.5 {
                warnings.push(format!(
                    "Low similarity threshold ({}) may return many irrelevant results",
                    similarity_threshold
                ));
            }
            if similarity_threshold > 0.95 {
                warnings.push(format!(
                    "High similarity threshold ({}) may return very few results",
                    similarity_threshold
                ));
            }
            if batch_size > 100 {
                warnings.push(format!(
                    "Large batch size ({}) may cause timeout issues",
                    batch_size
                ));
            }

            // Determine availability based on test result or previous success
            let available = test_result.as_ref().map(|t| t.success).unwrap_or_else(|| {
                // If no test was requested, assume available since it was configured
                true
            });

            return Ok(EmbeddingProviderStatusOutput {
                configured: true,
                available,
                provider,
                model: model_name,
                dimension,
                similarity_threshold,
                batch_size,
                cache_enabled,
                metadata,
                test_result,
                warnings,
            });
        }

        // No semantic service configured
        Ok(EmbeddingProviderStatusOutput {
            configured: false,
            available: false,
            provider: "not-configured".to_string(),
            model: "none".to_string(),
            dimension: 384,
            similarity_threshold: 0.7,
            batch_size: 32,
            cache_enabled: false,
            metadata: serde_json::json!({"status": "not_configured"}),
            test_result: None,
            warnings: vec![
                "Semantic embeddings not configured. Use configure_embeddings to enable embedding features.".to_string()
            ],
        })
    }
}
