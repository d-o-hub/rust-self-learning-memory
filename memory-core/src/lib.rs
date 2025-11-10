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

pub mod episode;
pub mod error;
pub mod extraction;
pub mod learning;
pub mod memory;
pub mod pattern;
pub mod patterns;
pub mod reflection;
pub mod reward;
pub mod storage;
pub mod sync;
pub mod types;

// Re-export commonly used types
pub use episode::{Episode, ExecutionStep, PatternId};
pub use error::{Error, Result};
pub use extraction::PatternExtractor;
pub use learning::queue::{PatternExtractionQueue, QueueConfig, QueueStats};
pub use memory::SelfLearningMemory;
pub use pattern::{Heuristic, Pattern};
pub use patterns::{
    ClusterCentroid, ClusteringConfig, EffectivenessTracker, EpisodeCluster, PatternClusterer,
    PatternMetrics, PatternUsage, PatternValidator, UsageStats, ValidationConfig,
};
pub use reflection::ReflectionGenerator;
pub use reward::RewardCalculator;
pub use storage::StorageBackend;
pub use types::{
    ComplexityLevel, Evidence, ExecutionResult, MemoryConfig, OutcomeStats, Reflection,
    RewardScore, StorageConfig, TaskContext, TaskOutcome, TaskType,
};
