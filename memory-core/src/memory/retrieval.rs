//! Episode and pattern retrieval

use crate::episode::Episode;
use crate::extraction::{deduplicate_patterns, rank_patterns};
use crate::pattern::Pattern;
use crate::spatiotemporal::RetrievalQuery;
use crate::types::TaskContext;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

use super::SelfLearningMemory;

/// Maximum size for caching episodes (100KB)
const MAX_CACHEABLE_SIZE: usize = 100 * 1024;

/// Check if episodes should be cached based on estimated size
///
/// Skips caching for large result sets to avoid expensive clone operations.
/// Estimates size based on step count and artifact sizes.
fn should_cache_episodes(episodes: &[Episode]) -> bool {
    // Quick check: if >50 episodes, likely too large
    if episodes.len() > 50 {
        return false;
    }

    // Estimate total size
    let estimated_size: usize = episodes
        .iter()
        .map(|ep| {
            // Base episode overhead: ~1KB
            let mut size = 1024;

            // Steps: ~100 bytes each
            size += ep.steps.len() * 100;

            // Outcome artifacts (can be large)
            if let Some(ref outcome) = ep.outcome {
                match outcome {
                    crate::types::TaskOutcome::Success { artifacts, .. } => {
                        size += artifacts.iter().map(|a| a.len()).sum::<usize>();
                    }
                    crate::types::TaskOutcome::PartialSuccess {
                        completed, failed, ..
                    } => {
                        size += completed.iter().map(|a| a.len()).sum::<usize>();
                        size += failed.iter().map(|a| a.len()).sum::<usize>();
                    }
                    crate::types::TaskOutcome::Failure { .. } => {}
                }
            }

            // Reflection insights
            if let Some(ref reflection) = ep.reflection {
                size += reflection.insights.iter().map(|i| i.len()).sum::<usize>();
            }

            size
        })
        .sum();

    estimated_size < MAX_CACHEABLE_SIZE
}

/// Generate a simple embedding for an episode based on its metadata
/// This is a placeholder until full embedding integration is complete
fn generate_simple_embedding(episode: &Episode) -> Vec<f32> {
    let mut embedding = Vec::with_capacity(10);

    // Domain hash
    let domain_hash = episode
        .context
        .domain
        .chars()
        .fold(0u32, |acc, c| acc.wrapping_add(c as u32));
    embedding.push((domain_hash % 100) as f32 / 100.0);

    // Task type encoding
    embedding.push(match episode.task_type {
        crate::types::TaskType::CodeGeneration => 0.9,
        crate::types::TaskType::Analysis => 0.7,
        crate::types::TaskType::Testing => 0.5,
        crate::types::TaskType::Debugging => 0.3,
        crate::types::TaskType::Refactoring => 0.2,
        crate::types::TaskType::Documentation => 0.1,
        crate::types::TaskType::Other => 0.0,
    });

    // Complexity encoding
    embedding.push(match episode.context.complexity {
        crate::types::ComplexityLevel::Simple => 0.2,
        crate::types::ComplexityLevel::Moderate => 0.5,
        crate::types::ComplexityLevel::Complex => 0.8,
    });

    // Language/framework presence
    embedding.push(if episode.context.language.is_some() {
        1.0
    } else {
        0.0
    });
    embedding.push(if episode.context.framework.is_some() {
        1.0
    } else {
        0.0
    });

    // Number of steps (normalized)
    let step_count = episode.steps.len().min(50) as f32 / 50.0;
    embedding.push(step_count);

    // Reward component (if available)
    let reward_value = episode.reward.as_ref().map_or(0.5, |r| r.total.min(1.0));
    embedding.push(reward_value);

    // Duration component
    if let Some(end) = episode.end_time {
        let duration = end - episode.start_time;
        let duration_secs = duration.num_seconds().clamp(0, 3600) as f32 / 3600.0;
        embedding.push(duration_secs);
    } else {
        embedding.push(0.5);
    }

    // Tag count (normalized)
    let tag_count = episode.context.tags.len().min(10) as f32 / 10.0;
    embedding.push(tag_count);

    // Outcome encoding
    embedding.push(match &episode.outcome {
        Some(crate::types::TaskOutcome::Success { .. }) => 1.0,
        Some(crate::types::TaskOutcome::PartialSuccess { .. }) => 0.5,
        Some(crate::types::TaskOutcome::Failure { .. }) => 0.0,
        None => 0.5,
    });

    embedding
}

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
    /// Vector of episodes sorted by relevance (highest first), limited to `limit` items.
    /// Returns empty vector if no relevant episodes found.
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
    /// for episode in relevant_episodes {
    ///     println!("Similar task: {}", episode.task_description);
    ///     println!("Reward: {:?}", episode.reward);
    ///
    ///     if let Some(reflection) = episode.reflection {
    ///         println!("Key insights:");
    ///         for insight in reflection.insights {
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
    ) -> Vec<Episode> {
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

            return cached_episodes;
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

            // Prefer cache first
            if let Some(cache) = &self.cache_storage {
                if let Ok(fetched) = cache.query_episodes_since(since).await {
                    if !fetched.is_empty() {
                        let mut episodes = self.episodes_fallback.write().await;
                        for ep in fetched {
                            episodes.entry(ep.episode_id).or_insert(ep);
                        }
                    }
                }
            }

            // Then durable storage
            if let Some(turso) = &self.turso_storage {
                if let Ok(fetched) = turso.query_episodes_since(since).await {
                    if !fetched.is_empty() {
                        let mut episodes = self.episodes_fallback.write().await;
                        for ep in fetched {
                            episodes.entry(ep.episode_id).or_insert(ep);
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

        // Collect completed episodes
        let completed_episodes: Vec<Episode> = episodes
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

                        let semantic_episodes: Vec<Episode> =
                            results.into_iter().map(|result| result.item).collect();

                        // v0.1.12: Cache the results before returning
                        // Skip caching if result set is too large (>100KB estimated)
                        if should_cache_episodes(&semantic_episodes) {
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

            let query = RetrievalQuery {
                query_text: task_description.clone(),
                query_embedding,
                domain: Some(context.domain.clone()),
                task_type: None,  // Could extract from context if needed
                limit: limit * 2, // Retrieve more candidates for diversity maximization
            };

            match retriever.retrieve(&query, &completed_episodes).await {
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
            let mut relevant: Vec<Episode> = completed_episodes
                .into_iter()
                .filter(|e| self.is_relevant_episode(e, &context, &task_description))
                .collect();

            relevant.sort_by(|a, b| {
                let a_score = self.calculate_relevance_score(a, &context, &task_description);
                let b_score = self.calculate_relevance_score(b, &context, &task_description);
                b_score
                    .partial_cmp(&a_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            relevant.truncate(limit);
            info!(
                retrieved_count = relevant.len(),
                "Retrieved episodes using legacy method"
            );

            // v0.1.12: Cache the results before returning
            if should_cache_episodes(&relevant) {
                self.query_cache.put(cache_key.clone(), relevant.clone());
            } else {
                debug!(
                    episode_count = relevant.len(),
                    "Skipping cache for large result set"
                );
            }

            return relevant;
        }

        let scored_episodes = scored_episodes.unwrap();

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
            let result_episodes: Vec<Episode> = diverse_scored
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
                retrieved_count = result_episodes.len(),
                diversity_score = diversity_score,
                "Retrieved diverse, relevant episodes using Phase 3 hierarchical retrieval + MMR"
            );

            // v0.1.12: Cache the results before returning
            if should_cache_episodes(&result_episodes) {
                self.query_cache
                    .put(cache_key.clone(), result_episodes.clone());
            } else {
                debug!(
                    episode_count = result_episodes.len(),
                    "Skipping cache for large result set"
                );
            }

            return result_episodes;
        }

        // Diversity maximization disabled - just use top scored episodes
        let result_episodes: Vec<Episode> = scored_episodes
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
            retrieved_count = result_episodes.len(),
            "Retrieved episodes using hierarchical retrieval (diversity disabled)"
        );

        // v0.1.12: Cache the results before returning
        if should_cache_episodes(&result_episodes) {
            self.query_cache.put(cache_key, result_episodes.clone());
        } else {
            debug!(
                episode_count = result_episodes.len(),
                "Skipping cache for large result set"
            );
        }

        result_episodes
    }

    /// Retrieve relevant patterns for a task context
    ///
    /// Finds patterns that match the given context and ranks them
    /// by relevance and success rate.
    ///
    /// # Arguments
    ///
    /// * `context` - Task context to match against
    /// * `limit` - Maximum number of patterns to return
    ///
    /// # Returns
    ///
    /// Vector of relevant patterns, sorted by relevance and quality
    #[instrument(skip(self))]
    pub async fn retrieve_relevant_patterns(
        &self,
        context: &TaskContext,
        limit: usize,
    ) -> Vec<Pattern> {
        let mut patterns = self.patterns_fallback.write().await;

        debug!(
            total_patterns = patterns.len(),
            limit = limit,
            "Retrieving relevant patterns"
        );

        let all_patterns: Vec<Pattern> = patterns.values().cloned().collect();

        // Rank patterns by relevance and quality (includes effectiveness scoring)
        let mut ranked = rank_patterns(all_patterns, context);

        // Deduplicate
        ranked = deduplicate_patterns(ranked);

        // Record retrieval for effectiveness tracking
        for pattern in &mut ranked.iter_mut().take(limit) {
            pattern.record_retrieval();
            // Update the stored pattern with retrieval count
            patterns.insert(pattern.id(), pattern.clone());
        }

        // Limit results
        ranked.truncate(limit);

        info!(
            retrieved_count = ranked.len(),
            "Retrieved relevant patterns with effectiveness tracking"
        );

        ranked
    }

    /// Retrieve a single pattern by ID
    ///
    /// # Arguments
    ///
    /// * `pattern_id` - The unique ID of the pattern to retrieve
    ///
    /// # Returns
    ///
    /// The pattern if found, or None if not found
    ///
    /// # Errors
    ///
    /// Returns error if storage operation fails
    #[instrument(skip(self))]
    pub async fn get_pattern(&self, pattern_id: Uuid) -> crate::Result<Option<Pattern>> {
        // Try storage backends first
        if let Some(storage) = &self.turso_storage {
            match storage.get_pattern(pattern_id).await {
                Ok(pattern) => return Ok(pattern),
                Err(e) => {
                    debug!("Failed to get pattern from Turso storage: {}", e);
                    // Fall back to cache or in-memory
                }
            }
        }

        if let Some(cache) = &self.cache_storage {
            match cache.get_pattern(pattern_id).await {
                Ok(pattern) => return Ok(pattern),
                Err(e) => {
                    debug!("Failed to get pattern from cache storage: {}", e);
                    // Fall back to in-memory
                }
            }
        }

        // Fall back to in-memory storage
        let patterns = self.patterns_fallback.read().await;
        Ok(patterns.get(&pattern_id).cloned())
    }

    /// Check if episode is relevant to the query
    fn is_relevant_episode(
        &self,
        episode: &Episode,
        context: &TaskContext,
        task_description: &str,
    ) -> bool {
        // Match on domain
        if episode.context.domain == context.domain {
            return true;
        }

        // Match on language
        if episode.context.language == context.language && episode.context.language.is_some() {
            return true;
        }

        // Match on framework
        if episode.context.framework == context.framework && episode.context.framework.is_some() {
            return true;
        }

        // Match on tags
        let common_tags: Vec<_> = episode
            .context
            .tags
            .iter()
            .filter(|t| context.tags.contains(t))
            .collect();

        if !common_tags.is_empty() {
            return true;
        }

        // Simple text matching on description (very basic)
        let desc_lower = task_description.to_lowercase();
        let episode_desc_lower = episode.task_description.to_lowercase();

        let common_words: Vec<_> = desc_lower
            .split_whitespace()
            .filter(|w| w.len() > 3) // Ignore short words
            .filter(|w| episode_desc_lower.contains(w))
            .collect();

        !common_words.is_empty()
    }

    /// Calculate relevance score for an episode
    fn calculate_relevance_score(
        &self,
        episode: &Episode,
        context: &TaskContext,
        task_description: &str,
    ) -> f32 {
        let mut score = 0.0;

        // Reward quality (30% weight)
        if let Some(reward) = &episode.reward {
            score += reward.total * 0.3;
        }

        // Context match (40% weight)
        let mut context_score = 0.0;

        if episode.context.domain == context.domain {
            context_score += 0.4;
        }

        if episode.context.language == context.language && episode.context.language.is_some() {
            context_score += 0.3;
        }

        if episode.context.framework == context.framework && episode.context.framework.is_some() {
            context_score += 0.2;
        }

        let common_tags: Vec<_> = episode
            .context
            .tags
            .iter()
            .filter(|t| context.tags.contains(t))
            .collect();

        if !common_tags.is_empty() {
            context_score += 0.1 * common_tags.len() as f32;
        }

        score += context_score.min(0.4);

        // Description similarity (30% weight)
        let desc_lower = task_description.to_lowercase();
        let episode_desc_lower = episode.task_description.to_lowercase();

        let desc_words: Vec<_> = desc_lower.split_whitespace().collect();
        let common_words: Vec<_> = desc_words
            .iter()
            .filter(|w| w.len() > 3)
            .filter(|w| episode_desc_lower.contains(**w))
            .collect();

        if !desc_words.is_empty() {
            let similarity = common_words.len() as f32 / desc_words.len() as f32;
            score += similarity * 0.3;
        }

        score
    }

    /// Retrieve relevant heuristics for a given task context
    ///
    /// Finds heuristics that match the given context and ranks them
    /// by confidence weighted by relevance score.
    ///
    /// # Algorithm
    ///
    /// 1. Query heuristics from in-memory fallback
    /// 2. Calculate relevance score based on context similarity:
    ///    - Domain exact match: +1.0
    ///    - Language exact match: +0.8
    ///    - Framework match: +0.5
    ///    - Tag overlap: +0.3 per matching tag
    /// 3. Rank by: confidence × `relevance_score`
    /// 4. Return top N heuristics sorted by score DESC
    ///
    /// # Arguments
    ///
    /// * `context` - Task context to match against
    /// * `limit` - Maximum number of heuristics to return
    ///
    /// # Returns
    ///
    /// Vector of relevant heuristics, sorted by relevance and confidence
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::{SelfLearningMemory, TaskContext, ComplexityLevel};
    ///
    /// # async fn example() {
    /// let memory = SelfLearningMemory::new();
    ///
    /// let context = TaskContext {
    ///     language: Some("rust".to_string()),
    ///     framework: Some("tokio".to_string()),
    ///     complexity: ComplexityLevel::Moderate,
    ///     domain: "async-processing".to_string(),
    ///     tags: vec!["concurrency".to_string()],
    /// };
    ///
    /// // Retrieve top 5 relevant heuristics
    /// let heuristics = memory.retrieve_relevant_heuristics(&context, 5).await;
    ///
    /// for heuristic in heuristics {
    ///     println!("Condition: {}", heuristic.condition);
    ///     println!("Action: {}", heuristic.action);
    ///     println!("Confidence: {}", heuristic.confidence);
    /// }
    /// # }
    /// ```
    #[instrument(skip(self))]
    pub async fn retrieve_relevant_heuristics(
        &self,
        context: &TaskContext,
        limit: usize,
    ) -> Vec<crate::pattern::Heuristic> {
        let heuristics = self.heuristics_fallback.read().await;

        debug!(
            total_heuristics = heuristics.len(),
            limit = limit,
            "Retrieving relevant heuristics"
        );

        // Calculate weighted score for each heuristic
        let mut scored_heuristics: Vec<_> = heuristics
            .values()
            .map(|h| {
                let relevance = self.calculate_heuristic_relevance(h, context);
                let weighted_score = h.confidence * relevance;
                (h.clone(), weighted_score)
            })
            .filter(|(_, score)| *score > 0.0) // Only include relevant heuristics
            .collect();

        // Sort by weighted score (descending)
        scored_heuristics
            .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Limit results
        let result: Vec<_> = scored_heuristics
            .into_iter()
            .take(limit)
            .map(|(h, _)| h)
            .collect();

        info!(
            retrieved_count = result.len(),
            "Retrieved relevant heuristics"
        );

        result
    }

    /// Calculate relevance score for a heuristic based on context
    ///
    /// Scoring:
    /// - Domain exact match: +1.0
    /// - Language exact match: +0.8
    /// - Framework match: +0.5
    /// - Tag overlap: +0.3 per matching tag
    fn calculate_heuristic_relevance(
        &self,
        heuristic: &crate::pattern::Heuristic,
        context: &TaskContext,
    ) -> f32 {
        let mut score = 0.0;

        // Extract context from the heuristic condition
        // Heuristics store context information in their condition string
        let condition_lower = heuristic.condition.to_lowercase();

        // Check domain match (look for domain in condition string)
        if condition_lower.contains(&context.domain.to_lowercase()) {
            score += 1.0;
        }

        // Check language match
        if let Some(lang) = &context.language {
            if condition_lower.contains(&lang.to_lowercase()) {
                score += 0.8;
            }
        }

        // Check framework match
        if let Some(framework) = &context.framework {
            if condition_lower.contains(&framework.to_lowercase()) {
                score += 0.5;
            }
        }

        // Check tag overlap
        for tag in &context.tags {
            if condition_lower.contains(&tag.to_lowercase()) {
                score += 0.3;
            }
        }

        // If no specific matches, give a small baseline score for general heuristics
        if score == 0.0 {
            score = 0.1;
        }

        score
    }
}
