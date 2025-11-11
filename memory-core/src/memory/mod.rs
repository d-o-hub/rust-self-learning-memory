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

mod episode;
mod learning;
mod retrieval;
pub mod validation;

use crate::episode::{Episode, PatternId};
use crate::extraction::PatternExtractor;
use crate::learning::queue::{PatternExtractionQueue, QueueConfig};
use crate::pattern::Pattern;
use crate::reflection::ReflectionGenerator;
use crate::reward::RewardCalculator;
use crate::storage::StorageBackend;
use crate::types::MemoryConfig;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
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
    pub(super) reward_calculator: RewardCalculator,
    /// Reflection generator
    pub(super) reflection_generator: ReflectionGenerator,
    /// Pattern extractor
    pub(super) pattern_extractor: PatternExtractor,
    /// Durable storage backend (Turso)
    pub(super) turso_storage: Option<Arc<dyn StorageBackend>>,
    /// Cache storage backend (redb)
    pub(super) cache_storage: Option<Arc<dyn StorageBackend>>,
    /// In-memory fallback for episodes (used when no storage configured)
    pub(super) episodes_fallback: Arc<RwLock<HashMap<Uuid, Episode>>>,
    /// In-memory fallback for patterns (used when no storage configured)
    pub(super) patterns_fallback: Arc<RwLock<HashMap<PatternId, Pattern>>>,
    /// Async pattern extraction queue (optional)
    pub(super) pattern_queue: Option<Arc<PatternExtractionQueue>>,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episode::ExecutionStep;
    use crate::types::{ComplexityLevel, ExecutionResult, TaskContext, TaskOutcome, TaskType};

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
        assert!(matches!(
            result.unwrap_err(),
            crate::error::Error::NotFound(_)
        ));
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
        assert!(matches!(
            result.unwrap_err(),
            crate::error::Error::NotFound(_)
        ));
    }
}
