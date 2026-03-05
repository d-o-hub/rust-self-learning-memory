//! Property-based tests for episode lifecycle and capacity management
//!
//! These tests use proptest to verify state machine invariants for the
//! episode lifecycle (creation → steps → completion) and capacity management
//! (can_store, eviction, relevance scoring).

use memory_core::episodic::{CapacityManager, EvictionPolicy};
use memory_core::*;
use proptest::prelude::*;

// ============================================================================
// Episode Lifecycle State Machine Properties
// ============================================================================

proptest! {
    /// New episodes are always in the initial (incomplete) state
    #[test]
    fn new_episode_initial_state(
        task_description in "[a-zA-Z0-9 ]{1,100}",
        domain in "[a-z]{3,15}",
        task_type in prop::sample::select(vec![
            TaskType::CodeGeneration,
            TaskType::Testing,
            TaskType::Debugging,
            TaskType::Refactoring,
            TaskType::Analysis,
            TaskType::Documentation,
        ]),
    ) {
        let episode = Episode::new(
            task_description,
            TaskContext {
                language: Some("rust".to_string()),
                framework: None,
                complexity: ComplexityLevel::Moderate,
                domain,
                tags: vec![],
            },
            task_type,
        );

        prop_assert!(!episode.is_complete());
        prop_assert!(episode.outcome.is_none());
        prop_assert!(episode.end_time.is_none());
        prop_assert!(episode.reward.is_none());
        prop_assert!(episode.steps.is_empty());
        prop_assert!(episode.duration().is_none());
    }

    /// Adding steps never makes an episode complete
    #[test]
    fn adding_steps_preserves_incomplete_state(
        num_steps in 1usize..100usize,
    ) {
        let mut episode = Episode::new(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::CodeGeneration,
        );

        for i in 0..num_steps {
            let step = ExecutionStep::new(
                i + 1,
                format!("tool_{i}"),
                format!("Action {i}"),
            );
            episode.add_step(step);

            prop_assert!(!episode.is_complete());
            prop_assert!(episode.outcome.is_none());
            prop_assert!(episode.end_time.is_none());
        }

        prop_assert_eq!(episode.steps.len(), num_steps);
    }

    /// Completing an episode transitions it to the complete state
    #[test]
    fn completion_transitions_to_complete_state(
        num_steps in 0usize..50usize,
        outcome_kind in 0u8..3u8,
    ) {
        let mut episode = Episode::new(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        );

        for i in 0..num_steps {
            let step = ExecutionStep::new(i + 1, "tool".to_string(), "action".to_string());
            episode.add_step(step);
        }

        let outcome = match outcome_kind {
            0 => TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            },
            1 => TaskOutcome::PartialSuccess {
                verdict: "Partial".to_string(),
                artifacts: vec![],
            },
            _ => TaskOutcome::Failure {
                verdict: "Failed".to_string(),
                error: Some("error".to_string()),
            },
        };

        episode.complete(outcome);

        prop_assert!(episode.is_complete());
        prop_assert!(episode.outcome.is_some());
        prop_assert!(episode.end_time.is_some());
        prop_assert!(episode.duration().is_some());
        prop_assert_eq!(episode.steps.len(), num_steps);
    }

    /// Step counts are always accurate
    #[test]
    fn step_counts_invariant(
        success_count in 0usize..30usize,
        error_count in 0usize..30usize,
    ) {
        let mut episode = Episode::new(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::CodeGeneration,
        );

        for i in 0..success_count {
            let mut step = ExecutionStep::new(
                i + 1,
                "tool".to_string(),
                "action".to_string(),
            );
            step.result = Some(ExecutionResult::Success {
                output: "ok".to_string(),
            });
            episode.add_step(step);
        }

        for i in 0..error_count {
            let mut step = ExecutionStep::new(
                success_count + i + 1,
                "tool".to_string(),
                "action".to_string(),
            );
            step.result = Some(ExecutionResult::Error {
                message: "err".to_string(),
            });
            episode.add_step(step);
        }

        prop_assert_eq!(episode.steps.len(), success_count + error_count);
        prop_assert_eq!(episode.successful_steps_count(), success_count);
        prop_assert_eq!(episode.failed_steps_count(), error_count);
    }
}

// ============================================================================
// Capacity Manager Properties
// ============================================================================

proptest! {
    /// can_store returns true only when under capacity
    #[test]
    fn can_store_under_capacity_invariant(
        max_episodes in 1usize..1000usize,
        current_count in 0usize..2000usize,
    ) {
        let manager = CapacityManager::new(max_episodes, EvictionPolicy::LRU);
        let can_store = manager.can_store(current_count);

        if current_count < max_episodes {
            prop_assert!(can_store);
        } else {
            prop_assert!(!can_store);
        }
    }

    /// Eviction never returns more IDs than episodes available
    #[test]
    fn eviction_count_bounded_by_episode_count(
        num_episodes in 1usize..20usize,
        max_capacity in 1usize..20usize,
        policy in prop::sample::select(vec![
            EvictionPolicy::LRU,
            EvictionPolicy::RelevanceWeighted,
        ]),
    ) {
        let manager = CapacityManager::new(max_capacity, policy);

        let episodes: Vec<Episode> = (0..num_episodes)
            .map(|i| Episode::new(
                format!("Task {i}"),
                TaskContext::default(),
                TaskType::Testing,
            ))
            .collect();

        let to_evict = manager.evict_if_needed(&episodes);

        prop_assert!(to_evict.len() <= num_episodes);
    }

    /// Eviction returns empty when under capacity
    #[test]
    fn no_eviction_under_capacity(
        num_episodes in 0usize..10usize,
    ) {
        let max_capacity = num_episodes + 5; // Always well under capacity
        let manager = CapacityManager::new(max_capacity, EvictionPolicy::LRU);

        let episodes: Vec<Episode> = (0..num_episodes)
            .map(|i| Episode::new(
                format!("Task {i}"),
                TaskContext::default(),
                TaskType::Testing,
            ))
            .collect();

        let to_evict = manager.evict_if_needed(&episodes);
        prop_assert!(to_evict.is_empty());
    }

    /// Eviction returns unique episode IDs
    #[test]
    fn eviction_returns_unique_ids(
        num_episodes in 2usize..15usize,
        policy in prop::sample::select(vec![
            EvictionPolicy::LRU,
            EvictionPolicy::RelevanceWeighted,
        ]),
    ) {
        let max_capacity = 1; // Force eviction
        let manager = CapacityManager::new(max_capacity, policy);

        let episodes: Vec<Episode> = (0..num_episodes)
            .map(|i| Episode::new(
                format!("Task {i}"),
                TaskContext::default(),
                TaskType::Testing,
            ))
            .collect();

        let to_evict = manager.evict_if_needed(&episodes);

        let unique_count = to_evict.iter().collect::<std::collections::HashSet<_>>().len();
        prop_assert_eq!(to_evict.len(), unique_count);
    }

    /// Relevance scores are always in [0.0, 1.0]
    #[test]
    fn relevance_score_bounded(
        reward_total in 0.0f32..3.0f32,
        has_reward in proptest::bool::ANY,
    ) {
        let manager = CapacityManager::new(100, EvictionPolicy::RelevanceWeighted);

        let mut episode = Episode::new(
            "Test".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        );

        if has_reward {
            episode.reward = Some(RewardScore {
                total: reward_total,
                base: 0.5,
                efficiency: 1.0,
                complexity_bonus: 1.0,
                quality_multiplier: 1.0,
                learning_bonus: 0.0,
            });
        }

        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let score = manager.relevance_score(&episode);
        prop_assert!((0.0..=1.0).contains(&score),
            "Relevance score {} out of bounds", score);
    }

    /// Quality score is always in [0.0, 1.0]
    #[test]
    fn quality_score_bounded(
        reward_total in 0.0f32..5.0f32,
    ) {
        let manager = CapacityManager::new(100, EvictionPolicy::RelevanceWeighted);

        let mut episode = Episode::new(
            "Test".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        );

        episode.reward = Some(RewardScore {
            total: reward_total,
            base: 0.5,
            efficiency: 1.0,
            complexity_bonus: 1.0,
            quality_multiplier: 1.0,
            learning_bonus: 0.0,
        });

        let score = manager.extract_quality_score(&episode);
        prop_assert!((0.0..=1.0).contains(&score),
            "Quality score {} out of bounds for reward total {}", score, reward_total);
    }

    /// Recency score is always in [0.0, 1.0]
    #[test]
    fn recency_score_bounded(
        has_end_time in proptest::bool::ANY,
    ) {
        let manager = CapacityManager::new(100, EvictionPolicy::RelevanceWeighted);

        let mut episode = Episode::new(
            "Test".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        );

        if has_end_time {
            episode.complete(TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            });
        }

        let score = manager.calculate_recency_score(&episode);
        prop_assert!((0.0..=1.0).contains(&score),
            "Recency score {} out of bounds", score);
    }
}
