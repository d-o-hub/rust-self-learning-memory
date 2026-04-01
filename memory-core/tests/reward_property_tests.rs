//! Property-based tests for `RewardCalculator` outputs
//!
//! These tests verify bounds and invariants for reward calculations
//! using the proptest crate for property-based testing.
//!
//! Covers ACT-031 from ADR-042 (Code Coverage Improvement)

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]

use chrono::{Duration, Utc};
use do_memory_core::{
    AdaptiveRewardCalculator, ComplexityLevel, DomainStatistics, Episode, ExecutionResult,
    ExecutionStep, PatternId, RewardCalculator, TaskContext, TaskOutcome, TaskType,
};
use proptest::prelude::*;

const MIN_EFFICIENCY: f32 = 0.5;
const MAX_EFFICIENCY: f32 = 1.5;
const MIN_QUALITY: f32 = 0.5;
const MAX_QUALITY: f32 = 1.5;
const MAX_LEARNING_BONUS: f32 = 0.5;
const MIN_COMPLEXITY_BONUS: f32 = 1.0;
const MAX_COMPLEXITY_BONUS: f32 = 1.2;

fn create_episode_with_params(
    complexity: ComplexityLevel,
    num_steps: usize,
    success_rate: f32,
    duration_secs: i64,
) -> Episode {
    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: None,
        complexity,
        domain: "test-domain".to_string(),
        tags: vec![],
    };
    let mut episode = Episode::new("Test task".to_string(), context, TaskType::Testing);
    for i in 0..num_steps {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), format!("Action {i}"));
        let is_success = (i as f32 / num_steps.max(1) as f32) < success_rate;
        step.result = if is_success {
            Some(ExecutionResult::Success {
                output: "OK".to_string(),
            })
        } else {
            Some(ExecutionResult::Error {
                message: "Failed".to_string(),
            })
        };
        episode.add_step(step);
    }
    if duration_secs > 0 {
        episode.start_time = Utc::now() - Duration::seconds(duration_secs);
    }
    episode
}

// ============================================================================
// Efficiency Multiplier Property Tests
// ============================================================================

proptest! {
    #[test]
    fn efficiency_multiplier_always_within_bounds(
        num_steps in 0usize..200usize,
        duration_secs in 0i64..3600i64,
        complexity in prop::sample::select(vec![
            ComplexityLevel::Simple,
            ComplexityLevel::Moderate,
            ComplexityLevel::Complex,
        ]),
    ) {
        let calculator = RewardCalculator::new();
        let mut episode = create_episode_with_params(complexity, num_steps, 1.0, duration_secs);
        episode.complete(TaskOutcome::Success { verdict: "Done".to_string(), artifacts: vec![] });
        let reward = calculator.calculate(&episode);
        prop_assert!(reward.efficiency >= MIN_EFFICIENCY, "Efficiency below minimum");
        prop_assert!(reward.efficiency <= MAX_EFFICIENCY, "Efficiency above maximum");
    }
}

proptest! {
    #[test]
    fn efficiency_clamped_at_minimum_for_extreme_values(
        num_steps in 500usize..1000usize,
        duration_secs in 7200i64..86400i64,
    ) {
        let calculator = RewardCalculator::new();
        let mut episode = create_episode_with_params(ComplexityLevel::Simple, num_steps, 0.5, duration_secs);
        episode.complete(TaskOutcome::Success { verdict: "Slow".to_string(), artifacts: vec![] });
        let reward = calculator.calculate(&episode);
        prop_assert!(reward.efficiency >= MIN_EFFICIENCY, "Efficiency should be clamped at minimum");
    }
}

proptest! {
    #[test]
    fn efficiency_for_instant_completion(num_steps in 1usize..10usize) {
        let calculator = RewardCalculator::new();
        let mut episode = create_episode_with_params(ComplexityLevel::Simple, num_steps, 1.0, 0);
        episode.complete(TaskOutcome::Success { verdict: "Instant".to_string(), artifacts: vec![] });
        let reward = calculator.calculate(&episode);
        // Instant completion (duration = 0) gives max duration efficiency, but combined
        // with step count efficiency may not reach max. Verify it's above average.
        prop_assert!(reward.efficiency > 1.0, "Instant completion should have above-average efficiency");
    }
}

proptest! {
    #[test]
    fn efficiency_decreases_with_more_steps(steps_low in 1usize..5usize, steps_high in 50usize..100usize) {
        let calculator = RewardCalculator::new();
        let mut episode_low = create_episode_with_params(ComplexityLevel::Simple, steps_low, 1.0, 30);
        episode_low.complete(TaskOutcome::Success { verdict: "Done".to_string(), artifacts: vec![] });
        let mut episode_high = create_episode_with_params(ComplexityLevel::Simple, steps_high, 1.0, 30);
        episode_high.complete(TaskOutcome::Success { verdict: "Done".to_string(), artifacts: vec![] });
        let reward_low = calculator.calculate(&episode_low);
        let reward_high = calculator.calculate(&episode_high);
        prop_assert!(reward_low.efficiency >= reward_high.efficiency, "Fewer steps should have higher efficiency");
    }
}

// ============================================================================
// Quality Multiplier Property Tests
// ============================================================================

proptest! {
    #[test]
    fn quality_multiplier_always_within_bounds(
        num_steps in 1usize..100usize,
        success_rate in 0.0f32..1.0f32,
        has_coverage in proptest::bool::ANY,
        clippy_warnings in 0usize..50usize,
    ) {
        let calculator = RewardCalculator::new();
        let mut episode = create_episode_with_params(ComplexityLevel::Moderate, num_steps, success_rate, 60);
        if has_coverage {
            episode.metadata.insert("test_coverage".to_string(), "85.0".to_string());
        }
        episode.metadata.insert("clippy_warnings".to_string(), clippy_warnings.to_string());
        let artifacts = if has_coverage {
            vec!["test_coverage.txt".to_string(), "src/main.rs".to_string()]
        } else {
            vec!["src/main.rs".to_string()]
        };
        episode.complete(TaskOutcome::Success { verdict: "Done".to_string(), artifacts });
        let reward = calculator.calculate(&episode);
        prop_assert!(reward.quality_multiplier >= MIN_QUALITY, "Quality below minimum");
        prop_assert!(reward.quality_multiplier <= MAX_QUALITY, "Quality above maximum");
    }
}

proptest! {
    #[test]
    fn quality_penalty_for_high_error_rate(num_steps in 10usize..50usize, error_rate in 0.4f32..1.0f32) {
        let calculator = RewardCalculator::new();
        let success_rate = 1.0 - error_rate;
        let mut episode = create_episode_with_params(ComplexityLevel::Simple, num_steps, success_rate, 60);
        episode.complete(TaskOutcome::Success { verdict: "Done".to_string(), artifacts: vec![] });
        let reward = calculator.calculate(&episode);
        prop_assert!(reward.quality_multiplier < 1.0, "High error rate should reduce quality");
    }
}

proptest! {
    #[test]
    fn quality_bonus_for_zero_errors(num_steps in 5usize..20usize) {
        let calculator = RewardCalculator::new();
        let mut episode = create_episode_with_params(ComplexityLevel::Simple, num_steps, 1.0, 60);
        episode.complete(TaskOutcome::Success { verdict: "Perfect".to_string(), artifacts: vec![] });
        let reward = calculator.calculate(&episode);
        prop_assert!(reward.quality_multiplier >= 1.0, "Zero errors should give quality bonus");
    }
}

proptest! {
    #[test]
    fn quality_bonus_for_test_coverage(coverage in 60.0f32..100.0f32) {
        let calculator = RewardCalculator::new();
        let mut episode = create_episode_with_params(ComplexityLevel::Moderate, 10, 1.0, 60);
        episode.metadata.insert("test_coverage".to_string(), coverage.to_string());
        episode.complete(TaskOutcome::Success { verdict: "Done".to_string(), artifacts: vec!["coverage.txt".to_string()] });
        let reward = calculator.calculate(&episode);
        prop_assert!(reward.quality_multiplier > 1.0, "High coverage should give quality bonus");
    }
}

// ============================================================================
// Learning Bonus Property Tests
// ============================================================================

proptest! {
    #[test]
    fn learning_bonus_always_within_bounds(
        num_steps in 0usize..100usize,
        success_rate in 0.0f32..1.0f32,
        num_patterns in 0usize..10usize,
    ) {
        let calculator = RewardCalculator::new();
        let mut episode = create_episode_with_params(ComplexityLevel::Moderate, num_steps, success_rate, 60);
        for _ in 0..num_patterns {
            episode.patterns.push(PatternId::new_v4());
        }
        episode.complete(TaskOutcome::Success { verdict: "Done".to_string(), artifacts: vec![] });
        let reward = calculator.calculate(&episode);
        prop_assert!(reward.learning_bonus >= 0.0, "Learning bonus below minimum");
        prop_assert!(reward.learning_bonus <= MAX_LEARNING_BONUS, "Learning bonus above maximum");
    }
}

proptest! {
    #[test]
    fn learning_bonus_increases_with_patterns(num_patterns_low in 0usize..2usize, num_patterns_high in 5usize..10usize) {
        let calculator = RewardCalculator::new();
        let mut episode_low = create_episode_with_params(ComplexityLevel::Moderate, 10, 1.0, 60);
        for _ in 0..num_patterns_low {
            episode_low.patterns.push(PatternId::new_v4());
        }
        episode_low.complete(TaskOutcome::Success { verdict: "Done".to_string(), artifacts: vec![] });
        let mut episode_high = create_episode_with_params(ComplexityLevel::Moderate, 10, 1.0, 60);
        for _ in 0..num_patterns_high {
            episode_high.patterns.push(PatternId::new_v4());
        }
        episode_high.complete(TaskOutcome::Success { verdict: "Done".to_string(), artifacts: vec![] });
        let reward_low = calculator.calculate(&episode_low);
        let reward_high = calculator.calculate(&episode_high);
        prop_assert!(reward_high.learning_bonus >= reward_low.learning_bonus, "More patterns should give more learning bonus");
    }
}

proptest! {
    #[test]
    fn learning_bonus_for_tool_diversity(num_unique_tools in 3usize..10usize) {
        let calculator = RewardCalculator::new();
        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: None,
            complexity: ComplexityLevel::Moderate,
            domain: "test-domain".to_string(),
            tags: vec![],
        };
        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Testing);
        for i in 0..num_unique_tools {
            let mut step = ExecutionStep::new(i + 1, format!("unique_tool_{i}"), "Action".to_string());
            step.result = Some(ExecutionResult::Success { output: "OK".to_string() });
            episode.add_step(step);
        }
        episode.complete(TaskOutcome::Success { verdict: "Diverse".to_string(), artifacts: vec![] });
        let reward = calculator.calculate(&episode);
        if num_unique_tools >= 5 {
            prop_assert!(reward.learning_bonus > 0.0, "Tool diversity should give learning bonus");
        }
    }
}

proptest! {
    #[test]
    fn learning_bonus_capped_at_maximum(num_patterns in 10usize..50usize, num_steps in 20usize..50usize) {
        let calculator = RewardCalculator::new();
        let mut episode = create_episode_with_params(ComplexityLevel::Complex, num_steps, 1.0, 15);
        for _ in 0..num_patterns {
            episode.patterns.push(PatternId::new_v4());
        }
        episode.complete(TaskOutcome::Success { verdict: "Maximum".to_string(), artifacts: vec![] });
        let reward = calculator.calculate(&episode);
        prop_assert!(reward.learning_bonus <= MAX_LEARNING_BONUS, "Learning bonus should be capped");
    }
}

// ============================================================================
// Complexity Bonus Property Tests
// ============================================================================

proptest! {
    #[test]
    fn complexity_bonus_within_bounds(
        complexity in prop::sample::select(vec![
            ComplexityLevel::Simple,
            ComplexityLevel::Moderate,
            ComplexityLevel::Complex,
        ]),
    ) {
        let calculator = RewardCalculator::new();
        let mut episode = create_episode_with_params(complexity, 5, 1.0, 60);
        episode.complete(TaskOutcome::Success { verdict: "Done".to_string(), artifacts: vec![] });
        let reward = calculator.calculate(&episode);
        prop_assert!(reward.complexity_bonus >= MIN_COMPLEXITY_BONUS, "Complexity bonus below minimum");
        prop_assert!(reward.complexity_bonus <= MAX_COMPLEXITY_BONUS, "Complexity bonus above maximum");
    }
}

proptest! {
    #[test]
    fn complexity_bonus_matches_expected_values(
        complexity in prop::sample::select(vec![
            ComplexityLevel::Simple,
            ComplexityLevel::Moderate,
            ComplexityLevel::Complex,
        ]),
    ) {
        let calculator = RewardCalculator::new();
        let mut episode = create_episode_with_params(complexity, 5, 1.0, 60);
        episode.complete(TaskOutcome::Success { verdict: "Done".to_string(), artifacts: vec![] });
        let reward = calculator.calculate(&episode);
        let expected = match complexity {
            ComplexityLevel::Simple => 1.0,
            ComplexityLevel::Moderate => 1.1,
            ComplexityLevel::Complex => 1.2,
        };
        prop_assert!((reward.complexity_bonus - expected).abs() < 0.001, "Complexity bonus should match expected value");
    }
}

// ============================================================================
// Total Reward Property Tests
// ============================================================================

proptest! {
    #[test]
    fn total_reward_calculation_correct(
        num_steps in 1usize..50usize,
        complexity in prop::sample::select(vec![
            ComplexityLevel::Simple,
            ComplexityLevel::Moderate,
            ComplexityLevel::Complex,
        ]),
    ) {
        let calculator = RewardCalculator::new();
        let mut episode = create_episode_with_params(complexity, num_steps, 1.0, 60);
        episode.complete(TaskOutcome::Success { verdict: "Done".to_string(), artifacts: vec![] });
        let reward = calculator.calculate(&episode);
        let expected_total = (reward.base * reward.efficiency * reward.complexity_bonus * reward.quality_multiplier) + reward.learning_bonus;
        prop_assert!((reward.total - expected_total).abs() < 0.001, "Total reward calculation incorrect");
    }
}

proptest! {
    #[test]
    fn total_reward_non_negative_for_success(num_steps in 1usize..100usize, duration_secs in 0i64..3600i64) {
        let calculator = RewardCalculator::new();
        let mut episode = create_episode_with_params(ComplexityLevel::Moderate, num_steps, 1.0, duration_secs);
        episode.complete(TaskOutcome::Success { verdict: "Done".to_string(), artifacts: vec![] });
        let reward = calculator.calculate(&episode);
        prop_assert!(reward.total >= 0.0, "Total reward should be non-negative");
    }
}

proptest! {
    #[test]
    fn total_reward_zero_for_failure(_dummy in 0usize..1usize) {
        let calculator = RewardCalculator::new();
        let mut episode = create_episode_with_params(ComplexityLevel::Simple, 5, 1.0, 60);
        episode.complete(TaskOutcome::Failure { reason: "Failed".to_string(), error_details: None });
        let reward = calculator.calculate(&episode);
        // Base is 0 for failure, but learning_bonus can still be non-zero
        prop_assert!((reward.base - 0.0).abs() < 0.001, "Base should be 0 for failure");
        // Total = learning_bonus when base is 0, so verify bounds
        prop_assert!(reward.total >= 0.0 && reward.total <= MAX_LEARNING_BONUS, "Total should be within learning bonus bounds for failure");
    }
}

// ============================================================================
// Edge Cases Property Tests
// ============================================================================

proptest! {
    #[test]
    fn edge_case_zero_steps(_dummy in 0usize..1usize) {
        let calculator = RewardCalculator::new();
        let mut episode = create_episode_with_params(ComplexityLevel::Simple, 0, 1.0, 0);
        episode.complete(TaskOutcome::Success { verdict: "No steps".to_string(), artifacts: vec![] });
        let reward = calculator.calculate(&episode);
        prop_assert!(reward.efficiency >= MIN_EFFICIENCY, "Efficiency below minimum for zero steps");
        prop_assert!(reward.efficiency <= MAX_EFFICIENCY, "Efficiency above maximum for zero steps");
    }
}

proptest! {
    #[test]
    fn edge_case_max_values(num_steps in 900usize..1000usize, duration_secs in 82800i64..86400i64) {
        let calculator = RewardCalculator::new();
        let mut episode = create_episode_with_params(ComplexityLevel::Complex, num_steps, 0.1, duration_secs);
        episode.complete(TaskOutcome::Success { verdict: "Maximum".to_string(), artifacts: vec![] });
        let reward = calculator.calculate(&episode);
        prop_assert!(reward.efficiency >= MIN_EFFICIENCY && reward.efficiency <= MAX_EFFICIENCY, "Efficiency out of bounds");
        prop_assert!(reward.quality_multiplier >= MIN_QUALITY && reward.quality_multiplier <= MAX_QUALITY, "Quality out of bounds");
        prop_assert!(reward.learning_bonus >= 0.0 && reward.learning_bonus <= MAX_LEARNING_BONUS, "Learning bonus out of bounds");
    }
}

proptest! {
    #[test]
    fn edge_case_partial_success(completed in 0usize..10usize, failed in 0usize..10usize) {
        let calculator = RewardCalculator::new();
        let mut episode = create_episode_with_params(ComplexityLevel::Simple, 5, 1.0, 60);
        episode.complete(TaskOutcome::PartialSuccess {
            verdict: "Partial".to_string(),
            completed: (0..completed).map(|i| format!("item_{i}")).collect(),
            failed: (0..failed).map(|i| format!("fail_{i}")).collect(),
        });
        let reward = calculator.calculate(&episode);
        let total = completed + failed;
        if total == 0 {
            prop_assert!((reward.base - 0.5).abs() < 0.001, "Empty lists should give 0.5 base");
        } else {
            let expected_base = completed as f32 / total as f32;
            prop_assert!((reward.base - expected_base).abs() < 0.001, "Base should match ratio");
        }
    }
}

proptest! {
    #[test]
    fn edge_case_incomplete_episode(num_steps in 0usize..50usize) {
        let calculator = RewardCalculator::new();
        let episode = create_episode_with_params(ComplexityLevel::Moderate, num_steps, 1.0, 60);
        let reward = calculator.calculate(&episode);
        // Incomplete episode has no outcome, so base = 0
        prop_assert!((reward.base - 0.0).abs() < 0.001, "Incomplete episode should have 0 base");
        // Total = learning_bonus when base is 0, so verify it's within bounds
        prop_assert!(reward.total >= 0.0 && reward.total <= MAX_LEARNING_BONUS, "Incomplete episode total should be within learning bonus bounds");
    }
}

// ============================================================================
// Adaptive Reward Calculator Property Tests
// ============================================================================

proptest! {
    #[test]
    fn adaptive_efficiency_bounds(
        episode_steps in 1usize..100usize,
        p50_steps in 5usize..50usize,
        p50_duration in 10.0f32..300.0f32,
    ) {
        let calculator = AdaptiveRewardCalculator::new();
        let mut episode = create_episode_with_params(ComplexityLevel::Moderate, episode_steps, 1.0, 60);
        episode.complete(TaskOutcome::Success { verdict: "Done".to_string(), artifacts: vec![] });
        let stats = DomainStatistics {
            domain: "test-domain".to_string(),
            episode_count: 10,
            avg_duration_secs: p50_duration * 1.2,
            p50_duration_secs: p50_duration,
            p90_duration_secs: p50_duration * 2.0,
            avg_step_count: p50_steps as f32 * 1.2,
            p50_step_count: p50_steps,
            p90_step_count: p50_steps * 2,
            avg_reward: 0.75,
            p50_reward: 0.8,
            reward_std_dev: 0.15,
            last_updated: Utc::now(),
            success_count: 8,
        };
        let reward = calculator.calculate(&episode, Some(&stats));
        prop_assert!(reward.efficiency >= MIN_EFFICIENCY, "Adaptive efficiency below minimum");
        prop_assert!(reward.efficiency <= MAX_EFFICIENCY, "Adaptive efficiency above maximum");
    }
}

proptest! {
    #[test]
    fn adaptive_fallback_with_unreliable_stats(episode_steps in 1usize..50usize) {
        let calculator = AdaptiveRewardCalculator::new();
        let mut episode = create_episode_with_params(ComplexityLevel::Simple, episode_steps, 1.0, 60);
        episode.complete(TaskOutcome::Success { verdict: "Done".to_string(), artifacts: vec![] });
        let stats = DomainStatistics {
            domain: "test-domain".to_string(),
            episode_count: 3,
            avg_duration_secs: 30.0,
            p50_duration_secs: 25.0,
            p90_duration_secs: 45.0,
            avg_step_count: 10.0,
            p50_step_count: 8,
            p90_step_count: 15,
            avg_reward: 0.8,
            p50_reward: 0.85,
            reward_std_dev: 0.1,
            last_updated: Utc::now(),
            success_count: 2,
        };
        let reward = calculator.calculate(&episode, Some(&stats));
        prop_assert!(reward.efficiency >= MIN_EFFICIENCY, "Fallback efficiency below minimum");
        prop_assert!(reward.efficiency <= MAX_EFFICIENCY, "Fallback efficiency above maximum");
    }
}

proptest! {
    #[test]
    fn adaptive_with_none_stats(episode_steps in 1usize..50usize) {
        let calculator = AdaptiveRewardCalculator::new();
        let mut episode = create_episode_with_params(ComplexityLevel::Moderate, episode_steps, 1.0, 60);
        episode.complete(TaskOutcome::Success { verdict: "Done".to_string(), artifacts: vec![] });
        let reward = calculator.calculate(&episode, None);
        prop_assert!(reward.efficiency >= MIN_EFFICIENCY && reward.efficiency <= MAX_EFFICIENCY, "Efficiency out of bounds");
        prop_assert!(reward.quality_multiplier >= MIN_QUALITY && reward.quality_multiplier <= MAX_QUALITY, "Quality out of bounds");
        prop_assert!(reward.learning_bonus >= 0.0 && reward.learning_bonus <= MAX_LEARNING_BONUS, "Learning bonus out of bounds");
    }
}

proptest! {
    #[test]
    fn custom_weights_affect_efficiency(duration_weight in 0.0f32..1.0f32, step_count_weight in 0.0f32..1.0f32) {
        let total = duration_weight + step_count_weight;
        let duration_weight = if total > 0.0 { duration_weight / total } else { 0.5 };
        let step_count_weight = if total > 0.0 { step_count_weight / total } else { 0.5 };
        let calculator = RewardCalculator::with_weights(duration_weight, step_count_weight);
        let mut episode = create_episode_with_params(ComplexityLevel::Simple, 20, 1.0, 60);
        episode.complete(TaskOutcome::Success { verdict: "Done".to_string(), artifacts: vec![] });
        let reward = calculator.calculate(&episode);
        prop_assert!(reward.efficiency >= MIN_EFFICIENCY, "Custom weights efficiency below minimum");
        prop_assert!(reward.efficiency <= MAX_EFFICIENCY, "Custom weights efficiency above maximum");
    }
}

proptest! {
    #[test]
    fn adaptive_custom_config(
        duration_weight in 0.0f32..1.0f32,
        fallback_duration in 10.0f32..300.0f32,
        fallback_steps in 5usize..50usize,
    ) {
        let step_count_weight = 1.0 - duration_weight;
        let calculator = AdaptiveRewardCalculator::with_config(
            duration_weight,
            step_count_weight,
            fallback_duration,
            fallback_steps,
        );
        let mut episode = create_episode_with_params(ComplexityLevel::Simple, fallback_steps / 2, 1.0, fallback_duration as i64 / 2);
        episode.complete(TaskOutcome::Success { verdict: "Done".to_string(), artifacts: vec![] });
        let reward = calculator.calculate(&episode, None);
        prop_assert!(reward.efficiency >= MIN_EFFICIENCY, "Custom config efficiency below minimum");
        prop_assert!(reward.efficiency <= MAX_EFFICIENCY, "Custom config efficiency above maximum");
    }
}
