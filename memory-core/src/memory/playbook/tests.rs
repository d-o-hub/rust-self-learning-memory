//! Tests for playbook module

use super::*;
use crate::pattern::Pattern;
use crate::pattern::PatternEffectiveness;
use crate::semantic::EpisodeSummary;
use crate::types::{ComplexityLevel, OutcomeStats, TaskContext, TaskType};
use chrono::Utc;
use uuid::Uuid;

fn create_tool_sequence_pattern() -> Pattern {
    Pattern::ToolSequence {
        id: Uuid::new_v4(),
        tools: vec![
            "read_file".to_string(),
            "analyze".to_string(),
            "edit_file".to_string(),
        ],
        context: TaskContext {
            domain: "web-api".to_string(),
            language: Some("rust".to_string()),
            framework: None,
            complexity: ComplexityLevel::Moderate,
            tags: vec!["api".to_string()],
        },
        success_rate: 0.9,
        avg_latency: chrono::Duration::milliseconds(150),
        occurrence_count: 15,
        effectiveness: PatternEffectiveness::new(),
    }
}

fn create_decision_point_pattern() -> Pattern {
    Pattern::DecisionPoint {
        id: Uuid::new_v4(),
        condition: "test_coverage < 80%".to_string(),
        action: "add_more_tests".to_string(),
        outcome_stats: OutcomeStats {
            success_count: 8,
            failure_count: 2,
            total_count: 10,
            avg_duration_secs: 30.0,
        },
        context: TaskContext {
            domain: "testing".to_string(),
            language: Some("rust".to_string()),
            framework: None,
            complexity: ComplexityLevel::Simple,
            tags: vec!["unit-test".to_string()],
        },
        effectiveness: PatternEffectiveness::new(),
    }
}

fn create_error_recovery_pattern() -> Pattern {
    Pattern::ErrorRecovery {
        id: Uuid::new_v4(),
        error_type: "network_timeout".to_string(),
        recovery_steps: vec![
            "retry_with_backoff".to_string(),
            "use_cached_data".to_string(),
        ],
        success_rate: 0.75,
        context: TaskContext {
            domain: "networking".to_string(),
            language: None,
            framework: None,
            complexity: ComplexityLevel::Complex,
            tags: vec!["resilience".to_string()],
        },
        effectiveness: PatternEffectiveness::new(),
    }
}

fn create_context_pattern() -> Pattern {
    Pattern::ContextPattern {
        id: Uuid::new_v4(),
        context_features: vec!["async".to_string(), "tokio".to_string()],
        recommended_approach: "Use async/await pattern".to_string(),
        evidence: vec![Uuid::new_v4(), Uuid::new_v4()],
        success_rate: 0.85,
        effectiveness: PatternEffectiveness::new(),
    }
}

fn create_episode_summary() -> EpisodeSummary {
    EpisodeSummary {
        episode_id: Uuid::new_v4(),
        summary_text: "Successfully implemented authentication with JWT tokens".to_string(),
        key_concepts: vec![
            "jwt".to_string(),
            "authentication".to_string(),
            "security".to_string(),
        ],
        key_steps: vec!["Generate token".to_string(), "Validate token".to_string()],
        summary_embedding: None,
        created_at: Utc::now(),
    }
}

fn create_reflection_data() -> ReflectionData {
    ReflectionData {
        episode_id: Uuid::new_v4(),
        successes: vec!["Clean implementation".to_string()],
        improvements: vec!["Add more tests".to_string()],
        insights: vec!["JWT tokens work well".to_string()],
        failed_steps: vec!["Initial validation failed".to_string()],
    }
}

#[test]
fn test_playbook_step_builder() {
    let step = PlaybookStep::new(1, "Analyze code".to_string())
        .with_tool_hint("code_analyzer")
        .with_expected_result("Code analysis report");

    assert_eq!(step.order, 1);
    assert_eq!(step.action, "Analyze code");
    assert_eq!(step.tool_hint, Some("code_analyzer".to_string()));
    assert_eq!(
        step.expected_result,
        Some("Code analysis report".to_string())
    );
}

#[test]
fn test_playbook_pitfall_builder() {
    let pitfall = PlaybookPitfall::new("Don't skip tests", "Skipping tests leads to bugs")
        .with_mitigation("Always run tests before merging");

    assert_eq!(pitfall.warning, "Don't skip tests");
    assert_eq!(pitfall.reason, "Skipping tests leads to bugs");
    assert_eq!(
        pitfall.mitigation,
        Some("Always run tests before merging".to_string())
    );
}

#[test]
fn test_synthesis_source_tracking() {
    let mut source = PlaybookSynthesisSource::new();
    let p1 = Uuid::new_v4();
    let p2 = Uuid::new_v4();
    let e1 = Uuid::new_v4();

    source.add_pattern(p1);
    source.add_pattern(p1); // Duplicate
    source.add_pattern(p2);
    source.add_episode(e1);
    source.add_summary(e1);

    assert_eq!(source.pattern_ids.len(), 2);
    assert_eq!(source.episode_ids.len(), 1);
    assert_eq!(source.summary_episode_ids.len(), 1);
    assert_eq!(source.total_sources(), 4);
}

#[test]
fn test_recommended_playbook_quality_score() {
    let mut playbook = RecommendedPlaybook::new(Uuid::new_v4(), 0.85);
    playbook.confidence = 0.9;
    playbook.supporting_pattern_ids.push(Uuid::new_v4());
    playbook.supporting_episode_ids.push(Uuid::new_v4());
    playbook.supporting_episode_ids.push(Uuid::new_v4());

    let score = playbook.quality_score();
    assert!(score > 0.0 && score <= 1.0);
    // High confidence and good match should give a decent score
    assert!(score > 0.5);
}

#[test]
fn test_playbook_request_builder() {
    let request = PlaybookRequest::new("Implement auth", "security")
        .with_task_type(TaskType::CodeGeneration)
        .with_max_steps(10);

    assert_eq!(request.task_description, "Implement auth");
    assert_eq!(request.domain, "security");
    assert_eq!(request.task_type, TaskType::CodeGeneration);
    assert_eq!(request.max_steps, 10);
}

#[tokio::test]
async fn test_generator_with_tool_sequence() {
    let generator = PlaybookGenerator::new();
    let request = PlaybookRequest::new("Write code", "web-api");

    let patterns = vec![create_tool_sequence_pattern()];
    let summaries = vec![];
    let reflections = vec![];

    let playbook = generator
        .generate(&request, &patterns, &summaries, &reflections)
        .unwrap();

    assert!(!playbook.ordered_steps.is_empty());
    assert!(playbook.task_match_score > 0.0);
    assert!(!playbook.when_to_apply.is_empty());
}

#[tokio::test]
async fn test_generator_with_decision_point() {
    let generator = PlaybookGenerator::new();
    let request = PlaybookRequest::new("Improve tests", "testing");

    let patterns = vec![create_decision_point_pattern()];
    let summaries = vec![];
    let reflections = vec![];

    let playbook = generator
        .generate(&request, &patterns, &summaries, &reflections)
        .unwrap();

    assert!(!playbook.ordered_steps.is_empty());
    // Should have both apply and not-apply conditions
    assert!(!playbook.when_not_to_apply.is_empty());
}

#[tokio::test]
async fn test_generator_with_error_recovery() {
    let generator = PlaybookGenerator::new();
    let request = PlaybookRequest::new("Handle errors", "networking");

    let patterns = vec![create_error_recovery_pattern()];
    let summaries = vec![];
    let reflections = vec![];

    let playbook = generator
        .generate(&request, &patterns, &summaries, &reflections)
        .unwrap();

    // Should have error handling steps
    assert!(
        playbook
            .ordered_steps
            .iter()
            .any(|s| s.action.contains("error") || s.action.contains("Handle"))
    );
}

#[tokio::test]
async fn test_generator_with_context_pattern() {
    let generator = PlaybookGenerator::new();
    let request = PlaybookRequest::new("Write async code", "concurrency");

    let patterns = vec![create_context_pattern()];
    let summaries = vec![];
    let reflections = vec![];

    let playbook = generator
        .generate(&request, &patterns, &summaries, &reflections)
        .unwrap();

    assert!(!playbook.ordered_steps.is_empty());
    assert!(!playbook.supporting_episode_ids.is_empty()); // From evidence
}

#[tokio::test]
async fn test_generator_with_reflections() {
    let generator = PlaybookGenerator::new();
    let request = PlaybookRequest::new("Implement auth", "security");

    let patterns = vec![create_tool_sequence_pattern()];
    let summaries = vec![create_episode_summary()];
    let reflections = vec![create_reflection_data()];

    let playbook = generator
        .generate(&request, &patterns, &summaries, &reflections)
        .unwrap();

    // Should have pitfalls from reflections
    assert!(!playbook.pitfalls.is_empty());
    // Should have expected outcome from summaries
    assert!(!playbook.expected_outcome.is_empty());
}

#[tokio::test]
async fn test_generator_confidence_calculation() {
    let generator = PlaybookGenerator::new();
    let request = PlaybookRequest::new("Task", "domain");

    // High success rate patterns
    let patterns = vec![create_tool_sequence_pattern()];
    let summaries = vec![create_episode_summary(), create_episode_summary()];
    let reflections = vec![];

    let playbook = generator
        .generate(&request, &patterns, &summaries, &reflections)
        .unwrap();

    // With high success patterns and summaries, confidence should be good
    assert!(playbook.confidence > 0.3);
}

#[tokio::test]
async fn test_generator_max_steps_limit() {
    let generator = PlaybookGenerator::new();
    let request = PlaybookRequest::new("Task", "domain").with_max_steps(3);

    let patterns = vec![
        create_tool_sequence_pattern(),
        create_decision_point_pattern(),
        create_error_recovery_pattern(),
    ];
    let summaries = vec![];
    let reflections = vec![];

    let playbook = generator
        .generate(&request, &patterns, &summaries, &reflections)
        .unwrap();

    // Should not exceed max_steps
    assert!(playbook.ordered_steps.len() <= 3);
}

#[tokio::test]
async fn test_generator_empty_inputs() {
    let generator = PlaybookGenerator::new();
    let request = PlaybookRequest::new("Task", "domain");

    let playbook = generator.generate(&request, &[], &[], &[]).unwrap();

    assert_eq!(playbook.task_match_score, 0.0);
    assert!(playbook.ordered_steps.is_empty());
    assert!(playbook.pitfalls.is_empty());
    assert_eq!(playbook.confidence, 0.0);
}

#[tokio::test]
async fn test_generator_multiple_pattern_types() {
    let generator = PlaybookGenerator::new();
    let request = PlaybookRequest::new("Complex task", "general").with_max_steps(10);

    let patterns = vec![
        create_tool_sequence_pattern(),
        create_decision_point_pattern(),
        create_error_recovery_pattern(),
        create_context_pattern(),
    ];
    let summaries = vec![create_episode_summary()];
    let reflections = vec![create_reflection_data()];

    let playbook = generator
        .generate(&request, &patterns, &summaries, &reflections)
        .unwrap();

    // Should synthesize from all pattern types
    assert!(playbook.ordered_steps.len() >= 3);
    assert!(!playbook.when_to_apply.is_empty());
    assert!(!playbook.supporting_pattern_ids.is_empty());
    assert!(playbook.quality_score() > 0.0);
}
