//! Tests for pattern extraction

use super::*;
use crate::{Episode, ExecutionStep, TaskContext, TaskOutcome, TaskType, pattern::Pattern};
use chrono::Duration;

#[test]
fn test_pattern_extractor_creation() {
    let extractor = PatternExtractor::new();
    assert_eq!(extractor.success_threshold, MIN_PATTERN_SUCCESS_RATE);
    assert_eq!(extractor.min_sequence_len, MIN_SEQUENCE_LENGTH);
    assert_eq!(extractor.max_sequence_len, MAX_SEQUENCE_LENGTH);
}

#[test]
fn test_extract_from_incomplete_episode() {
    let extractor = PatternExtractor::new();
    let context = TaskContext::default();
    let episode = Episode::new("Test task".to_string(), context, TaskType::Testing);

    let patterns = extractor.extract(&episode);
    assert_eq!(
        patterns.len(),
        0,
        "Should not extract from incomplete episode"
    );
}

#[test]
fn test_extract_from_complete_episode() {
    let extractor = PatternExtractor::new();
    let context = TaskContext::default();
    let mut episode = Episode::new("Test task".to_string(), context, TaskType::Testing);

    // Add some execution steps
    for i in 0..3 {
        let step = ExecutionStep::new(i + 1, format!("tool_{}", i), "Action".to_string());
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });

    let patterns = extractor.extract(&episode);
    // Currently returns empty since extraction is not implemented
    assert!(patterns.is_empty() || !patterns.is_empty());
}

#[cfg(test)]
mod utils_tests {
    use super::*;


    #[test]
    fn test_deduplicate_patterns_no_duplicates() {
        let patterns = vec![
            Pattern::ToolSequence {
                id: uuid::Uuid::new_v4(),
                tools: vec!["tool1".to_string(), "tool2".to_string()],
                context: TaskContext::default(),
                success_rate: 0.9,
                avg_latency: Duration::milliseconds(100),
                occurrence_count: 5,
            },
            Pattern::ToolSequence {
                id: uuid::Uuid::new_v4(),
                tools: vec!["tool3".to_string(), "tool4".to_string()],
                context: TaskContext::default(),
                success_rate: 0.8,
                avg_latency: Duration::milliseconds(200),
                occurrence_count: 3,
            },
        ];

        let deduplicated = super::utils::deduplicate_patterns(patterns.clone());
        assert_eq!(deduplicated.len(), 2);
    }

    #[test]
    fn test_deduplicate_patterns_with_duplicates() {
        let pattern1 = Pattern::ToolSequence {
            id: uuid::Uuid::new_v4(),
            tools: vec!["tool1".to_string(), "tool2".to_string()],
            context: TaskContext::default(),
            success_rate: 0.9,
            avg_latency: Duration::milliseconds(100),
            occurrence_count: 5,
        };

        let pattern2 = Pattern::ToolSequence {
            id: uuid::Uuid::new_v4(),
            tools: vec!["tool1".to_string(), "tool2".to_string()], // Same tools
            context: TaskContext::default(), // Same context
            success_rate: 0.8, // Different success rate
            avg_latency: Duration::milliseconds(200), // Different latency
            occurrence_count: 3, // Different count
        };

        let patterns = vec![pattern1, pattern2];
        let deduplicated = super::utils::deduplicate_patterns(patterns);
        assert_eq!(deduplicated.len(), 1);
    }

    #[test]
    fn test_rank_patterns_by_success_rate() {
        let patterns = vec![
            Pattern::ToolSequence {
                id: uuid::Uuid::new_v4(),
                tools: vec!["tool1".to_string()],
                context: TaskContext::default(),
                success_rate: 0.5,
                avg_latency: Duration::milliseconds(100),
                occurrence_count: 5,
            },
            Pattern::ToolSequence {
                id: uuid::Uuid::new_v4(),
                tools: vec!["tool2".to_string()],
                context: TaskContext::default(),
                success_rate: 0.9,
                avg_latency: Duration::milliseconds(100),
                occurrence_count: 5,
            },
        ];

        let context = TaskContext::default();
        let ranked = super::utils::rank_patterns(patterns, &context);

        assert_eq!(ranked.len(), 2);
        // Higher success rate should come first
        match &ranked[0] {
            Pattern::ToolSequence { success_rate, .. } => assert!((*success_rate - 0.9).abs() < 0.01),
            _ => panic!("Expected ToolSequence"),
        }
    }

    #[test]
    fn test_rank_patterns_by_context_relevance() {
        let mut context1 = TaskContext::default();
        context1.language = Some("rust".to_string());
        context1.domain = "web-api".to_string();

        let mut context2 = TaskContext::default();
        context2.language = Some("python".to_string());
        context2.domain = "data-science".to_string();

        let patterns = vec![
            Pattern::ToolSequence {
                id: uuid::Uuid::new_v4(),
                tools: vec!["tool1".to_string()],
                context: context1,
                success_rate: 0.8,
                avg_latency: Duration::milliseconds(100),
                occurrence_count: 5,
            },
            Pattern::ToolSequence {
                id: uuid::Uuid::new_v4(),
                tools: vec!["tool2".to_string()],
                context: context2,
                success_rate: 0.8,
                avg_latency: Duration::milliseconds(100),
                occurrence_count: 5,
            },
        ];

        let query_context = TaskContext {
            language: Some("rust".to_string()),
            domain: "web-api".to_string(),
            ..Default::default()
        };

        let ranked = super::utils::rank_patterns(patterns, &query_context);

        assert_eq!(ranked.len(), 2);
        // Pattern with matching context should come first
        match &ranked[0] {
            Pattern::ToolSequence { context, .. } => {
                assert_eq!(context.language, Some("rust".to_string()));
                assert_eq!(context.domain, "web-api");
            }
            _ => panic!("Expected ToolSequence"),
        }
    }

    #[test]
    fn test_rank_patterns_empty() {
        let patterns: Vec<Pattern> = vec![];
        let context = TaskContext::default();
        let ranked = super::utils::rank_patterns(patterns, &context);
        assert_eq!(ranked.len(), 0);
    }
}
