//! Episode completion and storage

use crate::error::{Error, Result};
use crate::security::audit::{AuditContext, episode_completed};
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
    /// `Ok(())` on success after all configured backends persist the episode.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] if the episode ID doesn't exist.
    ///
    /// Returns [`Error::ValidationFailed`] if the episode quality score is below threshold.
    ///
    /// Returns [`Error::Storage`] if any configured storage backend (`cache_storage`
    /// and/or `turso_storage`) fails to store the completed episode (ADR-075).
    /// Pattern extraction and in-memory map update are aborted on store failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use do_memory_core::{SelfLearningMemory, TaskContext, TaskType, TaskOutcome};
    /// use do_memory_core::{ExecutionStep, ExecutionResult};
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

        // ADR-071: Auto-create checkpoint on abstention before storage
        crate::memory::checkpoint::maybe_create_abstention_checkpoint(&mut episode);

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

                    // S1.4: durable backend deletion (idempotent per backend)
                    let mut backend_failures = 0usize;
                    for evicted_id in &evicted_ids {
                        if let Some(cache) = &self.cache_storage {
                            if let Err(e) = cache.delete_episode(*evicted_id).await {
                                backend_failures += 1;
                                warn!(
                                    episode_id = %evicted_id,
                                    error = %e,
                                    "Failed to delete capacity-evicted episode from cache storage"
                                );
                            }
                        }
                        if let Some(turso) = &self.turso_storage {
                            if let Err(e) = turso.delete_episode(*evicted_id).await {
                                backend_failures += 1;
                                warn!(
                                    episode_id = %evicted_id,
                                    error = %e,
                                    "Failed to delete capacity-evicted episode from durable storage"
                                );
                            }
                        }
                        // Drop embeddings for the evicted episode when backends support it
                        let embedding_key = evicted_id.to_string();
                        if let Some(cache) = &self.cache_storage {
                            let _ = cache.delete_embedding(&embedding_key).await;
                        }
                        if let Some(turso) = &self.turso_storage {
                            let _ = turso.delete_embedding(&embedding_key).await;
                        }
                    }

                    if backend_failures > 0 {
                        warn!(
                            evicted_count = evicted_ids.len(),
                            backend_failures,
                            "Capacity eviction partially failed; in-memory map updated, backends may need reconciliation"
                        );
                    } else {
                        info!(
                            evicted_ids = ?evicted_ids,
                            "Capacity-evicted episodes deleted from memory and storage backends"
                        );
                    }

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

        // ADR-075: durable complete is all-or-nothing for configured backends.
        // Collect every store failure, then hard-fail before claiming success
        // (skip in-memory map update and pattern extraction on failure).
        let mut store_failures: Vec<String> = Vec::new();

        if let Some(cache) = &self.cache_storage {
            if let Err(e) = cache.store_episode(episode_ref).await {
                warn!(
                    episode_id = %episode_id,
                    error = %e,
                    "Failed to store completed episode in cache"
                );
                store_failures.push(format!("cache: {e}"));
            }
        }

        if let Some(turso) = &self.turso_storage {
            if let Err(e) = turso.store_episode(episode_ref).await {
                warn!(
                    episode_id = %episode_id,
                    error = %e,
                    "Failed to store completed episode in Turso"
                );
                store_failures.push(format!("turso: {e}"));
            }
        }

        if !store_failures.is_empty() {
            return Err(Error::Storage(format!(
                "Failed to store completed episode to configured backend(s): {}",
                store_failures.join("; ")
            )));
        }

        // Store episode summary if generated
        // ADR-044 Feature 1: Persist semantic summary for playbook generation
        if let Some(ref summary) = summary {
            // Store in summaries cache for retrieval during playbook generation
            {
                let mut summaries = self.summaries_fallback.write().await;
                summaries.insert(episode_id, summary.clone());
            }
            info!(
                episode_id = %episode_id,
                summary_words = summary.summary_text.split_whitespace().count(),
                key_concepts = summary.key_concepts.len(),
                "Stored semantic summary"
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

                // Update ANN index for hybrid search (v0.1.34)
                if let Some(ref retriever) = self.semantic_retriever {
                    if let Ok(embeddings) = semantic.get_embeddings_batch(&[episode_id]).await {
                        if let Some(Some(embedding)) = embeddings.first() {
                            let _ = retriever.upsert(&episode_id.to_string(), embedding.clone());
                        }
                    }
                }
            }
        }

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
            TaskOutcome::Abstained { reason, .. } => reason.clone(),
        };
        let success = matches!(outcome, TaskOutcome::Success { .. });
        let audit_entry = episode_completed(&context, episode_id, &outcome_str, success);
        self.audit_logger.log(audit_entry);

        // WG-163 / ADR-055: emit lifecycle event to internal broadcast + external CloudEvents.
        self.emit_event_with_cloud(crate::types::MemoryEvent::EpisodeCompleted {
            id: episode_id.to_string(),
            reward: reward.total,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        });

        Ok(())
    }
}
