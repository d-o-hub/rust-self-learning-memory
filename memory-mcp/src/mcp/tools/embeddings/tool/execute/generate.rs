//! Generate embedding and search by embedding tool implementations.

use super::super::definitions::EmbeddingTools;
use crate::mcp::tools::embeddings::types::{
    EmbeddingSearchResult, GenerateEmbeddingInput, GenerateEmbeddingOutput, SearchByEmbeddingInput,
    SearchByEmbeddingOutput,
};
use anyhow::{Result, anyhow};
use do_memory_core::TaskOutcome;
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

        if input.text.len() > crate::constants::MAX_EMBEDDING_TEXT_LEN {
            return Err(anyhow::anyhow!(
                "Text length {} exceeds maximum {} bytes ({}KB)",
                input.text.len(),
                crate::constants::MAX_EMBEDDING_TEXT_LEN,
                crate::constants::MAX_EMBEDDING_TEXT_LEN / 1024
            ));
        }

        // Use live_semantic_service to pick up dynamically activated providers.
        if let Some(semantic_service) = self.memory.live_semantic_service().await {
            let mut embedding = semantic_service
                .provider
                .embed_text(&input.text)
                .await
                .map_err(|e| anyhow!("Failed to generate embedding: {}", e))?;

            let config = semantic_service.config();
            let model_name = config.provider.model_name();
            let dimension = config.provider.effective_dimension();
            let provider = format!("{:?}", config.provider);

            let normalized = input.normalize;
            if normalized {
                embedding = do_memory_core::embeddings::normalize_vector(embedding);
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
                token_count: None,
            });
        }

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

        // Use live_semantic_service to pick up dynamically activated providers.
        let service_opt = self.memory.live_semantic_service().await;

        let expected_dimension = service_opt
            .as_ref()
            .map(|s| s.config().provider.effective_dimension())
            .unwrap_or(384);

        if input.embedding.len() != expected_dimension {
            return Err(anyhow!(
                "Embedding dimension mismatch: got {}, expected {}. Use the same model that generated your embeddings.",
                input.embedding.len(),
                expected_dimension
            ));
        }

        if let Some(semantic_service) = service_opt {
            let config = semantic_service.config();
            let provider = format!("{:?}", config.provider);

            let clamped_limit = input.limit.clamp(
                crate::constants::MIN_QUERY_LIMIT,
                crate::constants::MAX_QUERY_LIMIT,
            );

            let similar_episodes = semantic_service
                .find_episodes_by_embedding(
                    input.embedding.clone(),
                    clamped_limit,
                    input.similarity_threshold,
                )
                .await
                .map_err(|e| anyhow!("Failed to search by embedding: {}", e))?;

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
                        TaskOutcome::Abstained { reason, .. } => {
                            format!("Abstained: {}", reason)
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

        warn!("Semantic service not available, cannot search by embedding");
        Err(anyhow!(
            "Semantic embeddings not configured. Use configure_embeddings first to enable embedding-based search."
        ))
    }
}
