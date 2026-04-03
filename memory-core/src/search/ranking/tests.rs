use super::*;
use crate::episode::Episode;
use crate::search::{SearchField, SearchMode, SearchResult};
use crate::types::{TaskContext, TaskType};
use chrono::{Duration, Utc};

#[test]
fn test_default_weights_sum_to_one() {
    let weights = RankingWeights::default();
    assert!(weights.validate().is_ok());
}

#[test]
fn test_custom_weights_validation() {
    let valid = RankingWeights::new(0.5, 0.2, 0.2, 0.05, 0.05);
    assert!(valid.validate().is_ok());

    let invalid = RankingWeights::new(0.5, 0.2, 0.2, 0.2, 0.2);
    assert!(invalid.validate().is_err());
}

#[test]
fn test_exact_relevance_score() {
    assert_eq!(calculate_relevance_score(&SearchMode::Exact, 1.0), 1.0);
    assert_eq!(calculate_relevance_score(&SearchMode::Exact, 0.9), 0.0);
}

#[test]
fn test_fuzzy_relevance_score() {
    let mode = SearchMode::Fuzzy { threshold: 0.8 };

    // Perfect match
    assert_eq!(calculate_relevance_score(&mode, 1.0), 1.0);

    // At threshold
    assert_eq!(calculate_relevance_score(&mode, 0.8), 0.0);

    // Midway between threshold and perfect
    assert_eq!(calculate_relevance_score(&mode, 0.9), 0.5);

    // Below threshold
    assert_eq!(calculate_relevance_score(&mode, 0.7), 0.0);
}

#[test]
fn test_regex_relevance_score() {
    assert_eq!(calculate_relevance_score(&SearchMode::Regex, 1.0), 0.9);
}

#[test]
fn test_recency_score() {
    let now = Utc::now();

    // Recent episode (1 day old)
    let recent = now - Duration::days(1);
    let score = calculate_recency_score(recent, now);
    assert!(score > 0.9);

    // 30 days old (half-life)
    let month_old = now - Duration::days(30);
    let score = calculate_recency_score(month_old, now);
    assert!((score - 0.5).abs() < 0.01);

    // 60 days old
    let two_months_old = now - Duration::days(60);
    let score = calculate_recency_score(two_months_old, now);
    assert!((score - 0.25).abs() < 0.01);
}

#[test]
fn test_success_score() {
    let context = TaskContext::default();
    let mut episode = Episode::new("test".to_string(), context, TaskType::CodeGeneration);

    // No outcome
    assert_eq!(calculate_success_score(&episode), 0.5);

    // Success
    episode.outcome = Some(crate::types::TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });
    assert_eq!(calculate_success_score(&episode), 1.0);

    // Partial success
    episode.outcome = Some(crate::types::TaskOutcome::PartialSuccess {
        verdict: "Partial".to_string(),
        completed: vec![],
        failed: vec![],
    });
    assert_eq!(calculate_success_score(&episode), 0.6);

    // Failure
    episode.outcome = Some(crate::types::TaskOutcome::Failure {
        reason: "Failed".to_string(),
        error_details: None,
    });
    assert_eq!(calculate_success_score(&episode), 0.2);
}

#[test]
fn test_completeness_score() {
    let context = TaskContext::default();
    let mut episode = Episode::new("test".to_string(), context, TaskType::CodeGeneration);

    // Incomplete, no steps
    assert_eq!(calculate_completeness_score(&episode), 0.0);

    // Add some steps but not complete
    for i in 1..=5 {
        let step = crate::episode::ExecutionStep::new(i, "tool".to_string(), "action".to_string());
        episode.steps.push(step);
    }
    let score = calculate_completeness_score(&episode);
    assert!(score > 0.0 && score < 1.0);

    // Complete (needs both outcome and end_time)
    episode.outcome = Some(crate::types::TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });
    episode.end_time = Some(Utc::now());
    assert_eq!(calculate_completeness_score(&episode), 1.0);
}

#[test]
fn test_field_importance_score() {
    assert_eq!(
        calculate_field_importance_score(&SearchField::Description),
        1.0
    );
    assert_eq!(calculate_field_importance_score(&SearchField::Outcome), 0.8);
    assert_eq!(calculate_field_importance_score(&SearchField::Steps), 0.6);
    assert_eq!(calculate_field_importance_score(&SearchField::Tags), 0.5);
    assert_eq!(calculate_field_importance_score(&SearchField::Domain), 0.4);
}

#[test]
fn test_calculate_ranking_score() {
    let context = TaskContext::default();
    let episode = Episode::new("test".to_string(), context, TaskType::CodeGeneration);

    let weights = RankingWeights::default();
    let mode = SearchMode::Exact;
    let field = SearchField::Description;

    let score = calculate_ranking_score(&episode, &mode, 1.0, &field, &weights);

    // Score should be between 0 and 1
    assert!((0.0..=1.0).contains(&score));
}

#[test]
fn test_rank_search_results() {
    let low_score = SearchResult {
        item: "episode1",
        score: 0.5,
        matches: vec![],
    };
    let high_score = SearchResult {
        item: "episode2",
        score: 0.9,
        matches: vec![],
    };
    let mid_score = SearchResult {
        item: "episode3",
        score: 0.7,
        matches: vec![],
    };

    let results = vec![low_score, high_score, mid_score];
    let ranked = rank_search_results(results);

    // Should be sorted by score (highest first)
    assert_eq!(ranked[0].score, 0.9);
    assert_eq!(ranked[1].score, 0.7);
    assert_eq!(ranked[2].score, 0.5);
}
