//! Episode lifecycle management

use crate::episode::{Episode, ExecutionStep};
use crate::error::{Error, Result};
use crate::security::audit::{episode_created, AuditContext};
use crate::types::{TaskContext, TaskType};
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

use super::SelfLearningMemory;

impl SelfLearningMemory {
    /// Start a new episode to track a task execution.
    ///
    /// Creates a new episode record and stores it in all configured backends
    /// (cache, durable storage, and in-memory). This marks the beginning of
    /// the learning cycle for a task.
    ///
    /// The episode is created with a unique ID and current timestamp. Initially,
    /// it has no steps, outcome, or analysis. Steps should be logged as the
    /// task executes, and the episode should be completed when finished.
    ///
    /// # Arguments
    ///
    /// * `task_description` - Clear, human-readable description of what needs to be done
    /// * `context` - Contextual metadata (language, domain, framework, etc.) used for retrieval
    /// * `task_type` - Classification of task (`CodeGeneration`, Debugging, etc.)
    ///
    /// # Returns
    ///
    /// Unique episode ID that should be used for subsequent [`log_step()`](SelfLearningMemory::log_step)
    /// and [`complete_episode()`](SelfLearningMemory::complete_episode) calls.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::{SelfLearningMemory, TaskContext, TaskType, ComplexityLevel};
    ///
    /// # async fn example() {
    /// let memory = SelfLearningMemory::new();
    ///
    /// // Start tracking a code generation task
    /// let context = TaskContext {
    ///     language: Some("rust".to_string()),
    ///     framework: Some("tokio".to_string()),
    ///     complexity: ComplexityLevel::Moderate,
    ///     domain: "async-io".to_string(),
    ///     tags: vec!["networking".to_string(), "http".to_string()],
    /// };
    ///
    /// let episode_id = memory.start_episode(
    ///     "Implement async HTTP client with retry logic".to_string(),
    ///     context,
    ///     TaskType::CodeGeneration,
    /// ).await;
    ///
    /// // Use episode_id to log steps and complete the episode
    /// println!("Started episode: {}", episode_id);
    /// # }
    /// ```
    #[instrument(skip(self), fields(task_type = %task_type))]
    pub async fn start_episode(
        &self,
        task_description: String,
        context: TaskContext,
        task_type: TaskType,
    ) -> Uuid {
        // Validate task description length
        // Note: Validation warnings are logged but don't prevent episode creation
        // to maintain backward compatibility with existing API
        if let Err(e) = super::validation::validate_task_description(&task_description) {
            warn!(
                error = %e,
                description_len = task_description.len(),
                "Task description validation failed - creating episode anyway for backward compatibility"
            );
        }

        let episode = Episode::new(task_description.clone(), context, task_type);
        let episode_id = episode.episode_id;

        info!(
            episode_id = %episode_id,
            task_description = %task_description,
            "Started new episode"
        );

        // Store in cache first (fast), then Turso (durable)
        if let Some(cache) = &self.cache_storage {
            if let Err(e) = cache.store_episode(&episode).await {
                warn!("Failed to store episode in cache: {}", e);
            }
        }

        if let Some(turso) = &self.turso_storage {
            if let Err(e) = turso.store_episode(&episode).await {
                warn!("Failed to store episode in Turso: {}", e);
            }
        }

        // Always store in fallback for in-memory access
        // Store as Arc to avoid cloning when sharing
        let mut episodes = self.episodes_fallback.write().await;
        episodes.insert(episode_id, Arc::new(episode));

        // Audit log: episode created
        let context = AuditContext::system();
        let audit_entry = episode_created(
            &context,
            episode_id,
            &task_description,
            &task_type.to_string(),
        );
        self.audit_logger.log(audit_entry);

        episode_id
    }

    /// Log an execution step for an ongoing episode.
    ///
    /// Records a single discrete action or operation performed during task execution.
    /// Steps should be logged sequentially as they occur to maintain accurate timing,
    /// ordering, and execution trace information.
    ///
    /// Each step captures what was done, which tool did it, what the result was,
    /// and how long it took. This detailed trace enables pattern extraction and
    /// learning from successful execution sequences.
    ///
    /// # Batching Behavior
    ///
    /// If step batching is enabled in the configuration, steps are buffered in memory
    /// and flushed to storage when:
    /// - Buffer size reaches `max_batch_size`, OR
    /// - Time since last flush exceeds `flush_interval_ms`, OR
    /// - Episode is completed (automatic flush)
    ///
    /// If batching is disabled, steps are persisted immediately (legacy behavior).
    ///
    /// # Arguments
    ///
    /// * `episode_id` - ID of the episode to add the step to (from [`start_episode()`](SelfLearningMemory::start_episode))
    /// * `step` - Execution step details including tool, action, result, timing
    ///
    /// # Behavior
    ///
    /// - Validates the step before adding
    /// - Buffers step if batching enabled and conditions not met
    /// - Flushes buffered steps when conditions met
    /// - Logs warning (doesn't error) if episode not found
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::{SelfLearningMemory, TaskContext, TaskType};
    /// use memory_core::{ExecutionStep, ExecutionResult};
    ///
    /// # async fn example() {
    /// let memory = SelfLearningMemory::new();
    ///
    /// let episode_id = memory.start_episode(
    ///     "Parse configuration file".to_string(),
    ///     TaskContext::default(),
    ///     TaskType::CodeGeneration,
    /// ).await;
    ///
    /// // Log each step as it happens
    /// let mut step1 = ExecutionStep::new(1, "file_reader".to_string(), "Read config.toml".to_string());
    /// step1.parameters = serde_json::json!({ "path": "config.toml" });
    /// step1.result = Some(ExecutionResult::Success {
    ///     output: "File read: 1024 bytes".to_string(),
    /// });
    /// step1.latency_ms = 5;
    /// memory.log_step(episode_id, step1).await;
    ///
    /// let mut step2 = ExecutionStep::new(2, "toml_parser".to_string(), "Parse TOML".to_string());
    /// step2.result = Some(ExecutionResult::Success {
    ///     output: "Parsed successfully".to_string(),
    /// });
    /// step2.latency_ms = 12;
    /// memory.log_step(episode_id, step2).await;
    ///
    /// // Verify steps were logged (may need flush first if batching enabled)
    /// let episode = memory.get_episode(episode_id).await.unwrap();
    /// # }
    /// ```
    #[instrument(skip(self, step), fields(episode_id = %episode_id, step_number = step.step_number))]
    pub async fn log_step(&self, episode_id: Uuid, step: ExecutionStep) {
        // Validate step first (before acquiring locks)
        // Use Arc::clone to check without consuming
        {
            let episodes = self.episodes_fallback.read().await;
            if let Some(episode) = episodes.get(&episode_id) {
                if let Err(e) = super::validation::validate_execution_step(episode, &step) {
                    warn!(
                        episode_id = %episode_id,
                        step_number = step.step_number,
                        error = %e,
                        "Execution step validation failed - skipping step"
                    );
                    return;
                }
            } else {
                warn!("Attempted to log step for non-existent episode");
                return;
            }
        }

        debug!(
            step_number = step.step_number,
            tool = %step.tool,
            "Logging execution step"
        );

        // Check if batching is enabled
        if let Some(batch_config) = self.batch_config() {
            // Use step buffering
            let mut buffers = self.step_buffers.write().await;
            let buffer = buffers.entry(episode_id).or_insert_with(|| {
                super::step_buffer::StepBuffer::new(episode_id, (*batch_config).clone())
            });

            // Add step to buffer
            if let Err(e) = buffer.add_step(step) {
                warn!("Failed to add step to buffer: {}", e);
                return;
            }

            // Check if we should flush
            let should_flush = buffer.should_flush();
            drop(buffers); // Release lock before flush

            if should_flush {
                self.flush_steps_internal(episode_id).await;
            }
        } else {
            // Legacy immediate persistence (batching disabled)
            // We need to clone, modify, and reinsert since we can't mutate through Arc
            let mut episodes = self.episodes_fallback.write().await;
            if let Some(episode_arc) = episodes.get(&episode_id) {
                // Clone the Episode from the Arc to mutate it
                // episode_arc is &Arc<Episode>, so *episode_arc gives Arc<Episode>
                // We need **episode_arc to get Episode via Deref, then clone it
                let mut episode = (**episode_arc).clone();
                episode.add_step(step);

                // Update in storage backends
                if let Some(cache) = &self.cache_storage {
                    if let Err(e) = cache.store_episode(&episode).await {
                        warn!("Failed to update episode in cache: {}", e);
                    }
                }

                if let Some(turso) = &self.turso_storage {
                    if let Err(e) = turso.store_episode(&episode).await {
                        warn!("Failed to update episode in Turso: {}", e);
                    }
                }

                // Re-insert the updated episode as Arc
                episodes.insert(episode_id, Arc::new(episode));
            }
        }
    }

    /// Flush buffered steps for an episode to storage.
    ///
    /// Manually flushes any buffered steps for the given episode to persistent storage.
    /// This is useful when you want to ensure all steps are persisted before
    /// performing operations that depend on complete step history.
    ///
    /// If step batching is disabled, this is a no-op.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - ID of the episode to flush steps for
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if flush fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::{SelfLearningMemory, TaskContext, TaskType};
    /// use memory_core::ExecutionStep;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let memory = SelfLearningMemory::new();
    ///
    /// let episode_id = memory.start_episode(
    ///     "Task".to_string(),
    ///     TaskContext::default(),
    ///     TaskType::Testing,
    /// ).await;
    ///
    /// // Log many steps
    /// for i in 1..=100 {
    ///     let step = ExecutionStep::new(i, "tool".to_string(), "action".to_string());
    ///     memory.log_step(episode_id, step).await;
    /// }
    ///
    /// // Ensure all steps are persisted
    /// memory.flush_steps(episode_id).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(episode_id = %episode_id))]
    pub async fn flush_steps(&self, episode_id: Uuid) -> crate::error::Result<()> {
        self.flush_steps_internal(episode_id).await;
        Ok(())
    }

    /// Internal flush implementation (no error wrapping)
    async fn flush_steps_internal(&self, episode_id: Uuid) {
        // Take steps from buffer
        let steps_to_flush = {
            let mut buffers = self.step_buffers.write().await;
            if let Some(buffer) = buffers.get_mut(&episode_id) {
                let steps = buffer.take_steps();
                if steps.is_empty() {
                    return; // Nothing to flush
                }
                debug!(
                    episode_id = %episode_id,
                    step_count = steps.len(),
                    "Flushing buffered steps to storage"
                );
                steps
            } else {
                return; // No buffer for this episode
            }
        };

        // Add steps to episode and persist
        // We need to clone, modify, and reinsert since we can't mutate through Arc
        let mut episodes = self.episodes_fallback.write().await;
        if let Some(episode_arc) = episodes.get(&episode_id) {
            // Clone the Episode from the Arc to mutate it
            // episode_arc is &Arc<Episode>, so **episode_arc gives Episode via Deref
            let episode = (**episode_arc).clone();
            let total_steps = episode.steps.len();
            let mut episode = episode;
            for step in steps_to_flush {
                episode.add_step(step);
            }

            // Update in storage backends
            if let Some(cache) = &self.cache_storage {
                if let Err(e) = cache.store_episode(&episode).await {
                    warn!("Failed to flush episode to cache: {}", e);
                }
            }

            if let Some(turso) = &self.turso_storage {
                if let Err(e) = turso.store_episode(&episode).await {
                    warn!("Failed to flush episode to Turso: {}", e);
                }
            }

            // Re-insert the updated episode as Arc
            episodes.insert(episode_id, Arc::new(episode));

            info!(
                episode_id = %episode_id,
                total_steps = total_steps,
                "Successfully flushed buffered steps"
            );
        } else {
            warn!("Attempted to flush steps for non-existent episode");
        }
    }

    /// Get an episode by ID
    ///
    /// # Arguments
    ///
    /// * `episode_id` - ID of the episode to retrieve
    ///
    /// # Returns
    ///
    /// The episode if found
    ///
    /// # Errors
    ///
    /// Returns `Error::NotFound` if the episode doesn't exist
    pub async fn get_episode(&self, episode_id: Uuid) -> Result<Episode> {
        // 1) Try in-memory cache first
        if let Some(ep) = {
            let episodes = self.episodes_fallback.read().await;
            episodes.get(&episode_id).cloned()
        } {
            // ep is Arc<Episode>, dereference to get Episode
            return Ok((*ep).clone());
        }

        // 2) Try cache storage (redb) next
        if let Some(cache) = &self.cache_storage {
            match cache.get_episode(episode_id).await {
                Ok(Some(episode)) => {
                    // Store in cache as Arc<Episode> for cheap cloning
                    let mut episodes = self.episodes_fallback.write().await;
                    episodes.insert(episode_id, Arc::new(episode.clone()));
                    return Ok(episode);
                }
                Ok(None) => {
                    // Not found in cache; continue to durable storage
                }
                Err(_e) => {
                    // Cache access failed; continue to durable storage
                }
            }
        }

        // 3) Try durable storage (Turso)
        if let Some(turso) = &self.turso_storage {
            match turso.get_episode(episode_id).await {
                Ok(Some(episode)) => {
                    // Populate in-memory cache for subsequent accesses
                    let mut episodes = self.episodes_fallback.write().await;
                    episodes.insert(episode_id, Arc::new(episode.clone()));
                    return Ok(episode);
                }
                Ok(None) => {}
                Err(_e) => {}
            }
        }

        Err(Error::NotFound(episode_id))
    }
}
