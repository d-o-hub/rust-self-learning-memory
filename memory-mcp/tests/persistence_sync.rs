//! Storage backend synchronization tests

mod persistence_helper;

use do_memory_core::{ComplexityLevel, ExecutionStep, TaskContext, TaskOutcome, TaskType};
use do_memory_mcp::{MemoryMCPServer, SandboxConfig};
use std::sync::Arc;

#[tokio::test]
async fn test_storage_backend_synchronization() {
    let (memory, _temp_dir) = persistence_helper::setup_persistent_memory().await.unwrap();

    // Create episode and verify it's stored
    let episode_id = memory
        .start_episode(
            "Sync Test Episode".to_string(),
            TaskContext {
                domain: "sync".to_string(),
                language: Some("rust".to_string()),
                framework: Some("tokio".to_string()),
                complexity: ComplexityLevel::Simple,
                tags: vec!["sync".to_string(), "test".to_string()],
            },
            TaskType::Testing,
        )
        .await;

    // Add multiple steps
    for i in 1..=3 {
        let step = ExecutionStep::new(
            i,
            format!("sync_tool_{i}"),
            format!("Synchronization step {i}"),
        );
        memory.log_step(episode_id, step).await;
    }

    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Storage synchronization test completed".to_string(),
                artifacts: vec!["sync_test.log".to_string()],
            },
        )
        .await
        .unwrap();

    // Verify through multiple access methods
    let direct_episode = memory.get_episode(episode_id).await.unwrap();
    assert_eq!(direct_episode.steps.len(), 3);

    // Verify through MCP
    let mcp_server = Arc::new(
        MemoryMCPServer::new(SandboxConfig::restrictive(), memory.clone())
            .await
            .unwrap(),
    );

    let query_result = mcp_server
        .query_memory(
            "Sync Test".to_string(),
            "sync".to_string(),
            None,
            10,
            "relevance".to_string(),
            None,
        )
        .await
        .unwrap();

    let episodes = query_result["episodes"].as_array().unwrap();
    assert_eq!(episodes.len(), 1);

    let mcp_episode = &episodes[0];
    assert_eq!(mcp_episode["steps"].as_array().unwrap().len(), 3);
}
