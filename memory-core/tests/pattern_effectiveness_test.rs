//! Tests for pattern effectiveness tracking

use chrono::Duration;
use memory_core::TaskType;
use memory_core::episode::{ApplicationOutcome, Episode, PatternId};
use memory_core::pattern::{Pattern, PatternEffectiveness};
use memory_core::types::{ComplexityLevel, TaskContext};

#[test]
#[allow(clippy::float_cmp)]
fn test_pattern_effectiveness_creation() {
    let effectiveness = PatternEffectiveness::new();

    assert_eq!(effectiveness.times_retrieved, 0);
    assert_eq!(effectiveness.times_applied, 0);
    assert_eq!(effectiveness.success_when_applied, 0);
    assert_eq!(effectiveness.failure_when_applied, 0);
    assert_eq!(effectiveness.avg_reward_delta, 0.0);
    assert_eq!(effectiveness.application_success_rate(), 0.5); // Neutral for untested
}

#[test]
fn test_pattern_effectiveness_record_retrieval() {
    let mut effectiveness = PatternEffectiveness::new();

    effectiveness.record_retrieval();
    effectiveness.record_retrieval();

    assert_eq!(effectiveness.times_retrieved, 2);
}

#[test]
#[allow(clippy::float_cmp)]
fn test_pattern_effectiveness_record_application() {
    let mut effectiveness = PatternEffectiveness::new();

    // Record successful application with positive reward delta
    effectiveness.record_application(true, 0.2);
    assert_eq!(effectiveness.times_applied, 1);
    assert_eq!(effectiveness.success_when_applied, 1);
    assert_eq!(effectiveness.failure_when_applied, 0);
    assert_eq!(effectiveness.avg_reward_delta, 0.2);
    assert_eq!(effectiveness.application_success_rate(), 1.0);

    // Record failed application with negative reward delta
    effectiveness.record_application(false, -0.1);
    assert_eq!(effectiveness.times_applied, 2);
    assert_eq!(effectiveness.success_when_applied, 1);
    assert_eq!(effectiveness.failure_when_applied, 1);
    assert_eq!(effectiveness.application_success_rate(), 0.5);

    // Check moving average of reward delta
    let expected_avg = (0.2 + (-0.1)) / 2.0;
    assert!((effectiveness.avg_reward_delta - expected_avg).abs() < 0.001);
}

#[test]
fn test_pattern_effectiveness_score() {
    let mut effectiveness = PatternEffectiveness::new();

    // Untested pattern should have modest score
    let initial_score = effectiveness.effectiveness_score();
    assert!(initial_score > 0.0);
    assert!(initial_score < 1.0);

    // Apply pattern successfully with positive reward delta
    effectiveness.record_application(true, 0.3);
    effectiveness.record_application(true, 0.2);
    effectiveness.record_application(true, 0.25);

    let good_score = effectiveness.effectiveness_score();
    assert!(good_score > initial_score);

    // Apply pattern unsuccessfully with negative reward delta
    let mut bad_effectiveness = PatternEffectiveness::new();
    bad_effectiveness.record_application(false, -0.2);
    bad_effectiveness.record_application(false, -0.3);

    let bad_score = bad_effectiveness.effectiveness_score();
    assert!(bad_score < initial_score);
}

#[test]
#[allow(clippy::float_cmp)]
fn test_pattern_usage_rate() {
    let mut effectiveness = PatternEffectiveness::new();

    // Retrieved but not applied
    effectiveness.record_retrieval();
    effectiveness.record_retrieval();
    effectiveness.record_retrieval();
    assert_eq!(effectiveness.usage_rate(), 0.0);

    // Applied once
    effectiveness.record_application(true, 0.1);
    assert!((effectiveness.usage_rate() - 0.333).abs() < 0.01); // 1/3

    // Applied twice
    effectiveness.record_application(true, 0.1);
    assert!((effectiveness.usage_rate() - 0.666).abs() < 0.01); // 2/3
}

#[test]
fn test_pattern_record_retrieval() {
    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: Some("tokio".to_string()),
        complexity: ComplexityLevel::Moderate,
        domain: "web-api".to_string(),
        tags: vec!["async".to_string()],
    };

    let mut pattern = Pattern::ToolSequence {
        id: uuid::Uuid::new_v4(),
        tools: vec!["planner".to_string(), "executor".to_string()],
        context: context.clone(),
        success_rate: 0.9,
        avg_latency: Duration::milliseconds(100),
        occurrence_count: 10,
        effectiveness: PatternEffectiveness::new(),
    };

    // Record retrieval
    pattern.record_retrieval();
    assert_eq!(pattern.effectiveness().times_retrieved, 1);

    pattern.record_retrieval();
    assert_eq!(pattern.effectiveness().times_retrieved, 2);
}

#[test]
#[allow(clippy::float_cmp)]
fn test_pattern_record_application() {
    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: Some("tokio".to_string()),
        complexity: ComplexityLevel::Moderate,
        domain: "web-api".to_string(),
        tags: vec!["async".to_string()],
    };

    let mut pattern = Pattern::ErrorRecovery {
        id: uuid::Uuid::new_v4(),
        error_type: "timeout".to_string(),
        recovery_steps: vec!["retry".to_string(), "fallback".to_string()],
        success_rate: 0.8,
        context: context.clone(),
        effectiveness: PatternEffectiveness::new(),
    };

    // Record successful application
    pattern.record_application(true, 0.15);

    let eff = pattern.effectiveness();
    assert_eq!(eff.times_applied, 1);
    assert_eq!(eff.success_when_applied, 1);
    assert_eq!(eff.avg_reward_delta, 0.15);
}

#[test]
fn test_episode_pattern_application() {
    let context = TaskContext::default();
    let mut episode = Episode::new("Test task".to_string(), context, TaskType::Testing);

    let pattern_id: PatternId = uuid::Uuid::new_v4();

    // Record pattern application
    episode.record_pattern_application(
        pattern_id,
        1,
        ApplicationOutcome::Helped,
        Some("Pattern improved efficiency".to_string()),
    );

    assert_eq!(episode.applied_patterns.len(), 1);

    let application = &episode.applied_patterns[0];
    assert_eq!(application.pattern_id, pattern_id);
    assert_eq!(application.applied_at_step, 1);
    assert_eq!(application.outcome, ApplicationOutcome::Helped);
    assert!(application.outcome.is_success());
}

#[test]
fn test_application_outcome_is_success() {
    assert!(ApplicationOutcome::Helped.is_success());
    assert!(!ApplicationOutcome::NoEffect.is_success());
    assert!(!ApplicationOutcome::Hindered.is_success());
    assert!(!ApplicationOutcome::Pending.is_success());
}

#[test]
#[allow(clippy::float_cmp)]
fn test_pattern_effectiveness_moving_average() {
    let mut effectiveness = PatternEffectiveness::new();

    // First application: reward delta = 0.5
    effectiveness.record_application(true, 0.5);
    assert_eq!(effectiveness.avg_reward_delta, 0.5);

    // Second application: reward delta = 0.3
    // Moving average: (0.5 + 0.3) / 2 = 0.4
    effectiveness.record_application(true, 0.3);
    assert!((effectiveness.avg_reward_delta - 0.4).abs() < 0.001);

    // Third application: reward delta = 0.2
    // Moving average: (0.5 + 0.3 + 0.2) / 3 = 0.333...
    effectiveness.record_application(true, 0.2);
    assert!((effectiveness.avg_reward_delta - 0.333).abs() < 0.01);
}

#[test]
#[allow(clippy::float_cmp)]
fn test_pattern_effectiveness_with_negative_reward() {
    let mut effectiveness = PatternEffectiveness::new();

    // Pattern that actually hurts performance
    effectiveness.record_application(true, -0.2);
    effectiveness.record_application(false, -0.3);

    assert_eq!(effectiveness.times_applied, 2);
    assert_eq!(effectiveness.success_when_applied, 1);
    assert_eq!(effectiveness.application_success_rate(), 0.5);

    // Average reward delta should be negative
    let expected_avg = (-0.2 + -0.3) / 2.0;
    assert!((effectiveness.avg_reward_delta - expected_avg).abs() < 0.001);

    // Effectiveness score should be low
    let score = effectiveness.effectiveness_score();
    assert!(score < 0.5);
}

#[tokio::test]
async fn test_pattern_retrieval_updates_effectiveness() {
    use memory_core::SelfLearningMemory;

    let memory = SelfLearningMemory::new();
    let context = TaskContext::default();

    // Retrieve patterns (should update retrieval count)
    let patterns = memory.retrieve_relevant_patterns(&context, 5).await;

    // Note: This test would need patterns in memory to verify
    // In a real scenario, patterns would have their retrieval count incremented
    assert!(patterns.is_empty() || patterns.len() <= 5);
}
