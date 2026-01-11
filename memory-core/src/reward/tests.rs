//! Tests for the reward module.

use super::*;
use crate::episode::ExecutionStep;
use crate::types::{ComplexityLevel, ExecutionResult, TaskContext, TaskType};

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
    use uuid::Uuid;
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
    use uuid::Uuid;
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
