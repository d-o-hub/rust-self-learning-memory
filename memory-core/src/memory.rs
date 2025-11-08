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
use crate::types::{MemoryConfig, TaskContext, TaskOutcome, TaskType};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

/// In-memory storage for episodes (will be replaced with actual storage later)
type EpisodeStore = Arc<RwLock<HashMap<Uuid, Episode>>>;

/// In-memory storage for patterns (will be replaced with actual storage later)
type PatternStore = Arc<RwLock<HashMap<PatternId, Pattern>>>;

/// Main self-learning memory system
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
    /// Episodes storage (in-memory for now)
    episodes: EpisodeStore,
    /// Patterns storage (in-memory for now)
    patterns: PatternStore,
    /// Async pattern extraction queue (optional)
    pattern_queue: Option<Arc<PatternExtractionQueue>>,
}

impl Default for SelfLearningMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfLearningMemory {
    /// Create a new self-learning memory system with default configuration
    pub fn new() -> Self {
        Self::with_config(MemoryConfig::default())
    }

    /// Create a memory system with custom configuration
    pub fn with_config(config: MemoryConfig) -> Self {
        let pattern_extractor =
            PatternExtractor::with_thresholds(config.pattern_extraction_threshold, 2, 5);

        Self {
            config,
            reward_calculator: RewardCalculator::new(),
            reflection_generator: ReflectionGenerator::new(),
            pattern_extractor,
            episodes: Arc::new(RwLock::new(HashMap::new())),
            patterns: Arc::new(RwLock::new(HashMap::new())),
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

    /// Start a new episode
    ///
    /// Creates and stores a new episode for task tracking.
    ///
    /// # Arguments
    ///
    /// * `task_description` - Human-readable description of the task
    /// * `context` - Task context including language, domain, complexity, tags
    /// * `task_type` - Type of task being performed
    ///
    /// # Returns
    ///
    /// The unique episode ID for this task
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

        let mut episodes = self.episodes.write().await;
        episodes.insert(episode_id, episode);

        episode_id
    }

    /// Log an execution step for an episode
    ///
    /// Records a single step in the task execution. Steps should be logged
    /// as they occur to maintain accurate timing and sequence information.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - ID of the episode to add the step to
    /// * `step` - Execution step details
    ///
    /// # Errors
    ///
    /// Returns `Error::NotFound` if the episode doesn't exist
    #[instrument(skip(self, step), fields(episode_id = %episode_id, step_number = step.step_number))]
    pub async fn log_step(&self, episode_id: Uuid, step: ExecutionStep) {
        let mut episodes = self.episodes.write().await;

        if let Some(episode) = episodes.get_mut(&episode_id) {
            debug!(
                step_number = step.step_number,
                tool = %step.tool,
                "Logged execution step"
            );
            episode.add_step(step);
        } else {
            warn!("Attempted to log step for non-existent episode");
        }
    }

    /// Complete an episode and perform learning analysis
    ///
    /// Finalizes the episode, calculates rewards, generates reflections,
    /// and extracts reusable patterns. This is where the learning happens.
    ///
    /// If async pattern extraction is enabled, patterns are extracted
    /// asynchronously in the background. Otherwise, patterns are extracted
    /// synchronously before returning.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - ID of the episode to complete
    /// * `outcome` - Final outcome of the task
    ///
    /// # Errors
    ///
    /// Returns `Error::NotFound` if the episode doesn't exist
    #[instrument(skip(self, outcome), fields(episode_id = %episode_id))]
    pub async fn complete_episode(&self, episode_id: Uuid, outcome: TaskOutcome) -> Result<()> {
        let mut episodes = self.episodes.write().await;

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
        let mut episodes = self.episodes.write().await;
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
        let mut patterns = self.patterns.write().await;
        let mut pattern_ids = Vec::new();

        for pattern in extracted_patterns {
            let pattern_id = pattern.id();
            pattern_ids.push(pattern_id);
            patterns.insert(pattern_id, pattern);
        }

        episode.patterns = pattern_ids;

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
        let mut episodes = self.episodes.write().await;
        let episode = episodes
            .get_mut(&episode_id)
            .ok_or(Error::NotFound(episode_id))?;

        let mut patterns = self.patterns.write().await;
        let mut pattern_ids = Vec::new();

        for pattern in extracted_patterns {
            let pattern_id = pattern.id();
            pattern_ids.push(pattern_id);
            patterns.insert(pattern_id, pattern);
        }

        episode.patterns = pattern_ids;

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

    /// Retrieve relevant context for a new task
    ///
    /// Searches for similar episodes and patterns that can inform
    /// the execution of a new task.
    ///
    /// # Arguments
    ///
    /// * `task_description` - Description of the new task
    /// * `context` - Context of the new task
    /// * `limit` - Maximum number of results to return
    ///
    /// # Returns
    ///
    /// Vector of relevant episodes, sorted by relevance
    #[instrument(skip(self))]
    pub async fn retrieve_relevant_context(
        &self,
        task_description: String,
        context: TaskContext,
        limit: usize,
    ) -> Vec<Episode> {
        let episodes = self.episodes.read().await;

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
        let patterns = self.patterns.read().await;

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
        let episodes = self.episodes.read().await;
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
        let episodes = self.episodes.read().await;
        let patterns = self.patterns.read().await;

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
