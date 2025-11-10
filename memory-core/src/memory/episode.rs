//! Episode lifecycle management

use crate::episode::{Episode, ExecutionStep};
use crate::error::{Error, Result};
use crate::types::{TaskContext, TaskType};
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
    /// * `task_type` - Classification of task (CodeGeneration, Debugging, etc.)
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
        let mut episodes = self.episodes_fallback.write().await;
        episodes.insert(episode_id, episode);

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
    /// # Arguments
    ///
    /// * `episode_id` - ID of the episode to add the step to (from [`start_episode()`](SelfLearningMemory::start_episode))
    /// * `step` - Execution step details including tool, action, result, timing
    ///
    /// # Behavior
    ///
    /// - Updates all storage backends (cache, durable, in-memory)
    /// - Logs warning (doesn't error) if episode not found
    /// - Updates persisted episode immediately (not buffered)
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
    /// // Verify steps were logged
    /// let episode = memory.get_episode(episode_id).await.unwrap();
    /// assert_eq!(episode.steps.len(), 2);
    /// assert_eq!(episode.successful_steps_count(), 2);
    /// # }
    /// ```
    #[instrument(skip(self, step), fields(episode_id = %episode_id, step_number = step.step_number))]
    pub async fn log_step(&self, episode_id: Uuid, step: ExecutionStep) {
        let mut episodes = self.episodes_fallback.write().await;

        if let Some(episode) = episodes.get_mut(&episode_id) {
            debug!(
                step_number = step.step_number,
                tool = %step.tool,
                "Logged execution step"
            );
            episode.add_step(step);

            // Update in storage backends
            if let Some(cache) = &self.cache_storage {
                if let Err(e) = cache.store_episode(episode).await {
                    warn!("Failed to update episode in cache: {}", e);
                }
            }

            if let Some(turso) = &self.turso_storage {
                if let Err(e) = turso.store_episode(episode).await {
                    warn!("Failed to update episode in Turso: {}", e);
                }
            }
        } else {
            warn!("Attempted to log step for non-existent episode");
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
        let episodes = self.episodes_fallback.read().await;
        episodes
            .get(&episode_id)
            .cloned()
            .ok_or(Error::NotFound(episode_id))
    }
}
