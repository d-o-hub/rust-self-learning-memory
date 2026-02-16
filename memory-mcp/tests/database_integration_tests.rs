//! Integration tests for memory MCP server with database verification
//!
//! These tests verify that the MCP server correctly integrates with the memory system
//! and that all database operations work as expected.

use memory_core::{
    ComplexityLevel, ExecutionResult, ExecutionStep, MemoryConfig, SelfLearningMemory, TaskContext,
    TaskOutcome, TaskType,
};
use memory_mcp::{ExecutionContext, MemoryMCPServer, SandboxConfig};
use serde_json::json;
use std::sync::Arc;

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Disable WASM sandbox for all tests to prevent rquickjs GC crashes
    fn disable_wasm_for_tests() {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            std::env::set_var("MCP_USE_WASM", "false");
        });
    }

    async fn setup_test_environment() -> (Arc<SelfLearningMemory>, Arc<MemoryMCPServer>) {
        disable_wasm_for_tests();
        let memory = Arc::new(SelfLearningMemory::with_config(MemoryConfig {
            quality_threshold: 0.0,
            batch_config: None, // Disable batching for tests for test episodes
            ..Default::default()
        }));
        let sandbox_config = SandboxConfig::default();
        let mcp_server = Arc::new(
            MemoryMCPServer::new(sandbox_config, memory.clone())
                .await
                .unwrap(),
        );
        (memory, mcp_server)
    }

    #[tokio::test]
    async fn test_mcp_server_initialization() {
        let (_memory, mcp_server) = setup_test_environment().await;

        // Test core tools (loaded by default)
        // Core tools: query_memory, health_check, get_metrics, analyze_patterns,
        // create_episode, add_episode_step, complete_episode, get_episode
        let core_tools = mcp_server.list_tools().await;
        assert_eq!(core_tools.len(), 8);

        let core_tool_names: Vec<String> = core_tools.iter().map(|t| t.name.clone()).collect();
        assert!(core_tool_names.contains(&"query_memory".to_string()));
        assert!(core_tool_names.contains(&"analyze_patterns".to_string()));
        assert!(core_tool_names.contains(&"health_check".to_string()));
        assert!(core_tool_names.contains(&"get_metrics".to_string()));

        // execute_agent_code should NOT be available when WASM is disabled
        assert!(!core_tool_names.contains(&"execute_agent_code".to_string()));

        // Load extended tools
        let extended_tool_names = vec![
            "advanced_pattern_analysis",
            "quality_metrics",
            "configure_embeddings",
            "query_semantic_memory",
            "test_embeddings",
            "search_patterns",
            "recommend_patterns",
            "bulk_episodes",
        ];

        for tool_name in &extended_tool_names {
            let tool = mcp_server.get_tool(tool_name).await;
            assert!(
                tool.is_some(),
                "Extended tool '{}' should be available",
                tool_name
            );
        }

        // After loading extended tools, verify they're in the list
        let all_tools = mcp_server.list_tools().await;
        assert_eq!(all_tools.len(), 8 + extended_tool_names.len()); // 8 core + 8 extended

        let all_tool_names: Vec<String> = all_tools.iter().map(|t| t.name.clone()).collect();

        // Verify extended tools are now present
        assert!(all_tool_names.contains(&"advanced_pattern_analysis".to_string()));
        assert!(all_tool_names.contains(&"quality_metrics".to_string()));
        assert!(all_tool_names.contains(&"configure_embeddings".to_string()));
        assert!(all_tool_names.contains(&"query_semantic_memory".to_string()));
        assert!(all_tool_names.contains(&"test_embeddings".to_string()));
        assert!(all_tool_names.contains(&"search_patterns".to_string()));
        assert!(all_tool_names.contains(&"recommend_patterns".to_string()));
        assert!(all_tool_names.contains(&"bulk_episodes".to_string()));
    }

    #[tokio::test]
    async fn test_memory_query_functionality() {
        let (memory, mcp_server) = setup_test_environment().await;

        // Create a test episode
        let episode_id = memory
            .start_episode(
                "Database test episode".to_string(),
                TaskContext {
                    domain: "database".to_string(),
                    language: Some("rust".to_string()),
                    framework: Some("sqlx".to_string()),
                    complexity: ComplexityLevel::Simple,
                    tags: vec!["test".to_string()],
                },
                TaskType::CodeGeneration,
            )
            .await;

        // Complete the episode
        memory
            .complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: "Test episode completed".to_string(),
                    artifacts: vec!["test.rs".to_string()],
                },
            )
            .await
            .unwrap();

        // Test MCP memory query
        let result = mcp_server
            .query_memory(
                "database test".to_string(),
                "database".to_string(),
                None,
                10,
                "relevance".to_string(),
                None,
            )
            .await
            .unwrap();

        let episodes = result["episodes"].as_array().unwrap();
        assert_eq!(episodes.len(), 1);

        let episode = &episodes[0];
        assert_eq!(episode["task_description"], "Database test episode");
        assert!(episode["outcome"].is_object());
    }

    #[tokio::test]
    async fn test_database_tool_usage_tracking() {
        let (_memory, mcp_server) = setup_test_environment().await;

        // Perform some tool operations
        let _ = mcp_server
            .query_memory(
                "test".to_string(),
                "test".to_string(),
                None,
                5,
                "relevance".to_string(),
                None,
            )
            .await
            .unwrap();

        let _ = mcp_server
            .analyze_patterns("test".to_string(), 0.7, 5, None)
            .await
            .unwrap();

        // Verify tool usage tracking
        let usage = mcp_server.get_tool_usage().await;
        assert!(usage.contains_key("query_memory"));
        assert!(usage.contains_key("analyze_patterns"));
        assert!(*usage.get("query_memory").unwrap_or(&0) >= 1);
        assert!(*usage.get("analyze_patterns").unwrap_or(&0) >= 1);
    }

    #[tokio::test]
    async fn test_execution_statistics_tracking() {
        let (_memory, mcp_server) = setup_test_environment().await;

        // Execute some code (may fail, but should be tracked)
        let _ = mcp_server
            .execute_agent_code(
                "return 42;".to_string(),
                ExecutionContext::new("test".to_string(), json!({})),
            )
            .await;

        // Verify execution statistics are tracked
        let stats = mcp_server.get_stats().await;
        assert!(stats.total_executions >= 1);
        // Note: success/failure depends on Node.js availability
    }

    #[tokio::test]
    async fn test_database_pattern_retrieval() {
        let (memory, _mcp_server) = setup_test_environment().await;

        // Create a test episode
        let episode_id = memory
            .start_episode(
                "Pattern test episode".to_string(),
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

        memory
            .complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: "Pattern test completed".to_string(),
                    artifacts: vec!["pattern.rs".to_string()],
                },
            )
            .await
            .unwrap();

        // Test pattern retrieval
        let _patterns = memory
            .retrieve_relevant_patterns(
                &TaskContext {
                    domain: "patterns".to_string(),
                    language: None,
                    framework: None,
                    complexity: ComplexityLevel::Simple,
                    tags: vec!["pattern".to_string()],
                },
                10,
            )
            .await;

        // Patterns may be empty initially, but the query should work
        // This assertion just verifies the method doesn't panic
    }

    #[tokio::test]
    async fn test_full_episode_lifecycle_with_database_verification() {
        let (memory, _mcp_server) = setup_test_environment().await;

        // Start episode
        let episode_id = memory
            .start_episode(
                "Test API implementation".to_string(),
                TaskContext {
                    domain: "web-api".to_string(),
                    language: Some("rust".to_string()),
                    framework: Some("axum".to_string()),
                    complexity: ComplexityLevel::Simple,
                    tags: vec!["test".to_string(), "api".to_string()],
                },
                TaskType::CodeGeneration,
            )
            .await;

        // Log execution steps using the correct ExecutionStep structure
        let mut step1 = ExecutionStep::new(1, "cargo".to_string(), "create project".to_string());
        step1.result = Some(ExecutionResult::Success {
            output: "Project created".to_string(),
        });
        step1.latency_ms = 100;
        step1.tokens_used = Some(25);

        let mut step2 =
            ExecutionStep::new(2, "rust_analyzer".to_string(), "implement API".to_string());
        step2.result = Some(ExecutionResult::Success {
            output: "API implemented".to_string(),
        });
        step2.latency_ms = 200;
        step2.tokens_used = Some(50);

        memory.log_step(episode_id, step1).await;
        memory.log_step(episode_id, step2).await;

        // Complete episode
        let outcome = TaskOutcome::Success {
            verdict: "Successfully implemented test API".to_string(),
            artifacts: vec!["src/api.rs".to_string()],
        };

        memory.complete_episode(episode_id, outcome).await.unwrap();

        // Verify database entries
        let episodes = memory
            .retrieve_relevant_context(
                "test API".to_string(),
                TaskContext {
                    domain: "web-api".to_string(),
                    language: None,
                    framework: None,
                    complexity: ComplexityLevel::Simple,
                    tags: vec![],
                },
                10,
            )
            .await;

        assert_eq!(episodes.len(), 1);
        let episode = &episodes[0];
        assert_eq!(episode.task_description, "Test API implementation");
        assert_eq!(episode.steps.len(), 2);
        assert!(episode.end_time.is_some());
    }
}
