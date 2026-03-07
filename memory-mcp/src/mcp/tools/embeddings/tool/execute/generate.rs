//! Generate embedding and search by embedding tool implementations.

use super::super::definitions::EmbeddingTools;
use crate::mcp::tools::embeddings::types::{
    EmbeddingSearchResult, GenerateEmbeddingInput, GenerateEmbeddingOutput, SearchByEmbeddingInput,
    SearchByEmbeddingOutput,
};
use anyhow::{Result, anyhow};
use memory_core::TaskOutcome;
use tracing::{debug, info, instrument, warn};

impl EmbeddingTools {
    /// Execute the generate_embedding tool
    #[instrument(skip(self, input), fields(text_len = input.text.len()))]
    pub async fn execute_generate_embedding(
        &self,
        input: GenerateEmbeddingInput,
    ) -> Result<GenerateEmbeddingOutput> {
        let start_time = std::time::Instant::now();

        info!("Generating embedding for text ({} chars)", input.text.len());

        // Check if semantic_service is available
        if let Some(semantic_service) = self.memory.semantic_service() {
            // Generate the embedding
            let mut embedding = semantic_service
                .provider
                .embed_text(&input.text)
                .await
                .map_err(|e| anyhow!("Failed to generate embedding: {}", e))?;

            let config = semantic_service.config();
            let model_name = config.provider.model_name();
            let dimension = config.provider.effective_dimension();
            let provider = format!("{:?}", config.provider);

            // Normalize if requested
            let normalized = input.normalize;
            if normalized {
                embedding = memory_core::embeddings::normalize_vector(embedding);
            }

            let generation_time_ms = start_time.elapsed().as_micros() as f64 / 1000.0;

            debug!(
                "Generated {}-dimensional embedding in {}ms",
                dimension, generation_time_ms
            );

            return Ok(GenerateEmbeddingOutput {
                embedding,
                dimension,
                model: model_name,
                provider,
                generation_time_ms,
                normalized,
                token_count: None, // Would need tokenizer integration
            });
        }

        // No semantic service configured
        warn!("Semantic service not available, cannot generate embedding");
        Err(anyhow!(
            "Semantic embeddings not configured. Use configure_embeddings first."
        ))
    }

    /// Execute the search_by_embedding tool
    #[instrument(skip(self, input), fields(embedding_dim = input.embedding.len()))]
    pub async fn execute_search_by_embedding(
        &self,
        input: SearchByEmbeddingInput,
    ) -> Result<SearchByEmbeddingOutput> {
        let start_time = std::time::Instant::now();

        info!(
            "Searching by embedding (dimension: {}, limit: {}, threshold: {})",
            input.embedding.len(),
            input.limit,
            input.similarity_threshold
        );

        // Validate embedding dimension
        let expected_dimension = if let Some(semantic_service) = self.memory.semantic_service() {
            semantic_service.config().provider.effective_dimension()
        } else {
            384 // Default dimension
        };

        if input.embedding.len() != expected_dimension {
            return Err(anyhow!(
                "Embedding dimension mismatch: got {}, expected {}. Use the same model that generated your embeddings.",
                input.embedding.len(),
                expected_dimension
            ));
        }

        // Check if semantic_service is available
        if let Some(semantic_service) = self.memory.semantic_service() {
            let config = semantic_service.config();
            let provider = format!("{:?}", config.provider);

            // Search for similar episodes using the embedding directly
            let similar_episodes = semantic_service
                .find_episodes_by_embedding(
                    input.embedding.clone(),
                    input.limit,
                    input.similarity_threshold,
                )
                .await
                .map_err(|e| anyhow!("Failed to search by embedding: {}", e))?;

            // Convert to search results
            let results: Vec<EmbeddingSearchResult> = similar_episodes
                .into_iter()
                .map(|result| {
                    let episode = result.item;
                    let outcome = episode.outcome.as_ref().map(|o| match o {
                        TaskOutcome::Success { verdict, .. } => {
                            format!("Success: {}", verdict)
                        }
                        TaskOutcome::PartialSuccess { verdict, .. } => {
                            format!("Partial: {}", verdict)
                        }
                        TaskOutcome::Failure { reason, .. } => {
                            format!("Failure: {}", reason)
                        }
                    });

                    EmbeddingSearchResult {
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

            let search_time_ms = start_time.elapsed().as_micros() as f64 / 1000.0;

            debug!(
                "Embedding search completed in {}ms, found {} results",
                search_time_ms,
                results.len()
            );

            return Ok(SearchByEmbeddingOutput {
                results_found: results.len(),
                results,
                embedding_dimension: expected_dimension,
                search_time_ms,
                provider,
            });
        }

        // No semantic service configured - fallback to standard retrieval with warning
        warn!("Semantic service not available, cannot search by embedding");
        Err(anyhow!(
            "Semantic embeddings not configured. Use configure_embeddings first to enable embedding-based search."
        ))
    }
}
