//! Episode completion and pattern learning

use crate::error::{Error, Result};
use crate::pattern::Pattern;
use crate::types::TaskOutcome;
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
        if self.config.batch_config.is_some() {
            debug!(
                episode_id = %episode_id,
                "Flushing buffered steps before episode completion"
            );
            self.flush_steps(episode_id).await?;
        }

        let mut episodes = self.episodes_fallback.write().await;

        let episode = episodes
            .get_mut(&episode_id)
            .ok_or(Error::NotFound(episode_id))?;

        // Mark episode as complete
        episode.complete(outcome);

        // Validate total episode size before processing
        super::validation::validate_episode_size(episode)?;

        // Calculate reward score
        let reward = self.reward_calculator.calculate(episode);
        episode.reward = Some(reward.clone());

        info!(
            episode_id = %episode_id,
            reward_total = reward.total,
            reward_base = reward.base,
            reward_efficiency = reward.efficiency,
            "Calculated reward score"
        );

        // Generate reflection
        let reflection = self.reflection_generator.generate(episode);
        episode.reflection = Some(reflection.clone());

        debug!(
            successes = reflection.successes.len(),
            improvements = reflection.improvements.len(),
            insights = reflection.insights.len(),
            "Generated reflection"
        );

        // Store updated episode in backends
        if let Some(cache) = &self.cache_storage {
            if let Err(e) = cache.store_episode(episode).await {
                warn!("Failed to store completed episode in cache: {}", e);
            }
        }

        if let Some(turso) = &self.turso_storage {
            if let Err(e) = turso.store_episode(episode).await {
                warn!("Failed to store completed episode in Turso: {}", e);
            }
        }

        // Release the write lock before pattern extraction
        drop(episodes);

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

        Ok(())
    }

    /// Extract patterns synchronously (internal helper)
    ///
    /// Used when async extraction is not enabled.
    pub(super) async fn extract_patterns_sync(&self, episode_id: Uuid) -> Result<()> {
        let mut episodes = self.episodes_fallback.write().await;
        let episode = episodes
            .get_mut(&episode_id)
            .ok_or(Error::NotFound(episode_id))?;

        // Extract patterns
        let extracted_patterns = self.pattern_extractor.extract(episode);

        debug!(
            pattern_count = extracted_patterns.len(),
            "Extracted patterns synchronously"
        );

        // Store patterns and link to episode
        let mut patterns = self.patterns_fallback.write().await;
        let mut pattern_ids = Vec::new();

        for pattern in extracted_patterns {
            let pattern_id = pattern.id();
            pattern_ids.push(pattern_id);

            // Store in backends
            if let Some(cache) = &self.cache_storage {
                if let Err(e) = cache.store_pattern(&pattern).await {
                    warn!("Failed to store pattern in cache: {}", e);
                }
            }

            if let Some(turso) = &self.turso_storage {
                if let Err(e) = turso.store_pattern(&pattern).await {
                    warn!("Failed to store pattern in Turso: {}", e);
                }
            }

            patterns.insert(pattern_id, pattern);
        }

        episode.patterns = pattern_ids;

        // Extract heuristics
        match self.heuristic_extractor.extract(episode).await {
            Ok(extracted_heuristics) => {
                debug!(
                    heuristic_count = extracted_heuristics.len(),
                    "Extracted heuristics synchronously"
                );

                // Store heuristics and link to episode
                let mut heuristic_ids = Vec::new();
                let mut heuristics_map = self.heuristics_fallback.write().await;

                for heuristic in &extracted_heuristics {
                    heuristic_ids.push(heuristic.heuristic_id);

                    // Store in backends
                    if let Some(cache) = &self.cache_storage {
                        if let Err(e) = cache.store_heuristic(heuristic).await {
                            warn!("Failed to store heuristic in cache: {}", e);
                        }
                    }

                    if let Some(turso) = &self.turso_storage {
                        if let Err(e) = turso.store_heuristic(heuristic).await {
                            warn!("Failed to store heuristic in Turso: {}", e);
                        }
                    }

                    // Store in in-memory fallback
                    heuristics_map.insert(heuristic.heuristic_id, heuristic.clone());
                }

                episode.heuristics = heuristic_ids;
            }
            Err(e) => {
                warn!("Failed to extract heuristics: {}", e);
                episode.heuristics = Vec::new();
            }
        }

        // Update episode with pattern and heuristic IDs
        if let Some(cache) = &self.cache_storage {
            if let Err(e) = cache.store_episode(episode).await {
                warn!(
                    "Failed to update episode with patterns and heuristics in cache: {}",
                    e
                );
            }
        }

        if let Some(turso) = &self.turso_storage {
            if let Err(e) = turso.store_episode(episode).await {
                warn!(
                    "Failed to update episode with patterns and heuristics in Turso: {}",
                    e
                );
            }
        }

        Ok(())
    }

    /// Store patterns (for use by async extraction workers)
    ///
    /// Links patterns to an episode. This is public so the queue workers
    /// can call it after extracting patterns asynchronously.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - Episode these patterns came from
    /// * `patterns` - Patterns to store
    ///
    /// # Errors
    ///
    /// Returns error if episode not found
    pub async fn store_patterns(
        &self,
        episode_id: Uuid,
        extracted_patterns: Vec<Pattern>,
    ) -> Result<()> {
        let mut episodes = self.episodes_fallback.write().await;
        let episode = episodes
            .get_mut(&episode_id)
            .ok_or(Error::NotFound(episode_id))?;

        let mut patterns = self.patterns_fallback.write().await;
        let mut pattern_ids = Vec::new();

        for pattern in extracted_patterns {
            let pattern_id = pattern.id();
            pattern_ids.push(pattern_id);

            // Store in backends
            if let Some(cache) = &self.cache_storage {
                if let Err(e) = cache.store_pattern(&pattern).await {
                    warn!("Failed to store pattern in cache: {}", e);
                }
            }

            if let Some(turso) = &self.turso_storage {
                if let Err(e) = turso.store_pattern(&pattern).await {
                    warn!("Failed to store pattern in Turso: {}", e);
                }
            }

            patterns.insert(pattern_id, pattern);
        }

        episode.patterns = pattern_ids;

        // Update episode with pattern IDs
        if let Some(cache) = &self.cache_storage {
            if let Err(e) = cache.store_episode(episode).await {
                warn!("Failed to update episode with patterns in cache: {}", e);
            }
        }

        if let Some(turso) = &self.turso_storage {
            if let Err(e) = turso.store_episode(episode).await {
                warn!("Failed to update episode with patterns in Turso: {}", e);
            }
        }

        Ok(())
    }

    /// Get queue statistics (if async extraction enabled)
    ///
    /// Returns statistics about the pattern extraction queue,
    /// or None if async extraction is not enabled.
    pub async fn get_queue_stats(&self) -> Option<crate::learning::queue::QueueStats> {
        if let Some(queue) = &self.pattern_queue {
            Some(queue.get_stats().await)
        } else {
            None
        }
    }

    /// Update heuristic confidence based on new episode outcome
    ///
    /// Updates a heuristic's confidence score by incorporating evidence from
    /// a new episode. The heuristic is retrieved from storage, updated with
    /// the new evidence, and persisted back to all storage backends.
    ///
    /// # Algorithm
    ///
    /// 1. Retrieve heuristic from in-memory fallback (or storage if needed)
    /// 2. Call `heuristic.update_evidence(episode_id, is_success)`
    /// 3. Recalculate confidence: `success_rate` × √`sample_size`
    /// 4. Store updated heuristic to both Turso and redb
    /// 5. Update in-memory fallback
    ///
    /// # Arguments
    ///
    /// * `heuristic_id` - ID of the heuristic to update
    /// * `episode_id` - ID of the episode providing new evidence
    /// * `outcome` - Outcome of the episode
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the heuristic doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] if the heuristic ID doesn't exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::{SelfLearningMemory, TaskOutcome};
    /// use uuid::Uuid;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let memory = SelfLearningMemory::new();
    ///
    /// let heuristic_id = Uuid::new_v4(); // From previous heuristic extraction
    /// let episode_id = Uuid::new_v4();   // From current episode
    ///
    /// // Update with successful outcome
    /// memory.update_heuristic_confidence(
    ///     heuristic_id,
    ///     episode_id,
    ///     TaskOutcome::Success {
    ///         verdict: "Applied heuristic successfully".to_string(),
    ///         artifacts: vec![],
    ///     },
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, outcome), fields(heuristic_id = %heuristic_id, episode_id = %episode_id))]
    pub async fn update_heuristic_confidence(
        &self,
        heuristic_id: Uuid,
        episode_id: Uuid,
        outcome: TaskOutcome,
    ) -> Result<()> {
        let mut heuristics = self.heuristics_fallback.write().await;

        let heuristic = heuristics
            .get_mut(&heuristic_id)
            .ok_or(Error::NotFound(heuristic_id))?;

        // Determine if the outcome was successful
        let is_success = matches!(
            outcome,
            TaskOutcome::Success { .. } | TaskOutcome::PartialSuccess { .. }
        );

        debug!(
            heuristic_id = %heuristic_id,
            episode_id = %episode_id,
            is_success = is_success,
            old_confidence = heuristic.confidence,
            old_success_rate = heuristic.evidence.success_rate,
            old_sample_size = heuristic.evidence.sample_size,
            "Updating heuristic confidence"
        );

        // Update evidence
        heuristic.update_evidence(episode_id, is_success);

        // Recalculate confidence: success_rate × √sample_size
        let new_confidence =
            heuristic.evidence.success_rate * (heuristic.evidence.sample_size as f32).sqrt();
        heuristic.confidence = new_confidence;

        info!(
            heuristic_id = %heuristic_id,
            new_confidence = new_confidence,
            new_success_rate = heuristic.evidence.success_rate,
            new_sample_size = heuristic.evidence.sample_size,
            "Updated heuristic confidence"
        );

        // Store updated heuristic in backends
        if let Some(cache) = &self.cache_storage {
            if let Err(e) = cache.store_heuristic(heuristic).await {
                warn!("Failed to store updated heuristic in cache: {}", e);
            }
        }

        if let Some(turso) = &self.turso_storage {
            if let Err(e) = turso.store_heuristic(heuristic).await {
                warn!("Failed to store updated heuristic in Turso: {}", e);
            }
        }

        Ok(())
    }
}
