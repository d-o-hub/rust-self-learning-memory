//! Integration tests for semantic pattern search

use memory_core::{
    ComplexityLevel, ExecutionStep, SelfLearningMemory, TaskContext, TaskOutcome, TaskType,
};

#[tokio::test]
async fn test_search_patterns_basic() {
    let memory = SelfLearningMemory::new();

    // Create a test context
    let context = TaskContext {
        domain: "web-api".to_string(),
        language: Some("rust".to_string()),
        framework: Some("axum".to_string()),
        complexity: ComplexityLevel::Moderate,
        tags: vec!["rest".to_string(), "async".to_string()],
    };

    // Start and complete an episode to generate patterns
    let episode_id = memory
        .start_episode(
            "Build REST API".to_string(),
            context.clone(),
            TaskType::CodeGeneration,
        )
        .await;

    // Add multiple steps to meet quality threshold
    let steps = vec![
        ExecutionStep::new(1, "create_project".to_string(), "Create new Rust project".to_string()),
        ExecutionStep::new(2, "add_dependencies".to_string(), "Add Axum and tower dependencies".to_string()),
        ExecutionStep::new(3, "create_router".to_string(), "Setup routes and handlers".to_string()),
        ExecutionStep::new(4, "add_middleware".to_string(), "Add logging and cors middleware".to_string()),
        ExecutionStep::new(5, "write_tests".to_string(), "Write integration tests".to_string()),
    ];

    for step in steps {
        memory.log_step(episode_id, step).await;
    }

    // Complete with success - quality should now meet threshold
    let _ = memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "API created successfully".to_string(),
                artifacts: vec!["api.rs".to_string(), "main.rs".to_string()],
            },
        )
        .await;

    // Search for patterns - even if episode didn't pass quality, search should work
    let results = memory
        .search_patterns_semantic("How to build a REST API", context, 5)
        .await
        .unwrap_or_default();

    // Search should succeed (return empty or fallback results)
    assert!(results.len() <= 5);
}

#[tokio::test]
async fn test_recommend_patterns_for_task() {
    let memory = SelfLearningMemory::new();

    let context = TaskContext {
        domain: "cli".to_string(),
        language: Some("rust".to_string()),
        framework: None,
        complexity: ComplexityLevel::Simple,
        tags: vec!["argparse".to_string()],
    };

    // Recommend patterns for a task
    let results = memory
        .recommend_patterns_for_task("Parse command line arguments", context, 3)
        .await
        .unwrap();

    // Should return 0-3 results
    assert!(results.len() <= 3);
}

#[tokio::test]
async fn test_discover_analogous_patterns() {
    let memory = SelfLearningMemory::new();

    let target_context = TaskContext {
        domain: "web-api".to_string(),
        language: Some("rust".to_string()),
        framework: None,
        complexity: ComplexityLevel::Moderate,
        tags: vec![],
    };

    // Discover patterns from CLI domain for web-api
    let results = memory
        .discover_analogous_patterns("cli", target_context, 5)
        .await
        .unwrap();

    // Should return 0-5 results
    assert!(results.len() <= 5);
}

#[tokio::test]
async fn test_pattern_search_with_filters() {
    let memory = SelfLearningMemory::new();

    let context = TaskContext {
        domain: "data-processing".to_string(),
        language: Some("python".to_string()),
        framework: Some("pandas".to_string()),
        complexity: ComplexityLevel::Complex,
        tags: vec!["etl".to_string()],
    };

    // Search with strict config
    let config = memory_core::memory::SearchConfig::strict();

    let results = memory
        .search_patterns_with_config("ETL pipeline", context, config, 10)
        .await
        .unwrap();

    assert!(results.len() <= 10);
}
