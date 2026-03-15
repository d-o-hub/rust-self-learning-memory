//! Property-based tests for RewardCalculator outputs
//!
//! These tests verify bounds and invariants for reward calculations
//! using the proptest crate for property-based testing.
//!
//! Covers ACT-031 from ADR-042 (Code Coverage Improvement)

use chrono::{Duration, Utc};
use memory_core::{
    AdaptiveRewardCalculator, ComplexityLevel, Episode, ExecutionResult, ExecutionStep,
    RewardCalculator, TaskContext, TaskOutcome, TaskType,
};
use proptest::prelude::*;

// ============================================================================
// Constants for bounds verification
// ============================================================================

/// Minimum efficiency multiplier
const MIN_EFFICIENCY: f32 = 0.5;

/// Maximum efficiency multiplier
const MAX_EFFICIENCY: f32 = 1.5;

/// Minimum quality multiplier
const MIN_QUALITY: f32 = 0.5;

/// Maximum quality multiplier
const MAX_QUALITY: f32 = 1.5;

/// Maximum learning bonus
const MAX_LEARNING_BONUS: f32 = 0.5;

/// Minimum complexity bonus
const MIN_COMPLEXITY_BONUS: f32 = 1.0;

/// Maximum complexity bonus
const MAX_COMPLEXITY_BONUS: f32 = 1.2;

// ============================================================================
// Helper functions for test setup
// ============================================================================

/// Create a test episode with configurable parameters
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

    // Add steps with configurable success rate
    for i in 0..num_steps {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), format!("Action {i}"));

        // Determine if this step should succeed based on success_rate
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

    // Set start time in the past to control duration
    if duration_secs > 0 {
        episode.start_time = Utc::now() - Duration::seconds(duration_secs);
    }

    episode
}

/// Create an episode with specific tool diversity
fn create_episode_with_tools(tools: &[&str], results: &[bool]) -> Episode {
    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: None,
        complexity: ComplexityLevel::Moderate,
        domain: "test-domain".to_string(),
        tags: vec![],
    };

    let mut episode = Episode::new("Test task".to_string(), context, TaskType::Testing);

    for (i, (tool, &is_success)) in tools.iter().zip(results.iter()).enumerate() {
        let mut step = ExecutionStep::new(i + 1, tool.to_string(), "Action".to_string());

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

    episode
}

// ============================================================================
// Efficiency Multiplier Property Tests
// ============================================================================

proptest! {
    /// Test that efficiency multiplier is always within bounds [0.5, 1.5]
    /// regardless of input parameters
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

        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let reward = calculator.calculate(&episode);

        prop_assert!(
            reward.efficiency >= MIN_EFFICIENCY,
            "Efficiency {} is below minimum {}",
            reward.efficiency,
            MIN_EFFICIENCY
        );

        prop_assert!(
            reward.efficiency <= MAX_EFFICIENCY,
            "Efficiency {} exceeds maximum {}",
            reward.efficiency,
            MAX_EFFICIENCY
        );
    }

    /// Test efficiency clamping at minimum with extreme values
    #[test]
    fn efficiency_clamped_at_minimum_for_extreme_values(
        num_steps in 500usize..1000usize,
        duration_secs in 7200i64..86400i64, // 2-24 hours
    ) {
        let calculator = RewardCalculator::new();
        let mut episode = create_episode_with_params(
            ComplexityLevel::Simple,
            num_steps,
            0.5,
            duration_secs,
        );

        episode.complete(TaskOutcome::Success {
            verdict: "Slow execution".to_string(),
            artifacts: vec![],
        });

        let reward = calculator.calculate(&episode);

        prop_assert!(
            reward.efficiency >= MIN_EFFICIENCY,
            "Efficiency {} should be clamped to minimum {}",
            reward.efficiency,
            MIN_EFFICIENCY
        );
    }

    /// Test efficiency for instant completion (zero duration)
    #[test]
    fn efficiency_for_instant_completion(
        num_steps in 1usize..10usize,
    ) {
        let calculator = RewardCalculator::new();
        let mut episode = create_episode_with_params(
            ComplexityLevel::Simple,
            num_steps,
            1.0,
            0, // Zero duration
        );

        episode.complete(TaskOutcome::Success {
            verdict: "Instant".to_string(),
            artifacts: vec![],
        });

        let reward = calculator.calculate(&episode);

        // Instant completion should give maximum efficiency
        prop_assert!(
            reward.efficiency == MAX_EFFICIENCY,
            "Instant completion should give max efficiency {}, got {}",
            MAX_EFFICIENCY,
            reward.efficiency
        );
    }

    /// Test efficiency correlation with step count
    #[test]
    fn efficiency_decreases_with_more_steps(
        steps_low in 1usize..5usize,
        steps_high in 50usize..100usize,
    ) {
        let calculator = RewardCalculator::new();

        // Episode with few steps
        let mut episode_low = create_episode_with_params(
            ComplexityLevel::Simple,
            steps_low,
            1.0,
            30,
        );
        episode_low.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        // Episode with many steps
        let mut episode_high = create_episode_with_params(
            ComplexityLevel::Simple,
            steps_high,
            1.0,
            30,
        );
        episode_high.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let reward_low = calculator.calculate(&episode_low);
        let reward_high = calculator.calculate(&episode_high);

        // Fewer steps should have higher or equal efficiency
        prop_assert!(
            reward_low.efficiency >= reward_high.efficiency,
            "Fewer steps ({}) should have >= efficiency than more steps ({}). Got {} vs {}",
            steps_low,
            steps_high,
            reward_low.efficiency,
            reward_high.efficiency
        );
    }
}

// ============================================================================
// Quality Multiplier Property Tests
// ============================================================================

proptest! {
    /// Test that quality multiplier is always within bounds [0.5, 1.5]
    #[test]
    fn quality_multiplier_always_within_bounds(
        num_steps in 1usize..100usize,
        success_rate in 0.0f32..1.0f32,
        has_coverage in proptest::bool::ANY,
        clippy_warnings in 0usize..50usize,
    ) {
        let calculator = RewardCalculator::new();
        let mut episode = create_episode_with_params(
            ComplexityLevel::Moderate,
            num_steps,
            success_rate,
            60,
        );

        // Add metadata for quality indicators
        if has_coverage {
            episode.metadata.insert(
                "test_coverage".to_string(),
                "85.0".to_string(),
            );
        }
        episode.metadata.insert(
            "clippy_warnings".to_string(),
            clippy_warnings.to_string(),
        );

        // Create artifacts based on coverage flag
        let artifacts = if has_coverage {
            vec!["test_coverage.txt".to_string(), "src/main.rs".to_string()]
        } else {
            vec!["src/main.rs".to_string()]
        };

        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts,
        });

        let reward = calculator.calculate(&episode);

        prop_assert!(
            reward.quality_multiplier >= MIN_QUALITY,
            "Quality {} is below minimum {}",
            reward.quality_multiplier,
            MIN_QUALITY
        );

        prop_assert!(
            reward.quality_multiplier <= MAX_QUALITY,
            "Quality {} exceeds maximum {}",
            reward.quality_multiplier,
            MAX_QUALITY
        );
    }

    /// Test quality penalty for high error rates
    #[test]
    fn quality_penalty_for_high_error_rate(
        num_steps in 10usize..50usize,
        error_rate in 0.4f32..1.0f32, // High error rate
    ) {
        let calculator = RewardCalculator::new();
        let mut episode = create_episode_with_params(
            ComplexityLevel::Simple,
            num_steps,
            1.0 - error_rate, // Convert error rate to success rate
            60,
        );

        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let reward = calculator.calculate(&episode);

        // High error rate should result in lower quality
        prop_assert!(
            reward.quality_multiplier < 1.0,
            "High error rate ({}) should reduce quality below 1.0, got {}",
            error_rate,
            reward.quality_multiplier
        );
    }

    /// Test quality bonus for zero errors
    #[test]
    fn quality_bonus_for_zero_errors(
        num_steps in 5usize..20usize,
    ) {
        let calculator = RewardCalculator::new();
        let mut episode = create_episode_with_params(
            ComplexityLevel::Simple,
            num_steps,
            1.0, // 100% success rate
            60,
        );

        episode.complete(TaskOutcome::Success {
            verdict: "Perfect".to_string(),
            artifacts: vec![],
        });

        let reward = calculator.calculate(&episode);

        // Zero errors should give quality bonus
        prop_assert!(
            reward.quality_multiplier >= 1.0,
            "Zero errors should give quality >= 1.0, got {}",
            reward.quality_multiplier
        );
    }

    /// Test quality bonus for test coverage artifacts
    #[test]
    fn quality_bonus_for_test_coverage(
        coverage in 60.0f32..100.0f32,
    ) {
        let calculator = RewardCalculator::new();
        let mut episode = create_episode_with_params(
            ComplexityLevel::Moderate,
            10,
            1.0,
            60,
        );

        episode.metadata.insert(
            "test_coverage".to_string(),
            coverage.to_string(),
        );

        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec!["coverage.txt".to_string()],
        });

        let reward = calculator.calculate(&episode);

        // High coverage should give quality bonus
        prop_assert!(
            reward.quality_multiplier > 1.0,
            "Coverage {}% should give quality > 1.0, got {}",
            coverage,
            reward.quality_multiplier
        );
    }
}

// ============================================================================
// Learning Bonus Property Tests
// ============================================================================

proptest! {
    /// Test that learning bonus is always within bounds [0.0, 0.5]
    #[test]
    fn learning_bonus_always_within_bounds(
        num_steps in 0usize..100usize,
        success_rate in 0.0f32..1.0f32,
        num_patterns in 0usize..10usize,
    ) {
        let calculator = RewardCalculator::new();
        let mut episode = create_episode_with_params(
            ComplexityLevel::Moderate,
            num_steps,
            success_rate,
            60,
        );

        // Add patterns (simulated)
        for i in 0..num_patterns {
            episode.patterns.push(format!("pattern_{i}").into());
        }

        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let reward = calculator.calculate(&episode);

        prop_assert!(
            reward.learning_bonus >= 0.0,
            "Learning bonus {} is below minimum 0.0",
            reward.learning_bonus
        );

        prop_assert!(
            reward.learning_bonus <= MAX_LEARNING_BONUS,
            "Learning bonus {} exceeds maximum {}",
            reward.learning_bonus,
            MAX_LEARNING_BONUS
        );
    }

    /// Test learning bonus for pattern discovery
    #[test]
    fn learning_bonus_increases_with_patterns(
        num_patterns_low in 0usize..2usize,
        num_patterns_high in 5usize..10usize,
    ) {
        let calculator = RewardCalculator::new();

        // Episode with few patterns
        let mut episode_low = create_episode_with_params(
            ComplexityLevel::Moderate,
            10,
            1.0,
            60,
        );
        for i in 0..num_patterns_low {
            episode_low.patterns.push(format!("pattern_{i}").into());
        }
        episode_low.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        // Episode with many patterns
        let mut episode_high = create_episode_with_params(
            ComplexityLevel::Moderate,
            10,
            1.0,
            60,
        );
        for i in 0..num_patterns_high {
            episode_high.patterns.push(format!("pattern_{i}").into());
        }
        episode_high.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let reward_low = calculator.calculate(&episode_low);
        let reward_high = calculator.calculate(&episode_high);

        // More patterns should give more learning bonus
        prop_assert!(
            reward_high.learning_bonus >= reward_low.learning_bonus,
            "More patterns ({} vs {}) should give >= learning bonus. Got {} vs {}",
            num_patterns_high,
            num_patterns_low,
            reward_high.learning_bonus,
            reward_low.learning_bonus
        );
    }

    /// Test learning bonus for tool diversity (novelty)
    #[test]
    fn learning_bonus_for_tool_diversity(
        num_unique_tools in 3usize..10usize,
    ) {
        let calculator = RewardCalculator::new();

        // Create episode with diverse tools
        let tools: Vec<String> = (0..num_unique_tools)
            .map(|i| format!("unique_tool_{i}"))
            .collect();

        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: None,
            complexity: ComplexityLevel::Moderate,
            domain: "test-domain".to_string(),
            tags: vec![],
        };

        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Testing);

        for (i, tool) in tools.iter().enumerate() {
            let mut step = ExecutionStep::new(i + 1, tool.clone(), "Action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::Success {
            verdict: "Diverse".to_string(),
            artifacts: vec![],
        });

        let reward = calculator.calculate(&episode);

        // Tool diversity should contribute to learning bonus
        if num_unique_tools >= 5 {
            prop_assert!(
                reward.learning_bonus > 0.0,
                "Tool diversity ({} unique) should give learning bonus > 0, got {}",
                num_unique_tools,
                reward.learning_bonus
            );
        }
    }

    /// Test learning bonus for error recovery
    #[test]
    fn learning_bonus_for_error_recovery() {
        let calculator = RewardCalculator::new();

        // Episode with error recovery pattern
        let tools = vec!["tool1", "tool2", "tool3", "tool4"];
        let results = vec![false, true, false, true]; // Error -> Success pattern
        let mut episode = create_episode_with_tools(&tools, &results);

        episode.complete(TaskOutcome::Success {
            verdict: "Recovered".to_string(),
            artifacts: vec![],
        });

        let reward = calculator.calculate(&episode);

        // Error recovery should give learning bonus
        prop_assert!(
            reward.learning_bonus > 0.0,
            "Error recovery should give learning bonus > 0, got {}",
            reward.learning_bonus
        );
    }

    /// Test learning bonus cap
    #[test]
    fn learning_bonus_capped_at_maximum(
        num_patterns in 10usize..50usize,
        num_steps in 20usize..50usize,
    ) {
        let calculator = RewardCalculator::new();

        // Create episode with maximum potential learning bonus
        let mut episode = create_episode_with_params(
            ComplexityLevel::Complex,
            num_steps,
            1.0, // Perfect success rate
            15,  // Very fast (optimization bonus)
        );

        // Add many patterns
        for i in 0..num_patterns {
            episode.patterns.push(format!("pattern_{i}").into());
        }

        episode.complete(TaskOutcome::Success {
            verdict: "Maximum".to_string(),
            artifacts: vec![],
        });

        let reward = calculator.calculate(&episode);

        prop_assert!(
            reward.learning_bonus <= MAX_LEARNING_BONUS,
            "Learning bonus {} should be capped at {}",
            reward.learning_bonus,
            MAX_LEARNING_BONUS
        );
    }
}

// ============================================================================
// Complexity Bonus Property Tests
// ============================================================================

proptest! {
    /// Test complexity bonus is always within bounds [1.0, 1.2]
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

        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let reward = calculator.calculate(&episode);

        prop_assert!(
            reward.complexity_bonus >= MIN_COMPLEXITY_BONUS,
            "Complexity bonus {} is below minimum {}",
            reward.complexity_bonus,
            MIN_COMPLEXITY_BONUS
        );

        prop_assert!(
            reward.complexity_bonus <= MAX_COMPLEXITY_BONUS,
            "Complexity bonus {} exceeds maximum {}",
            reward.complexity_bonus,
            MAX_COMPLEXITY_BONUS
        );
    }

    /// Test complexity bonus values match expected levels
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

        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let reward = calculator.calculate(&episode);

        let expected = match complexity {
            ComplexityLevel::Simple => 1.0,
            ComplexityLevel::Moderate => 1.1,
            ComplexityLevel::Complex => 1.2,
        };

        prop_assert!(
            (reward.complexity_bonus - expected).abs() < 0.001,
            "Complexity {:?} should give bonus {}, got {}",
            complexity,
            expected,
            reward.complexity_bonus
        );
    }

    /// Test complexity ordering: Complex > Moderate > Simple
    #[test]
    fn complexity_bonus_ordering() {
        let calculator = RewardCalculator::new();

        let mut simple = create_episode_with_params(ComplexityLevel::Simple, 5, 1.0, 60);
        simple.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let mut moderate = create_episode_with_params(ComplexityLevel::Moderate, 5, 1.0, 60);
        moderate.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let mut complex = create_episode_with_params(ComplexityLevel::Complex, 5, 1.0, 60);
        complex.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let reward_simple = calculator.calculate(&simple);
        let reward_moderate = calculator.calculate(&moderate);
        let reward_complex = calculator.calculate(&complex);

        prop_assert!(
            reward_simple.complexity_bonus < reward_moderate.complexity_bonus,
            "Simple ({}) should be < Moderate ({})",
            reward_simple.complexity_bonus,
            reward_moderate.complexity_bonus
        );

        prop_assert!(
            reward_moderate.complexity_bonus < reward_complex.complexity_bonus,
            "Moderate ({}) should be < Complex ({})",
            reward_moderate.complexity_bonus,
            reward_complex.complexity_bonus
        );
    }
}

// ============================================================================
// Total Reward Property Tests
// ============================================================================

proptest! {
    /// Test that total reward is calculated correctly
    /// total = base * efficiency * complexity_bonus * quality_multiplier + learning_bonus
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

        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let reward = calculator.calculate(&episode);

        // Verify calculation matches formula
        let expected_total = (reward.base
            * reward.efficiency
            * reward.complexity_bonus
            * reward.quality_multiplier)
            + reward.learning_bonus;

        prop_assert!(
            (reward.total - expected_total).abs() < 0.001,
            "Total {} should equal calculation {}, difference: {}",
            reward.total,
            expected_total,
            (reward.total - expected_total).abs()
        );
    }

    /// Test total reward is non-negative for successful episodes
    #[test]
    fn total_reward_non_negative_for_success(
        num_steps in 1usize..100usize,
        duration_secs in 0i64..3600i64,
    ) {
        let calculator = RewardCalculator::new();
        let mut episode = create_episode_with_params(
            ComplexityLevel::Moderate,
            num_steps,
            1.0,
            duration_secs,
        );

        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let reward = calculator.calculate(&episode);

        prop_assert!(
            reward.total >= 0.0,
            "Total reward should be non-negative for success, got {}",
            reward.total
        );
    }

    /// Test total reward is zero for failure
    #[test]
    fn total_reward_zero_for_failure() {
        let calculator = RewardCalculator::new();
        let mut episode = create_episode_with_params(ComplexityLevel::Simple, 5, 1.0, 60);

        episode.complete(TaskOutcome::Failure {
            reason: "Failed".to_string(),
            error_details: None,
        });

        let reward = calculator.calculate(&episode);

        prop_assert_eq!(
            reward.base, 0.0,
            "Base should be 0 for failure"
        );

        prop_assert_eq!(
            reward.total, 0.0,
            "Total should be 0 for failure (base * ... = 0)"
        );
    }
}

// ============================================================================
// Edge Cases Property Tests
// ============================================================================

proptest! {
    /// Test episode with zero steps
    #[test]
    fn edge_case_zero_steps() {
        let calculator = RewardCalculator::new();
        let mut episode = create_episode_with_params(ComplexityLevel::Simple, 0, 1.0, 0);

        episode.complete(TaskOutcome::Success {
            verdict: "No steps".to_string(),
            artifacts: vec![],
        });

        let reward = calculator.calculate(&episode);

        // Zero steps should still produce valid bounds
        prop_assert!(
            reward.efficiency >= MIN_EFFICIENCY && reward.efficiency <= MAX_EFFICIENCY,
            "Zero steps efficiency {} out of bounds",
            reward.efficiency
        );
    }

    /// Test episode with maximum reasonable values
    #[test]
    fn edge_case_max_values(
        num_steps in 900usize..1000usize,
        duration_secs in 82800i64..86400i64, // ~23-24 hours
    ) {
        let calculator = RewardCalculator::new();
        let mut episode = create_episode_with_params(
            ComplexityLevel::Complex,
            num_steps,
            0.1, // Low success rate
            duration_secs,
        );

        episode.complete(TaskOutcome::Success {
            verdict: "Maximum".to_string(),
            artifacts: vec![],
        });

        let reward = calculator.calculate(&episode);

        // All values should still be within bounds
        prop_assert!(reward.efficiency >= MIN_EFFICIENCY && reward.efficiency <= MAX_EFFICIENCY);
        prop_assert!(reward.quality_multiplier >= MIN_QUALITY && reward.quality_multiplier <= MAX_QUALITY);
        prop_assert!(reward.learning_bonus >= 0.0 && reward.learning_bonus <= MAX_LEARNING_BONUS);
        prop_assert!(reward.complexity_bonus >= MIN_COMPLEXITY_BONUS && reward.complexity_bonus <= MAX_COMPLEXITY_BONUS);
    }

    /// Test partial success with various ratios
    #[test]
    fn edge_case_partial_success(
        completed in 0usize..10usize,
        failed in 0usize..10usize,
    ) {
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
            prop_assert_eq!(reward.base, 0.5, "Empty lists should give 0.5 base");
        } else {
            let expected_base = completed as f32 / total as f32;
            prop_assert!(
                (reward.base - expected_base).abs() < 0.001,
                "Base {} should equal {}/{} = {}",
                reward.base,
                completed,
                total,
                expected_base
            );
        }
    }

    /// Test incomplete episode (no outcome)
    #[test]
    fn edge_case_incomplete_episode(
        num_steps in 0usize..50usize,
    ) {
        let calculator = RewardCalculator::new();
        let episode = create_episode_with_params(ComplexityLevel::Moderate, num_steps, 1.0, 60);
        // Don't complete the episode

        let reward = calculator.calculate(&episode);

        prop_assert_eq!(
            reward.base, 0.0,
            "Incomplete episode should have 0 base"
        );

        prop_assert_eq!(
            reward.total, 0.0,
            "Incomplete episode should have 0 total"
        );
    }
}

// ============================================================================
// Adaptive Reward Calculator Property Tests
// ============================================================================

proptest! {
    /// Test adaptive calculator efficiency bounds with domain statistics
    #[test]
    fn adaptive_efficiency_bounds(
        episode_steps in 1usize..100usize,
        p50_steps in 5usize..50usize,
        p50_duration in 10.0f32..300.0f32,
    ) {
        use memory_core::DomainStatistics;

        let calculator = AdaptiveRewardCalculator::new();
        let mut episode = create_episode_with_params(
            ComplexityLevel::Moderate,
            episode_steps,
            1.0,
            60,
        );

        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let stats = DomainStatistics {
            domain: "test-domain".to_string(),
            episode_count: 10, // Reliable
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

        prop_assert!(
            reward.efficiency >= MIN_EFFICIENCY && reward.efficiency <= MAX_EFFICIENCY,
            "Adaptive efficiency {} out of bounds",
            reward.efficiency
        );
    }

    /// Test adaptive calculator fallback behavior
    #[test]
    fn adaptive_fallback_with_unreliable_stats(
        episode_steps in 1usize..50usize,
    ) {
        use memory_core::DomainStatistics;

        let calculator = AdaptiveRewardCalculator::new();
        let mut episode = create_episode_with_params(
            ComplexityLevel::Simple,
            episode_steps,
            1.0,
            60,
        );

        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        // Unreliable stats (< 5 episodes)
        let stats = DomainStatistics {
            domain: "test-domain".to_string(),
            episode_count: 3, // Unreliable
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

        // Should still produce valid bounds with fallback
        prop_assert!(
            reward.efficiency >= MIN_EFFICIENCY && reward.efficiency <= MAX_EFFICIENCY,
            "Fallback efficiency {} out of bounds",
            reward.efficiency
        );
    }

    /// Test adaptive calculator with None stats (fallback)
    #[test]
    fn adaptive_with_none_stats(
        episode_steps in 1usize..50usize,
    ) {
        let calculator = AdaptiveRewardCalculator::new();
        let mut episode = create_episode_with_params(
            ComplexityLevel::Moderate,
            episode_steps,
            1.0,
            60,
        );

        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let reward = calculator.calculate(&episode, None);

        // Should produce valid bounds with fallback
        prop_assert!(reward.efficiency >= MIN_EFFICIENCY && reward.efficiency <= MAX_EFFICIENCY);
        prop_assert!(reward.quality_multiplier >= MIN_QUALITY && reward.quality_multiplier <= MAX_QUALITY);
        prop_assert!(reward.learning_bonus >= 0.0 && reward.learning_bonus <= MAX_LEARNING_BONUS);
    }
}

// ============================================================================
// Weight Configuration Property Tests
// ============================================================================

proptest! {
    /// Test custom weights affect efficiency calculation
    #[test]
    fn custom_weights_affect_efficiency(
        duration_weight in 0.0f32..1.0f32,
        step_count_weight in 0.0f32..1.0f32,
    ) {
        // Normalize weights to sum to 1.0
        let total = duration_weight + step_count_weight;
        let duration_weight = if total > 0.0 { duration_weight / total } else { 0.5 };
        let step_count_weight = if total > 0.0 { step_count_weight / total } else { 0.5 };

        let calculator = RewardCalculator::with_weights(duration_weight, step_count_weight);
        let mut episode = create_episode_with_params(
            ComplexityLevel::Simple,
            20,
            1.0,
            60,
        );

        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let reward = calculator.calculate(&episode);

        // Custom weights should still produce valid bounds
        prop_assert!(
            reward.efficiency >= MIN_EFFICIENCY && reward.efficiency <= MAX_EFFICIENCY,
            "Custom weights efficiency {} out of bounds",
            reward.efficiency
        );
    }

    /// Test adaptive calculator with custom configuration
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

        let mut episode = create_episode_with_params(
            ComplexityLevel::Simple,
            fallback_steps / 2, // Half of fallback
            1.0,
            fallback_duration as i64 / 2,
        );

        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let reward = calculator.calculate(&episode, None);

        // Custom config should still produce valid bounds
        prop_assert!(reward.efficiency >= MIN_EFFICIENCY && reward.efficiency <= MAX_EFFICIENCY);
    }
}
