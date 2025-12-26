//! Integration tests for pre-storage quality assessment.

use chrono::Utc;
use memory_core::pre_storage::{QualityAssessor, QualityConfig, QualityFeature};
use memory_core::types::{ExecutionResult, Reflection};
use memory_core::{Episode, ExecutionStep, TaskContext, TaskOutcome, TaskType};

#[test]
fn test_quality_assessor_basic_usage() {
    let config = QualityConfig::default();
    let assessor = QualityAssessor::new(config);

    let episode = Episode::new(
        "Test task".to_string(),
        TaskContext::default(),
        TaskType::Testing,
    );

    let score = assessor.assess_episode(&episode);
    assert!(
        score >= 0.0 && score <= 1.0,
        "Quality score must be in range 0.0-1.0"
    );
}

#[test]
fn test_high_quality_episode_scores_above_threshold() {
    let config = QualityConfig::default();
    let assessor = QualityAssessor::new(config);

    let mut episode = Episode::new(
        "Complex implementation task".to_string(),
        TaskContext::default(),
        TaskType::CodeGeneration,
    );

    // Add 15 diverse steps with multiple tools
    for i in 0..15 {
        let mut step = ExecutionStep::new(
            i + 1,
            format!("tool_{}", i % 5),
            format!("Implementing feature part {}", i),
        );
        step.result = Some(ExecutionResult::Success {
            output: format!("Part {} completed successfully", i),
        });
        episode.add_step(step);
    }

    // Add comprehensive reflection
    episode.reflection = Some(Reflection {
        successes: vec![
            "Good error handling".to_string(),
            "Efficient algorithm".to_string(),
            "Clean code structure".to_string(),
        ],
        improvements: vec![
            "Could add more tests".to_string(),
            "Documentation could be better".to_string(),
        ],
        insights: vec![
            "Builder pattern works well here".to_string(),
            "Async approach improved performance".to_string(),
        ],
        generated_at: Utc::now(),
    });

    // Complete successfully
    episode.complete(TaskOutcome::Success {
        verdict: "Implementation complete".to_string(),
        artifacts: vec!["feature.rs".to_string(), "feature_test.rs".to_string()],
    });

    let score = assessor.assess_episode(&episode);
    // Should score well above the default 0.7 threshold
    assert!(
        score >= 0.7,
        "High-quality episode should score >= 0.7, got {}",
        score
    );
    assert!(
        assessor.should_store(&episode),
        "High-quality episode should pass storage threshold"
    );
}

#[test]
fn test_low_quality_episode_scores_below_threshold() {
    let config = QualityConfig::default();
    let assessor = QualityAssessor::new(config);

    let mut episode = Episode::new(
        "Simple task".to_string(),
        TaskContext::default(),
        TaskType::Other,
    );

    // Add only a single simple step
    let mut step = ExecutionStep::new(1, "tool".to_string(), "Simple action".to_string());
    step.result = Some(ExecutionResult::Success {
        output: "OK".to_string(),
    });
    episode.add_step(step);

    // No reflection
    // Complete
    episode.complete(TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });

    let score = assessor.assess_episode(&episode);
    // Single simple step should score below threshold
    assert!(
        score < 0.7,
        "Low-quality episode should score < 0.7 (below threshold), got {}",
        score
    );
    assert!(
        !assessor.should_store(&episode),
        "Low-quality episode should not pass storage threshold"
    );
}

#[test]
fn test_error_recovery_improves_quality() {
    let config = QualityConfig::default();
    let assessor = QualityAssessor::new(config);

    let mut episode = Episode::new(
        "Task with error recovery".to_string(),
        TaskContext::default(),
        TaskType::Debugging,
    );

    // Mix of errors and successes showing error recovery
    for i in 0..12 {
        let mut step =
            ExecutionStep::new(i + 1, format!("tool_{}", i % 4), format!("Action {}", i));
        // Every 4th step fails, others succeed (showing recovery)
        step.result = if i % 4 == 0 {
            Some(ExecutionResult::Error {
                message: "Temporary error".to_string(),
            })
        } else {
            Some(ExecutionResult::Success {
                output: "Success after recovery".to_string(),
            })
        };
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "Recovered from errors successfully".to_string(),
        artifacts: vec!["fix.rs".to_string()],
    });

    let score = assessor.assess_episode(&episode);
    assert!(
        score > 0.5,
        "Error recovery should result in moderate-to-good quality, got {}",
        score
    );
}

#[test]
fn test_custom_quality_threshold() {
    let config = QualityConfig::new(0.9); // Very high threshold
    let assessor = QualityAssessor::new(config);

    let mut episode = Episode::new(
        "Moderately good task".to_string(),
        TaskContext::default(),
        TaskType::Testing,
    );

    // Add moderate complexity
    for i in 0..8 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i % 3), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "Complete".to_string(),
        artifacts: vec!["output.rs".to_string()],
    });

    let score = assessor.assess_episode(&episode);
    // Should not pass very high threshold
    assert!(
        !assessor.should_store(&episode),
        "Episode with score {} should not pass 0.9 threshold",
        score
    );
}

#[test]
fn test_custom_feature_weights() {
    let mut config = QualityConfig::default();
    // Heavily weight task complexity
    config.set_weight(QualityFeature::TaskComplexity, 0.6);
    config.set_weight(QualityFeature::StepDiversity, 0.1);
    config.set_weight(QualityFeature::ErrorRate, 0.1);
    config.set_weight(QualityFeature::ReflectionDepth, 0.1);
    config.set_weight(QualityFeature::PatternNovelty, 0.1);

    let assessor = QualityAssessor::new(config);

    let mut episode = Episode::new(
        "Complex task".to_string(),
        TaskContext::default(),
        TaskType::CodeGeneration,
    );

    // Add many steps with diverse tools (high complexity)
    for i in 0..20 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i), format!("Action {}", i));
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "Complete".to_string(),
        artifacts: vec![],
    });

    let score = assessor.assess_episode(&episode);
    // Should score well due to high complexity weight
    assert!(
        score > 0.6,
        "High complexity with heavy weight should score well, got {}",
        score
    );
}

#[test]
fn test_reflection_depth_impact() {
    let config = QualityConfig::default();
    let assessor = QualityAssessor::new(config);

    // Create two identical episodes except for reflection
    let mut episode_without_reflection = Episode::new(
        "Task".to_string(),
        TaskContext::default(),
        TaskType::Testing,
    );

    for i in 0..5 {
        let mut step = ExecutionStep::new(i + 1, "tool".to_string(), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode_without_reflection.add_step(step);
    }

    episode_without_reflection.complete(TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });

    let score_without = assessor.assess_episode(&episode_without_reflection);

    // Same episode with reflection
    let mut episode_with_reflection = episode_without_reflection.clone();
    episode_with_reflection.reflection = Some(Reflection {
        successes: vec!["Success 1".to_string(), "Success 2".to_string()],
        improvements: vec!["Improvement 1".to_string()],
        insights: vec!["Insight 1".to_string(), "Insight 2".to_string()],
        generated_at: Utc::now(),
    });

    let score_with = assessor.assess_episode(&episode_with_reflection);

    assert!(
        score_with > score_without,
        "Reflection should improve quality score"
    );
}

#[test]
fn test_all_quality_features_contribute() {
    let config = QualityConfig::default();
    let assessor = QualityAssessor::new(config);

    let mut episode = Episode::new(
        "Comprehensive task".to_string(),
        TaskContext::default(),
        TaskType::CodeGeneration,
    );

    // High complexity: many diverse steps
    for i in 0..15 {
        let mut step = ExecutionStep::new(
            i + 1,
            format!("tool_{}", i % 6),
            format!("Varied action {}", i),
        );
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    // Good reflection
    episode.reflection = Some(Reflection {
        successes: vec!["Success".to_string(); 4],
        improvements: vec!["Improvement".to_string(); 2],
        insights: vec!["Insight".to_string(); 3],
        generated_at: Utc::now(),
    });

    // Simulate pattern discovery (in real usage, these would be set by pattern extraction)
    // Note: We can't easily test this without mocking pattern extraction

    episode.complete(TaskOutcome::Success {
        verdict: "Complete".to_string(),
        artifacts: vec!["output.rs".to_string()],
    });

    let score = assessor.assess_episode(&episode);

    // Should score well across all features
    assert!(
        score > 0.7,
        "Episode with good scores across all features should be high quality, got {}",
        score
    );
}

#[test]
fn test_score_stability() {
    let config = QualityConfig::default();
    let assessor = QualityAssessor::new(config);

    let mut episode = Episode::new(
        "Task".to_string(),
        TaskContext::default(),
        TaskType::Testing,
    );

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

    // Score should be consistent across multiple calls
    let score1 = assessor.assess_episode(&episode);
    let score2 = assessor.assess_episode(&episode);
    let score3 = assessor.assess_episode(&episode);

    assert_eq!(score1, score2, "Quality score should be stable");
    assert_eq!(score2, score3, "Quality score should be stable");
}
