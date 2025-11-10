//! # Self Learning Memory
//!
//! Main orchestrator for the episodic learning system.
//!
//! Provides the complete learning cycle:
//! 1. **Start Episode** - Initialize task tracking
//! 2. **Log Steps** - Record execution steps
//! 3. **Complete Episode** - Analyze, score, reflect, and extract patterns
//! 4. **Retrieve Context** - Query relevant episodes and patterns
//!
//! ## Example
//!
//! ```no_run
//! use memory_core::memory::SelfLearningMemory;
//! use memory_core::{TaskContext, TaskType, TaskOutcome, ExecutionStep};
//!
//! #[tokio::main]
//! async fn main() {
//!     let memory = SelfLearningMemory::new();
//!
//!     // Start an episode
//!     let context = TaskContext::default();
//!     let episode_id = memory.start_episode(
//!         "Implement user authentication".to_string(),
//!         context,
//!         TaskType::CodeGeneration,
//!     ).await;
//!
//!     // Log execution steps
//!     let step = ExecutionStep::new(1, "read_file".to_string(), "Read config".to_string());
//!     memory.log_step(episode_id, step).await;
//!
//!     // Complete the episode
//!     let outcome = TaskOutcome::Success {
//!         verdict: "Authentication implemented successfully".to_string(),
//!         artifacts: vec!["auth.rs".to_string()],
//!     };
//!     memory.complete_episode(episode_id, outcome).await.unwrap();
//!
//!     // Retrieve relevant context for future tasks
//!     let relevant = memory.retrieve_relevant_context(
//!         "Add authorization logic".to_string(),
//!         TaskContext::default(),
//!         5,
//!     ).await;
//! }
//! ```

use crate::episode::{Episode, ExecutionStep, PatternId};
use crate::error::{Error, Result};
use crate::extraction::{deduplicate_patterns, rank_patterns, PatternExtractor};
use crate::learning::queue::{PatternExtractionQueue, QueueConfig};
use crate::pattern::Pattern;
use crate::reflection::ReflectionGenerator;
use crate::reward::RewardCalculator;
use crate::storage::StorageBackend;
use crate::types::{MemoryConfig, TaskContext, TaskOutcome, TaskType};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

/// Main self-learning memory system orchestrating the complete learning cycle.
///
/// `SelfLearningMemory` is the primary interface for episodic learning. It manages:
/// - **Episode lifecycle**: Create, track, and complete task executions
/// - **Learning analysis**: Calculate rewards, generate reflections, extract patterns
/// - **Pattern storage**: Persist learnings to durable (Turso) and cache (redb) storage
/// - **Context retrieval**: Find relevant past episodes for new tasks
///
/// # Architecture
///
/// The system uses a dual-storage approach:
/// - **Turso (libSQL)**: Durable, queryable storage for long-term retention
/// - **redb**: Fast embedded cache for hot data and quick lookups
/// - **In-memory**: Fallback when external storage is not configured
///
/// # Learning Cycle
///
/// 1. **Start Episode** - [`start_episode()`](SelfLearningMemory::start_episode) creates a new task record
/// 2. **Log Steps** - [`log_step()`](SelfLearningMemory::log_step) tracks execution steps
/// 3. **Complete** - [`complete_episode()`](SelfLearningMemory::complete_episode) finalizes and analyzes
/// 4. **Retrieve** - [`retrieve_relevant_context()`](SelfLearningMemory::retrieve_relevant_context) queries for similar episodes
///
/// # Examples
///
/// ## Basic Usage (In-Memory)
///
/// ```
/// use memory_core::{SelfLearningMemory, TaskContext, TaskType, TaskOutcome, ExecutionStep, ExecutionResult};
///
/// # async fn example() {
/// let memory = SelfLearningMemory::new();
///
/// // Start tracking a task
/// let episode_id = memory.start_episode(
///     "Implement file parser".to_string(),
///     TaskContext::default(),
///     TaskType::CodeGeneration,
/// ).await;
///
/// // Log execution steps
/// let mut step = ExecutionStep::new(1, "parser".to_string(), "Parse TOML file".to_string());
/// step.result = Some(ExecutionResult::Success {
///     output: "Parsed successfully".to_string(),
/// });
/// memory.log_step(episode_id, step).await;
///
/// // Complete and learn
/// memory.complete_episode(
///     episode_id,
///     TaskOutcome::Success {
///         verdict: "Parser implemented with tests".to_string(),
///         artifacts: vec!["parser.rs".to_string()],
///     },
/// ).await.unwrap();
///
/// // Later: retrieve for similar tasks
/// let relevant = memory.retrieve_relevant_context(
///     "Parse JSON file".to_string(),
///     TaskContext::default(),
///     5,
/// ).await;
/// # }
/// ```
///
/// ## With External Storage
///
/// ```no_run
/// use memory_core::{SelfLearningMemory, MemoryConfig};
/// use std::sync::Arc;
///
/// # async fn example() -> anyhow::Result<()> {
/// # let turso_backend: Arc<dyn memory_core::StorageBackend> = unimplemented!();
/// # let redb_backend: Arc<dyn memory_core::StorageBackend> = unimplemented!();
/// let memory = SelfLearningMemory::with_storage(
///     MemoryConfig::default(),
///     turso_backend,   // Durable storage
///     redb_backend,    // Fast cache
/// );
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct SelfLearningMemory {
    /// Configuration
    #[allow(dead_code)]
    config: MemoryConfig,
    /// Reward calculator
    reward_calculator: RewardCalculator,
    /// Reflection generator
    reflection_generator: ReflectionGenerator,
    /// Pattern extractor
    pattern_extractor: PatternExtractor,
    /// Durable storage backend (Turso)
    turso_storage: Option<Arc<dyn StorageBackend>>,
    /// Cache storage backend (redb)
    cache_storage: Option<Arc<dyn StorageBackend>>,
    /// In-memory fallback for episodes (used when no storage configured)
    episodes_fallback: Arc<RwLock<HashMap<Uuid, Episode>>>,
    /// In-memory fallback for patterns (used when no storage configured)
    patterns_fallback: Arc<RwLock<HashMap<PatternId, Pattern>>>,
    /// Async pattern extraction queue (optional)
    pattern_queue: Option<Arc<PatternExtractionQueue>>,
}

impl Default for SelfLearningMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfLearningMemory {
    /// Create a new self-learning memory system with default configuration (in-memory only)
    pub fn new() -> Self {
        Self::with_config(MemoryConfig::default())
    }

    /// Create a memory system with custom configuration (in-memory only)
    pub fn with_config(config: MemoryConfig) -> Self {
        let pattern_extractor =
            PatternExtractor::with_thresholds(config.pattern_extraction_threshold, 2, 5);

        Self {
            config,
            reward_calculator: RewardCalculator::new(),
            reflection_generator: ReflectionGenerator::new(),
            pattern_extractor,
            turso_storage: None,
            cache_storage: None,
            episodes_fallback: Arc::new(RwLock::new(HashMap::new())),
            patterns_fallback: Arc::new(RwLock::new(HashMap::new())),
            pattern_queue: None,
        }
    }

    /// Create a memory system with storage backends
    ///
    /// # Arguments
    ///
    /// * `config` - Memory configuration
    /// * `turso` - Durable storage backend (typically Turso)
    /// * `cache` - Cache storage backend (typically redb)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use memory_core::{SelfLearningMemory, MemoryConfig};
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// // Assuming turso and cache are already created StorageBackend implementations
    /// # let turso: Arc<dyn memory_core::StorageBackend> = unimplemented!();
    /// # let cache: Arc<dyn memory_core::StorageBackend> = unimplemented!();
    /// let memory = SelfLearningMemory::with_storage(
    ///     MemoryConfig::default(),
    ///     turso,
    ///     cache,
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_storage(
        config: MemoryConfig,
        turso: Arc<dyn StorageBackend>,
        cache: Arc<dyn StorageBackend>,
    ) -> Self {
        let pattern_extractor =
            PatternExtractor::with_thresholds(config.pattern_extraction_threshold, 2, 5);

        Self {
            config,
            reward_calculator: RewardCalculator::new(),
            reflection_generator: ReflectionGenerator::new(),
            pattern_extractor,
            turso_storage: Some(turso),
            cache_storage: Some(cache),
            episodes_fallback: Arc::new(RwLock::new(HashMap::new())),
            patterns_fallback: Arc::new(RwLock::new(HashMap::new())),
            pattern_queue: None,
        }
    }

    /// Enable async pattern extraction with a worker pool
    ///
    /// Sets up the pattern extraction queue and starts worker tasks.
    /// After this is called, `complete_episode` will enqueue episodes
    /// for async pattern extraction instead of processing them synchronously.
    ///
    /// # Arguments
    ///
    /// * `queue_config` - Configuration for the queue and workers
    pub fn enable_async_extraction(mut self, queue_config: QueueConfig) -> Self {
        let memory_arc = Arc::new(self.clone());
        let queue = Arc::new(PatternExtractionQueue::new(queue_config, memory_arc));
        self.pattern_queue = Some(queue);
        self
    }

    /// Start async pattern extraction workers
    ///
    /// Must be called after `enable_async_extraction`.
    /// Spawns worker tasks that process the queue.
    pub async fn start_workers(&self) {
        if let Some(queue) = &self.pattern_queue {
            queue.start_workers().await;
        }
    }

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
    /// # Errors
    ///
    /// Returns [`Error::InvalidInput`] if the task description exceeds [`MAX_DESCRIPTION_LEN`](crate::types::MAX_DESCRIPTION_LEN).
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
    ) -> Result<Uuid> {
        // Validate input to prevent DoS attacks via unbounded inputs
        if task_description.len() > crate::types::MAX_DESCRIPTION_LEN {
            return Err(Error::InvalidInput(format!(
                "Task description too long: {} bytes > {} bytes (MAX_DESCRIPTION_LEN)",
                task_description.len(),
                crate::types::MAX_DESCRIPTION_LEN
            )));
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
        let mut episodes = self.episodes_fallback.write().await;
        episodes.insert(episode_id, episode);

        Ok(episode_id)
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
        let mut episodes = self.episodes_fallback.write().await;

        let episode = episodes
            .get_mut(&episode_id)
            .ok_or(Error::NotFound(episode_id))?;

        // Mark episode as complete
        episode.complete(outcome);

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
    async fn extract_patterns_sync(&self, episode_id: Uuid) -> Result<()> {
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
        let episodes = self.episodes_fallback.read().await;

        debug!(
            total_episodes = episodes.len(),
            limit = limit,
            "Retrieving relevant context"
        );

        // Find relevant episodes
        let mut relevant: Vec<Episode> = episodes
            .values()
            .filter(|e| e.is_complete())
            .filter(|e| self.is_relevant_episode(e, &context, &task_description))
            .cloned()
            .collect();

        // Sort by relevance (using reward as proxy for quality)
        relevant.sort_by(|a, b| {
            let a_score = self.calculate_relevance_score(a, &context, &task_description);
            let b_score = self.calculate_relevance_score(b, &context, &task_description);

            b_score
                .partial_cmp(&a_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit results
        relevant.truncate(limit);

        info!(
            retrieved_count = relevant.len(),
            "Retrieved relevant episodes"
        );

        relevant
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
        let patterns = self.patterns_fallback.read().await;

        debug!(
            total_patterns = patterns.len(),
            limit = limit,
            "Retrieving relevant patterns"
        );

        let all_patterns: Vec<Pattern> = patterns.values().cloned().collect();

        // Rank patterns by relevance and quality
        let mut ranked = rank_patterns(all_patterns, context);

        // Deduplicate
        ranked = deduplicate_patterns(ranked);

        // Limit results
        ranked.truncate(limit);

        info!(
            retrieved_count = ranked.len(),
            "Retrieved relevant patterns"
        );

        ranked
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

    /// Get statistics about the memory system
    ///
    /// # Returns
    ///
    /// Tuple of (total episodes, completed episodes, total patterns)
    pub async fn get_stats(&self) -> (usize, usize, usize) {
        let episodes = self.episodes_fallback.read().await;
        let patterns = self.patterns_fallback.read().await;

        let total_episodes = episodes.len();
        let completed_episodes = episodes.values().filter(|e| e.is_complete()).count();
        let total_patterns = patterns.len();

        (total_episodes, completed_episodes, total_patterns)
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ComplexityLevel, ExecutionResult};

    #[tokio::test]
    async fn test_start_episode() {
        let memory = SelfLearningMemory::new();

        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: Some("tokio".to_string()),
            complexity: ComplexityLevel::Moderate,
            domain: "testing".to_string(),
            tags: vec!["async".to_string()],
        };

        let episode_id = memory
            .start_episode("Test task".to_string(), context.clone(), TaskType::Testing)
            .await;

        // Verify episode was created
        let episode = memory.get_episode(episode_id).await.unwrap();
        assert_eq!(episode.task_description, "Test task");
        assert_eq!(episode.context.domain, "testing");
        assert!(!episode.is_complete());
    }

    #[tokio::test]
    async fn test_log_steps() {
        let memory = SelfLearningMemory::new();

        let episode_id = memory
            .start_episode(
                "Test task".to_string(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;

        // Log some steps
        for i in 0..3 {
            let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i), "Action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            memory.log_step(episode_id, step).await;
        }

        // Verify steps were logged
        let episode = memory.get_episode(episode_id).await.unwrap();
        assert_eq!(episode.steps.len(), 3);
    }

    #[tokio::test]
    async fn test_complete_episode() {
        let memory = SelfLearningMemory::new();

        let episode_id = memory
            .start_episode(
                "Test task".to_string(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;

        // Log a step
        let mut step = ExecutionStep::new(1, "test_tool".to_string(), "Run tests".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "All tests passed".to_string(),
        });
        memory.log_step(episode_id, step).await;

        // Complete the episode
        let outcome = TaskOutcome::Success {
            verdict: "Tests passed".to_string(),
            artifacts: vec!["test_results.json".to_string()],
        };

        memory.complete_episode(episode_id, outcome).await.unwrap();

        // Verify episode was completed and analyzed
        let episode = memory.get_episode(episode_id).await.unwrap();
        assert!(episode.is_complete());
        assert!(episode.reward.is_some());
        assert!(episode.reflection.is_some());

        // Check that patterns were extracted
        let stats = memory.get_stats().await;
        assert!(stats.2 > 0); // Should have some patterns
    }

    #[tokio::test]
    async fn test_retrieve_relevant_context() {
        let memory = SelfLearningMemory::new();

        // Create and complete several episodes
        for i in 0..3 {
            let context = TaskContext {
                language: Some("rust".to_string()),
                framework: None,
                complexity: ComplexityLevel::Moderate,
                domain: "web-api".to_string(),
                tags: vec![],
            };

            let episode_id = memory
                .start_episode(format!("API task {}", i), context, TaskType::CodeGeneration)
                .await;

            let mut step = ExecutionStep::new(1, "builder".to_string(), "Build API".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "Built".to_string(),
            });
            memory.log_step(episode_id, step).await;

            memory
                .complete_episode(
                    episode_id,
                    TaskOutcome::Success {
                        verdict: "API built successfully".to_string(),
                        artifacts: vec![],
                    },
                )
                .await
                .unwrap();
        }

        // Create one episode with different context
        let different_context = TaskContext {
            language: Some("python".to_string()),
            framework: None,
            complexity: ComplexityLevel::Simple,
            domain: "data-science".to_string(),
            tags: vec![],
        };

        let different_id = memory
            .start_episode(
                "Data analysis".to_string(),
                different_context.clone(),
                TaskType::Analysis,
            )
            .await;

        memory
            .complete_episode(
                different_id,
                TaskOutcome::Success {
                    verdict: "Analysis done".to_string(),
                    artifacts: vec![],
                },
            )
            .await
            .unwrap();

        // Retrieve relevant context for web-api task
        let query_context = TaskContext {
            language: Some("rust".to_string()),
            domain: "web-api".to_string(),
            ..Default::default()
        };

        let relevant = memory
            .retrieve_relevant_context("Build REST API".to_string(), query_context, 5)
            .await;

        // Should retrieve the web-api episodes, not the data-science one
        assert!(relevant.len() >= 3);
        assert!(relevant
            .iter()
            .all(|e| e.context.domain == "web-api" || e.task_description.contains("API")));
    }

    #[tokio::test]
    async fn test_retrieve_relevant_patterns() {
        let memory = SelfLearningMemory::new();

        // Create and complete an episode to generate patterns
        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: Some("tokio".to_string()),
            complexity: ComplexityLevel::Moderate,
            domain: "async-processing".to_string(),
            tags: vec!["concurrency".to_string()],
        };

        let episode_id = memory
            .start_episode(
                "Process data concurrently".to_string(),
                context.clone(),
                TaskType::CodeGeneration,
            )
            .await;

        // Add multiple successful steps to generate patterns
        for i in 0..4 {
            let mut step =
                ExecutionStep::new(i + 1, format!("async_tool_{}", i), "Process".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "Processed".to_string(),
            });
            memory.log_step(episode_id, step).await;
        }

        memory
            .complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: "Processing complete".to_string(),
                    artifacts: vec![],
                },
            )
            .await
            .unwrap();

        // Retrieve patterns for similar context
        let patterns = memory.retrieve_relevant_patterns(&context, 10).await;

        assert!(!patterns.is_empty());
    }

    #[tokio::test]
    async fn test_get_stats() {
        let memory = SelfLearningMemory::new();

        // Initially no episodes
        let (total, completed, patterns) = memory.get_stats().await;
        assert_eq!(total, 0);
        assert_eq!(completed, 0);
        assert_eq!(patterns, 0);

        // Create an incomplete episode
        let _ = memory
            .start_episode(
                "Task 1".to_string(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;

        let (total, completed, _) = memory.get_stats().await;
        assert_eq!(total, 1);
        assert_eq!(completed, 0);

        // Complete the episode
        let episode_id = memory
            .start_episode(
                "Task 2".to_string(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;

        memory
            .complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: "Done".to_string(),
                    artifacts: vec![],
                },
            )
            .await
            .unwrap();

        let (total, completed, patterns) = memory.get_stats().await;
        assert_eq!(total, 2);
        assert_eq!(completed, 1);
        assert!(patterns > 0);
    }

    #[tokio::test]
    async fn test_episode_not_found() {
        let memory = SelfLearningMemory::new();

        let fake_id = Uuid::new_v4();
        let result = memory.get_episode(fake_id).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::NotFound(_)));
    }

    #[tokio::test]
    async fn test_complete_nonexistent_episode() {
        let memory = SelfLearningMemory::new();

        let fake_id = Uuid::new_v4();
        let result = memory
            .complete_episode(
                fake_id,
                TaskOutcome::Success {
                    verdict: "Done".to_string(),
                    artifacts: vec![],
                },
            )
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::NotFound(_)));
    }
}
