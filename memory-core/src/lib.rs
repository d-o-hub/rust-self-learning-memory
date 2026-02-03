#![allow(clippy::cast_precision_loss)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::unused_self)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::assigning_clones)]
#![allow(clippy::borrowed_box)]
#![allow(clippy::float_cmp)]
#![allow(clippy::ref_option)]

//! # Memory Core
//!
//! Core data structures and types for the self-learning memory system with episodic learning.
//!
//! This crate provides the fundamental building blocks for AI agents to learn from execution:
//!
//! ## Core Concepts
//!
//! - **Episodes**: Complete task execution records with steps, outcomes, and metadata
//! - **Patterns**: Reusable patterns extracted from episodes using statistical analysis
//! - **Heuristics**: Learned condition-action rules for future decision-making
//! - **Reflections**: Generated insights after episode completion
//! - **Rewards**: Quantitative scoring of episode success
//!
//! ## Module Organization
//!
//! ### Primary APIs
//! - [`memory`]: Main orchestrator for the learning cycle
//! - [`episode`]: Episode creation, logging, and management
//! - [`patterns`]: Pattern extraction, clustering, and validation
//! - [`embeddings`]: Semantic embedding generation and similarity search
//!
//! ### Support Modules
//! - [`types`]: Common types used across the system
//! - [`storage`]: Storage backend abstractions
//! - [`retrieval`]: Memory retrieval and caching
//! - [`search`]: Search and ranking functionality
//! - [`security`]: Audit logging and security
//! - [`monitoring`]: Agent performance monitoring
//!
//! ## Quick Start
//!
//! ### Basic Episode Recording
//!
//! ```no_run
//! use memory_core::memory::SelfLearningMemory;
//! use memory_core::{TaskContext, TaskType, TaskOutcome, ExecutionStep, ExecutionResult};
//!
//! #[tokio::main]
//! async fn main() {
//!     let memory = SelfLearningMemory::new();
//!
//!     // 1. Start an episode for a task
//!     let context = TaskContext::default();
//!     let episode_id = memory.start_episode(
//!         "Implement authentication".to_string(),
//!         context,
//!         TaskType::CodeGeneration,
//!     ).await;
//!
//!     // 2. Log execution steps
//!     let mut step = ExecutionStep::new(
//!         1,
//!         "analyzer".to_string(),
//!         "Analyze requirements".to_string()
//!     );
//!     step.result = Some(ExecutionResult::Success {
//!         output: "Requirements clear".to_string()
//!     });
//!     memory.log_step(episode_id, step).await;
//!
//!     // 3. Complete episode with learning
//!     memory.complete_episode(episode_id, TaskOutcome::Success {
//!         verdict: "Auth implemented successfully".to_string(),
//!         artifacts: vec!["auth.rs".to_string()],
//!     }).await.unwrap();
//!
//!     // 4. Retrieve relevant context for future tasks
//!     let relevant = memory.retrieve_relevant_context(
//!         "Add authorization".to_string(),
//!         TaskContext::default(),
//!         5,
//!     ).await;
//!
//!     println!("Found {} relevant episodes", relevant.len());
//! }
//! ```
//!
//! ### Pattern-Based Learning
//!
//! ```no_run
//! use memory_core::memory::SelfLearningMemory;
//! use memory_core::patterns::PatternExtractor;
//! use memory_core::patterns::DBSCANConfig;
//!
//! # #[tokio::main]
//! # async fn main() {
//! # let memory = SelfLearningMemory::new();
//! // Configure pattern extraction with DBSCAN clustering
//! let config = DBSCANConfig::default();
//! let extractor = PatternExtractor::new(config);
//!
//! // Extract patterns from completed episodes
//! let patterns = extractor.extract_patterns(&memory).await.unwrap();
//!
//! for pattern in &patterns {
//!     println!("Pattern: {} (effectiveness: {:.2})",
//!              pattern.description,
//!              pattern.effectiveness.score());
//! }
//! # }
//! ```
//!
//! ### Semantic Search with Embeddings
//!
//! ```no_run
//! use memory_core::embeddings::{SemanticService, EmbeddingConfig};
//! use memory_core::episode::Episode;
//!
//! # #[tokio::main]
//! # async fn main() {
//! # let config = EmbeddingConfig::default();
//! // Use semantic similarity to find related episodes
//! # use memory_core::embeddings::storage::InMemoryEmbeddingStorage;
//! # let storage = Box::new(InMemoryEmbeddingStorage::new());
//! let semantic = SemanticService::default(storage).await.unwrap();
//!
//! let related_episodes = semantic
//!     .find_similar_episodes(
//!         "fix authentication bug",
//!         &TaskContext::default(),
//!         10
//!     )
//!     .await
//!     .unwrap();
//! # }
//! ```
//!
//! ## Learning Cycle
//!
//! 1. **Start Episode**: Create a new episode with task context
//! 2. **Log Steps**: Record each execution step with outcomes
//! 3. **Complete Episode**: Mark episode as success/failure
//! 4. **Extract Patterns**: Identify reusable patterns
//! 5. **Generate Reflection**: Create insights and lessons learned
//! 6. **Calculate Reward**: Score episode success
//! 7. **Retrieve**: Use learned patterns for future tasks
//!
//! ## Error Handling
//!
//! Most functions return [`Result<T>`] for proper error handling:
//!
//! ```no_run
//! use memory_core::{Error, Result};
//!
//! async fn example() -> Result<()> {
//!     // Operations that can fail
//!     // .await?
//!     Ok(())
//! }
//! ```
//!
//! ## Feature Flags
//!
//! - `openai`: Enable OpenAI embeddings
//! - `local-embeddings`: Enable local ONNX-based embeddings
//! - `turso`: Enable Turso/libSQL storage backend
//! - `redb`: Enable redb cache storage
//! - `full`: Enable all features
//!

pub mod constants;
pub mod embeddings;
pub mod episode;
pub mod episodic;
pub mod error;
pub mod extraction;
pub mod indexing;
pub mod learning;
pub mod memory;
pub mod monitoring;
pub mod pattern;
pub mod patterns;
pub mod pre_storage;
pub mod reflection;
pub mod retrieval;
pub mod reward;
pub mod search;
pub mod security;
pub mod semantic;
pub mod spatiotemporal;
pub mod storage;
pub mod sync;
pub use sync::StorageSynchronizer;
pub mod types;

// Semantic embeddings module (simplified version)
pub mod embeddings_simple;

// Re-export commonly used types
pub use episode::{Episode, ExecutionStep, PatternId};
pub use episodic::{CapacityManager, EvictionPolicy};
pub use error::{CacheError, Error, RelationshipError, Result};
pub use extraction::PatternExtractor;
pub use indexing::{
    hierarchical::{HierarchicalIndex, HierarchicalIndexStats, HierarchicalQuery},
    spatiotemporal::{IndexStats, QueryOptions, SpatiotemporalIndex, TimeBucket},
    BenchmarkResult, IndexMetrics, IndexableMemory, QueryPerformance,
};
pub use learning::queue::{PatternExtractionQueue, QueueConfig, QueueStats};
pub use memory::filters::{EpisodeFilter, EpisodeFilterBuilder, OutcomeType};
pub use memory::step_buffer::BatchConfig;
pub use memory::SelfLearningMemory;
pub use monitoring::{AgentMetrics, AgentMonitor, AgentType, MonitoringConfig, TaskMetrics};
pub use pattern::{Heuristic, Pattern, PatternEffectiveness};
pub use patterns::{
    Anomaly, AnomalyReason, ChangeDirection, ChangeType, Changepoint, ChangepointConfig,
    ChangepointDetector, ClusterCentroid, ClusteringConfig, DBSCANAnomalyDetector,
    DBSCANClusterResult, DBSCANConfig, DBSCANStats, EffectivenessTracker, EpisodeCluster,
    FeatureWeights, PatternClusterer, PatternMetrics, PatternUsage, PatternValidator,
    SegmentComparison, SegmentComparisonConfig, SegmentStats, UsageStats, ValidationConfig,
};
pub use reflection::ReflectionGenerator;
pub use retrieval::{CacheKey, CacheMetrics, QueryCache};
pub use reward::{
    AdaptiveRewardCalculator, DomainStatistics, DomainStatisticsCache, RewardCalculator,
};
pub use security::audit::{
    ActorType, AuditConfig, AuditContext, AuditEntry, AuditEventType, AuditLogLevel, AuditLogger,
    AuditOutput, AuditResult,
};
pub use storage::StorageBackend;
pub use types::{
    ComplexityLevel, ConcurrencyConfig, Evidence, ExecutionResult, MemoryConfig, OutcomeStats,
    Reflection, RewardScore, StorageConfig, TaskContext, TaskOutcome, TaskType,
};
