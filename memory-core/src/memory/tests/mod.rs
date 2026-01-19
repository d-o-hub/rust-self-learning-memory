//! Tests for `SelfLearningMemory`
//!
//! This module contains integration tests for the memory system.

pub mod episode_tests;
pub mod lazy_loading_tests;
pub mod retrieval_tests;
pub mod semantic_tests;

pub use episode_tests::{test_complete_episode, test_log_steps, test_start_episode};
pub use lazy_loading_tests::{test_get_all_episodes_lazy_loading, test_get_episode_lazy_loading};
pub use retrieval_tests::{test_retrieve_relevant_context, test_retrieve_relevant_patterns};
pub use semantic_tests::{
    test_embedding_generation_on_completion, test_semantic_fallback_to_keyword,
    test_semantic_service_initialization, test_with_semantic_config,
};
