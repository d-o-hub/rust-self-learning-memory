//! Execution-backed retrieval pipeline and report (issue #1079).
//!
//! Extracted from `context.rs` so the public entry point stays thin and the
//! pipeline can return metadata describing what actually ran. Provenance builds
//! from this report instead of re-probing the query cache or fabricating
//! candidate counts from result counts.

// expect used with preceding invariant check - pattern is intentional
#![allow(clippy::expect_used)]

use crate::MAX_QUERY_LIMIT;
use crate::episode::Episode;
use crate::spatiotemporal::RetrievalQuery;
use crate::types::TaskContext;
use std::sync::Arc;
use tracing::{debug, info};

use super::super::SelfLearningMemory;
use super::helpers::{cache_episodes_if_eligible, generate_simple_embedding};
use super::report::RetrievalExecution;

impl SelfLearningMemory {
    /// Execute retrieval once and return the results with execution metadata.
    ///
    /// Performs exactly one query-cache lookup. On a miss it runs hybrid,
    /// semantic, hierarchical, then legacy keyword retrieval in order and
    /// reports the tier and fallback that actually served the request. Unknown
    /// candidate counts stay `None` rather than being inferred from results.
    pub(super) async fn retrieve_relevant_context_with_execution(
        &self,
        task_description: String,
        context: TaskContext,
        limit: usize,
    ) -> (Vec<Arc<Episode>>, RetrievalExecution) {
        use chrono::{TimeZone, Utc};

        // v0.1.12: Check query cache first
        // ADR-074 / S1.2: full identity = TaskContext + mode + provider + ranking + generation
        // The provider identity follows the runtime-activated provider (issue #1072),
        // not the construction-time semantic config.
        let cache_key = crate::retrieval::CacheKey::new(task_description.clone())
            .with_task_context(&context)
            .with_limit(limit)
            .with_retrieval_mode(self.config.retrieval_mode.to_string())
            .with_provider_identity(self.effective_provider_identity())
            .with_ranking_config_version(crate::retrieval::RANKING_CONFIG_VERSION)
            .with_index_generation(self.query_cache.index_generation());
        let query_start = std::time::Instant::now();

        // Snapshot the live provider before any provider `.await` so cache
        // identity and the service used for embedding stay coherent even if an
        // activation swaps the provider mid-retrieval.
        let live_semantic = self.live_semantic_service().await;

        if let Some(cached_episodes) = self.query_cache.get(&cache_key) {
            debug!(
                cached_count = cached_episodes.len(),
                "Query cache HIT - returning cached episodes"
            );

            // Log cache metrics periodically (every 100 hits)
            let metrics = self.query_cache.metrics();
            if metrics.hits % 100 == 0 {
                info!(
                    hit_rate = format!("{:.1}%", metrics.hit_rate() * 100.0),
                    cache_size = format!("{}/{}", metrics.size, metrics.capacity),
                    hits = metrics.hits,
                    misses = metrics.misses,
                    evictions = metrics.evictions,
                    "Query cache metrics"
                );
            }

            // Return Arc-clones (cheap reference count increment)
            Self::record_query_outcome(
                &query_start,
                crate::monitoring::metrics::RetrievalTier::Cache,
                cached_episodes.len(),
                None,
            );
            let result_count = cached_episodes.len();
            let execution = RetrievalExecution::from_cache_hit(result_count, query_start);
            return (cached_episodes.clone(), execution);
        }

        debug!("Query cache MISS - performing retrieval");

        // Ensure we have some episodes in memory; if not, try to backfill from storage
        let mut need_backfill = false;
        {
            let episodes = self.episodes_fallback.read().await;
            let completed_count = episodes.values().filter(|e| e.is_complete()).count();
            if completed_count < limit {
                need_backfill = true;
                debug!(
                    completed_count,
                    limit, "Insufficient in-memory episodes, attempting backfill from storage"
                );
            }
        }

        if need_backfill {
            // Oldest timestamp to fetch from
            let since = Utc
                .timestamp_millis_opt(0)
                .single()
                .unwrap_or_else(Utc::now);

            // Prefer cache first with higher limit for backfill
            if let Some(cache) = &self.cache_storage {
                if let Ok(fetched) = cache
                    .query_episodes_since(since, Some(MAX_QUERY_LIMIT))
                    .await
                {
                    self.merge_backfilled_episodes(fetched).await;
                }
            }

            // Then durable storage with higher limit for backfill
            if let Some(turso) = &self.turso_storage {
                if let Ok(fetched) = turso
                    .query_episodes_since(since, Some(MAX_QUERY_LIMIT))
                    .await
                {
                    self.merge_backfilled_episodes(fetched).await;
                }
            }
        }

        let episodes = self.episodes_fallback.read().await;

        debug!(
            total_episodes = episodes.len(),
            limit = limit,
            "Retrieving relevant context with Phase 3 hierarchical retrieval"
        );

        // Collect completed episodes - store as Arc to enable cheap cloning during filtering
        let completed_episodes: Vec<Arc<Episode>> = episodes
            .values()
            .filter(|e| e.is_complete())
            .cloned()
            .collect();

        if completed_episodes.is_empty() {
            info!("No completed episodes found for retrieval");
            Self::record_query_outcome(
                &query_start,
                crate::monitoring::metrics::RetrievalTier::None,
                0,
                None,
            );
            return (
                vec![],
                RetrievalExecution::from_execution(
                    crate::monitoring::metrics::RetrievalTier::None,
                    Some(0),
                    0,
                    false,
                    query_start,
                ),
            );
        }

        // Hybrid Search (v0.1.34) - Improved ANN-backed retrieval
        let hybrid_attempted = self.config.retrieval_mode == crate::types::RetrievalMode::Hybrid
            && self.semantic_retriever.is_some();
        if let Some(outcome) = self
            .try_hybrid_retrieval(
                &task_description,
                &context,
                limit,
                &cache_key,
                &completed_episodes,
                query_start,
            )
            .await
        {
            let result_count = outcome.episodes.len();
            let execution = RetrievalExecution::from_execution(
                outcome.tier,
                outcome.candidate_count,
                result_count,
                false,
                query_start,
            );
            return (outcome.episodes, execution);
        }
        // A configured hybrid path that did not serve is a fallback for the
        // tier that eventually does.
        let mut fallback = hybrid_attempted;

        // Semantic Search - Try semantic similarity first
        if let Some(semantic) = &live_semantic {
            if let Some(outcome) = self
                .try_semantic_retrieval(
                    semantic,
                    &task_description,
                    &context,
                    limit,
                    &cache_key,
                    query_start,
                )
                .await
            {
                let result_count = outcome.episodes.len();
                let execution = RetrievalExecution::from_execution(
                    outcome.tier,
                    outcome.candidate_count,
                    result_count,
                    fallback,
                    query_start,
                );
                return (outcome.episodes, execution);
            }
            // The semantic provider was available but did not serve.
            fallback = true;
        }

        // ============================================================================
        // Fallback to keyword-based retrieval
        // ============================================================================

        // Phase 3: Use hierarchical retriever for efficient search (if enabled)
        let scored_episodes = if let Some(ref retriever) = self.hierarchical_retriever {
            // Generate query embedding if semantic service is available
            let query_embedding = if let Some(semantic) = &live_semantic {
                match semantic.embed_query_text(&task_description).await {
                    Ok(embedding) => {
                        debug!(
                            embedding_dim = embedding.len(),
                            "Generated query embedding for hierarchical retrieval"
                        );
                        Some(embedding)
                    }
                    Err(e) => {
                        debug!(
                            error = %e,
                            "Failed to generate query embedding, falling back to keyword search"
                        );
                        None
                    }
                }
            } else {
                None
            };

            // Preload episode embeddings for semantic similarity scoring
            // Note: Using empty map for now - individual lookups will be done in the retriever
            let episode_embeddings = std::collections::HashMap::new();

            let query = RetrievalQuery {
                query_text: task_description.clone(),
                query_embedding,
                domain: Some(context.domain.clone()),
                task_type: None,    // Could extract from context if needed
                limit: limit * 2,   // Retrieve more candidates for diversity maximization
                episode_embeddings, // Preloaded embeddings
            };

            match retriever
                .retrieve(&query, completed_episodes.as_slice())
                .await
            {
                Ok(scored) => Some(scored),
                Err(e) => {
                    debug!(
                        "Hierarchical retrieval failed: {}, falling back to legacy method",
                        e
                    );
                    None
                }
            }
        } else {
            None
        };

        // If hierarchical retrieval failed or is disabled, use legacy method
        if scored_episodes.is_none() {
            // v0.1.32: Pre-calculate query data for optimized legacy retrieval
            let desc_lower = task_description.to_lowercase();
            let query_words: Vec<&str> = desc_lower.split_whitespace().collect();
            let query_words_gt3: Vec<&str> = query_words
                .iter()
                .filter(|w| w.len() > 3)
                .copied()
                .collect();
            let query_tags: std::collections::HashSet<&String> = context.tags.iter().collect();

            // Optimization: Use Schwartzian Transform (decorate-sort-undecorate)
            // to ensure each candidate is scored exactly once.
            // 1. Filter and decorate (calculate scores)
            let mut decorated: Vec<(f32, Arc<Episode>)> = completed_episodes
                .iter()
                .map(|e| (e.task_description.to_lowercase(), e))
                .filter(|(desc_lower, e)| {
                    self.is_relevant_episode(e, &context, &query_tags, &query_words_gt3, desc_lower)
                })
                .map(|(desc_lower, e)| {
                    let score = self.calculate_relevance_score(
                        e,
                        &context,
                        &query_tags,
                        &query_words,
                        &query_words_gt3,
                        &desc_lower,
                    );
                    (score, Arc::clone(e))
                })
                .collect();

            // 2. Sort by score DESC
            // Optimization: Use sort_unstable_by as stability isn't required for search results.
            decorated.sort_unstable_by(|a, b| {
                b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal)
            });

            // 3. Undecorate and truncate
            let candidate_count = decorated.len();
            let relevant: Vec<Arc<Episode>> =
                decorated.into_iter().take(limit).map(|(_, e)| e).collect();

            info!(
                retrieved_count = relevant.len(),
                "Retrieved episodes using legacy method"
            );

            cache_episodes_if_eligible(&self.query_cache, cache_key.clone(), &relevant);
            Self::record_query_outcome(
                &query_start,
                crate::monitoring::metrics::RetrievalTier::Keyword,
                relevant.len(),
                Some(candidate_count),
            );
            let result_count = relevant.len();
            let execution = RetrievalExecution::from_execution(
                crate::monitoring::metrics::RetrievalTier::Keyword,
                Some(candidate_count),
                result_count,
                fallback,
                query_start,
            );
            return (relevant, execution);
        }

        let scored_episodes = scored_episodes
            .expect("scored_episodes is Some: None case handled by early return above");

        // Phase 3: Apply MMR diversity maximization (if enabled)
        if let Some(ref maximizer) = self.diversity_maximizer {
            // Convert scored episodes to diversity format with embeddings
            let diversity_candidates: Vec<crate::spatiotemporal::diversity::ScoredEpisode> =
                scored_episodes
                    .iter()
                    .filter_map(|scored| {
                        completed_episodes
                            .iter()
                            .find(|e| e.episode_id == scored.episode_id)
                            .map(|episode| {
                                let embedding = generate_simple_embedding(episode);
                                crate::spatiotemporal::diversity::ScoredEpisode::new(
                                    episode.episode_id.to_string(),
                                    scored.relevance_score,
                                    embedding,
                                )
                            })
                    })
                    .collect();

            // Apply MMR diversity maximization
            let diverse_scored = maximizer.maximize_diversity(diversity_candidates, limit);

            // Calculate and log diversity score
            let diversity_score = maximizer.calculate_diversity_score(&diverse_scored);
            debug!(
                diversity_score = diversity_score,
                target = 0.7,
                "Applied MMR diversity maximization"
            );

            // Extract episodes from diverse results
            // Already have Arc<Episode> from completed_episodes, just collect
            let result_arc_episodes: Vec<Arc<Episode>> = diverse_scored
                .iter()
                .filter_map(|scored| {
                    let episode_id = uuid::Uuid::parse_str(scored.episode_id()).ok()?;
                    completed_episodes
                        .iter()
                        .find(|e| e.episode_id == episode_id)
                        .cloned()
                })
                .collect();

            info!(
                retrieved_count = result_arc_episodes.len(),
                diversity_score = diversity_score,
                "Retrieved diverse, relevant episodes using Phase 3 hierarchical retrieval + MMR"
            );

            cache_episodes_if_eligible(&self.query_cache, cache_key.clone(), &result_arc_episodes);
            Self::record_query_outcome(
                &query_start,
                crate::monitoring::metrics::RetrievalTier::Hierarchical,
                result_arc_episodes.len(),
                Some(scored_episodes.len()),
            );
            let result_count = result_arc_episodes.len();
            let execution = RetrievalExecution::from_execution(
                crate::monitoring::metrics::RetrievalTier::Hierarchical,
                Some(scored_episodes.len()),
                result_count,
                fallback,
                query_start,
            );
            return (result_arc_episodes, execution);
        }

        // Diversity maximization disabled - top scored episodes only
        let result_arc_episodes: Vec<Arc<Episode>> = scored_episodes
            .iter()
            .take(limit)
            .filter_map(|scored| {
                completed_episodes
                    .iter()
                    .find(|e| e.episode_id == scored.episode_id)
                    .cloned()
            })
            .collect();

        info!(
            retrieved_count = result_arc_episodes.len(),
            "Retrieved episodes using hierarchical retrieval (diversity disabled)"
        );

        cache_episodes_if_eligible(&self.query_cache, cache_key, &result_arc_episodes);
        Self::record_query_outcome(
            &query_start,
            crate::monitoring::metrics::RetrievalTier::Hierarchical,
            result_arc_episodes.len(),
            Some(scored_episodes.len()),
        );
        let result_count = result_arc_episodes.len();
        (
            result_arc_episodes,
            RetrievalExecution::from_execution(
                crate::monitoring::metrics::RetrievalTier::Hierarchical,
                Some(scored_episodes.len()),
                result_count,
                fallback,
                query_start,
            ),
        )
    }

    /// Merge fetched episodes into the in-memory fallback map, ignoring empties.
    async fn merge_backfilled_episodes(&self, fetched: Vec<Episode>) {
        if fetched.is_empty() {
            return;
        }
        let mut episodes = self.episodes_fallback.write().await;
        for ep in fetched {
            episodes
                .entry(ep.episode_id)
                .or_insert_with(|| Arc::new(ep));
        }
    }
}
