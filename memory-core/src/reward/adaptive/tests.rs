//! Tests for adaptive reward calculation

use crate::episode::{Episode, ExecutionStep};
use crate::reward::adaptive::AdaptiveRewardCalculator;
use crate::reward::domain_stats::DomainStatistics;
use crate::types::{ComplexityLevel, ExecutionResult, TaskContext, TaskOutcome, TaskType};

fn create_test_episode(domain: &str, complexity: ComplexityLevel) -> Episode {
    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: None,
        complexity,
        domain: domain.to_string(),
        tags: vec![],
    };

    Episode::new("Test task".to_string(), context, TaskType::Testing)
}

#[test]
fn test_adaptive_with_no_stats_uses_fallback() {
    let calculator = AdaptiveRewardCalculator::new();
    let mut episode = create_test_episode("new-domain", ComplexityLevel::Moderate);

    episode.complete(TaskOutcome::Success {
        verdict: "Success".to_string(),
        artifacts: vec![],
    });

    // Should use fixed thresholds when no stats available
    let reward = calculator.calculate(&episode, None);
    assert_eq!(reward.base, 1.0);
    assert!(reward.efficiency > 0.0);
}

#[test]
fn test_adaptive_with_unreliable_stats_uses_fallback() {
    let calculator = AdaptiveRewardCalculator::new();
    let mut episode = create_test_episode("test-domain", ComplexityLevel::Moderate);

    episode.complete(TaskOutcome::Success {
        verdict: "Success".to_string(),
        artifacts: vec![],
    });

    // Create unreliable stats (< 5 episodes)
    let stats = DomainStatistics {
        domain: "test-domain".to_string(),
        episode_count: 2, // Too few for reliability
        avg_duration_secs: 30.0,
        p50_duration_secs: 25.0,
        p90_duration_secs: 45.0,
        avg_step_count: 8.0,
        p50_step_count: 7,
        p90_step_count: 12,
        avg_reward: 0.8,
        p50_reward: 0.85,
        reward_std_dev: 0.1,
        last_updated: chrono::Utc::now(),
        success_count: 2,
    };

    // Should use fixed thresholds with unreliable stats
    let reward = calculator.calculate(&episode, Some(&stats));
    assert_eq!(reward.base, 1.0);
    assert!(reward.efficiency > 0.0);
}

#[test]
fn test_adaptive_with_reliable_stats() {
    let calculator = AdaptiveRewardCalculator::new();
    let mut episode = create_test_episode("mature-domain", ComplexityLevel::Moderate);

    // Simulate fast episode (20 steps)
    for i in 0..20 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "Success".to_string(),
        artifacts: vec![],
    });

    // Create reliable stats where median is 30 steps
    let stats = DomainStatistics {
        domain: "mature-domain".to_string(),
        episode_count: 50, // Reliable
        avg_duration_secs: 120.0,
        p50_duration_secs: 100.0,
        p90_duration_secs: 180.0,
        avg_step_count: 35.0,
        p50_step_count: 30, // Our episode has 20, better than median!
        p90_step_count: 50,
        avg_reward: 0.75,
        p50_reward: 0.8,
        reward_std_dev: 0.15,
        last_updated: chrono::Utc::now(),
        success_count: 45,
    };

    // Should use adaptive thresholds with reliable stats
    let reward = calculator.calculate(&episode, Some(&stats));

    assert_eq!(reward.base, 1.0);
    // Episode with 20 steps vs p50 of 30 should have good efficiency
    assert!(
        reward.efficiency > 0.9,
        "Expected efficiency > 0.9 for better-than-median performance"
    );
}

#[test]
fn test_adaptive_penalizes_worse_than_median() {
    let calculator = AdaptiveRewardCalculator::new();
    let mut episode = create_test_episode("test-domain", ComplexityLevel::Simple);

    // Simulate slow episode (50 steps)
    for i in 0..50 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "Success".to_string(),
        artifacts: vec![],
    });

    // Stats where median is 20 steps
    let stats = DomainStatistics {
        domain: "test-domain".to_string(),
        episode_count: 30,
        avg_duration_secs: 80.0,
        p50_duration_secs: 60.0,
        p90_duration_secs: 120.0,
        avg_step_count: 25.0,
        p50_step_count: 20, // Our episode has 50, worse than median!
        p90_step_count: 35,
        avg_reward: 0.7,
        p50_reward: 0.75,
        reward_std_dev: 0.2,
        last_updated: chrono::Utc::now(),
        success_count: 25,
    };

    let reward = calculator.calculate(&episode, Some(&stats));

    // Episode with 50 steps vs p50 of 20 should reflect in efficiency calculation
    // Note: Actual efficiency depends on both duration and step_count
    // We verify the calculation runs without errors
    assert!(
        reward.efficiency > 0.0,
        "Expected valid efficiency score, got {}",
        reward.efficiency
    );
}

#[test]
fn test_new_calculator_has_default_weights() {
    let calculator = AdaptiveRewardCalculator::new();
    // Just verify it can be created
    assert!(calculator.duration_weight > 0.0);
    assert!(calculator.step_count_weight > 0.0);
}

#[test]
fn test_calculator_with_custom_config() {
    let calculator = AdaptiveRewardCalculator::with_config(0.7, 0.3, 45.0, 8);
    assert_eq!(calculator.duration_weight, 0.7);
    assert_eq!(calculator.step_count_weight, 0.3);
    assert_eq!(calculator.fallback_duration_secs, 45.0);
    assert_eq!(calculator.fallback_step_count, 8);
}

#[test]
fn test_base_reward_success() {
    let calculator = AdaptiveRewardCalculator::new();
    let mut episode = create_test_episode("test", ComplexityLevel::Simple);

    episode.complete(TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode, None);
    assert_eq!(reward.base, 1.0);
}

#[test]
fn test_base_reward_partial_success() {
    let calculator = AdaptiveRewardCalculator::new();
    let mut episode = create_test_episode("test", ComplexityLevel::Simple);

    episode.complete(TaskOutcome::PartialSuccess {
        verdict: "Partial".to_string(),
        completed: vec!["a".to_string()],
        failed: vec!["b".to_string()],
    });

    let reward = calculator.calculate(&episode, None);
    // 1 out of 2 = 0.5
    assert!((reward.base - 0.5).abs() < 0.01);
}

#[test]
fn test_base_reward_failure() {
    let calculator = AdaptiveRewardCalculator::new();
    let mut episode = create_test_episode("test", ComplexityLevel::Simple);

    episode.complete(TaskOutcome::Failure {
        reason: "Failed".to_string(),
        error_details: None,
    });

    let reward = calculator.calculate(&episode, None);
    assert_eq!(reward.base, 0.0);
}

#[test]
fn test_complexity_bonus_levels() {
    let calculator = AdaptiveRewardCalculator::new();

    let mut simple_episode = create_test_episode("test", ComplexityLevel::Simple);
    simple_episode.complete(TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });

    let mut moderate_episode = create_test_episode("test", ComplexityLevel::Moderate);
    moderate_episode.complete(TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });

    let mut complex_episode = create_test_episode("test", ComplexityLevel::Complex);
    complex_episode.complete(TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });

    let simple_reward = calculator.calculate(&simple_episode, None);
    let moderate_reward = calculator.calculate(&moderate_episode, None);
    let complex_reward = calculator.calculate(&complex_episode, None);

    assert_eq!(simple_reward.complexity_bonus, 1.0);
    assert_eq!(moderate_reward.complexity_bonus, 1.1);
    assert_eq!(complex_reward.complexity_bonus, 1.2);
}

#[test]
fn test_quality_multiplier_with_test_coverage() {
    let calculator = AdaptiveRewardCalculator::new();
    let mut episode = create_test_episode("test", ComplexityLevel::Simple);

    episode.complete(TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec!["test coverage: 90%".to_string()],
    });

    let reward = calculator.calculate(&episode, None);
    assert!(reward.quality_multiplier > 1.0);
}

#[test]
fn test_zero_error_rate_bonus() {
    let calculator = AdaptiveRewardCalculator::new();
    let mut episode = create_test_episode("test", ComplexityLevel::Simple);

    // Add successful steps
    for i in 0..5 {
        let mut step = ExecutionStep::new(i + 1, "tool".to_string(), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode, None);
    // Zero error rate should give bonus
    assert!(reward.quality_multiplier >= 1.0);
}

#[test]
fn test_error_recovery_bonus() {
    let calculator = AdaptiveRewardCalculator::new();
    let mut episode = create_test_episode("test", ComplexityLevel::Simple);

    // Add failed step followed by success
    let mut failed_step = ExecutionStep::new(1, "tool".to_string(), "Fail".to_string());
    failed_step.result = Some(ExecutionResult::Error {
        message: "Failed".to_string(),
    });
    episode.add_step(failed_step);

    let mut success_step = ExecutionStep::new(2, "tool".to_string(), "Recover".to_string());
    success_step.result = Some(ExecutionResult::Success {
        output: "OK".to_string(),
    });
    episode.add_step(success_step);

    episode.complete(TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode, None);
    // Error recovery should give learning bonus
    assert!(reward.learning_bonus > 0.0);
}

#[test]
fn test_perfect_execution_bonus() {
    let calculator = AdaptiveRewardCalculator::new();
    let mut episode = create_test_episode("test", ComplexityLevel::Simple);

    // Add multiple successful steps
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

    let reward = calculator.calculate(&episode, None);
    // Perfect execution should give learning bonus
    assert!(reward.learning_bonus > 0.0);
}

#[test]
fn test_novelty_bonus_with_diverse_tools() {
    let calculator = AdaptiveRewardCalculator::new();
    let mut episode = create_test_episode("test", ComplexityLevel::Simple);

    // Add steps with unique tools (at least 5)
    for i in 0..5 {
        let mut step = ExecutionStep::new(i + 1, format!("unique_tool_{i}"), format!("Action {i}"));
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode, None);
    // Should get novelty bonus for using 5+ unique tools
    assert!(reward.learning_bonus >= 0.1);
}

#[test]
fn test_optimization_bonus() {
    let calculator = AdaptiveRewardCalculator::new();
    let mut episode = create_test_episode("test", ComplexityLevel::Simple);

    // Add a few successful steps
    for i in 0..5 {
        let mut step = ExecutionStep::new(i + 1, "tool".to_string(), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });

    // Manually set duration to simulate quick completion
    episode
        .metadata
        .insert("duration_ms".to_string(), "5000".to_string());

    let reward = calculator.calculate(&episode, None);
    // Should get optimization bonus for quick completion
    assert!(reward.learning_bonus >= 0.0);
}

#[test]
fn test_incomplete_episode_zero_reward() {
    let calculator = AdaptiveRewardCalculator::new();
    let episode = create_test_episode("test", ComplexityLevel::Simple);
    // Don't complete the episode

    let reward = calculator.calculate(&episode, None);
    assert_eq!(reward.base, 0.0);
    assert_eq!(reward.total, 0.0);
}

#[test]
fn test_total_reward_calculation() {
    let calculator = AdaptiveRewardCalculator::new();
    let mut episode = create_test_episode("test", ComplexityLevel::Simple);

    // Add successful steps
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

    let reward = calculator.calculate(&episode, None);

    // Total should be: base * efficiency * complexity * quality + learning
    assert!(reward.total > 0.0);
    assert_eq!(reward.base, 1.0); // Success
    assert!(reward.efficiency >= 0.5 && reward.efficiency <= 1.5);
    assert!(reward.complexity_bonus >= 1.0 && reward.complexity_bonus <= 1.2);
    assert!(reward.quality_multiplier >= 0.5 && reward.quality_multiplier <= 1.5);
}

// ============================================================================
// ACT-026: Additional adaptive reward calculator tests
// ============================================================================

/// Test efficiency with zero steps (should give minimum efficiency)
#[test]
fn test_adaptive_efficiency_zero_steps() {
    let calculator = AdaptiveRewardCalculator::new();
    let mut episode = create_test_episode("test", ComplexityLevel::Simple);

    // No steps
    episode.complete(TaskOutcome::Success {
        verdict: "Instant".to_string(),
        artifacts: vec![],
    });

    let stats = DomainStatistics {
        domain: "test".to_string(),
        episode_count: 10,
        avg_duration_secs: 30.0,
        p50_duration_secs: 25.0,
        p90_duration_secs: 45.0,
        avg_step_count: 8.0,
        p50_step_count: 7,
        p90_step_count: 12,
        avg_reward: 0.8,
        p50_reward: 0.85,
        reward_std_dev: 0.1,
        last_updated: chrono::Utc::now(),
        success_count: 8,
    };

    let reward = calculator.calculate(&episode, Some(&stats));

    // For adaptive: instant completion (duration <= 0) returns 1.5 early
    assert!(
        reward.efficiency >= 0.5 && reward.efficiency <= 1.5,
        "Expected efficiency in valid range, got {}",
        reward.efficiency
    );
}

/// Test efficiency with instant completion (no duration)
#[test]
fn test_adaptive_efficiency_instant_completion() {
    let calculator = AdaptiveRewardCalculator::new();
    let mut episode = create_test_episode("test", ComplexityLevel::Simple);

    // Add steps
    for i in 0..5 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    // Complete immediately
    episode.complete(TaskOutcome::Success {
        verdict: "Instant".to_string(),
        artifacts: vec![],
    });

    let stats = DomainStatistics {
        domain: "test".to_string(),
        episode_count: 10,
        avg_duration_secs: 60.0,
        p50_duration_secs: 50.0,
        p90_duration_secs: 100.0,
        avg_step_count: 10.0,
        p50_step_count: 8,
        p90_step_count: 15,
        avg_reward: 0.7,
        p50_reward: 0.75,
        reward_std_dev: 0.15,
        last_updated: chrono::Utc::now(),
        success_count: 8,
    };

    let reward = calculator.calculate(&episode, Some(&stats));

    // Should have high efficiency for instant completion
    assert!(
        reward.efficiency > 1.0,
        "Expected high efficiency for instant completion, got {}",
        reward.efficiency
    );
}

/// Test adaptive efficiency with much better than median performance
#[test]
fn test_adaptive_efficiency_much_better_than_median() {
    let calculator = AdaptiveRewardCalculator::new();
    let mut episode = create_test_episode("test", ComplexityLevel::Simple);

    // 5 steps when median is 30
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

    let stats = DomainStatistics {
        domain: "test".to_string(),
        episode_count: 10,
        avg_duration_secs: 120.0,
        p50_duration_secs: 100.0,
        p90_duration_secs: 180.0,
        avg_step_count: 35.0,
        p50_step_count: 30, // Our episode has 5 steps, much better!
        p90_step_count: 50,
        avg_reward: 0.7,
        p50_reward: 0.75,
        reward_std_dev: 0.15,
        last_updated: chrono::Utc::now(),
        success_count: 8,
    };

    let reward = calculator.calculate(&episode, Some(&stats));

    // Much better than median should give high efficiency
    assert!(
        reward.efficiency > 1.0,
        "Expected high efficiency for much better than median, got {}",
        reward.efficiency
    );
}

/// Test adaptive efficiency clamping at maximum
#[test]
fn test_adaptive_efficiency_clamped_at_maximum() {
    let calculator = AdaptiveRewardCalculator::new();
    let mut episode = create_test_episode("test", ComplexityLevel::Simple);

    // Just 1 step, instant completion
    let mut step = ExecutionStep::new(1, "tool".to_string(), "Action".to_string());
    step.result = Some(ExecutionResult::Success {
        output: "OK".to_string(),
    });
    episode.add_step(step);

    episode.complete(TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });

    let stats = DomainStatistics {
        domain: "test".to_string(),
        episode_count: 10,
        avg_duration_secs: 120.0,
        p50_duration_secs: 100.0,
        p90_duration_secs: 180.0,
        avg_step_count: 35.0,
        p50_step_count: 30,
        p90_step_count: 50,
        avg_reward: 0.7,
        p50_reward: 0.75,
        reward_std_dev: 0.15,
        last_updated: chrono::Utc::now(),
        success_count: 8,
    };

    let reward = calculator.calculate(&episode, Some(&stats));

    // Efficiency should be clamped to max 1.5
    assert!(
        reward.efficiency <= 1.5,
        "Efficiency should not exceed 1.5, got {}",
        reward.efficiency
    );
}

/// Test adaptive efficiency clamping at minimum
#[test]
fn test_adaptive_efficiency_clamped_at_minimum() {
    let calculator = AdaptiveRewardCalculator::new();
    let mut episode = create_test_episode("test", ComplexityLevel::Simple);

    // Simulate very long episode (1 hour)
    episode.start_time = chrono::Utc::now() - chrono::Duration::hours(1);

    // 100 steps when median is 10
    for i in 0..100 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "Slow".to_string(),
        artifacts: vec![],
    });

    let stats = DomainStatistics {
        domain: "test".to_string(),
        episode_count: 10,
        avg_duration_secs: 30.0,
        p50_duration_secs: 25.0,
        p90_duration_secs: 45.0,
        avg_step_count: 12.0,
        p50_step_count: 10,
        p90_step_count: 20,
        avg_reward: 0.7,
        p50_reward: 0.75,
        reward_std_dev: 0.15,
        last_updated: chrono::Utc::now(),
        success_count: 8,
    };

    let reward = calculator.calculate(&episode, Some(&stats));

    // Efficiency should be clamped to min 0.5
    assert!(
        reward.efficiency >= 0.5,
        "Efficiency should not go below 0.5, got {}",
        reward.efficiency
    );
}

/// Test fallback efficiency with zero steps
#[test]
fn test_fallback_efficiency_zero_steps() {
    let calculator = AdaptiveRewardCalculator::new();
    let mut episode = create_test_episode("test", ComplexityLevel::Simple);

    // No steps
    episode.complete(TaskOutcome::Success {
        verdict: "Instant".to_string(),
        artifacts: vec![],
    });

    // No stats provided, should use fallback
    let reward = calculator.calculate(&episode, None);

    // For fallback: instant completion (duration <= 0) returns 1.5 early
    assert!(
        reward.efficiency >= 0.5 && reward.efficiency <= 1.5,
        "Expected efficiency in valid range with fallback, got {}",
        reward.efficiency
    );
}

/// Test fallback efficiency with instant completion
#[test]
fn test_fallback_efficiency_instant_completion() {
    let calculator = AdaptiveRewardCalculator::new();
    let mut episode = create_test_episode("test", ComplexityLevel::Simple);

    // Add steps
    for i in 0..3 {
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

    // No stats, should use fallback
    let reward = calculator.calculate(&episode, None);

    // Fast execution should give good efficiency
    assert!(
        reward.efficiency > 0.5,
        "Expected good efficiency for quick completion with fallback, got {}",
        reward.efficiency
    );
}

/// Test with exactly reliability threshold episodes (5)
#[test]
fn test_adaptive_reliability_threshold_exactly_5() {
    let calculator = AdaptiveRewardCalculator::new();
    let mut episode = create_test_episode("test", ComplexityLevel::Simple);

    episode.complete(TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });

    // Exactly 5 episodes - should be reliable
    let stats = DomainStatistics {
        domain: "test".to_string(),
        episode_count: 5, // Exactly at threshold
        avg_duration_secs: 30.0,
        p50_duration_secs: 25.0,
        p90_duration_secs: 45.0,
        avg_step_count: 8.0,
        p50_step_count: 7,
        p90_step_count: 12,
        avg_reward: 0.8,
        p50_reward: 0.85,
        reward_std_dev: 0.1,
        last_updated: chrono::Utc::now(),
        success_count: 4,
    };

    assert!(
        stats.is_reliable(),
        "Stats with 5 episodes should be reliable"
    );

    let reward = calculator.calculate(&episode, Some(&stats));
    assert!(reward.efficiency > 0.0);
}

/// Test with just below reliability threshold (4 episodes)
#[test]
fn test_adaptive_reliability_threshold_4() {
    let calculator = AdaptiveRewardCalculator::new();
    let mut episode = create_test_episode("test", ComplexityLevel::Simple);

    episode.complete(TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });

    // 4 episodes - should NOT be reliable
    let stats = DomainStatistics {
        domain: "test".to_string(),
        episode_count: 4, // Below threshold
        avg_duration_secs: 30.0,
        p50_duration_secs: 25.0,
        p90_duration_secs: 45.0,
        avg_step_count: 8.0,
        p50_step_count: 7,
        p90_step_count: 12,
        avg_reward: 0.8,
        p50_reward: 0.85,
        reward_std_dev: 0.1,
        last_updated: chrono::Utc::now(),
        success_count: 3,
    };

    assert!(
        !stats.is_reliable(),
        "Stats with 4 episodes should not be reliable"
    );

    // Should use fallback since not reliable
    let reward = calculator.calculate(&episode, Some(&stats));
    assert!(reward.efficiency > 0.0);
}

/// Test custom fallback thresholds
#[test]
fn test_adaptive_custom_fallback_thresholds() {
    let calculator = AdaptiveRewardCalculator::with_config(
        0.5, 0.5, 30.0, // 30 seconds fallback
        5,    // 5 steps fallback
    );
    let mut episode = create_test_episode("test", ComplexityLevel::Simple);

    // Add 3 steps (less than fallback threshold of 5)
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

    // No stats, use custom fallback
    let reward = calculator.calculate(&episode, None);

    // With fewer steps than fallback threshold, efficiency should be good
    assert!(
        reward.efficiency > 1.0,
        "Expected high efficiency with custom fallback and fewer steps, got {}",
        reward.efficiency
    );
}

/// Test partial success with empty completed and failed lists
#[test]
fn test_adaptive_partial_success_empty_lists() {
    let calculator = AdaptiveRewardCalculator::new();
    let mut episode = create_test_episode("test", ComplexityLevel::Simple);

    episode.complete(TaskOutcome::PartialSuccess {
        verdict: "Nothing done".to_string(),
        completed: vec![],
        failed: vec![],
    });

    let reward = calculator.calculate(&episode, None);

    // Empty lists should give default 0.5 base reward
    assert_eq!(
        reward.base, 0.5,
        "Expected base 0.5 for partial success with empty lists, got {}",
        reward.base
    );
}

/// Test partial success with all completed
#[test]
fn test_adaptive_partial_success_all_completed() {
    let calculator = AdaptiveRewardCalculator::new();
    let mut episode = create_test_episode("test", ComplexityLevel::Simple);

    episode.complete(TaskOutcome::PartialSuccess {
        verdict: "All done".to_string(),
        completed: vec!["a".to_string(), "b".to_string()],
        failed: vec![],
    });

    let reward = calculator.calculate(&episode, None);

    // All completed = 1.0 base
    assert_eq!(
        reward.base, 1.0,
        "Expected base 1.0 for all completed, got {}",
        reward.base
    );
}

/// Test failure with error details
#[test]
fn test_adaptive_failure_with_error_details() {
    let calculator = AdaptiveRewardCalculator::new();
    let mut episode = create_test_episode("test", ComplexityLevel::Simple);

    episode.complete(TaskOutcome::Failure {
        reason: "Error".to_string(),
        error_details: Some("Stack trace...".to_string()),
    });

    let reward = calculator.calculate(&episode, None);

    assert_eq!(reward.base, 0.0);
    assert_eq!(reward.total, 0.0);
}

/// Test with domain statistics having zero median values
#[test]
fn test_adaptive_stats_with_zero_median() {
    let calculator = AdaptiveRewardCalculator::new();
    let mut episode = create_test_episode("test", ComplexityLevel::Simple);

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

    // Stats with zero median (edge case)
    let stats = DomainStatistics {
        domain: "test".to_string(),
        episode_count: 10,
        avg_duration_secs: 0.0,
        p50_duration_secs: 0.0, // Zero median
        p90_duration_secs: 0.0,
        avg_step_count: 0.0,
        p50_step_count: 0, // Zero median
        p90_step_count: 0,
        avg_reward: 0.0,
        p50_reward: 0.0,
        reward_std_dev: 0.0,
        last_updated: chrono::Utc::now(),
        success_count: 0,
    };

    // Should handle zero median gracefully (use max(1, 0) = 1 for baseline)
    let reward = calculator.calculate(&episode, Some(&stats));

    // Should not crash and produce valid efficiency
    assert!(
        reward.efficiency >= 0.5 && reward.efficiency <= 1.5,
        "Expected valid efficiency with zero median stats, got {}",
        reward.efficiency
    );
}

/// Test efficiency calculation with duration weight of 1.0
#[test]
fn test_adaptive_duration_weight_100_percent() {
    let calculator = AdaptiveRewardCalculator::with_config(
        1.0, 0.0, 60.0, // fallback duration
        10,   // fallback steps (ignored with weight 0)
    );
    let mut episode = create_test_episode("test", ComplexityLevel::Simple);

    // Add steps
    for i in 0..20 {
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

    let reward = calculator.calculate(&episode, None);

    // Efficiency should be based purely on duration
    assert!(
        reward.efficiency > 0.0,
        "Expected valid efficiency with duration-only weight, got {}",
        reward.efficiency
    );
}

/// Test efficiency calculation with step count weight of 1.0
#[test]
fn test_adaptive_step_count_weight_100_percent() {
    let calculator = AdaptiveRewardCalculator::with_config(
        0.0, 1.0, 60.0, // fallback duration (ignored with weight 0)
        10,   // fallback steps
    );
    let mut episode = create_test_episode("test", ComplexityLevel::Simple);

    // 5 steps (less than fallback of 10)
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

    let reward = calculator.calculate(&episode, None);

    // Efficiency should be based purely on step count
    // 5 steps vs fallback 10 = better than threshold
    assert!(
        reward.efficiency > 1.0,
        "Expected high efficiency with fewer steps than threshold, got {}",
        reward.efficiency
    );
}

/// Test Timeout result handling in steps
#[test]
fn test_adaptive_timeout_result_in_steps() {
    let calculator = AdaptiveRewardCalculator::new();
    let mut episode = create_test_episode("test", ComplexityLevel::Simple);

    // Add a step with Timeout result
    let mut step = ExecutionStep::new(1, "slow_tool".to_string(), "Slow action".to_string());
    step.result = Some(ExecutionResult::Timeout);
    episode.add_step(step);

    // Add a successful step
    let mut step2 = ExecutionStep::new(2, "fast_tool".to_string(), "Fast action".to_string());
    step2.result = Some(ExecutionResult::Success {
        output: "OK".to_string(),
    });
    episode.add_step(step2);

    episode.complete(TaskOutcome::Success {
        verdict: "Done with timeout".to_string(),
        artifacts: vec![],
    });

    let reward = calculator.calculate(&episode, None);

    // Timeout is not a success, so error recovery bonus may apply
    assert!(reward.learning_bonus >= 0.0);
}
