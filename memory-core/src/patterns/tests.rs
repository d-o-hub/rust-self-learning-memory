//! Tests for pattern extraction and management

use crate::patterns::*;
use crate::types::ComplexityLevel;
use chrono::Duration;
use uuid::Uuid;

#[test]
fn test_pattern_id() {
    let pattern = Pattern::ToolSequence {
        id: Uuid::new_v4(),
        tools: vec!["tool1".to_string(), "tool2".to_string()],
        context: TaskContext::default(),
        success_rate: 0.9,
        avg_latency: Duration::milliseconds(100),
        occurrence_count: 5,
        effectiveness: PatternEffectiveness::new(),
    };

    assert!(pattern.success_rate() > 0.8);
    assert!(pattern.context().is_some());
}

#[test]
fn test_pattern_similarity_key() {
    let pattern1 = Pattern::ToolSequence {
        id: Uuid::new_v4(),
        tools: vec!["read".to_string(), "write".to_string()],
        context: TaskContext {
            domain: "web-api".to_string(),
            ..Default::default()
        },
        success_rate: 0.9,
        avg_latency: Duration::milliseconds(100),
        occurrence_count: 5,
        effectiveness: PatternEffectiveness::new(),
    };

    let pattern2 = Pattern::ToolSequence {
        id: Uuid::new_v4(),
        tools: vec!["read".to_string(), "write".to_string()],
        context: TaskContext {
            domain: "web-api".to_string(),
            ..Default::default()
        },
        success_rate: 0.8,
        avg_latency: Duration::milliseconds(120),
        occurrence_count: 3,
        effectiveness: PatternEffectiveness::new(),
    };

    // Same tools and domain = same key
    assert_eq!(pattern1.similarity_key(), pattern2.similarity_key());
}

#[test]
fn test_pattern_similarity_score() {
    let pattern1 = Pattern::ToolSequence {
        id: Uuid::new_v4(),
        tools: vec!["read".to_string(), "write".to_string()],
        context: TaskContext {
            domain: "web-api".to_string(),
            language: Some("rust".to_string()),
            ..Default::default()
        },
        success_rate: 0.9,
        avg_latency: Duration::milliseconds(100),
        occurrence_count: 5,
        effectiveness: PatternEffectiveness::new(),
    };

    let pattern2 = Pattern::ToolSequence {
        id: Uuid::new_v4(),
        tools: vec!["read".to_string(), "write".to_string()],
        context: TaskContext {
            domain: "web-api".to_string(),
            language: Some("rust".to_string()),
            ..Default::default()
        },
        success_rate: 0.8,
        avg_latency: Duration::milliseconds(120),
        occurrence_count: 3,
        effectiveness: PatternEffectiveness::new(),
    };

    let similarity = pattern1.similarity_score(&pattern2);

    // Identical tools and context should have high similarity
    assert!(similarity > 0.9);
}

#[test]
fn test_pattern_confidence() {
    let pattern = Pattern::ToolSequence {
        id: Uuid::new_v4(),
        tools: vec!["tool1".to_string()],
        context: TaskContext::default(),
        success_rate: 0.8,
        avg_latency: Duration::milliseconds(100),
        occurrence_count: 16, // sqrt(16) = 4
        effectiveness: PatternEffectiveness::new(),
    };

    let confidence = pattern.confidence();

    // 0.8 * sqrt(16) = 0.8 * 4 = 3.2
    assert!((confidence - 3.2).abs() < 0.01);
}

#[test]
fn test_pattern_merge() {
    let mut pattern1 = Pattern::ToolSequence {
        id: Uuid::new_v4(),
        tools: vec!["read".to_string(), "write".to_string()],
        context: TaskContext::default(),
        success_rate: 0.8,
        avg_latency: Duration::milliseconds(100),
        occurrence_count: 10,
        effectiveness: PatternEffectiveness::new(),
    };

    let pattern2 = Pattern::ToolSequence {
        id: Uuid::new_v4(),
        tools: vec!["read".to_string(), "write".to_string()],
        context: TaskContext::default(),
        success_rate: 0.9,
        avg_latency: Duration::milliseconds(200),
        occurrence_count: 10,
        effectiveness: PatternEffectiveness::new(),
    };

    pattern1.merge_with(&pattern2);

    // Should have combined occurrence count
    match pattern1 {
        Pattern::ToolSequence {
            occurrence_count,
            success_rate,
            ..
        } => {
            assert_eq!(occurrence_count, 20);
            // Average: (0.8 * 10 + 0.9 * 10) / 20 = 0.85
            assert!((success_rate - 0.85).abs() < 0.01);
        }
        _ => panic!("Expected ToolSequence"),
    }
}

#[test]
fn test_pattern_relevance() {
    let pattern_context = TaskContext {
        language: Some("rust".to_string()),
        framework: None,
        complexity: ComplexityLevel::Moderate,
        domain: "web-api".to_string(),
        tags: vec!["async".to_string()],
    };

    let pattern = Pattern::ToolSequence {
        id: Uuid::new_v4(),
        tools: vec![],
        context: pattern_context.clone(),
        success_rate: 0.9,
        avg_latency: Duration::milliseconds(100),
        occurrence_count: 1,
        effectiveness: PatternEffectiveness::new(),
    };

    // Should match on domain
    let query_context = TaskContext {
        domain: "web-api".to_string(),
        ..Default::default()
    };
    assert!(pattern.is_relevant_to(&query_context));

    // Should match on language
    let query_context2 = TaskContext {
        language: Some("rust".to_string()),
        domain: "cli".to_string(),
        ..Default::default()
    };
    assert!(pattern.is_relevant_to(&query_context2));

    // Should not match
    let query_context3 = TaskContext {
        language: Some("python".to_string()),
        domain: "data-science".to_string(),
        ..Default::default()
    };
    assert!(!pattern.is_relevant_to(&query_context3));
}

#[test]
fn test_heuristic_evidence_update() {
    let mut heuristic = Heuristic::new(
        "When refactoring async code".to_string(),
        "Use tokio::spawn for CPU-intensive tasks".to_string(),
        0.7,
    );

    assert_eq!(heuristic.evidence.sample_size, 0);

    // Add successful evidence
    heuristic.update_evidence(Uuid::new_v4(), true);
    assert_eq!(heuristic.evidence.sample_size, 1);
    assert_eq!(heuristic.evidence.success_rate, 1.0);

    // Add failed evidence
    heuristic.update_evidence(Uuid::new_v4(), false);
    assert_eq!(heuristic.evidence.sample_size, 2);
    assert_eq!(heuristic.evidence.success_rate, 0.5);

    // Add more successful evidence
    heuristic.update_evidence(Uuid::new_v4(), true);
    assert_eq!(heuristic.evidence.sample_size, 3);
    assert!((heuristic.evidence.success_rate - 0.666).abs() < 0.01);
}

fn create_decision_point(
    condition: &str,
    action: &str,
    success_count: usize,
    failure_count: usize,
) -> Pattern {
    Pattern::DecisionPoint {
        id: Uuid::new_v4(),
        condition: condition.to_string(),
        action: action.to_string(),
        outcome_stats: crate::types::OutcomeStats {
            success_count,
            failure_count,
            total_count: success_count + failure_count,
            avg_duration_secs: 1.0,
        },
        context: TaskContext {
            domain: "web-api".to_string(),
            language: Some("rust".to_string()),
            tags: vec!["db".to_string()],
            ..Default::default()
        },
        effectiveness: PatternEffectiveness::new(),
    }
}

fn create_error_recovery(error_type: &str, steps: Vec<&str>, success_rate: f32) -> Pattern {
    Pattern::ErrorRecovery {
        id: Uuid::new_v4(),
        error_type: error_type.to_string(),
        recovery_steps: steps.into_iter().map(String::from).collect(),
        success_rate,
        context: TaskContext {
            domain: "web-api".to_string(),
            language: Some("rust".to_string()),
            tags: vec!["db".to_string()],
            ..Default::default()
        },
        effectiveness: PatternEffectiveness::new(),
    }
}

fn create_context_pattern(
    features: Vec<&str>,
    approach: &str,
    evidence: Vec<Uuid>,
    success_rate: f32,
) -> Pattern {
    Pattern::ContextPattern {
        id: Uuid::new_v4(),
        context_features: features.into_iter().map(String::from).collect(),
        recommended_approach: approach.to_string(),
        evidence,
        success_rate,
        effectiveness: PatternEffectiveness::new(),
    }
}

#[test]
fn test_similarity_score_decision_point_identical() {
    let p1 = create_decision_point("cond_a", "act_a", 5, 0);
    let p2 = create_decision_point("cond_a", "act_a", 5, 0);
    let score = p1.similarity_score(&p2);
    // Identical decision points should have exact match score
    assert_eq!(score, 1.0);
}

#[test]
fn test_similarity_score_error_recovery_same_type() {
    let p1 = create_error_recovery("io_error", vec!["retry", "log"], 0.8);
    let p2 = create_error_recovery("io_error", vec!["retry", "log"], 0.8);
    let score = p1.similarity_score(&p2);
    // Identical error recovery patterns should have high similarity score
    assert_eq!(score, 1.0);
}

#[test]
fn test_similarity_score_context_pattern_identical() {
    let p1 = create_context_pattern(
        vec!["feat1", "feat2"],
        "approach_a",
        vec![Uuid::new_v4()],
        0.9,
    );
    let p2 = create_context_pattern(
        vec!["feat1", "feat2"],
        "approach_a",
        vec![Uuid::new_v4()],
        0.9,
    );
    let score = p1.similarity_score(&p2);
    // Identical context patterns should have exact match score
    assert_eq!(score, 1.0);
}

#[test]
fn test_merge_with_decision_point_combines_outcome_stats() {
    let mut p1 = create_decision_point("cond_a", "act_a", 5, 0);
    let p2 = create_decision_point("cond_a", "act_a", 5, 0);
    p1.merge_with(&p2);

    if let Pattern::DecisionPoint { outcome_stats, .. } = p1 {
        assert_eq!(outcome_stats.success_count, 10);
        assert_eq!(outcome_stats.total_count, 10);
    } else {
        panic!("Expected DecisionPoint");
    }
}

#[test]
fn test_merge_with_error_recovery_updates_success_rate() {
    let mut p1 = create_error_recovery("io_error", vec!["retry"], 0.6);
    let p2 = create_error_recovery("io_error", vec!["retry"], 0.8);
    p1.merge_with(&p2);

    if let Pattern::ErrorRecovery { success_rate, .. } = p1 {
        // (0.6 + 0.8) / 2 = 0.7
        assert!((success_rate - 0.7).abs() < 0.01);
    } else {
        panic!("Expected ErrorRecovery");
    }
}

#[test]
fn test_merge_with_context_pattern_extends_evidence() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let mut p1 = create_context_pattern(vec!["feat1"], "approach", vec![id1], 0.6);
    let p2 = create_context_pattern(vec!["feat1"], "approach", vec![id2], 0.8);
    p1.merge_with(&p2);

    if let Pattern::ContextPattern {
        evidence,
        success_rate,
        ..
    } = p1
    {
        assert_eq!(evidence.len(), 2);
        assert!(evidence.contains(&id1));
        assert!(evidence.contains(&id2));
        // (0.6 * 1 + 0.8 * 1) / 2 = 0.7
        assert!((success_rate - 0.7).abs() < 0.01);
    } else {
        panic!("Expected ContextPattern");
    }
}
