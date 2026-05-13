//! Bulk data persistence tests

mod persistence_helper;

use do_memory_core::{ComplexityLevel, ExecutionStep, TaskContext, TaskOutcome, TaskType};
use do_memory_mcp::{MemoryMCPServer, SandboxConfig};
use std::sync::Arc;

#[tokio::test]
async fn test_bulk_data_persistence() {
    let (memory, _temp_dir) = persistence_helper::setup_persistent_memory().await.unwrap();

    // Create multiple episodes
    let episode_count = 5;
    let mut episode_ids = Vec::new();

    for i in 1..=episode_count {
        let episode_id = memory
            .start_episode(
                format!("Bulk Test Episode {i}"),
                TaskContext {
                    domain: "bulk".to_string(),
                    language: Some("rust".to_string()),
                    framework: Some("tokio".to_string()),
                    complexity: ComplexityLevel::Simple,
                    tags: vec!["bulk".to_string(), "test".to_string()],
                },
                TaskType::Testing,
            )
            .await;

        // Add some steps
        for j in 1..=2 {
            let step = ExecutionStep::new(
                j,
                format!("bulk_tool_{j}"),
                format!("Bulk operation {j} for episode {i}"),
            );
            memory.log_step(episode_id, step).await;
        }

        memory
            .complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: format!("Bulk episode {i} completed"),
                    artifacts: vec![format!("bulk_{}.log", i)],
                },
            )
            .await
            .unwrap();

        episode_ids.push(episode_id);
    }

    // Verify all episodes persist
    let mcp_server = Arc::new(
        MemoryMCPServer::new(SandboxConfig::restrictive(), memory.clone())
            .await
            .unwrap(),
    );

    let query_result = mcp_server
        .query_memory(
            "Bulk Test".to_string(),
            "bulk".to_string(),
            None,
            10,
            "relevance".to_string(),
            None,
        )
        .await
        .unwrap();

    let episodes = query_result["episodes"].as_array().unwrap();
    assert_eq!(episodes.len(), episode_count);
}
