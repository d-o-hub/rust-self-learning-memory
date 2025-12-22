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
//! Core data structures and types for the self-learning memory system.
//!
//! This crate provides the fundamental building blocks for episodic learning:
//! - Episodes: Complete task execution records
//! - Patterns: Reusable patterns extracted from episodes
//! - Heuristics: Learned condition-action rules
//! - Types: Common types and enums
//! - Memory: Main orchestrator for the learning cycle
//!
//! ## Example - Complete Learning Cycle
//!
//! ```no_run
//! use memory_core::memory::SelfLearningMemory;
//! use memory_core::{TaskContext, TaskType, TaskOutcome, ExecutionStep, ExecutionResult};
//!
//! #[tokio::main]
//! async fn main() {
//!     let memory = SelfLearningMemory::new();
//!
//!     // 1. Start an episode
//!     let context = TaskContext::default();
//!     let episode_id = memory.start_episode(
//!         "Implement authentication".to_string(),
//!         context,
//!         TaskType::CodeGeneration,
//!     ).await;
//!
//!     // 2. Log execution steps
//!     let mut step = ExecutionStep::new(1, "analyzer".to_string(), "Analyze requirements".to_string());
//!     step.result = Some(ExecutionResult::Success { output: "Requirements clear".to_string() });
//!     memory.log_step(episode_id, step).await;
//!
//!     // 3. Complete episode with learning
//!     memory.complete_episode(episode_id, TaskOutcome::Success {
//!         verdict: "Auth implemented successfully".to_string(),
//!         artifacts: vec!["auth.rs".to_string()],
//!     }).await.unwrap();
//!
//!     // 4. Retrieve for future tasks
//!     let relevant = memory.retrieve_relevant_context(
//!         "Add authorization".to_string(),
//!         TaskContext::default(),
//!         5,
//!     ).await;
//! }
//! ```

pub mod embeddings;
pub mod episode;
pub mod error;
pub mod extraction;
pub mod learning;
pub mod memory;
pub mod monitoring;
pub mod pattern;
pub mod patterns;
pub mod reflection;
pub mod reward;
pub mod storage;
pub mod sync;
pub use sync::StorageSynchronizer;
pub mod types;

// Semantic embeddings module (simplified version)
pub mod embeddings_simple;

// Re-export commonly used types
pub use episode::{Episode, ExecutionStep, PatternId};
pub use error::{Error, Result};
pub use extraction::PatternExtractor;
pub use learning::queue::{PatternExtractionQueue, QueueConfig, QueueStats};
pub use memory::step_buffer::BatchConfig;
pub use memory::SelfLearningMemory;
pub use monitoring::{AgentMetrics, AgentMonitor, AgentType, MonitoringConfig, TaskMetrics};
pub use pattern::{Heuristic, Pattern};
pub use patterns::{
    ClusterCentroid, ClusteringConfig, EffectivenessTracker, EpisodeCluster, PatternClusterer,
    PatternMetrics, PatternUsage, PatternValidator, UsageStats, ValidationConfig,
};
pub use reflection::ReflectionGenerator;
pub use reward::RewardCalculator;
pub use storage::StorageBackend;
pub use types::{
    ComplexityLevel, ConcurrencyConfig, Evidence, ExecutionResult, MemoryConfig, OutcomeStats,
    Reflection, RewardScore, StorageConfig, TaskContext, TaskOutcome, TaskType,
};
