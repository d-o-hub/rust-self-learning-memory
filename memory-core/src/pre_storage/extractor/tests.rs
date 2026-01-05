//! Tests for salient feature extraction.

use super::*;
use crate::episode::ExecutionStep;
use crate::types::{ExecutionResult, Reflection, TaskContext, TaskOutcome, TaskType};
use chrono::Utc;

fn create_test_episode() -> Episode {
    Episode::new(
        "Test task".to_string(),
        TaskContext::default(),
        TaskType::Testing,
    )
}

#[test]
fn test_salient_features_new() {
    let features = SalientFeatures::new();
    assert!(features.is_empty());
    assert_eq!(features.count(), 0);
}

#[test]
fn test_salient_features_count() {
    let mut features = SalientFeatures::new();
    assert_eq!(features.count(), 0);

    features.critical_decisions.push("Decision 1".to_string());
    assert_eq!(features.count(), 1);

    features
        .tool_combinations
        .push(vec!["tool1".to_string(), "tool2".to_string()]);
    assert_eq!(features.count(), 2);

    features.key_insights.push("Insight 1".to_string());
    features.key_insights.push("Insight 2".to_string());
    assert_eq!(features.count(), 4);
}

#[test]
fn test_extract_empty_episode() {
    let extractor = SalientExtractor::new();
    let episode = create_test_episode();
    let features = extractor.extract(&episode);

    assert!(features.is_empty());
}

#[test]
fn test_extract_critical_decisions() {
    let extractor = SalientExtractor::new();
    let mut episode = create_test_episode();

    // Add step with decision keyword
    let mut step1 = ExecutionStep::new(
        1,
        "planner".to_string(),
        "Choose async implementation strategy".to_string(),
    );
    step1.result = Some(ExecutionResult::Success {
        output: "Strategy selected".to_string(),
    });
    episode.add_step(step1);

    // Add step with strategy parameter
    let mut step2 = ExecutionStep::new(
        2,
        "executor".to_string(),
        "Execute with chosen strategy".to_string(),
    );
    step2.parameters = serde_json::json!({
        "strategy": "async",
        "approach": "tokio"
    });
    step2.result = Some(ExecutionResult::Success {
        output: "Executed".to_string(),
    });
    episode.add_step(step2);

    episode.complete(TaskOutcome::Success {
        verdict: "Successfully implemented async solution".to_string(),
        artifacts: vec![],
    });

    let features = extractor.extract(&episode);
    assert!(!features.critical_decisions.is_empty());
    assert!(features
        .critical_decisions
        .iter()
        .any(|d| d.contains("Choose async")));
    assert!(features
        .critical_decisions
        .iter()
        .any(|d| d.contains("strategy")));
}

#[test]
fn test_extract_tool_combinations() {
    let extractor = SalientExtractor::new();
    let mut episode = create_test_episode();

    // Add successful sequence
    for i in 0..4 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), format!("action_{i}"));
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    let features = extractor.extract(&episode);
    assert!(!features.tool_combinations.is_empty());
    assert_eq!(features.tool_combinations[0].len(), 4);
    assert_eq!(features.tool_combinations[0][0], "tool_0");
    assert_eq!(features.tool_combinations[0][3], "tool_3");
}

#[test]
fn test_extract_tool_combinations_with_failures() {
    let extractor = SalientExtractor::new();
    let mut episode = create_test_episode();

    // Add success sequence
    for i in 0..3 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    // Add failure
    let mut fail_step = ExecutionStep::new(4, "tool_3".to_string(), "action".to_string());
    fail_step.result = Some(ExecutionResult::Error {
        message: "Error".to_string(),
    });
    episode.add_step(fail_step);

    // Add another success sequence
    for i in 4..6 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    let features = extractor.extract(&episode);
    assert_eq!(features.tool_combinations.len(), 2);
    assert_eq!(features.tool_combinations[0].len(), 3); // First sequence
    assert_eq!(features.tool_combinations[1].len(), 2); // Second sequence
}

#[test]
fn test_extract_error_recovery_patterns() {
    let extractor = SalientExtractor::new();
    let mut episode = create_test_episode();

    // Add error step
    let mut error_step = ExecutionStep::new(
        1,
        "connector".to_string(),
        "Connect to database".to_string(),
    );
    error_step.result = Some(ExecutionResult::Error {
        message: "Connection timeout".to_string(),
    });
    episode.add_step(error_step);

    // Add recovery step
    let mut recovery_step =
        ExecutionStep::new(2, "connector".to_string(), "Retry with backoff".to_string());
    recovery_step.result = Some(ExecutionResult::Success {
        output: "Connected".to_string(),
    });
    episode.add_step(recovery_step);

    let features = extractor.extract(&episode);
    assert!(!features.error_recovery_patterns.is_empty());
    assert!(features.error_recovery_patterns[0].contains("Connection timeout"));
    assert!(features.error_recovery_patterns[0].contains("Retry with backoff"));
}

#[test]
fn test_extract_multi_step_error_recovery() {
    let extractor = SalientExtractor::new();
    let mut episode = create_test_episode();

    // Add error
    let mut error_step = ExecutionStep::new(1, "parser".to_string(), "Parse input".to_string());
    error_step.result = Some(ExecutionResult::Error {
        message: "Invalid format".to_string(),
    });
    episode.add_step(error_step);

    // Add multiple recovery steps
    let recovery_actions = ["Sanitize input", "Validate format", "Re-parse"];
    for (i, action) in recovery_actions.iter().enumerate() {
        let mut step = ExecutionStep::new(i + 2, "parser".to_string(), (*action).to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    let features = extractor.extract(&episode);
    assert!(!features.error_recovery_patterns.is_empty());
    // Should capture multi-step recovery
    assert!(features
        .error_recovery_patterns
        .iter()
        .any(|p| p.contains('[')));
}

#[test]
fn test_extract_key_insights_from_reflection() {
    let extractor = SalientExtractor::new();
    let mut episode = create_test_episode();

    episode.reflection = Some(Reflection {
        successes: vec![
            "Efficient error handling pattern".to_string(),
            "Good test coverage".to_string(),
        ],
        improvements: vec!["Could reduce duplication".to_string()],
        insights: vec![
            "Builder pattern works well".to_string(),
            "Async improves performance".to_string(),
        ],
        generated_at: Utc::now(),
    });

    let features = extractor.extract(&episode);
    assert!(!features.key_insights.is_empty());
    assert!(features
        .key_insights
        .iter()
        .any(|i| i.contains("Builder pattern")));
    assert!(features
        .key_insights
        .iter()
        .any(|i| i.contains("Async improves")));
}

#[test]
fn test_extract_key_insights_from_outcome() {
    let extractor = SalientExtractor::new();
    let mut episode = create_test_episode();

    episode.complete(TaskOutcome::Success {
        verdict: "Implementation complete".to_string(),
        artifacts: vec!["auth.rs".to_string(), "auth_test.rs".to_string()],
    });

    let features = extractor.extract(&episode);
    assert!(!features.key_insights.is_empty());
    assert!(features
        .key_insights
        .iter()
        .any(|i| i.contains("Artifacts produced")));
}

#[test]
fn test_extract_comprehensive_features() {
    let extractor = SalientExtractor::new();
    let mut episode = create_test_episode();

    // Add diverse steps with decisions
    let mut step1 = ExecutionStep::new(
        1,
        "planner".to_string(),
        "Choose implementation strategy".to_string(),
    );
    step1.parameters = serde_json::json!({"strategy": "async"});
    step1.result = Some(ExecutionResult::Success {
        output: "Strategy chosen".to_string(),
    });
    episode.add_step(step1);

    // Add tool sequence
    for i in 1..4 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), format!("action_{i}"));
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    // Add error and recovery
    let mut error_step =
        ExecutionStep::new(5, "validator".to_string(), "Validate result".to_string());
    error_step.result = Some(ExecutionResult::Error {
        message: "Validation failed".to_string(),
    });
    episode.add_step(error_step);

    let mut recovery_step = ExecutionStep::new(
        6,
        "validator".to_string(),
        "Re-validate with fix".to_string(),
    );
    recovery_step.result = Some(ExecutionResult::Success {
        output: "Valid".to_string(),
    });
    episode.add_step(recovery_step);

    // Add reflection and outcome
    episode.reflection = Some(Reflection {
        successes: vec!["Async strategy worked well".to_string()],
        improvements: vec!["Better error messages needed".to_string()],
        insights: vec!["Validation should happen earlier".to_string()],
        generated_at: Utc::now(),
    });

    episode.complete(TaskOutcome::Success {
        verdict: "Successfully implemented".to_string(),
        artifacts: vec!["implementation.rs".to_string()],
    });

    let features = extractor.extract(&episode);

    // Should have extracted features in all categories
    assert!(!features.critical_decisions.is_empty());
    assert!(!features.tool_combinations.is_empty());
    assert!(!features.error_recovery_patterns.is_empty());
    assert!(!features.key_insights.is_empty());

    assert!(features.count() > 5);
}

#[test]
fn test_extract_handles_partial_success() {
    let extractor = SalientExtractor::new();
    let mut episode = create_test_episode();

    episode.complete(TaskOutcome::PartialSuccess {
        verdict: "Core functionality working".to_string(),
        completed: vec!["login".to_string()],
        failed: vec!["logout".to_string()],
    });

    let features = extractor.extract(&episode);
    assert!(!features.critical_decisions.is_empty());
    assert!(features
        .critical_decisions
        .iter()
        .any(|d| d.contains("Partial success")));
}

#[test]
fn test_extract_handles_failure() {
    let extractor = SalientExtractor::new();
    let mut episode = create_test_episode();

    episode.complete(TaskOutcome::Failure {
        reason: "Compilation errors".to_string(),
        error_details: Some("Type mismatch".to_string()),
    });

    let features = extractor.extract(&episode);
    assert!(!features.critical_decisions.is_empty());
    assert!(features
        .critical_decisions
        .iter()
        .any(|d| d.contains("Failure reason")));
}

#[test]
fn test_no_tool_combinations_for_short_sequences() {
    let extractor = SalientExtractor::new();
    let mut episode = create_test_episode();

    // Add only one successful step
    let mut step = ExecutionStep::new(1, "tool".to_string(), "action".to_string());
    step.result = Some(ExecutionResult::Success {
        output: "OK".to_string(),
    });
    episode.add_step(step);

    let features = extractor.extract(&episode);
    assert!(features.tool_combinations.is_empty());
}

#[test]
fn test_timeout_error_recovery() {
    let extractor = SalientExtractor::new();
    let mut episode = create_test_episode();

    // Add timeout error
    let mut timeout_step = ExecutionStep::new(1, "fetcher".to_string(), "Fetch data".to_string());
    timeout_step.result = Some(ExecutionResult::Timeout);
    episode.add_step(timeout_step);

    // Add recovery
    let mut recovery_step =
        ExecutionStep::new(2, "fetcher".to_string(), "Retry with timeout".to_string());
    recovery_step.result = Some(ExecutionResult::Success {
        output: "Fetched".to_string(),
    });
    episode.add_step(recovery_step);

    let features = extractor.extract(&episode);
    assert!(!features.error_recovery_patterns.is_empty());
    assert!(features.error_recovery_patterns[0].contains("Timeout"));
}
