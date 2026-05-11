//! DuckDB persistence tests for episodes and patterns
#![allow(clippy::explicit_iter_loop)]
#![allow(missing_docs)]

#[cfg(feature = "duckdb")]
mod duckdb_tests {
    mod common;

    use common::setup_duckdb_persistent_memory;
    use do_memory_core::{ComplexityLevel, ExecutionStep, TaskContext, TaskOutcome, TaskType};
    use do_memory_mcp::{MemoryMCPServer, SandboxConfig};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_episode_persistence_in_duckdb() {
        println!("🧪 Testing Episode Persistence in DuckDB");
        println!("========================================");

        let (memory, _temp_dir) = setup_duckdb_persistent_memory().await.unwrap();

        // Create episode
        let episode_id = memory
            .start_episode(
                "DuckDB Persistent Episode Test".to_string(),
                TaskContext {
                    domain: "persistence".to_string(),
                    language: Some("rust".to_string()),
                    framework: Some("tokio".to_string()),
                    complexity: ComplexityLevel::Simple,
                    tags: vec!["test".to_string(), "duckdb".to_string()],
                },
                TaskType::Testing,
            )
            .await;

        println!("✅ Episode created: {episode_id}");

        // Log steps
        let step = ExecutionStep::new(
            1,
            "duckdb_test_tool".to_string(),
            "Testing DuckDB persistence".to_string(),
        );
        memory.log_step(episode_id, step).await;

        // Complete episode
        let outcome = TaskOutcome::Success {
            verdict: "Episode persisted successfully in DuckDB".to_string(),
            artifacts: vec!["duckdb_test_result.txt".to_string()],
        };
        memory.complete_episode(episode_id, outcome).await.unwrap();

        println!("✅ Episode completed and steps logged");

        // Verify episode exists in storage
        let retrieved_episode = memory.get_episode(episode_id).await.unwrap();
        assert_eq!(
            retrieved_episode.task_description,
            "DuckDB Persistent Episode Test"
        );
        assert_eq!(retrieved_episode.steps.len(), 1);
        assert!(retrieved_episode.is_complete());

        println!("✅ Episode verified in DuckDB storage");

        // Test memory query through MCP
        let mcp_server = Arc::new(
            MemoryMCPServer::new(SandboxConfig::restrictive(), memory.clone())
                .await
                .unwrap(),
        );

        let query_result = mcp_server
            .query_memory(
                "DuckDB Persistent".to_string(),
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
        assert_eq!(episode["task_description"], "DuckDB Persistent Episode Test");

        println!("✅ MCP query verified episode in DuckDB storage");
    }
}
