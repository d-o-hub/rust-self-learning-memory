//!
//! Integration tests for Phase 3 Spatiotemporal Memory Organization
//!
//! Validates end-to-end hierarchical retrieval, diversity maximization,
//! temporal bias, and performance characteristics.
//!
//! ## Test Coverage
//! - Hierarchical retrieval by domain and task type
//! - Temporal bias (recent episodes ranked higher)
//! - Diversity maximization (MMR algorithm)
//! - Query latency (<=100ms target)
//! - Index synchronization
//! - Large-scale retrieval (1000+ episodes)
//! - Backward compatibility
//!

use memory_core::memory::SelfLearningMemory;
use memory_core::spatiotemporal::{DiversityMaximizer, ScoredEpisode};
use memory_core::{
    ComplexityLevel, ExecutionResult, ExecutionStep, MemoryConfig, TaskContext, TaskOutcome, TaskType,
};
use std::collections::HashSet;
use std::time::Instant;
use uuid::Uuid;

mod hierarchical_retrieval;
mod diversity_maximization;

use hierarchical_retrieval::{
    test_end_to_end_hierarchical_retrieval,
    test_hierarchical_retrieval_by_domain,
    test_hierarchical_retrieval_by_task_type,
    test_temporal_bias_recent_episodes_ranked_higher,
    test_query_latency_under_100ms,
    test_index_synchronization_on_storage,
    test_backward_compatibility_flat_retrieval,
    test_combined_filtering_domain_and_task_type,
    test_large_scale_retrieval_1000_episodes,
};

use diversity_maximization::{
    test_diversity_reduces_redundancy,
    test_diversity_score_calculation,
    test_diversity_lambda_parameter,
    test_diversity_disabled_fallback,
    test_diversity_improves_result_quality,
};

/// Helper: Create a test episode with specific attributes
async fn create_test_episode(
    memory: &SelfLearningMemory,
    domain: &str,
    task_type: TaskType,
    description: &str,
    num_steps: usize,
) -> Uuid {
    let context = TaskContext {
        domain: domain.to_string(),
        language: Some("rust".to_string()),
        framework: Some("tokio".to_string()),
        complexity: ComplexityLevel::Moderate,
        tags: vec!["test".to_string()],
    };

    let episode_id = memory
        .start_episode(description.to_string(), context, task_type)
        .await;

    // Log execution steps
    for i in 0..num_steps {
        let mut step = ExecutionStep::new(
            i + 1,
            format!("tool_{}", i % 5),
            format!("Step {i} for {description}"),
        );
        step.result = Some(ExecutionResult::Success {
            output: format!("Output for step {i}"),
        });
        memory.log_step(episode_id, step).await;
    }

    // Complete episode
    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: format!("{description} completed successfully"),
                artifacts: vec![format!("{}.rs", domain)],
            },
        )
        .await
        .unwrap();

    episode_id
}

// Re-export tests from submodules
pub use hierarchical_retrieval::test_end_to_end_hierarchical_retrieval;
pub use hierarchical_retrieval::test_hierarchical_retrieval_by_domain;
pub use hierarchical_retrieval::test_hierarchical_retrieval_by_task_type;
pub use hierarchical_retrieval::test_temporal_bias_recent_episodes_ranked_higher;
pub use hierarchical_retrieval::test_query_latency_under_100ms;
pub use hierarchical_retrieval::test_index_synchronization_on_storage;
pub use hierarchical_retrieval::test_backward_compatibility_flat_retrieval;
pub use hierarchical_retrieval::test_combined_filtering_domain_and_task_type;
pub use hierarchical_retrieval::test_large_scale_retrieval_1000_episodes;
pub use diversity_maximization::test_diversity_reduces_redundancy;
pub use diversity_maximization::test_diversity_score_calculation;
pub use diversity_maximization::test_diversity_lambda_parameter;
pub use diversity_maximization::test_diversity_disabled_fallback;
pub use diversity_maximization::test_diversity_improves_result_quality;
