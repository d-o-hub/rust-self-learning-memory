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
pub mod filters;
mod init;
mod learning;
mod management;
mod monitoring;
mod pattern_search;
mod queries;
mod retrieval;
pub mod step_buffer;
#[cfg(test)]
mod tests;
pub mod validation;

use crate::embeddings::{EmbeddingConfig, SemanticService};
use crate::episode::{Episode, PatternId};
use crate::extraction::PatternExtractor;
use crate::learning::queue::{PatternExtractionQueue, QueueConfig};
use crate::monitoring::{AgentMetrics, AgentMonitor};
use crate::pattern::{Heuristic, Pattern};
use crate::patterns::extractors::HeuristicExtractor;
use crate::pre_storage::{QualityAssessor, SalientExtractor};
use crate::reflection::ReflectionGenerator;
use crate::reward::RewardCalculator;
use crate::storage::StorageBackend;
use crate::types::{MemoryConfig, TaskContext};
use crate::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use uuid::Uuid;

// Re-export pattern search types for public API
pub use pattern_search::{PatternSearchResult, ScoreBreakdown, SearchConfig};

use step_buffer::StepBuffer;

/// Main self-learning memory system with semantic search capabilities.
///
/// `SelfLearningMemory` is the primary interface for episodic learning. It manages:
/// - **Episode lifecycle**: Create, track, and complete task executions
/// - **Pre-storage reasoning**: Quality assessment and salient feature extraction (`PREMem`)
/// - **Learning analysis**: Calculate rewards, generate reflections, extract patterns
/// - **Pattern storage**: Persist learnings to durable (Turso) and cache (redb) storage
/// - **Context retrieval**: Find relevant past episodes for new tasks using semantic search
/// - **Agent monitoring**: Track agent utilization, performance, and task completion rates
///
/// # Semantic Search
///
/// When available, `SelfLearningMemory` uses semantic embeddings to find
/// contextually relevant episodes:
/// - **Local provider**: Offline, privacy-preserving (default)
/// - **`OpenAI` provider**: Higher accuracy, requires API key
/// - **Fallback**: Automatic fallback chain (Local → `OpenAI` → Mock)
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
/// 5. **Monitor** - [`record_agent_execution()`](SelfLearningMemory::record_agent_execution) tracks agent performance
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
/// // In practice, use storage backends like:
/// // - memory_storage_turso::TursoStorage for durable SQL storage
/// // - memory_storage_redb::RedbStorage for fast key-value cache
/// //
/// // Example setup:
/// // let turso_url = std::env::var("TURSO_URL")?;
/// // let turso_backend = memory_storage_turso::TursoStorage::new(&turso_url).await?;
/// // let redb_backend = memory_storage_redb::RedbStorage::new("cache.redb").await?;
///
/// // For this example, we assume the backends are already configured
/// # let turso_backend: Arc<dyn memory_core::StorageBackend> = todo!("Configure TursoStorage backend");
/// # let redb_backend: Arc<dyn memory_core::StorageBackend> = todo!("Configure RedbStorage backend");
/// let memory = SelfLearningMemory::with_storage(
///     MemoryConfig::default(),
///     turso_backend,   // Durable storage
///     redb_backend,    // Fast cache
/// );
/// # Ok(())
/// # }
/// ```
///
/// ## Agent Monitoring
///
/// ```no_run
/// use memory_core::SelfLearningMemory;
/// use std::time::Instant;
///
/// # async fn example() -> anyhow::Result<()> {
/// let memory = SelfLearningMemory::new();
///
/// // Track agent execution
/// let start = Instant::now();
/// // ... agent work ...
/// let duration = start.elapsed();
///
/// memory.record_agent_execution("feature-implementer", true, duration).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct SelfLearningMemory {
    /// Configuration
    #[allow(dead_code)]
    config: MemoryConfig,
    /// Quality assessor for pre-storage reasoning
    pub(super) quality_assessor: QualityAssessor,
    /// Salient feature extractor for pre-storage reasoning
    pub(super) salient_extractor: SalientExtractor,
    /// Reward calculator
    pub(super) reward_calculator: RewardCalculator,
    /// Reflection generator
    pub(super) reflection_generator: ReflectionGenerator,
    /// Pattern extractor
    pub(super) pattern_extractor: PatternExtractor,
    /// Heuristic extractor
    pub(super) heuristic_extractor: HeuristicExtractor,
    /// Agent monitoring system
    pub(super) agent_monitor: AgentMonitor,
    /// Durable storage backend (Turso)
    pub(super) turso_storage: Option<Arc<dyn StorageBackend>>,
    /// Cache storage backend (redb)
    pub(super) cache_storage: Option<Arc<dyn StorageBackend>>,
    /// In-memory fallback for episodes (used when no storage configured)
    pub(super) episodes_fallback: Arc<RwLock<HashMap<Uuid, Episode>>>,
    /// In-memory fallback for patterns (used when no storage configured)
    pub(super) patterns_fallback: Arc<RwLock<HashMap<PatternId, Pattern>>>,
    /// In-memory fallback for heuristics (used when no storage configured)
    pub(super) heuristics_fallback: Arc<RwLock<HashMap<Uuid, Heuristic>>>,
    /// Async pattern extraction queue (optional)
    pub(super) pattern_queue: Option<Arc<PatternExtractionQueue>>,
    /// Step buffers for batching I/O operations
    pub(super) step_buffers: Arc<RwLock<HashMap<Uuid, StepBuffer>>>,
    /// Semaphore to limit concurrent cache operations and prevent async runtime blocking
    #[allow(dead_code)]
    pub(super) cache_semaphore: Arc<Semaphore>,

    // Phase 2 (GENESIS) - Capacity management
    /// Capacity manager for episodic storage with eviction policies
    pub(super) capacity_manager: Option<crate::episodic::CapacityManager>,

    // Phase 2 (GENESIS) - Semantic summarization
    /// Semantic summarizer for episode compression
    pub(super) semantic_summarizer: Option<crate::semantic::SemanticSummarizer>,

    // Phase 3 (Spatiotemporal) - Hierarchical retrieval and indexing
    /// Spatiotemporal index for domain -> `task_type` -> temporal clustering
    pub(super) spatiotemporal_index:
        Option<Arc<RwLock<crate::spatiotemporal::SpatiotemporalIndex>>>,
    /// Hierarchical retriever for efficient episode search
    pub(super) hierarchical_retriever: Option<crate::spatiotemporal::HierarchicalRetriever>,
    /// Diversity maximizer using MMR for result set optimization
    pub(super) diversity_maximizer: Option<crate::spatiotemporal::DiversityMaximizer>,
    /// Context-aware embeddings for task-specific similarity (future)
    #[allow(dead_code)]
    pub(super) context_aware_embeddings: Option<crate::spatiotemporal::ContextAwareEmbeddings>,

    // Semantic Search Integration
    /// Semantic service for embedding generation and search
    #[allow(dead_code)]
    semantic_service: Option<Arc<SemanticService>>,
    /// Configuration for semantic search
    #[allow(dead_code)]
    semantic_config: EmbeddingConfig,

    // v0.1.12: Query Caching
    /// Query cache for retrieval performance (LRU + TTL)
    query_cache: Arc<crate::retrieval::QueryCache>,
}

impl Default for SelfLearningMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfLearningMemory {
    /// Create a new self-learning memory system with default configuration (in-memory only)
    #[must_use]
    pub fn new() -> Self {
        init::with_config(MemoryConfig::default())
    }

    /// Create a memory system with custom configuration (in-memory only)
    #[must_use]
    pub fn with_config(config: MemoryConfig) -> Self {
        init::with_config(config)
    }

    /// Create a memory system with storage backends
    pub fn with_storage(
        config: MemoryConfig,
        turso: Arc<dyn StorageBackend>,
        cache: Arc<dyn StorageBackend>,
    ) -> Self {
        init::with_storage(config, turso, cache)
    }

    /// Create memory with custom semantic config
    #[must_use]
    pub fn with_semantic_config(config: MemoryConfig, semantic_config: EmbeddingConfig) -> Self {
        init::with_semantic_config(config, semantic_config)
    }

    /// Enable async pattern extraction with a worker pool
    #[must_use]
    pub fn enable_async_extraction(self, queue_config: QueueConfig) -> Self {
        init::enable_async_extraction(self, queue_config)
    }

    /// Start async pattern extraction workers
    pub async fn start_workers(&self) {
        init::start_workers(self).await;
    }

    /// Get statistics about the memory system
    pub async fn get_stats(&self) -> (usize, usize, usize) {
        monitoring::get_stats(&self.episodes_fallback, &self.patterns_fallback).await
    }

    /// Record an agent execution for monitoring
    pub async fn record_agent_execution(
        &self,
        agent_name: &str,
        success: bool,
        duration: std::time::Duration,
    ) -> Result<()> {
        monitoring::record_agent_execution(&self.agent_monitor, agent_name, success, duration).await
    }

    /// Record detailed agent execution information
    pub async fn record_agent_execution_detailed(
        &self,
        agent_name: &str,
        success: bool,
        duration: std::time::Duration,
        task_description: Option<String>,
        error_message: Option<String>,
    ) -> Result<()> {
        monitoring::record_agent_execution_detailed(
            &self.agent_monitor,
            agent_name,
            success,
            duration,
            task_description,
            error_message,
        )
        .await
    }

    /// Get performance metrics for a specific agent
    pub async fn get_agent_metrics(&self, agent_name: &str) -> Option<AgentMetrics> {
        monitoring::get_agent_metrics(&self.agent_monitor, agent_name).await
    }

    /// Get metrics for all tracked agents
    pub async fn get_all_agent_metrics(&self) -> std::collections::HashMap<String, AgentMetrics> {
        monitoring::get_all_agent_metrics(&self.agent_monitor).await
    }

    /// Get monitoring system summary statistics
    pub async fn get_monitoring_summary(&self) -> crate::monitoring::MonitoringSummary {
        monitoring::get_monitoring_summary(&self.agent_monitor).await
    }

    /// Get query cache metrics (v0.1.12)
    #[must_use]
    pub fn get_cache_metrics(&self) -> crate::retrieval::CacheMetrics {
        monitoring::get_cache_metrics(&self.query_cache)
    }

    /// Clear query cache metrics (v0.1.12)
    pub fn clear_cache_metrics(&self) {
        monitoring::clear_cache_metrics(&self.query_cache);
    }

    /// Clear all cached query results (v0.1.12)
    pub fn clear_cache(&self) {
        monitoring::clear_cache(&self.query_cache);
    }

    /// Check if Turso storage is configured
    #[must_use]
    pub fn has_turso_storage(&self) -> bool {
        queries::has_turso_storage(&self.turso_storage)
    }

    /// Check if cache storage is configured
    #[must_use]
    pub fn has_cache_storage(&self) -> bool {
        queries::has_cache_storage(&self.cache_storage)
    }

    /// Get a reference to the Turso storage backend (if configured)
    #[must_use]
    pub fn turso_storage(&self) -> Option<&Arc<dyn StorageBackend>> {
        queries::turso_storage(&self.turso_storage)
    }

    /// Get a reference to the cache storage backend (if configured)
    #[must_use]
    pub fn cache_storage(&self) -> Option<&Arc<dyn StorageBackend>> {
        queries::cache_storage(&self.cache_storage)
    }

    /// Get a reference to the semantic service (if configured)
    #[must_use]
    pub fn semantic_service(&self) -> Option<&Arc<SemanticService>> {
        self.semantic_service.as_ref()
    }

    /// Get all episodes with proper lazy loading from storage backends.
    pub async fn get_all_episodes(&self) -> Result<Vec<Episode>> {
        queries::get_all_episodes(
            &self.episodes_fallback,
            self.cache_storage.as_ref(),
            self.turso_storage.as_ref(),
        )
        .await
    }

    /// Get all patterns with proper lazy loading from storage backends.
    pub async fn get_all_patterns(&self) -> Result<Vec<Pattern>> {
        queries::get_all_patterns(&self.patterns_fallback).await
    }

    /// List episodes with optional filtering, using proper lazy loading.
    pub async fn list_episodes(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
        completed_only: Option<bool>,
    ) -> Result<Vec<Episode>> {
        queries::list_episodes(
            &self.episodes_fallback,
            self.cache_storage.as_ref(),
            self.turso_storage.as_ref(),
            limit,
            offset,
            completed_only,
        )
        .await
    }

    /// List episodes with advanced filtering support.
    ///
    /// Provides rich filtering capabilities including tags, date ranges,
    /// task types, outcomes, and more. Use `EpisodeFilter::builder()` for a fluent API.
    ///
    /// # Arguments
    ///
    /// * `filter` - Episode filter criteria
    /// * `limit` - Maximum number of episodes to return
    /// * `offset` - Number of episodes to skip (for pagination)
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::{SelfLearningMemory, EpisodeFilter, TaskType};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let memory = SelfLearningMemory::new();
    ///
    /// // Get successful episodes with specific tags
    /// let filter = EpisodeFilter::builder()
    ///     .with_any_tags(vec!["async".to_string()])
    ///     .success_only(true)
    ///     .build();
    ///
    /// let episodes = memory.list_episodes_filtered(filter, Some(10), None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_episodes_filtered(
        &self,
        filter: filters::EpisodeFilter,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<Episode>> {
        queries::list_episodes_filtered(
            &self.episodes_fallback,
            self.cache_storage.as_ref(),
            self.turso_storage.as_ref(),
            filter,
            limit,
            offset,
        )
        .await
    }

    /// Get patterns extracted from a specific episode
    #[allow(clippy::unused_async)]
    pub async fn get_episode_patterns(&self, episode_id: Uuid) -> Result<Vec<Pattern>> {
        queries::get_episode_patterns(episode_id, &self.patterns_fallback).await
    }

    /// Search for patterns semantically similar to a query
    ///
    /// Uses multi-signal ranking combining:
    /// - Semantic similarity via embeddings
    /// - Context matching (domain, task type, tags)
    /// - Pattern effectiveness from past usage
    /// - Recency (recently used patterns score higher)
    /// - Success rate
    ///
    /// # Arguments
    /// * `query` - Natural language description of what you're looking for
    /// * `context` - Task context for filtering and scoring
    /// * `limit` - Maximum number of results to return
    ///
    /// # Example
    /// ```no_run
    /// use memory_core::{SelfLearningMemory, TaskContext};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let memory = SelfLearningMemory::new();
    /// let context = TaskContext {
    ///     language: Some("rust".to_string()),
    ///     domain: "web-api".to_string(),
    ///     tags: vec!["rest".to_string()],
    ///     ..Default::default()
    /// };
    ///
    /// let results = memory.search_patterns_semantic(
    ///     "How to handle API rate limiting with retries",
    ///     context,
    ///     5,
    /// ).await?;
    ///
    /// for result in results {
    ///     println!("Pattern: {:?}", result.pattern);
    ///     println!("Relevance: {:.2}", result.relevance_score);
    ///     println!("Breakdown: {:?}", result.score_breakdown);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search_patterns_semantic(
        &self,
        query: &str,
        context: TaskContext,
        limit: usize,
    ) -> Result<Vec<pattern_search::PatternSearchResult>> {
        let patterns = self.get_all_patterns().await?;
        pattern_search::search_patterns_semantic(
            query,
            patterns,
            &context,
            self.semantic_service.as_ref(),
            pattern_search::SearchConfig::default(),
            limit,
        )
        .await
    }

    /// Search patterns with custom configuration
    pub async fn search_patterns_with_config(
        &self,
        query: &str,
        context: TaskContext,
        config: pattern_search::SearchConfig,
        limit: usize,
    ) -> Result<Vec<pattern_search::PatternSearchResult>> {
        let patterns = self.get_all_patterns().await?;
        pattern_search::search_patterns_semantic(
            query,
            patterns,
            &context,
            self.semantic_service.as_ref(),
            config,
            limit,
        )
        .await
    }

    /// Recommend patterns for a specific task
    ///
    /// Similar to search but uses stricter filtering and emphasizes
    /// effectiveness and context match for high-quality recommendations.
    ///
    /// # Arguments
    /// * `task_description` - Description of the task you're working on
    /// * `context` - Task context
    /// * `limit` - Maximum number of recommendations
    ///
    /// # Example
    /// ```no_run
    /// use memory_core::{SelfLearningMemory, TaskContext};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let memory = SelfLearningMemory::new();
    /// let context = TaskContext {
    ///     language: Some("rust".to_string()),
    ///     domain: "web-api".to_string(),
    ///     tags: vec!["async".to_string()],
    ///     ..Default::default()
    /// };
    ///
    /// let recommendations = memory.recommend_patterns_for_task(
    ///     "Build an async HTTP client with connection pooling",
    ///     context,
    ///     3,
    /// ).await?;
    ///
    /// for rec in recommendations {
    ///     println!("Recommended: {:?}", rec.pattern);
    ///     println!("Relevance: {:.2}", rec.relevance_score);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn recommend_patterns_for_task(
        &self,
        task_description: &str,
        context: TaskContext,
        limit: usize,
    ) -> Result<Vec<pattern_search::PatternSearchResult>> {
        let patterns = self.get_all_patterns().await?;
        pattern_search::recommend_patterns_for_task(
            task_description,
            context,
            patterns,
            self.semantic_service.as_ref(),
            limit,
        )
        .await
    }

    /// Discover patterns from one domain that might apply to another
    ///
    /// Finds analogous patterns across domains, useful for:
    /// - Learning from similar problems in different contexts
    /// - Cross-pollinating ideas between domains
    /// - Finding universal patterns
    ///
    /// # Arguments
    /// * `source_domain` - Domain to learn from (e.g., "cli")
    /// * `target_context` - Context to apply patterns to (e.g., "web-api")
    /// * `limit` - Maximum number of patterns to discover
    ///
    /// # Example
    /// ```no_run
    /// use memory_core::{SelfLearningMemory, TaskContext};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let memory = SelfLearningMemory::new();
    /// let target = TaskContext {
    ///     language: Some("rust".to_string()),
    ///     domain: "web-api".to_string(),
    ///     tags: vec![],
    ///     ..Default::default()
    /// };
    ///
    /// // Find patterns from CLI work that might apply to web APIs
    /// let analogous = memory.discover_analogous_patterns(
    ///     "cli",
    ///     target,
    ///     5,
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn discover_analogous_patterns(
        &self,
        source_domain: &str,
        target_context: TaskContext,
        limit: usize,
    ) -> Result<Vec<pattern_search::PatternSearchResult>> {
        let patterns = self.get_all_patterns().await?;
        pattern_search::discover_analogous_patterns(
            source_domain,
            target_context,
            patterns,
            self.semantic_service.as_ref(),
            limit,
        )
        .await
    }
}
