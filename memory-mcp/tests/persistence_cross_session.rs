//! Cross-session persistence tests

use do_memory_core::{
    ComplexityLevel, ExecutionStep, MemoryConfig, SelfLearningMemory, TaskContext, TaskOutcome,
    TaskType,
};
use do_memory_mcp::{MemoryMCPServer, SandboxConfig};
use do_memory_storage_redb::RedbStorage;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_cross_session_persistence() {
    // Create temporary directory that persists across the test
    let temp_dir = TempDir::new().unwrap();
    let redb_path = temp_dir.path().join("test_memory.redb");

    // Create storage and memory instance
    let redb_storage = Arc::new(RedbStorage::new(&redb_path).await.unwrap());
    let memory = SelfLearningMemory::with_storage(
        MemoryConfig {
            quality_threshold: 0.0, // Zero threshold for test episodes
            ..Default::default()
        },
        redb_storage.clone(),
        redb_storage,
    );
    let memory = Arc::new(memory);

    // First session - create and store data
    let episode_id = memory
        .start_episode(
            "Cross-Session Test".to_string(),
            TaskContext {
                domain: "persistence".to_string(),
                language: Some("rust".to_string()),
                framework: Some("tokio".to_string()),
                complexity: ComplexityLevel::Simple,
                tags: vec!["cross-session".to_string()],
            },
            TaskType::Testing,
        )
        .await;

    let step = ExecutionStep::new(
        1,
        "persistence_test".to_string(),
        "Testing cross-session data persistence".to_string(),
    );
    memory.log_step(episode_id, step).await;

    let outcome = TaskOutcome::Success {
        verdict: "Cross-session persistence test completed".to_string(),
        artifacts: vec!["persistence_test.log".to_string()],
    };
    memory.complete_episode(episode_id, outcome).await.unwrap();

    // Verify episode exists
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert_eq!(episode.task_description, "Cross-Session Test");
    assert_eq!(episode.steps.len(), 1);

    // Test MCP query on the same instance
    let mcp_server = Arc::new(
        MemoryMCPServer::new(SandboxConfig::restrictive(), memory.clone())
            .await
            .unwrap(),
    );

    let query_result = mcp_server
        .query_memory(
            "Cross-Session".to_string(),
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
    assert_eq!(episode["task_description"], "Cross-Session Test");
    assert_eq!(episode["steps"].as_array().unwrap().len(), 1);
}
