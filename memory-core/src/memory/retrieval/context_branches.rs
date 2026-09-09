//! Semantic-sense branches of the context retrieval pipeline.
//!
//! Extracted from `context.rs` to honor the 500 LOC invariant; these are
//! inherent methods of [`SelfLearningMemory`], split across files. Each
//! helper returns `None` so the caller falls through to the next branch.

use std::sync::Arc;
use tracing::{info, warn};

use crate::episode::Episode;
use crate::types::TaskContext;

use super::super::SelfLearningMemory;
use super::helpers::cache_episodes_if_eligible;

impl SelfLearningMemory {
    /// Attempt hybrid ANN-backed retrieval (v0.1.34).
    ///
    /// Generates the query embedding, runs the semantic retriever, caches
    /// and records the hybrid-tier outcome on success. Returns `None` when
    /// hybrid mode is off, the service pair is unavailable, embedding or
    /// retrieval fails, or no hits were found.
    pub(super) async fn try_hybrid_retrieval(
        &self,
        task_description: &str,
        context: &TaskContext,
        limit: usize,
        cache_key: &crate::retrieval::CacheKey,
        completed_episodes: &[Arc<Episode>],
        query_start: std::time::Instant,
    ) -> Option<Vec<Arc<Episode>>> {
        if self.config.retrieval_mode != crate::types::RetrievalMode::Hybrid {
            return None;
        }
        let (retriever, semantic) = (
            self.semantic_retriever.as_ref()?,
            self.semantic_service.as_ref()?,
        );

        // Generate query embedding
        match semantic.embed_query_text(task_description).await {
            Ok(query_embedding) => {
                let episodes_map: std::collections::HashMap<uuid::Uuid, Arc<Episode>> =
                    completed_episodes
                        .iter()
                        .map(|e| (e.episode_id, e.clone()))
                        .collect();
                match retriever.retrieve(
                    task_description,
                    &query_embedding,
                    context,
                    episodes_map,
                    limit,
                ) {
                    Ok(hits) => {
                        if hits.is_empty() {
                            return None;
                        }
                        let hybrid_episodes: Vec<Arc<Episode>> =
                            hits.into_iter().map(|h| h.episode).collect();
                        cache_episodes_if_eligible(
                            &self.query_cache,
                            cache_key.clone(),
                            &hybrid_episodes,
                        );
                        info!(
                            retrieved_count = hybrid_episodes.len(),
                            "Retrieved episodes using hybrid search"
                        );
                        Self::record_query_outcome(
                            &query_start,
                            crate::monitoring::metrics::RetrievalTier::Hybrid,
                            hybrid_episodes.len(),
                            None,
                        );
                        Some(hybrid_episodes)
                    }
                    Err(e) => {
                        warn!(error = %e, "Hybrid retrieval failed, falling back");
                        None
                    }
                }
            }
            Err(e) => {
                warn!(error = %e, "Query embedding failed for hybrid search, falling back");
                None
            }
        }
    }

    /// Attempt semantic similarity retrieval.
    ///
    /// Truncates to `limit`, caches, and records the semantic-tier outcome
    /// on success. Returns `None` when the service finds nothing or fails,
    /// so the caller falls back to keyword/hierarchical retrieval.
    pub(super) async fn try_semantic_retrieval(
        &self,
        semantic: &crate::embeddings::SemanticService,
        task_description: &str,
        context: &TaskContext,
        limit: usize,
        cache_key: &crate::retrieval::CacheKey,
        query_start: std::time::Instant,
    ) -> Option<Vec<Arc<Episode>>> {
        match semantic
            .find_similar_episodes(task_description, context, limit)
            .await
        {
            Ok(mut results) => {
                if results.is_empty() {
                    return None;
                }
                info!(
                    semantic_results = results.len(),
                    "Found episodes via semantic search"
                );

                // Limit results and convert to Arc<Episode> for cheap cloning
                results.truncate(limit);
                let semantic_episodes: Vec<Arc<Episode>> = results
                    .into_iter()
                    .map(|result| Arc::new(result.item))
                    .collect();

                cache_episodes_if_eligible(
                    &self.query_cache,
                    cache_key.clone(),
                    &semantic_episodes,
                );

                Self::record_query_outcome(
                    &query_start,
                    crate::monitoring::metrics::RetrievalTier::Semantic,
                    semantic_episodes.len(),
                    None,
                );
                Some(semantic_episodes)
            }
            Err(e) => {
                warn!(
                    error = %e,
                    "Semantic search failed: {}. Falling back to keyword search.",
                    e
                );
                None
            }
        }
    }

    /// Record one finished query path: request, duration, and scored count.
    ///
    /// Never records query text, IDs, or scores (redaction contract, #962).
    pub(super) fn record_query_outcome(
        start: &std::time::Instant,
        tier: crate::monitoring::metrics::RetrievalTier,
        returned: usize,
        scored: Option<usize>,
    ) {
        use crate::monitoring::metrics::{
            RetrievalOperation, RetrievalOutcome, RetrievalStage, global_retrieval_metrics,
        };

        let telemetry = global_retrieval_metrics();
        telemetry.record_request(
            RetrievalOperation::Query,
            tier,
            RetrievalOutcome::from_count(returned),
            start.elapsed().as_millis().min(u128::from(u64::MAX)) as u64,
        );
        if let Some(scored) = scored {
            telemetry.record_candidates(RetrievalStage::Scored, scored);
        }
    }
}
