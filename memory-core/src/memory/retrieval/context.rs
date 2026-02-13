//! Context-based episode retrieval implementation

use crate::episode::Episode;
use crate::spatiotemporal::RetrievalQuery;
use crate::types::TaskContext;
use crate::MAX_QUERY_LIMIT;
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};

use super::super::SelfLearningMemory;
use super::helpers::{generate_simple_embedding, should_cache_episodes};

impl SelfLearningMemory {
    /// Retrieve relevant past episodes for a new task.
    ///
    /// Searches the memory for episodes similar to the given task, enabling
    /// the system to learn from past experience. Similarity is determined by:
    /// - **Domain match**: Same problem domain
    /// - **Language/framework**: Same technology stack
    /// - **Tags**: Overlapping tags
    /// - **Description**: Common keywords in task descriptions
    ///
    /// Results are ranked by a relevance score combining context match (40%),
    /// reward quality (30%), and description similarity (30%).
    ///
    /// # Search Strategy
    ///
    /// 1. Filters to completed episodes only
    /// 2. Matches on context fields (domain, language, framework, tags)
    /// 3. Performs basic text matching on descriptions
    /// 4. Scores and ranks by relevance
    /// 5. Returns top N results
    ///
    /// # Arguments
    ///
    /// * `task_description` - Description of the new task you're about to perform
    /// * `context` - Context for the new task (same structure as when starting episodes)
    /// * `limit` - Maximum number of episodes to return
    ///
    /// # Returns
    ///
    /// Vector of Arc-wrapped episodes sorted by relevance (highest first), limited to `limit` items.
    /// Returns empty vector if no relevant episodes found. The Arc wrapper enables cheap cloning
    /// when the episodes need to be shared across multiple consumers.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::{SelfLearningMemory, TaskContext, TaskType, ComplexityLevel};
    ///
    /// # async fn example() {
    /// let memory = SelfLearningMemory::new();
    ///
    /// // Query for relevant past episodes
    /// let context = TaskContext {
    ///     language: Some("rust".to_string()),
    ///     framework: Some("axum".to_string()),
    ///     complexity: ComplexityLevel::Moderate,
    ///     domain: "web-api".to_string(),
    ///     tags: vec!["rest".to_string(), "authentication".to_string()],
    /// };
    ///
    /// let relevant_episodes = memory.retrieve_relevant_context(
    ///     "Implement OAuth2 authentication".to_string(),
    ///     context,
    ///     5,  // Get top 5 most relevant
    /// ).await;
    ///
    /// // Use retrieved episodes to inform approach
    /// for arc_ep in relevant_episodes {
    ///     let episode = arc_ep.as_ref();
    ///     println!("Similar task: {}", episode.task_description);
    ///     println!("Reward: {:?}", episode.reward);
    ///
    ///     if let Some(reflection) = &episode.reflection {
    ///         println!("Key insights:");
    ///         for insight in &reflection.insights {
    ///             println!("  - {}", insight);
    ///         }
    ///     }
    /// }
    /// # }
    /// ```
    ///
    /// # See Also
    ///
    /// - [`retrieve_relevant_patterns()`](SelfLearningMemory::retrieve_relevant_patterns) - Get patterns instead of full episodes
    #[instrument(skip(self))]
    pub async fn retrieve_relevant_context(
        &self,
        task_description: String,
        context: TaskContext,
        limit: usize,
    ) -> Vec<Arc<Episode>> {
        use chrono::{TimeZone, Utc};

        // v0.1.12: Check query cache first
        let cache_key = crate::retrieval::CacheKey::new(task_description.clone())
            .with_domain(Some(context.domain.clone()))
            .with_limit(limit);

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
            return cached_episodes.clone();
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
                    if !fetched.is_empty() {
                        let mut episodes = self.episodes_fallback.write().await;
                        for ep in fetched {
                            episodes
                                .entry(ep.episode_id)
                                .or_insert_with(|| Arc::new(ep));
                        }
                    }
                }
            }

            // Then durable storage with higher limit for backfill
            if let Some(turso) = &self.turso_storage {
                if let Ok(fetched) = turso
                    .query_episodes_since(since, Some(MAX_QUERY_LIMIT))
                    .await
                {
                    if !fetched.is_empty() {
                        let mut episodes = self.episodes_fallback.write().await;
                        for ep in fetched {
                            episodes
                                .entry(ep.episode_id)
                                .or_insert_with(|| Arc::new(ep));
                        }
                    }
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
            return vec![];
        }

        // ============================================================================
        // Semantic Search - Try semantic similarity first
        // ============================================================================

        if let Some(ref semantic) = self.semantic_service {
            match semantic
                .find_similar_episodes(&task_description, &context, limit)
                .await
            {
                Ok(mut results) => {
                    if !results.is_empty() {
                        info!(
                            semantic_results = results.len(),
                            "Found episodes via semantic search"
                        );

                        // Limit results and extract episodes
                        results.truncate(limit);

                        // Convert to Arc<Episode> for cheap cloning
                        let semantic_episodes: Vec<Arc<Episode>> = results
                            .into_iter()
                            .map(|result| Arc::new(result.item))
                            .collect();

                        // v0.1.12: Cache the results before returning
                        // Skip caching if result set is too large (>100KB estimated)
                        if should_cache_episodes(&semantic_episodes) {
                            // Clone the Vec<Arc<Episode>> for caching (Arc clone is cheap)
                            self.query_cache
                                .put(cache_key.clone(), semantic_episodes.clone());
                        } else {
                            debug!(
                                episode_count = semantic_episodes.len(),
                                "Skipping cache for large result set"
                            );
                        }

                        return semantic_episodes;
                    }
                }
                Err(e) => {
                    warn!(
                        error = %e,
                        "Semantic search failed: {}. Falling back to keyword search.",
                        e
                    );
                }
            }
        }

        // ============================================================================
        // Fallback to keyword-based retrieval
        // ============================================================================

        // Phase 3: Use hierarchical retriever for efficient search (if enabled)
        let scored_episodes = if let Some(ref retriever) = self.hierarchical_retriever {
            // Generate query embedding if semantic service is available
            let query_embedding = if let Some(ref semantic) = self.semantic_service {
                match semantic.provider.embed_text(&task_description).await {
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
            // Use Arc::clone (cheap ref count inc) instead of Episode clone
            let mut relevant: Vec<Arc<Episode>> = completed_episodes
                .iter()
                .filter(|e| self.is_relevant_episode(e, &context, &task_description))
                .cloned()
                .collect();

            relevant.sort_by(|a, b| {
                let a_score = self.calculate_relevance_score(a, &context, &task_description);
                let b_score = self.calculate_relevance_score(b, &context, &task_description);
                b_score
                    .partial_cmp(&a_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            relevant.truncate(limit);

            // Already have Vec<Arc<Episode>>, no conversion needed
            info!(
                retrieved_count = relevant.len(),
                "Retrieved episodes using legacy method"
            );

            // v0.1.12: Cache the results before returning
            if should_cache_episodes(&relevant) {
                // Clone the Vec<Arc<Episode>> for caching (Arc clone is cheap)
                self.query_cache.put(cache_key.clone(), relevant.clone());
            } else {
                debug!(
                    episode_count = relevant.len(),
                    "Skipping cache for large result set"
                );
            }

            return relevant;
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

            // Already have Vec<Arc<Episode>>, no conversion needed
            info!(
                retrieved_count = result_arc_episodes.len(),
                diversity_score = diversity_score,
                "Retrieved diverse, relevant episodes using Phase 3 hierarchical retrieval + MMR"
            );

            // v0.1.12: Cache the results before returning
            if should_cache_episodes(&result_arc_episodes) {
                // Clone the Vec<Arc<Episode>> for caching (Arc clone is cheap)
                self.query_cache
                    .put(cache_key.clone(), result_arc_episodes.clone());
            } else {
                debug!(
                    episode_count = result_arc_episodes.len(),
                    "Skipping cache for large result set"
                );
            }

            return result_arc_episodes;
        }

        // Diversity maximization disabled - just use top scored episodes
        // Already have Vec<Arc<Episode>> from completed_episodes
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

        // Already have Vec<Arc<Episode>>, no conversion needed
        info!(
            retrieved_count = result_arc_episodes.len(),
            "Retrieved episodes using hierarchical retrieval (diversity disabled)"
        );

        // v0.1.12: Cache the results before returning
        if should_cache_episodes(&result_arc_episodes) {
            // Clone the Vec<Arc<Episode>> for caching (Arc clone is cheap)
            self.query_cache.put(cache_key, result_arc_episodes.clone());
        } else {
            debug!(
                episode_count = result_arc_episodes.len(),
                "Skipping cache for large result set"
            );
        }

        result_arc_episodes
    }
}
