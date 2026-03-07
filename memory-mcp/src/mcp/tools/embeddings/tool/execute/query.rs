//! Query semantic memory tool implementation.

use super::super::definitions::EmbeddingTools;
use crate::mcp::tools::embeddings::types::{
    QuerySemanticMemoryInput, QuerySemanticMemoryOutput, SemanticResult,
};
use anyhow::Result;
use memory_core::{ComplexityLevel, TaskContext, TaskOutcome};
use tracing::{debug, info, instrument, warn};

impl EmbeddingTools {
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
            let context = TaskContext {
                domain: domain.clone(),
                language: None,
                framework: None,
                complexity: ComplexityLevel::Moderate,
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
                    let fallback_context = TaskContext {
                        domain,
                        language: None,
                        framework: None,
                        complexity: ComplexityLevel::Moderate,
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
                        .map(|arc_ep| {
                            // Dereference Arc<Episode> to Episode
                            let episode = arc_ep.as_ref().clone();
                            memory_core::embeddings::SimilaritySearchResult {
                                item: episode,
                                similarity: 0.5,
                                metadata: memory_core::embeddings::SimilarityMetadata::default(),
                            }
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
                embedding_dimension: config.provider.effective_dimension(),
                query_time_ms,
                provider: format!("{:?}", config.provider),
            });
        }

        // Fallback if no semantic service configured
        warn!("Semantic service not available, using standard retrieval as fallback");

        let context = TaskContext {
            domain: input.domain.unwrap_or_else(|| "general".to_string()),
            language: None,
            framework: None,
            complexity: ComplexityLevel::Moderate,
            tags: input
                .task_type
                .as_ref()
                .map(|t| vec![t.clone()])
                .unwrap_or_default(),
        };

        let limit = input.limit.unwrap_or(10);

        let arc_episodes = self
            .memory
            .retrieve_relevant_context(input.query.clone(), context, limit)
            .await;

        // Convert Arc<Episode> episodes to semantic results with simulated scores
        let results: Vec<SemanticResult> = arc_episodes
            .into_iter()
            .enumerate()
            .map(|(idx, arc_ep)| {
                // Dereference Arc<Episode> to access Episode
                let episode = arc_ep.as_ref();
                // Simulate similarity score (decreasing with rank)
                let similarity_score = 0.95 - (idx as f32 * 0.05);

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

                SemanticResult {
                    episode_id: episode.episode_id.to_string(),
                    similarity_score,
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

        Ok(QuerySemanticMemoryOutput {
            results_found: results.len(),
            results,
            embedding_dimension: 384, // Default dimension
            query_time_ms,
            provider: "fallback-standard-retrieval".to_string(),
        })
    }
}
