// ============================================================================
// Type Definitions
//
// This module has been split into submodules for better organization.
// All types are re-exported here for backward compatibility.
// ============================================================================

// Re-export all types from submodules
pub mod config;
pub mod constants;
pub mod enums;
pub mod structs;

pub use config::{ConcurrencyConfig, MemoryConfig, StorageConfig};
pub use constants::{
    MAX_ARTIFACT_SIZE, MAX_DESCRIPTION_LEN, MAX_EPISODE_SIZE, MAX_OBSERVATION_LEN, MAX_STEP_COUNT,
};
pub use enums::{ComplexityLevel, ExecutionResult, TaskOutcome, TaskType};
pub use structs::{Evidence, OutcomeStats, Reflection, RewardScore, TaskContext};

pub use crate::memory::step_buffer::BatchConfig;

#[cfg(test)]
mod tests;
