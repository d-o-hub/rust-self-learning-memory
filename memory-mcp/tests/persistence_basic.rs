//! Basic persistence tests for episodes and patterns

mod persistence_helper;

use do_memory_core::{ComplexityLevel, ExecutionStep, TaskContext, TaskOutcome, TaskType};
use do_memory_mcp::{MemoryMCPServer, SandboxConfig};
use std::sync::Arc;

#[tokio::test]
async fn test_episode_persistence_in_redb() {
    let (memory, _temp_dir) = persistence_helper::setup_persistent_memory().await.unwrap();

    // Create episode
    let episode_id = memory
        .start_episode(
            "Persistent Episode Test".to_string(),
            TaskContext {
                domain: "persistence".to_string(),
                language: Some("rust".to_string()),
                framework: Some("tokio".to_string()),
                complexity: ComplexityLevel::Simple,
                tags: vec!["test".to_string(), "persistence".to_string()],
            },
            TaskType::Testing,
        )
        .await;

    // Log steps
    let step = ExecutionStep::new(
        1,
        "test_tool".to_string(),
        "Testing persistence".to_string(),
    );
    memory.log_step(episode_id, step).await;

    // Complete episode
    let outcome = TaskOutcome::Success {
        verdict: "Episode persisted successfully".to_string(),
        artifacts: vec!["test_result.txt".to_string()],
    };
    memory.complete_episode(episode_id, outcome).await.unwrap();

    // Verify episode exists in storage
    let retrieved_episode = memory.get_episode(episode_id).await.unwrap();
    assert_eq!(
        retrieved_episode.task_description,
        "Persistent Episode Test"
    );
    assert_eq!(retrieved_episode.steps.len(), 1);
    assert!(retrieved_episode.is_complete());

    // Test memory query through MCP
    let mcp_server = Arc::new(
        MemoryMCPServer::new(SandboxConfig::restrictive(), memory.clone())
            .await
            .unwrap(),
    );

    let query_result = mcp_server
        .query_memory(
            "Persistent Episode".to_string(),
            "persistence".to_string(),
            None,
            10,
            "relevance".to_string(),
            None,
        )
        .await
        .unwrap();

    let episodes = query_result["episodes"].as_array().unwrap();
    assert_eq!(episodes.len(), 1);

    let episode = &episodes[0];
    assert_eq!(episode["task_description"], "Persistent Episode Test");
}

#[tokio::test]
async fn test_pattern_persistence_in_redb() {
    let (memory, _temp_dir) = persistence_helper::setup_persistent_memory().await.unwrap();

    // Create multiple episodes to generate patterns
    for i in 1..=3 {
        let episode_id = memory
            .start_episode(
                format!("Pattern Test Episode {i}"),
                TaskContext {
                    domain: "patterns".to_string(),
                    language: Some("rust".to_string()),
                    framework: Some("tokio".to_string()),
                    complexity: ComplexityLevel::Simple,
                    tags: vec!["pattern".to_string(), "test".to_string()],
                },
                TaskType::CodeGeneration,
            )
            .await;

        // Log similar steps to create patterns
        let step1 = ExecutionStep::new(1, "cargo".to_string(), "create_project".to_string());
        let step2 = ExecutionStep::new(
            2,
            "rust_analyzer".to_string(),
            "implement_feature".to_string(),
        );

        memory.log_step(episode_id, step1).await;
        memory.log_step(episode_id, step2).await;

        let outcome = TaskOutcome::Success {
            verdict: format!("Pattern episode {i} completed"),
            artifacts: vec![format!("feature_{}.rs", i)],
        };
        memory.complete_episode(episode_id, outcome).await.unwrap();
    }

    // Test pattern retrieval
    let patterns = memory
        .retrieve_relevant_patterns(
            &TaskContext {
                domain: "patterns".to_string(),
                language: Some("rust".to_string()),
                framework: Some("tokio".to_string()),
                complexity: ComplexityLevel::Simple,
                tags: vec!["pattern".to_string()],
            },
            10,
        )
        .await;

    // Test MCP pattern analysis
    let mcp_server = Arc::new(
        MemoryMCPServer::new(SandboxConfig::restrictive(), memory.clone())
            .await
            .unwrap(),
    );

    let pattern_result = mcp_server
        .analyze_patterns("CodeGeneration".to_string(), 0.0, 10, None)
        .await
        .unwrap();

    let mcp_patterns = pattern_result["patterns"].as_array().unwrap();
    assert!(!mcp_patterns.is_empty() || patterns.is_empty());
}
