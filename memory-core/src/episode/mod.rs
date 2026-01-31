//! Episode management for tracking task execution.
//!
//! This module provides the core types for tracking and learning from task
//! executions, including episodes, execution steps, and pattern applications.
//!
//! # Examples
//!
//! ```
//! use memory_core::{Episode, TaskContext, TaskType, TaskOutcome, ExecutionStep};
//! use memory_core::ExecutionResult;
//!
//! // Create a new episode
//! let mut episode = Episode::new(
//!     "Implement user authentication".to_string(),
//!     TaskContext::default(),
//!     TaskType::CodeGeneration,
//! );
//!
//! // Add execution steps
//! let mut step = ExecutionStep::new(1, "planner".to_string(), "Plan implementation".to_string());
//! step.result = Some(ExecutionResult::Success {
//!     output: "Plan created".to_string(),
//! });
//! episode.add_step(step);
//!
//! // Complete the episode
//! episode.complete(TaskOutcome::Success {
//!     verdict: "Authentication implemented successfully".to_string(),
//!     artifacts: vec!["auth.rs".to_string()],
//! });
//!
//! assert!(episode.is_complete());
//! ```

pub mod graph_algorithms;
pub mod relationship_errors;
pub mod relationship_manager;
pub mod relationships;
pub mod structs;
pub mod validation;

pub use graph_algorithms::{
    find_all_cycles_from_node, find_path_dfs, get_ancestors, get_transitive_closure, has_cycle,
    has_path_dfs, topological_sort,
};
pub use relationship_errors::{GraphError, RemovalError, ValidationError};
pub use relationship_manager::RelationshipManager;
pub use relationships::{Direction, EpisodeRelationship, RelationshipMetadata, RelationshipType};
pub use structs::{ApplicationOutcome, Episode, ExecutionStep, PatternApplication, PatternId};
