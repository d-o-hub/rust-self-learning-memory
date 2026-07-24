//! Episode completion and pattern learning

use crate::error::{Error, Result};
use crate::patterns::Pattern;
use crate::types::TaskOutcome;
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

use super::SelfLearningMemory;

impl SelfLearningMemory {
    pub(super) async fn extract_patterns_sync(&self, episode_id: Uuid) -> Result<()> {
        // Get the episode Arc and clone it to work with Episode directly
        let episode_arc = {
            let episodes = self.episodes_fallback.read().await;
            episodes
                .get(&episode_id)
                .cloned()
                .ok_or(Error::NotFound(episode_id))?
        };
        let mut episode = (*episode_arc).clone();

        // Extract patterns
        let extracted_patterns = self.pattern_extractor.extract(&episode);

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
        match self.heuristic_extractor.extract(&episode).await {
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
                        #[expect(clippy::excessive_nesting)]
                        if let Err(e) = cache.store_heuristic(heuristic).await {
                            warn!("Failed to store heuristic in cache: {}", e);
                        }
                    }

                    if let Some(turso) = &self.turso_storage {
                        #[expect(clippy::excessive_nesting)]
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

        // Update episode with pattern and heuristic IDs in storage backends
        if let Some(cache) = &self.cache_storage {
            if let Err(e) = cache.store_episode(&episode).await {
                warn!(
                    "Failed to update episode with patterns and heuristics in cache: {}",
                    e
                );
            }
        }

        if let Some(turso) = &self.turso_storage {
            if let Err(e) = turso.store_episode(&episode).await {
                warn!(
                    "Failed to update episode with patterns and heuristics in Turso: {}",
                    e
                );
            }
        }

        // Re-insert the updated episode into the in-memory cache
        let mut episodes = self.episodes_fallback.write().await;
        episodes.insert(episode_id, Arc::new(episode));

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
        // Get the episode Arc and clone it to work with Episode directly
        let episode_arc = {
            let episodes = self.episodes_fallback.read().await;
            episodes
                .get(&episode_id)
                .cloned()
                .ok_or(Error::NotFound(episode_id))?
        };
        let mut episode = (*episode_arc).clone(); // Deref Arc<Episode> to Episode, then clone

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

        // Update episode with pattern IDs in storage backends
        if let Some(cache) = &self.cache_storage {
            if let Err(e) = cache.store_episode(&episode).await {
                warn!("Failed to update episode with patterns in cache: {}", e);
            }
        }

        if let Some(turso) = &self.turso_storage {
            if let Err(e) = turso.store_episode(&episode).await {
                warn!("Failed to update episode with patterns in Turso: {}", e);
            }
        }

        // Re-insert the updated episode into the in-memory cache
        let mut episodes = self.episodes_fallback.write().await;
        episodes.insert(episode_id, Arc::new(episode));

        Ok(())
    }

    /// Re-run pattern extraction for an already-completed episode (ADR-076 §5).
    ///
    /// This is the public entry-point used by the `pattern extract` CLI command.
    /// It validates that the episode exists and is in a completed state, then
    /// delegates to the same extractor pipeline that `complete_episode` uses.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - ID of the episode to re-extract patterns for
    ///
    /// # Returns
    ///
    /// The number of patterns extracted and stored.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] if the episode does not exist.
    /// Returns [`Error::ValidationFailed`] if the episode is not yet completed.
    pub async fn re_extract_patterns(&self, episode_id: Uuid) -> Result<usize> {
        // Validate: episode must exist and be complete.
        let episode_arc = {
            let episodes = self.episodes_fallback.read().await;
            episodes
                .get(&episode_id)
                .cloned()
                .ok_or(Error::NotFound(episode_id))?
        };

        if !episode_arc.is_complete() {
            return Err(Error::ValidationFailed(format!(
                "Episode {episode_id} is not in Completed state; \
                 re-extraction requires a completed episode"
            )));
        }

        // Count patterns before extraction so we can report the delta.
        let count_before = {
            let patterns = self.patterns_fallback.read().await;
            // Count patterns linked to this episode.
            episode_arc.patterns.len().min(patterns.len())
        };
        let linked_before = episode_arc.patterns.len();

        self.extract_patterns_sync(episode_id).await?;

        // Report how many pattern IDs are now linked to the episode.
        let linked_after = {
            let episodes = self.episodes_fallback.read().await;
            episodes
                .get(&episode_id)
                .map(|ep| ep.patterns.len())
                .unwrap_or(linked_before)
        };

        let _ = count_before; // silence unused warning
        Ok(linked_after)
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
    /// use do_memory_core::{SelfLearningMemory, TaskOutcome};
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
