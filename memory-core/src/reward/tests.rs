//! Tests for the reward module.

use super::*;
use crate::episode::ExecutionStep;
use crate::types::{ComplexityLevel, ExecutionResult, TaskContext, TaskType};
use uuid::Uuid;

fn create_test_episode(complexity: ComplexityLevel) -> Episode {
    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: None,
        complexity,
        domain: "testing".to_string(),
        tags: vec![],
    };

    Episode::new("Test task".to_string(), context, TaskType::Testing)
}

#[test]
fn test_successful_episode_reward() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Moderate);

    episode.complete(TaskOutcome::Success {
        verdict: "All tests passed".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    assert_eq!(reward.base, 1.0);
    assert!(reward.efficiency > 0.0);
    assert_eq!(reward.complexity_bonus, 1.1); // Moderate complexity
    assert!(reward.quality_multiplier > 0.0);
    assert!(reward.learning_bonus >= 0.0);
    assert!(reward.total > 0.0);
}

#[test]
fn test_failed_episode_reward() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    episode.complete(TaskOutcome::Failure {
        reason: "Tests failed".to_string(),
        error_details: None,
    });

    let reward = calculator.calculate(&episode);

    assert_eq!(reward.base, 0.0);
    assert_eq!(reward.total, 0.0); // Base is 0, so total is 0
}

#[test]
fn test_partial_success_reward() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Moderate);

    episode.complete(TaskOutcome::PartialSuccess {
        verdict: "Some tests passed".to_string(),
        completed: vec!["test1".to_string(), "test2".to_string()],
        failed: vec!["test3".to_string()],
    });

    let reward = calculator.calculate(&episode);

    // 2 out of 3 succeeded = 0.667
    assert!((reward.base - 0.667).abs() < 0.01);
    assert!(reward.total > 0.0);
}

#[test]
fn test_efficiency_fast_execution() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    // Add just a few steps
    for i in 0..3 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "Quick completion".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // Should have high efficiency (few steps, fast completion)
    assert!(reward.efficiency > 1.0);
}

#[test]
fn test_efficiency_slow_execution() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    // Simulate episode that started 5 minutes ago
    episode.start_time = chrono::Utc::now() - chrono::Duration::minutes(5);

    // Add many steps (more than efficient threshold)
    for i in 0..50 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "Slow completion".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // Should have lower efficiency (long duration + many steps)
    assert!(
        reward.efficiency < 1.0,
        "Expected efficiency < 1.0, got {}",
        reward.efficiency
    );
}

#[test]
fn test_complexity_bonus() {
    let calculator = RewardCalculator::new();

    let mut simple = create_test_episode(ComplexityLevel::Simple);
    simple.complete(TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });

    let mut moderate = create_test_episode(ComplexityLevel::Moderate);
    moderate.complete(TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });

    let mut complex = create_test_episode(ComplexityLevel::Complex);
    complex.complete(TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });

    let simple_reward = calculator.calculate(&simple);
    let moderate_reward = calculator.calculate(&moderate);
    let complex_reward = calculator.calculate(&complex);

    assert_eq!(simple_reward.complexity_bonus, 1.0);
    assert_eq!(moderate_reward.complexity_bonus, 1.1);
    assert_eq!(complex_reward.complexity_bonus, 1.2);

    // More complex tasks should have higher total rewards (all else equal)
    assert!(complex_reward.total > moderate_reward.total);
    assert!(moderate_reward.total > simple_reward.total);
}

#[test]
fn test_custom_weights() {
    // Heavily weight duration
    let calculator = RewardCalculator::with_weights(0.9, 0.1);
    let mut episode = create_test_episode(ComplexityLevel::Moderate);

    episode.complete(TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);
    assert!(reward.total > 0.0);
}

#[test]
fn test_incomplete_episode() {
    let calculator = RewardCalculator::new();
    let episode = create_test_episode(ComplexityLevel::Moderate);

    // Episode not completed
    let reward = calculator.calculate(&episode);

    assert_eq!(reward.base, 0.0);
    assert_eq!(reward.total, 0.0);
}

#[test]
fn test_quality_multiplier_with_test_coverage() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Moderate);

    // Add test coverage metadata
    episode
        .metadata
        .insert("test_coverage".to_string(), "85.5".to_string());

    episode.complete(TaskOutcome::Success {
        verdict: "Tests passed with coverage".to_string(),
        artifacts: vec!["coverage_report.html".to_string()],
    });

    let reward = calculator.calculate(&episode);

    // Should have quality bonus for high coverage
    assert!(reward.quality_multiplier > 1.0);
}

#[test]
fn test_quality_multiplier_with_zero_errors() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    // Add all successful steps
    for i in 0..5 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "Perfect execution".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // Zero errors should give quality bonus
    assert!(reward.quality_multiplier >= 1.0);
}

#[test]
fn test_quality_multiplier_with_high_error_rate() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    // Add many failed steps
    for i in 0..10 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        if i < 7 {
            step.result = Some(ExecutionResult::Error {
                message: "Error".to_string(),
            });
        } else {
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
        }
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "Eventually succeeded".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // High error rate should penalize quality
    assert!(reward.quality_multiplier < 1.0);
}

#[test]
fn test_learning_bonus_with_patterns() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Moderate);

    // Add some successful steps
    for i in 0..5 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    // Add pattern IDs to simulate pattern discovery
    episode.patterns.push(Uuid::new_v4());
    episode.patterns.push(Uuid::new_v4());

    episode.complete(TaskOutcome::Success {
        verdict: "Learned new patterns".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // Should have learning bonus for pattern discovery
    assert!(reward.learning_bonus > 0.0);
}

#[test]
fn test_learning_bonus_for_error_recovery() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Moderate);

    // Add error followed by recovery
    let mut error_step =
        ExecutionStep::new(1, "failing_tool".to_string(), "Failed action".to_string());
    error_step.result = Some(ExecutionResult::Error {
        message: "Error".to_string(),
    });
    episode.add_step(error_step);

    let mut recovery_step = ExecutionStep::new(
        2,
        "recovery_tool".to_string(),
        "Recovery action".to_string(),
    );
    recovery_step.result = Some(ExecutionResult::Success {
        output: "Recovered".to_string(),
    });
    episode.add_step(recovery_step);

    episode.complete(TaskOutcome::Success {
        verdict: "Recovered from error".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // Should have learning bonus for error recovery
    assert!(reward.learning_bonus > 0.0);
}

#[test]
fn test_learning_bonus_for_diverse_tools() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Complex);

    // Add many different tools (diverse approach)
    for i in 0..6 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "Used diverse toolset".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // Should have learning bonus for tool diversity
    assert!(reward.learning_bonus > 0.0);
}

#[test]
fn test_learning_bonus_for_efficient_execution() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    // Add successful steps with perfect execution
    for i in 0..5 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "Perfect execution".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // Should have learning bonus for high success rate
    assert!(reward.learning_bonus > 0.0);
}

#[test]
fn test_combined_quality_and_learning_bonuses() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Complex);

    // Add high-quality execution
    for i in 0..7 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    // Add quality indicators
    episode
        .metadata
        .insert("test_coverage".to_string(), "90.0".to_string());
    episode
        .metadata
        .insert("clippy_warnings".to_string(), "0".to_string());

    // Add patterns
    episode.patterns.push(Uuid::new_v4());

    episode.complete(TaskOutcome::Success {
        verdict: "High quality with learning".to_string(),
        artifacts: vec![
            "tests.rs".to_string(),
            "coverage.html".to_string(),
            "docs.md".to_string(),
        ],
    });

    let reward = calculator.calculate(&episode);

    // Should have both quality and learning bonuses
    assert!(reward.quality_multiplier > 1.0);
    assert!(reward.learning_bonus > 0.0);
    assert!(reward.total > reward.base);
}

// ============================================================================
// ACT-026: Additional tests for episode lifecycle and reward calculation
// ============================================================================

/// Test efficiency multiplier at minimum bound (zero steps)
#[test]
fn test_efficiency_minimum_bound_zero_steps() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    // No steps added - should result in minimum step count efficiency
    // But instant completion gives max duration efficiency
    episode.complete(TaskOutcome::Success {
        verdict: "Instant task".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // Zero steps = MIN step efficiency (0.5)
    // Instant completion = MAX duration efficiency (1.5)
    // Combined = (0.5 * 0.5) + (1.5 * 0.5) = 1.0
    assert!(
        reward.efficiency >= 0.5 && reward.efficiency <= 1.5,
        "Expected efficiency in valid range, got {}",
        reward.efficiency
    );
}

/// Test efficiency multiplier with maximum efficient execution (instant)
#[test]
fn test_efficiency_maximum_instant_completion() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    // Add minimal steps with instant completion (duration = 0)
    let step = ExecutionStep::new(1, "instant_tool".to_string(), "Instant action".to_string());
    episode.add_step(step);

    // Complete immediately (no time elapsed)
    episode.complete(TaskOutcome::Success {
        verdict: "Instant".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // Should have high efficiency for instant completion
    assert!(
        reward.efficiency > 1.0,
        "Expected high efficiency for instant completion, got {}",
        reward.efficiency
    );
}

/// Test efficiency multiplier clamping at maximum
#[test]
fn test_efficiency_clamped_at_maximum() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    // Add just 1 step with instant completion to maximize efficiency
    let mut step = ExecutionStep::new(1, "fast_tool".to_string(), "Fast action".to_string());
    step.result = Some(ExecutionResult::Success {
        output: "OK".to_string(),
    });
    episode.add_step(step);

    episode.complete(TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // Efficiency should be clamped to max 1.5
    assert!(
        reward.efficiency <= 1.5,
        "Efficiency should not exceed 1.5, got {}",
        reward.efficiency
    );
}

/// Test efficiency multiplier clamping at minimum for very slow execution
#[test]
fn test_efficiency_clamped_at_minimum() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    // Simulate very long episode (1 hour)
    episode.start_time = chrono::Utc::now() - chrono::Duration::hours(1);

    // Add many steps (100+)
    for i in 0..100 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "Very slow".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // Efficiency should be clamped to min 0.5
    assert!(
        reward.efficiency >= 0.5,
        "Efficiency should not go below 0.5, got {}",
        reward.efficiency
    );
}

/// Test quality multiplier with test coverage at 80% threshold
#[test]
fn test_quality_multiplier_coverage_at_80_threshold() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    // Coverage exactly at 80% threshold
    episode
        .metadata
        .insert("test_coverage".to_string(), "80.0".to_string());

    episode.complete(TaskOutcome::Success {
        verdict: "Coverage at threshold".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // Coverage > 80% should get bonus
    assert!(
        reward.quality_multiplier > 1.0,
        "Expected quality bonus for coverage > 80%, got {}",
        reward.quality_multiplier
    );
}

/// Test quality multiplier with test coverage just below 80%
#[test]
fn test_quality_multiplier_coverage_below_80_threshold() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    // Coverage just below 80% threshold but above 60%
    episode
        .metadata
        .insert("test_coverage".to_string(), "79.9".to_string());

    episode.complete(TaskOutcome::Success {
        verdict: "Coverage below threshold".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // Coverage between 60-80% should get smaller bonus
    // The bonus is +0.1 for coverage > 60%, so quality should be > 1.0
    assert!(
        reward.quality_multiplier > 1.0,
        "Expected quality bonus for coverage > 60%, got {}",
        reward.quality_multiplier
    );
}

/// Test quality multiplier with test coverage at 60% threshold
#[test]
fn test_quality_multiplier_coverage_at_60_threshold() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    // Coverage exactly at 60% threshold (doesn't trigger bonus, needs > 60%)
    episode
        .metadata
        .insert("test_coverage".to_string(), "60.0".to_string());

    episode.complete(TaskOutcome::Success {
        verdict: "Coverage at 60%".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // Coverage exactly at 60% doesn't get bonus (needs > 60%)
    // But might get bonus from other factors like zero errors
    assert!(
        reward.quality_multiplier >= 0.5,
        "Quality should be in valid range, got {}",
        reward.quality_multiplier
    );
}

/// Test quality multiplier with test coverage just above 60%
#[test]
fn test_quality_multiplier_coverage_above_60_threshold() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    // Coverage just above 60% threshold
    episode
        .metadata
        .insert("test_coverage".to_string(), "61.0".to_string());

    episode.complete(TaskOutcome::Success {
        verdict: "Coverage above 60%".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // Coverage > 60% should get bonus
    assert!(
        reward.quality_multiplier > 1.0,
        "Expected quality bonus for coverage > 60%, got {}",
        reward.quality_multiplier
    );
}

/// Test quality multiplier with test coverage below 60%
#[test]
fn test_quality_multiplier_coverage_below_60_threshold() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    // Coverage below 60%
    episode
        .metadata
        .insert("test_coverage".to_string(), "50.0".to_string());

    episode.complete(TaskOutcome::Success {
        verdict: "Low coverage".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // Coverage < 60% should not get bonus from coverage alone
    // But might get bonus from other factors
    assert!(
        reward.quality_multiplier >= 0.5,
        "Quality should be at least 0.5, got {}",
        reward.quality_multiplier
    );
}

/// Test quality multiplier with invalid test coverage value
#[test]
fn test_quality_multiplier_invalid_coverage_value() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    // Invalid coverage value
    episode
        .metadata
        .insert("test_coverage".to_string(), "invalid".to_string());

    episode.complete(TaskOutcome::Success {
        verdict: "Invalid coverage".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // Should not crash, quality should be in valid range
    assert!(
        reward.quality_multiplier >= 0.5 && reward.quality_multiplier <= 1.5,
        "Quality should be in valid range, got {}",
        reward.quality_multiplier
    );
}

/// Test quality multiplier with clippy_warnings = "0"
#[test]
fn test_quality_multiplier_zero_clippy_warnings() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    episode
        .metadata
        .insert("clippy_warnings".to_string(), "0".to_string());

    episode.complete(TaskOutcome::Success {
        verdict: "Clean code".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // Zero clippy warnings should give quality bonus
    assert!(
        reward.quality_multiplier > 1.0,
        "Expected quality bonus for zero clippy warnings, got {}",
        reward.quality_multiplier
    );
}

/// Test quality multiplier with non-zero clippy warnings
#[test]
fn test_quality_multiplier_nonzero_clippy_warnings() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    episode
        .metadata
        .insert("clippy_warnings".to_string(), "5".to_string());

    episode.complete(TaskOutcome::Success {
        verdict: "Some warnings".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // Non-zero clippy warnings should not give bonus
    // Quality should be 1.0 base
    assert!(
        reward.quality_multiplier >= 0.5,
        "Quality should be at least 0.5, got {}",
        reward.quality_multiplier
    );
}

/// Test quality multiplier clamping at maximum
#[test]
fn test_quality_multiplier_clamped_at_maximum() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Complex);

    // Add multiple quality bonuses
    episode
        .metadata
        .insert("test_coverage".to_string(), "95.0".to_string()); // +0.15
    episode
        .metadata
        .insert("clippy_warnings".to_string(), "0".to_string()); // +0.05

    // Add successful steps for zero error bonus
    for i in 0..10 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "High quality".to_string(),
        artifacts: vec![
            "test.rs".to_string(),
            "coverage.html".to_string(),
            "docs.md".to_string(),
        ], // +0.05 for 3+ artifacts, +0.1 for test/coverage keyword
    });

    let reward = calculator.calculate(&episode);

    // Quality should be clamped to max 1.5
    assert!(
        reward.quality_multiplier <= 1.5,
        "Quality should not exceed 1.5, got {}",
        reward.quality_multiplier
    );
}

/// Test quality multiplier clamping at minimum
#[test]
fn test_quality_multiplier_clamped_at_minimum() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    // Add many failed steps for high error rate penalty
    for i in 0..10 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        if i < 9 {
            // 90% error rate
            step.result = Some(ExecutionResult::Error {
                message: "Error".to_string(),
            });
        } else {
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
        }
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "Many errors".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // Quality should be clamped to min 0.5
    assert!(
        reward.quality_multiplier >= 0.5,
        "Quality should not go below 0.5, got {}",
        reward.quality_multiplier
    );
}

/// Test quality multiplier with error rate at boundary (30%)
#[test]
fn test_quality_multiplier_error_rate_at_30_percent() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    // 3 errors out of 10 = 30% error rate (at boundary)
    for i in 0..10 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        if i < 3 {
            step.result = Some(ExecutionResult::Error {
                message: "Error".to_string(),
            });
        } else {
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
        }
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "30% error rate".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // Error rate > 30% gets -0.2 penalty, 30% is not > 30%
    // So error rate > 10% should get -0.1 penalty
    assert!(
        reward.quality_multiplier < 1.0,
        "Expected quality penalty for error rate > 10%, got {}",
        reward.quality_multiplier
    );
}

/// Test quality multiplier with error rate just above 10%
#[test]
fn test_quality_multiplier_error_rate_above_10_percent() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    // 2 errors out of 10 = 20% error rate
    for i in 0..10 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        if i < 2 {
            step.result = Some(ExecutionResult::Error {
                message: "Error".to_string(),
            });
        } else {
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
        }
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "20% error rate".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // Error rate between 10-30% should get -0.1 penalty
    assert!(
        reward.quality_multiplier < 1.0,
        "Expected quality penalty for error rate > 10%, got {}",
        reward.quality_multiplier
    );
}

/// Test quality multiplier with error rate at 10% boundary
#[test]
fn test_quality_multiplier_error_rate_at_10_percent() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    // 1 error out of 10 = 10% error rate (at boundary)
    for i in 0..10 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        if i == 0 {
            step.result = Some(ExecutionResult::Error {
                message: "Error".to_string(),
            });
        } else {
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
        }
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "10% error rate".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // Error rate > 10% gets penalty, 10% is not > 10%
    // So no penalty, but also no zero-error bonus
    assert!(
        reward.quality_multiplier >= 1.0,
        "Expected no penalty for error rate <= 10%, got {}",
        reward.quality_multiplier
    );
}

/// Test TaskOutcome::Success with multiple artifacts
#[test]
fn test_task_outcome_success_with_multiple_artifacts() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Complex);

    // Add successful steps
    for i in 0..5 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "Complete success with artifacts".to_string(),
        artifacts: vec![
            "main.rs".to_string(),
            "test.rs".to_string(),
            "lib.rs".to_string(),
            "mod.rs".to_string(),
        ],
    });

    let reward = calculator.calculate(&episode);

    assert_eq!(reward.base, 1.0);
    // 4 artifacts >= 3, so should get bonus
    assert!(
        reward.quality_multiplier > 1.0,
        "Expected quality bonus for 3+ artifacts, got {}",
        reward.quality_multiplier
    );
}

/// Test TaskOutcome::Success with test-related artifacts
#[test]
fn test_task_outcome_success_with_test_artifacts() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    episode.complete(TaskOutcome::Success {
        verdict: "Tests pass".to_string(),
        artifacts: vec![
            "test_coverage.html".to_string(),
            "test_results.xml".to_string(),
        ],
    });

    let reward = calculator.calculate(&episode);

    assert_eq!(reward.base, 1.0);
    // Artifacts containing "coverage" or "test" should get bonus
    assert!(
        reward.quality_multiplier > 1.0,
        "Expected quality bonus for test-related artifacts, got {}",
        reward.quality_multiplier
    );
}

/// Test TaskOutcome::PartialSuccess with empty completed and failed lists
#[test]
fn test_task_outcome_partial_success_empty_lists() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    episode.complete(TaskOutcome::PartialSuccess {
        verdict: "Nothing done".to_string(),
        completed: vec![],
        failed: vec![],
    });

    let reward = calculator.calculate(&episode);

    // Empty lists should give default 0.5 base reward
    assert_eq!(
        reward.base, 0.5,
        "Expected base 0.5 for partial success with empty lists, got {}",
        reward.base
    );
}

/// Test TaskOutcome::PartialSuccess with all completed
#[test]
fn test_task_outcome_partial_success_all_completed() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    episode.complete(TaskOutcome::PartialSuccess {
        verdict: "All completed".to_string(),
        completed: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        failed: vec![],
    });

    let reward = calculator.calculate(&episode);

    // All completed = 1.0 base
    assert_eq!(
        reward.base, 1.0,
        "Expected base 1.0 for all completed, got {}",
        reward.base
    );
}

/// Test TaskOutcome::PartialSuccess with all failed
#[test]
fn test_task_outcome_partial_success_all_failed() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    episode.complete(TaskOutcome::PartialSuccess {
        verdict: "All failed".to_string(),
        completed: vec![],
        failed: vec!["a".to_string(), "b".to_string()],
    });

    let reward = calculator.calculate(&episode);

    // All failed = 0.0 base
    assert_eq!(
        reward.base, 0.0,
        "Expected base 0.0 for all failed, got {}",
        reward.base
    );
}

/// Test TaskOutcome::Failure with error details
#[test]
fn test_task_outcome_failure_with_error_details() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    episode.complete(TaskOutcome::Failure {
        reason: "Compilation failed".to_string(),
        error_details: Some("error[E0425]: cannot find value `x` in this scope".to_string()),
    });

    let reward = calculator.calculate(&episode);

    assert_eq!(reward.base, 0.0);
    assert_eq!(reward.total, 0.0);
}

/// Test TaskOutcome::Failure without error details
#[test]
fn test_task_outcome_failure_without_error_details() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Complex);

    episode.complete(TaskOutcome::Failure {
        reason: "Unknown error".to_string(),
        error_details: None,
    });

    let reward = calculator.calculate(&episode);

    assert_eq!(reward.base, 0.0);
    assert_eq!(reward.total, 0.0);
}

/// Test learning bonus with maximum patterns (capped at 0.3)
#[test]
fn test_learning_bonus_max_patterns_capped() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Complex);

    // Add many patterns (should be capped at 0.3 bonus)
    for _ in 0..10 {
        episode.patterns.push(Uuid::new_v4());
    }

    // Add steps
    for i in 0..5 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "Many patterns".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // Pattern bonus should be capped at 0.3
    // But total learning bonus has other components too
    assert!(
        reward.learning_bonus > 0.0,
        "Expected learning bonus for patterns, got {}",
        reward.learning_bonus
    );
}

/// Test learning bonus with tool diversity at threshold (5 unique tools)
#[test]
fn test_learning_bonus_tool_diversity_at_threshold() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Complex);

    // Add exactly 5 unique tools (threshold for 0.15 bonus)
    for i in 0..5 {
        let mut step = ExecutionStep::new(i + 1, format!("unique_tool_{i}"), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "Diverse tools".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // 5 unique tools should give 0.15 novelty bonus
    assert!(
        reward.learning_bonus >= 0.15,
        "Expected at least 0.15 learning bonus for 5 unique tools, got {}",
        reward.learning_bonus
    );
}

/// Test learning bonus with tool diversity at 3 unique tools
#[test]
fn test_learning_bonus_tool_diversity_3_tools() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    // Add 3 unique tools (threshold for 0.1 bonus)
    for i in 0..3 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "Some diversity".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // 3 unique tools should give 0.1 novelty bonus
    assert!(
        reward.learning_bonus >= 0.1,
        "Expected at least 0.1 learning bonus for 3 unique tools, got {}",
        reward.learning_bonus
    );
}

/// Test learning bonus for efficient problem solving (< 30s, < 10 steps)
#[test]
fn test_learning_bonus_efficient_problem_solving() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    // Add few steps (< 10) that complete quickly
    for i in 0..5 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "Quick".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // Should have some learning bonus for efficient execution
    assert!(
        reward.learning_bonus >= 0.0,
        "Expected learning bonus for efficient execution, got {}",
        reward.learning_bonus
    );
}

/// Test learning bonus for high success rate with 5+ steps
#[test]
fn test_learning_bonus_high_success_rate() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Moderate);

    // Add 10 steps with 100% success rate
    for i in 0..10 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "Perfect".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // Should get bonus for high success rate with 5+ steps
    assert!(
        reward.learning_bonus > 0.0,
        "Expected learning bonus for high success rate, got {}",
        reward.learning_bonus
    );
}

/// Test learning bonus for perfect execution with 3-4 steps
#[test]
fn test_learning_bonus_perfect_execution_3_steps() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    // Add exactly 3 steps with perfect success
    for i in 0..3 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "Perfect small task".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // Perfect execution with 3+ steps should give bonus
    assert!(
        reward.learning_bonus > 0.0,
        "Expected learning bonus for perfect execution, got {}",
        reward.learning_bonus
    );
}

/// Test learning bonus capped at 0.5
#[test]
fn test_learning_bonus_capped_at_maximum() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Complex);

    // Add many patterns (capped at 0.3)
    for _ in 0..5 {
        episode.patterns.push(Uuid::new_v4());
    }

    // Add diverse tools (0.15 for 5+)
    for i in 0..6 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    // Add error recovery (0.15)
    let mut error_step = ExecutionStep::new(10, "fail".to_string(), "Fail".to_string());
    error_step.result = Some(ExecutionResult::Error {
        message: "Error".to_string(),
    });
    episode.add_step(error_step);

    let mut recovery_step = ExecutionStep::new(11, "recover".to_string(), "Recover".to_string());
    recovery_step.result = Some(ExecutionResult::Success {
        output: "OK".to_string(),
    });
    episode.add_step(recovery_step);

    episode.complete(TaskOutcome::Success {
        verdict: "Max bonuses".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // Learning bonus should be capped at 0.5
    assert!(
        reward.learning_bonus <= 0.5,
        "Learning bonus should not exceed 0.5, got {}",
        reward.learning_bonus
    );
}

/// Test duration efficiency with no duration (None)
#[test]
fn test_duration_efficiency_none() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    // Don't complete the episode - no duration
    // But we need to complete it to have an outcome
    episode.complete(TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });

    // The episode is completed instantly, so duration should be very small
    let reward = calculator.calculate(&episode);

    // Should have valid efficiency
    assert!(
        reward.efficiency > 0.0,
        "Expected valid efficiency, got {}",
        reward.efficiency
    );
}

/// Test with custom weights affecting efficiency calculation
#[test]
fn test_custom_weights_efficiency_calculation() {
    let calculator = RewardCalculator::with_weights(1.0, 0.0);
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    // Add steps
    for i in 0..5 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // With 100% weight on duration and 0% on steps, efficiency should reflect duration only
    assert!(
        reward.efficiency > 0.0 && reward.efficiency <= 1.5,
        "Expected valid efficiency with custom weights, got {}",
        reward.efficiency
    );
}

/// Test total reward formula calculation
#[test]
fn test_total_reward_formula_verification() {
    let calculator = RewardCalculator::new();
    let mut episode = create_test_episode(ComplexityLevel::Simple);

    // Create a simple episode
    for i in 0..3 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode);

    // Verify formula: total = base * efficiency * complexity_bonus * quality_multiplier + learning_bonus
    let expected_total =
        (reward.base * reward.efficiency * reward.complexity_bonus * reward.quality_multiplier)
            + reward.learning_bonus;

    assert!(
        (reward.total - expected_total).abs() < 0.01,
        "Total should match formula. Expected {}, got {}",
        expected_total,
        reward.total
    );
}
