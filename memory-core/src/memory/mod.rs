//! # Self Learning Memory
//!
//! Main orchestrator for the episodic learning system.
//!
//! Provides a complete learning cycle:
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

mod api;
mod completion;
mod episode;
pub mod filters;
mod init;
mod learning;
mod management;
mod monitoring;
mod pattern_api;
mod pattern_search;
mod queries;
pub mod query_api;
pub mod relationship_query;
mod relationships;
mod retrieval;
pub mod step_buffer;
#[cfg(test)]
mod tests;
pub mod types;
pub mod validation;

use crate::embeddings::SemanticService;
use crate::types::MemoryConfig;
use std::sync::Arc;

// Re-export pattern search types for public API
pub use pattern_search::{PatternSearchResult, ScoreBreakdown, SearchConfig};
pub use types::SelfLearningMemory;

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
        turso: Arc<dyn crate::storage::StorageBackend>,
        cache: Arc<dyn crate::storage::StorageBackend>,
    ) -> Self {
        init::with_storage(config, turso, cache)
    }

    /// Create memory with custom semantic config
    #[must_use]
    pub fn with_semantic_config(
        config: MemoryConfig,
        semantic_config: crate::embeddings::EmbeddingConfig,
    ) -> Self {
        init::with_semantic_config(config, semantic_config)
    }

    /// Enable async pattern extraction with a worker pool
    #[must_use]
    pub fn enable_async_extraction(
        self,
        queue_config: crate::learning::queue::QueueConfig,
    ) -> Self {
        init::enable_async_extraction(self, queue_config)
    }

    /// Start async pattern extraction workers
    pub async fn start_workers(&self) {
        init::start_workers(self).await;
    }

    /// Get a reference to semantic service (if configured)
    #[must_use]
    pub fn semantic_service(&self) -> Option<&Arc<SemanticService>> {
        self.semantic_service.as_ref()
    }

    /// Get a reference to the batch configuration.
    ///
    /// Returns `Some(&BatchConfig)` if step batching is enabled,
    /// or `None` if batching is disabled.
    #[must_use]
    pub fn batch_config(&self) -> Option<&step_buffer::BatchConfig> {
        self.config.batch_config.as_ref()
    }

    /// Get the quality threshold for episode acceptance.
    ///
    /// Episodes with quality scores below this threshold are rejected
    /// during [`complete_episode()`](SelfLearningMemory::complete_episode).
    /// Value is between 0.0 and 1.0.
    #[must_use]
    pub fn quality_threshold(&self) -> f32 {
        self.config.quality_threshold
    }
}
