//! Comprehensive tests for semantic summarization.

use memory_core::pre_storage::SalientFeatures;
use memory_core::semantic::{EpisodeSummary, SemanticSummarizer};
use memory_core::{
    ComplexityLevel, Episode, ExecutionResult, ExecutionStep, TaskContext, TaskOutcome, TaskType,
};

fn create_test_episode() -> Episode {
    Episode::new(
        "Test task".to_string(),
        TaskContext::default(),
        TaskType::Testing,
    )
}

fn create_rich_context() -> TaskContext {
    TaskContext {
        language: Some("rust".to_string()),
        framework: Some("tokio".to_string()),
        complexity: ComplexityLevel::Moderate,
        domain: "authentication".to_string(),
        tags: vec!["security".to_string(), "async".to_string()],
    }
}

#[tokio::test]
async fn test_summarize_complete_episode() {
    let summarizer = SemanticSummarizer::new();
    let mut episode = Episode::new(
        "Implement user authentication".to_string(),
        create_rich_context(),
        TaskType::CodeGeneration,
    );

    // Add execution steps
    for i in 0..5 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i), format!("Action {}", i));
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    // Complete with outcome
    episode.complete(TaskOutcome::Success {
        verdict: "Authentication implemented successfully".to_string(),
        artifacts: vec!["auth.rs".to_string(), "auth_test.rs".to_string()],
    });

    let summary = summarizer.summarize_episode(&episode).await.unwrap();

    // Verify summary structure
    assert_eq!(summary.episode_id, episode.episode_id);
    assert!(!summary.summary_text.is_empty());
    assert!(!summary.key_concepts.is_empty());
    assert!(!summary.key_steps.is_empty());

    // Verify summary content
    assert!(summary.summary_text.contains("Task:"));
    assert!(summary.summary_text.contains("Outcome:"));
    assert!(summary.key_concepts.contains(&"rust".to_string()));
    assert!(summary.key_concepts.contains(&"authentication".to_string()));
}

#[tokio::test]
async fn test_summarize_incomplete_episode() {
    let summarizer = SemanticSummarizer::new();
    let mut episode = create_test_episode();

    // Add just one step, no completion
    let mut step = ExecutionStep::new(1, "tool".to_string(), "Action".to_string());
    step.result = Some(ExecutionResult::Success {
        output: "OK".to_string(),
    });
    episode.add_step(step);

    // Should handle gracefully
    let summary = summarizer.summarize_episode(&episode).await.unwrap();

    assert_eq!(summary.episode_id, episode.episode_id);
    assert!(!summary.summary_text.is_empty());
    assert!(summary.summary_text.contains("Task:"));
}

#[test]
fn test_extract_key_concepts() {
    let summarizer = SemanticSummarizer::new();
    let episode = Episode::new(
        "Implement JWT authentication with Redis caching".to_string(),
        create_rich_context(),
        TaskType::CodeGeneration,
    );

    let concepts = summarizer.extract_key_concepts(&episode);

    // Should extract from task description
    assert!(concepts.contains(&"authentication".to_string()));
    assert!(concepts.contains(&"caching".to_string()));

    // Should extract from context
    assert!(concepts.contains(&"rust".to_string()));
    assert!(concepts.contains(&"tokio".to_string()));
    assert!(concepts.contains(&"security".to_string()));

    // Should extract task type
    assert!(concepts.iter().any(|c| c.contains("code_generation")));

    // Should be normalized (lowercase)
    assert!(concepts
        .iter()
        .all(|c| c.chars().all(|ch| !ch.is_uppercase())));

    // Should be limited
    assert!(concepts.len() <= 20);
}

#[test]
fn test_extract_key_concepts_with_salient_features() {
    let summarizer = SemanticSummarizer::new();
    let mut episode = create_test_episode();

    // Add salient features
    let mut features = SalientFeatures::new();
    features
        .critical_decisions
        .push("Chose async implementation for better performance".to_string());
    features
        .key_insights
        .push("Builder pattern simplifies configuration".to_string());
    episode.salient_features = Some(features);

    let concepts = summarizer.extract_key_concepts(&episode);

    // Should extract from salient features
    assert!(concepts
        .iter()
        .any(|c| c.contains("async") || c.contains("implementation")));
    assert!(concepts
        .iter()
        .any(|c| c.contains("builder") || c.contains("pattern")));
}

#[test]
fn test_extract_key_steps() {
    let summarizer = SemanticSummarizer::new();
    let mut episode = create_test_episode();

    // Add diverse steps
    for i in 0..10 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i), format!("Action {}", i));
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    let key_steps = summarizer.extract_key_steps(&episode);

    // Should extract limited number of steps
    assert!(key_steps.len() <= 5);

    // Should include first step
    assert!(key_steps[0].contains("Step 1"));

    // Should include last step
    assert!(key_steps.last().unwrap().contains("Step 10"));
}

#[test]
fn test_extract_key_steps_with_errors() {
    let summarizer = SemanticSummarizer::new();
    let mut episode = create_test_episode();

    // Add successful steps
    for i in 0..3 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i), format!("Action {}", i));
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    // Add error step
    let mut error_step =
        ExecutionStep::new(4, "validator".to_string(), "Validate input".to_string());
    error_step.result = Some(ExecutionResult::Error {
        message: "Validation failed".to_string(),
    });
    episode.add_step(error_step);

    // Add recovery step
    let mut recovery_step =
        ExecutionStep::new(5, "validator".to_string(), "Re-validate".to_string());
    recovery_step.result = Some(ExecutionResult::Success {
        output: "Valid".to_string(),
    });
    episode.add_step(recovery_step);

    let key_steps = summarizer.extract_key_steps(&episode);

    // Should prioritize error steps
    assert!(key_steps
        .iter()
        .any(|s| s.contains("Step 4") && s.contains("[ERROR]")));
}

#[test]
fn test_extract_key_steps_empty_episode() {
    let summarizer = SemanticSummarizer::new();
    let episode = create_test_episode();

    let key_steps = summarizer.extract_key_steps(&episode);

    // Should handle empty episode
    assert!(key_steps.is_empty());
}

#[test]
fn test_extract_key_steps_single_step() {
    let summarizer = SemanticSummarizer::new();
    let mut episode = create_test_episode();

    let mut step = ExecutionStep::new(1, "tool".to_string(), "Action".to_string());
    step.result = Some(ExecutionResult::Success {
        output: "OK".to_string(),
    });
    episode.add_step(step);

    let key_steps = summarizer.extract_key_steps(&episode);

    // Should include the single step
    assert_eq!(key_steps.len(), 1);
    assert!(key_steps[0].contains("Step 1"));
}

#[test]
fn test_summary_length_constraints() {
    let summarizer = SemanticSummarizer::new();
    let mut episode = create_test_episode();

    // Add many steps to generate longer summary
    for i in 0..100 {
        let mut step = ExecutionStep::new(
            i + 1,
            format!("tool_{}", i),
            format!("Very long action description number {}", i),
        );
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "Task completed successfully with many steps and lots of details".to_string(),
        artifacts: vec![],
    });

    let summary_text = summarizer.generate_summary_text(&episode);
    let word_count = summary_text.split_whitespace().count();

    // Should respect max length (with some tolerance for truncation marker)
    assert!(word_count <= 205); // 200 + tolerance for "..."
}

#[test]
fn test_summary_with_salient_features() {
    let summarizer = SemanticSummarizer::new();
    let mut episode = create_test_episode();

    // Add steps
    let mut step = ExecutionStep::new(1, "planner".to_string(), "Plan approach".to_string());
    step.result = Some(ExecutionResult::Success {
        output: "Plan ready".to_string(),
    });
    episode.add_step(step);

    // Add salient features
    let mut features = SalientFeatures::new();
    features
        .critical_decisions
        .push("Chose async implementation".to_string());
    features
        .error_recovery_patterns
        .push("Timeout -> Retry with backoff".to_string());
    features
        .key_insights
        .push("Builder pattern works well".to_string());
    episode.salient_features = Some(features);

    episode.complete(TaskOutcome::Success {
        verdict: "Implemented successfully".to_string(),
        artifacts: vec![],
    });

    let summary_text = summarizer.generate_summary_text(&episode);

    // Should include salient features
    assert!(summary_text.contains("Key decision:") || summary_text.contains("async"));
    assert!(summary_text.contains("Recovery pattern:") || summary_text.contains("Timeout"));
    assert!(summary_text.contains("Insight:") || summary_text.contains("Builder"));
}

#[test]
fn test_summary_without_salient_features() {
    let summarizer = SemanticSummarizer::new();
    let mut episode = create_test_episode();

    let mut step = ExecutionStep::new(1, "tool".to_string(), "Action".to_string());
    step.result = Some(ExecutionResult::Success {
        output: "OK".to_string(),
    });
    episode.add_step(step);

    episode.complete(TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });

    let summary_text = summarizer.generate_summary_text(&episode);

    // Should still generate valid summary without salient features
    assert!(summary_text.contains("Task:"));
    assert!(summary_text.contains("Outcome:"));
}

#[test]
fn test_summary_edge_case_empty_episode() {
    let summarizer = SemanticSummarizer::new();
    let episode = create_test_episode();

    let summary_text = summarizer.generate_summary_text(&episode);

    // Should handle empty episode gracefully
    assert!(!summary_text.is_empty());
    assert!(summary_text.contains("Task:"));
}

#[test]
fn test_summary_edge_case_many_steps() {
    let summarizer = SemanticSummarizer::new();
    let mut episode = create_test_episode();

    // Add 150 steps
    for i in 0..150 {
        let mut step = ExecutionStep::new(i + 1, "tool".to_string(), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    episode.complete(TaskOutcome::Success {
        verdict: "Completed".to_string(),
        artifacts: vec![],
    });

    let summary_text = summarizer.generate_summary_text(&episode);

    // Should handle many steps without crashing
    assert!(summary_text.contains("150 steps"));
}

#[test]
fn test_custom_configuration() {
    let summarizer = SemanticSummarizer::with_config(50, 100, 3);

    let mut episode = create_test_episode();

    for i in 0..10 {
        let mut step = ExecutionStep::new(i + 1, "tool".to_string(), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    let key_steps = summarizer.extract_key_steps(&episode);

    // Should respect custom max_key_steps
    assert!(key_steps.len() <= 3);
}

#[test]
fn test_outcome_variations() {
    let summarizer = SemanticSummarizer::new();

    // Test Success outcome
    let mut episode1 = create_test_episode();
    episode1.complete(TaskOutcome::Success {
        verdict: "All good".to_string(),
        artifacts: vec!["file1.rs".to_string()],
    });
    let summary1 = summarizer.generate_summary_text(&episode1);
    assert!(summary1.contains("Success"));
    assert!(summary1.contains("file1.rs"));

    // Test PartialSuccess outcome
    let mut episode2 = create_test_episode();
    episode2.complete(TaskOutcome::PartialSuccess {
        verdict: "Mostly done".to_string(),
        completed: vec!["task1".to_string()],
        failed: vec!["task2".to_string()],
    });
    let summary2 = summarizer.generate_summary_text(&episode2);
    assert!(summary2.contains("Partial success"));

    // Test Failure outcome
    let mut episode3 = create_test_episode();
    episode3.complete(TaskOutcome::Failure {
        reason: "Compilation error".to_string(),
        error_details: Some("Type mismatch".to_string()),
    });
    let summary3 = summarizer.generate_summary_text(&episode3);
    assert!(summary3.contains("Failure"));
    assert!(summary3.contains("Type mismatch"));
}

#[tokio::test]
async fn test_episode_summary_serialization() {
    let summary = EpisodeSummary {
        episode_id: uuid::Uuid::new_v4(),
        summary_text: "Test summary".to_string(),
        key_concepts: vec!["concept1".to_string(), "concept2".to_string()],
        key_steps: vec!["Step 1: test".to_string()],
        summary_embedding: Some(vec![0.1, 0.2, 0.3]),
        created_at: chrono::Utc::now(),
    };

    // Test serialization
    let json = serde_json::to_string(&summary).unwrap();
    assert!(json.contains("Test summary"));

    // Test deserialization
    let deserialized: EpisodeSummary = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.summary_text, summary.summary_text);
    assert_eq!(deserialized.key_concepts, summary.key_concepts);
}

#[test]
fn test_stopword_filtering() {
    let summarizer = SemanticSummarizer::new();
    let episode = Episode::new(
        "The implementation will have been done with the framework".to_string(),
        TaskContext::default(),
        TaskType::CodeGeneration,
    );

    let concepts = summarizer.extract_key_concepts(&episode);

    // Should filter out stopwords
    assert!(!concepts.contains(&"the".to_string()));
    assert!(!concepts.contains(&"will".to_string()));
    assert!(!concepts.contains(&"have".to_string()));
    assert!(!concepts.contains(&"been".to_string()));
    assert!(!concepts.contains(&"with".to_string()));

    // Should keep meaningful words
    assert!(concepts.contains(&"implementation".to_string()));
    assert!(concepts.contains(&"framework".to_string()));
}

#[test]
fn test_concept_normalization() {
    let summarizer = SemanticSummarizer::new();
    let episode = Episode::new(
        "IMPLEMENT Authentication, Testing! Security.".to_string(),
        TaskContext::default(),
        TaskType::CodeGeneration,
    );

    let concepts = summarizer.extract_key_concepts(&episode);

    // All should be lowercase
    assert!(concepts
        .iter()
        .all(|c| c.chars().all(|ch| !ch.is_uppercase())));

    // Should strip punctuation
    assert!(concepts.contains(&"authentication".to_string()));
    assert!(concepts.contains(&"testing".to_string()));
    assert!(concepts.contains(&"security".to_string()));
}

// Note: extract_step_number is tested indirectly through extract_key_steps
// which uses salient features that reference step numbers
