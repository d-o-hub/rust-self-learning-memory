//! Episode completion and storage

use crate::error::{Error, Result};
use crate::security::audit::{episode_completed, AuditContext};
use crate::types::TaskOutcome;
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

use super::SelfLearningMemory;

impl SelfLearningMemory {
    /// Complete an episode and trigger learning analysis.
    ///
    /// Finalizes the episode by recording the outcome, then performs the learning
    /// cycle:
    /// 1. **Marks complete** - Sets end time and outcome
    /// 2. **Calculates reward** - Scores based on success, efficiency, complexity
    /// 3. **Generates reflection** - Identifies successes, improvements, insights
    /// 4. **Extracts patterns** - Finds reusable patterns from execution steps
    /// 5. **Stores everything** - Persists to all storage backends
    ///
    /// This is the core learning step. Patterns extracted here become available
    /// for future task retrieval.
    ///
    /// # Pattern Extraction Modes
    ///
    /// - **Synchronous** (default): Patterns extracted before returning
    /// - **Asynchronous**: If [`enable_async_extraction()`](SelfLearningMemory::enable_async_extraction)
    ///   was called, patterns are extracted in background workers
    ///
    /// # Arguments
    ///
    /// * `episode_id` - ID returned from [`start_episode()`](SelfLearningMemory::start_episode)
    /// * `outcome` - Final outcome describing success/failure and artifacts
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the episode doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] if the episode ID doesn't exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::{SelfLearningMemory, TaskContext, TaskType, TaskOutcome};
    /// use memory_core::{ExecutionStep, ExecutionResult};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let memory = SelfLearningMemory::new();
    ///
    /// // Start episode
    /// let episode_id = memory.start_episode(
    ///     "Fix authentication bug".to_string(),
    ///     TaskContext::default(),
    ///     TaskType::Debugging,
    /// ).await;
    ///
    /// // Log debugging steps
    /// let mut step = ExecutionStep::new(1, "debugger".to_string(), "Identify issue".to_string());
    /// step.result = Some(ExecutionResult::Success {
    ///     output: "Found null pointer in auth handler".to_string(),
    /// });
    /// memory.log_step(episode_id, step).await;
    ///
    /// // Complete with success
    /// memory.complete_episode(
    ///     episode_id,
    ///     TaskOutcome::Success {
    ///         verdict: "Bug fixed, tests added".to_string(),
    ///         artifacts: vec!["auth_fix.patch".to_string(), "auth_test.rs".to_string()],
    ///     },
    /// ).await?;
    ///
    /// // Episode now has reward, reflection, and patterns
    /// let episode = memory.get_episode(episode_id).await?;
    /// assert!(episode.reward.is_some());
    /// assert!(episode.reflection.is_some());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, outcome), fields(episode_id = %episode_id))]
    pub async fn complete_episode(&self, episode_id: Uuid, outcome: TaskOutcome) -> Result<()> {
        // Flush any buffered steps before completing the episode
        // This ensures all steps are persisted and available for analysis
        if self.batch_config().is_some() {
            debug!(
                episode_id = %episode_id,
                "Flushing buffered steps before episode completion"
            );
            self.flush_steps(episode_id).await?;
        }

        // Get the episode Arc and clone it to work with Episode directly
        let episode_arc = {
            let episodes = self.episodes_fallback.read().await;
            episodes
                .get(&episode_id)
                .cloned()
                .ok_or(Error::NotFound(episode_id))?
        };
        let mut episode = (*episode_arc).clone();

        // Mark episode as complete
        episode.complete(outcome.clone());

        // Validate total episode size before processing
        super::validation::validate_episode_size(&episode)?;

        // ============================================================================
        // Pre-Storage Reasoning (PREMem Phase 1)
        // ============================================================================

        // 1. Assess episode quality before storage
        let quality_score = self.quality_assessor.assess_episode(&episode);

        info!(
            episode_id = %episode_id,
            quality_score = quality_score,
            quality_threshold = self.quality_threshold(),
            "Assessed episode quality"
        );

        // 2. Check if episode meets quality threshold
        if quality_score < self.quality_threshold() {
            warn!(
                episode_id = %episode_id,
                quality_score = quality_score,
                quality_threshold = self.quality_threshold(),
                "Episode rejected: quality score below threshold"
            );

            // Return error - episode will not be stored
            return Err(Error::ValidationFailed(format!(
                "Episode quality score ({:.2}) below threshold ({:.2})",
                quality_score,
                self.quality_threshold()
            )));
        }

        // 3. Extract salient features for high-quality episodes
        let salient_features = self.salient_extractor.extract(&episode);
        episode.salient_features = Some(salient_features.clone());

        debug!(
            episode_id = %episode_id,
            feature_count = salient_features.count(),
            critical_decisions = salient_features.critical_decisions.len(),
            tool_combinations = salient_features.tool_combinations.len(),
            error_recovery_patterns = salient_features.error_recovery_patterns.len(),
            key_insights = salient_features.key_insights.len(),
            "Extracted salient features"
        );

        // ============================================================================
        // Learning Analysis (Existing Workflow)
        // ============================================================================

        // Calculate reward score
        let reward = self.reward_calculator.calculate(&episode);
        episode.reward = Some(reward.clone());

        info!(
            episode_id = %episode_id,
            reward_total = reward.total,
            reward_base = reward.base,
            reward_efficiency = reward.efficiency,
            "Calculated reward score"
        );

        // Generate reflection
        let reflection = self.reflection_generator.generate(&episode);
        episode.reflection = Some(reflection.clone());

        debug!(
            successes = reflection.successes.len(),
            improvements = reflection.improvements.len(),
            insights = reflection.insights.len(),
            "Generated reflection"
        );

        // ============================================================================
        // Phase 2 (GENESIS) - Semantic Summarization
        // ============================================================================

        // Generate semantic summary before storage (if enabled)
        let summary = if let Some(ref summarizer) = self.semantic_summarizer {
            match summarizer.summarize_episode(&episode).await {
                Ok(summary) => {
                    info!(
                        episode_id = %episode_id,
                        summary_words = summary.summary_text.split_whitespace().count(),
                        key_concepts = summary.key_concepts.len(),
                        key_steps = summary.key_steps.len(),
                        "Generated semantic summary"
                    );
                    Some(summary)
                }
                Err(e) => {
                    warn!("Failed to generate semantic summary: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // ============================================================================
        // Phase 2 (GENESIS) - Capacity-Constrained Storage
        // ============================================================================

        // Store with capacity enforcement if configured, otherwise use normal storage
        if let Some(ref capacity_mgr) = self.capacity_manager {
            // Get all episodes EXCEPT the current one for capacity calculation
            // (the current episode is being added, so we check if we need to evict others)
            let (current_count, all_episodes) = {
                let eps = self.episodes_fallback.read().await;
                let episodes: Vec<_> = eps
                    .iter()
                    .filter(|(id, _)| **id != episode_id) // Exclude current episode
                    .map(|(_, ep)| (**ep).clone()) // Dereference twice: &Arc<Episode> -> Episode
                    .collect();
                (episodes.len(), episodes)
            };

            // Check if eviction is needed
            if !capacity_mgr.can_store(current_count) {
                let evicted_ids = capacity_mgr.evict_if_needed(&all_episodes);

                if !evicted_ids.is_empty() {
                    info!(
                        episode_id = %episode_id,
                        evicted_count = evicted_ids.len(),
                        "Evicting episodes due to capacity constraints"
                    );

                    // Remove evicted episodes from in-memory storage
                    {
                        let mut episodes_map = self.episodes_fallback.write().await;
                        for evicted_id in &evicted_ids {
                            episodes_map.remove(evicted_id);
                        }
                    }

                    // Remove from storage backends
                    // Note: In Phase 2.4, storage backends will have store_episode_with_capacity()
                    // For now, we just log the eviction
                    debug!(
                        evicted_ids = ?evicted_ids,
                        "Episodes evicted (backend deletion to be implemented in Phase 2.4)"
                    );

                    // Phase 3: Remove evicted episodes from spatiotemporal index
                    if let Some(ref index) = self.spatiotemporal_index {
                        if let Ok(mut index_write) = index.try_write() {
                            let mut removed_count = 0;
                            for evicted_id in &evicted_ids {
                                index_write.remove(*evicted_id);
                                removed_count += 1;
                            }
                            debug!(
                                evicted_count = removed_count,
                                "Removed evicted episodes from spatiotemporal index"
                            );
                        }
                    }
                }
            }
        }

        // Use the episode for storage operations
        let episode_ref = &episode;

        // Store updated episode in backends
        if let Some(cache) = &self.cache_storage {
            if let Err(e) = cache.store_episode(episode_ref).await {
                warn!("Failed to store completed episode in cache: {}", e);
            }
        }

        if let Some(turso) = &self.turso_storage {
            if let Err(e) = turso.store_episode(episode_ref).await {
                warn!("Failed to store completed episode in Turso: {}", e);
            }
        }

        // Store episode summary if generated
        // Note: In Phase 2.4, storage backends will have store_episode_summary()
        if let Some(_summary) = summary {
            debug!(
                episode_id = %episode_id,
                "Summary generated (storage to be implemented in Phase 2.4)"
            );
        }

        // ============================================================================
        // Phase 3 (Spatiotemporal) - Update hierarchical index
        // ============================================================================

        // Update spatiotemporal index if enabled
        if let Some(ref index) = self.spatiotemporal_index {
            if let Ok(mut index_write) = index.try_write() {
                index_write.insert(episode_ref);
                debug!(
                    episode_id = %episode_id,
                    domain = %episode.context.domain,
                    task_type = %episode.task_type,
                    "Inserted episode into spatiotemporal index"
                );
            } else {
                debug!(
                    episode_id = %episode_id,
                    "Spatiotemporal index locked, skipping indexing"
                );
            }
        }

        // ============================================================================
        // Semantic Search - Generate and store embedding
        // ============================================================================

        // Generate and store embedding for semantic search
        if let Some(ref semantic) = self.semantic_service {
            if let Err(e) = semantic.embed_episode(episode_ref).await {
                warn!(
                    episode_id = %episode_id,
                    error = %e,
                    "Failed to generate embedding for episode. Continuing without embedding."
                );
                // Don't fail entire operation on embedding error
            } else {
                debug!(
                    episode_id = %episode_id,
                    "Successfully generated embedding for episode"
                );
            }
        }

        // ============================================================================
        // v0.1.12: Invalidate Query Cache
        // ============================================================================

        // Invalidate all cached queries since we added a new episode
        // This ensures future retrievals will include the new episode
        let metrics_before = self.query_cache.metrics();
        self.query_cache.invalidate_all();
        info!(
            episode_id = %episode_id,
            invalidated_entries = metrics_before.size,
            total_invalidations = metrics_before.invalidations + metrics_before.size as u64,
            "Invalidated query cache after episode completion"
        );

        // ============================================================================
        // Re-insert the updated episode into the in-memory cache
        // ============================================================================

        {
            let mut episodes = self.episodes_fallback.write().await;
            episodes.insert(episode_id, Arc::new(episode));
        }

        // Extract patterns - async if queue enabled, sync otherwise
        if let Some(queue) = &self.pattern_queue {
            // Async path: enqueue for background processing
            queue.enqueue_episode(episode_id).await?;
            info!(
                episode_id = %episode_id,
                "Episode completed, enqueued for async pattern extraction"
            );
        } else {
            // Sync path: extract patterns immediately
            self.extract_patterns_sync(episode_id).await?;
            info!(
                episode_id = %episode_id,
                "Episode completed and patterns extracted synchronously"
            );
        }

        // Audit log: episode completed
        let context = AuditContext::system();
        let outcome_str = match &outcome {
            TaskOutcome::Success { verdict, .. } => verdict.clone(),
            TaskOutcome::PartialSuccess { verdict, .. } => verdict.clone(),
            TaskOutcome::Failure { reason, .. } => reason.clone(),
        };
        let success = matches!(outcome, TaskOutcome::Success { .. });
        let audit_entry = episode_completed(&context, episode_id, &outcome_str, success);
        self.audit_logger.log(audit_entry);

        Ok(())
    }
}
