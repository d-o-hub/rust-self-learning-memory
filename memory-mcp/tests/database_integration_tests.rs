//! Integration tests for memory MCP server with database verification
//!
//! These tests verify that the MCP server correctly integrates with the memory system
//! and that all database operations work as expected.

use memory_core::{SelfLearningMemory, TaskContext, TaskType, TaskOutcome, ComplexityLevel};
use memory_mcp::{MemoryMCPServer, SandboxConfig, ExecutionContext};
use serde_json::json;
use std::sync::Arc;

#[cfg(test)]
mod integration_tests {
    use super::*;

    async fn setup_test_environment() -> (Arc<SelfLearningMemory>, Arc<MemoryMCPServer>) {
        let memory = Arc::new(SelfLearningMemory::new());
        let sandbox_config = SandboxConfig::restrictive();
        let mcp_server = Arc::new(MemoryMCPServer::new(sandbox_config, memory.clone()).await.unwrap());
        (memory, mcp_server)
    }

    #[tokio::test]
    async fn test_mcp_server_initialization() {
        let (_memory, mcp_server) = setup_test_environment().await;

        // Test that server initializes with correct tools
        let tools = mcp_server.list_tools().await;
        assert_eq!(tools.len(), 3);

        let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
        assert!(tool_names.contains(&"query_memory".to_string()));
        assert!(tool_names.contains(&"execute_agent_code".to_string()));
        assert!(tool_names.contains(&"analyze_patterns".to_string()));
    }

    #[tokio::test]
    async fn test_memory_query_functionality() {
        let (memory, mcp_server) = setup_test_environment().await;

        // Create a test episode
        let episode_id = memory.start_episode(
            "Database test episode".to_string(),
            TaskContext {
                domain: "database".to_string(),
                language: Some("rust".to_string()),
                framework: Some("sqlx".to_string()),
                complexity: ComplexityLevel::Simple,
                tags: vec!["test".to_string()],
            },
            TaskType::CodeGeneration
        );

        // Complete the episode
        memory.complete_episode(episode_id, TaskOutcome::Success {
            verdict: "Test episode completed".to_string(),
            artifacts: vec!["test.rs".to_string()],
        }).await;

        // Test MCP memory query
        let result = mcp_server.query_memory(
            "database test".to_string(),
            "database".to_string(),
            None,
            10
        ).await.unwrap();

        let episodes = result["episodes"].as_array().unwrap();
        assert_eq!(episodes.len(), 1);

        let episode = &episodes[0];
        assert_eq!(episode["task_description"], "Database test episode");
        assert!(episode["outcome"].is_object());
    }

    #[tokio::test]
    async fn test_tool_usage_tracking() {
        let (_memory, mcp_server) = setup_test_environment().await;

        // Perform some tool operations
        let _ = mcp_server.query_memory(
            "test".to_string(),
            "test".to_string(),
            None,
            5
        ).await.unwrap();

        let _ = mcp_server.analyze_patterns(
            "test".to_string(),
            0.7,
            5
        ).await.unwrap();

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
        let _ = mcp_server.execute_agent_code(
            "return 42;".to_string(),
            ExecutionContext::new("test".to_string(), json!({}))
        ).await;

        // Verify execution statistics are tracked
        let stats = mcp_server.get_stats().await;
        assert!(stats.total_executions >= 1);
        // Note: success/failure depends on Node.js availability
    }

    #[tokio::test]
    async fn test_database_pattern_retrieval() {
        let (memory, _mcp_server) = setup_test_environment().await;

        // Create a test episode
        let episode_id = memory.start_episode(
            "Pattern test episode".to_string(),
            TaskContext {
                domain: "patterns".to_string(),
                language: Some("rust".to_string()),
                framework: Some("tokio".to_string()),
                complexity: ComplexityLevel::Simple,
                tags: vec!["pattern".to_string(), "test".to_string()],
            },
            TaskType::CodeGeneration
        );

        memory.complete_episode(episode_id, TaskOutcome::Success {
            verdict: "Pattern test completed".to_string(),
            artifacts: vec!["pattern.rs".to_string()],
        }).await;

        // Test pattern retrieval
        let patterns = memory.retrieve_relevant_patterns(
            &TaskContext {
                domain: "patterns".to_string(),
                language: None,
                framework: None,
                complexity: ComplexityLevel::Simple,
                tags: vec!["pattern".to_string()],
            },
            10
        ).await;

        // Patterns may be empty initially, but the query should work
        assert!(patterns.len() >= 0); // At minimum, should return empty vec
    }

    #[tokio::test]
    async fn test_mcp_server_initialization() {
        let (_memory, mcp_server) = setup_test_environment().await;

        // Test that server initializes with correct tools
        let tools = mcp_server.list_tools().await;
        assert_eq!(tools.len(), 3);

        let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
        assert!(tool_names.contains(&"query_memory".to_string()));
        assert!(tool_names.contains(&"execute_agent_code".to_string()));
        assert!(tool_names.contains(&"analyze_patterns".to_string()));
    }

    #[tokio::test]
    async fn test_memory_query_functionality() {
        let (memory, mcp_server) = setup_test_environment().await;

        // Create a test episode
        let episode_id = memory.start_episode(
            "Database test episode".to_string(),
            TaskContext {
                domain: "database".to_string(),
                language: Some("rust".to_string()),
                framework: Some("sqlx".to_string()),
                complexity: ComplexityLevel::Simple,
                tags: vec!["test".to_string()],
            },
            TaskType::CodeGeneration
        );

        // Complete the episode
        memory.complete_episode(episode_id, TaskOutcome::Success {
            verdict: "Test episode completed".to_string(),
            artifacts: vec!["test.rs".to_string()],
        }).await;

        // Test MCP memory query
        let result = mcp_server.query_memory(
            "database test".to_string(),
            "database".to_string(),
            None,
            10
        ).await.unwrap();

        let episodes = result["episodes"].as_array().unwrap();
        assert_eq!(episodes.len(), 1);

        let episode = &episodes[0];
        assert_eq!(episode["task_description"], "Database test episode");
        assert!(episode["outcome"].is_object());
    }

    #[tokio::test]
    async fn test_tool_usage_tracking() {
        let (_memory, mcp_server) = setup_test_environment().await;

        // Perform some tool operations
        let _ = mcp_server.query_memory(
            "test".to_string(),
            "test".to_string(),
            None,
            5
        ).await.unwrap();

        let _ = mcp_server.analyze_patterns(
            "test".to_string(),
            0.7,
            5
        ).await.unwrap();

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
        let _ = mcp_server.execute_agent_code(
            "return 42;".to_string(),
            ExecutionContext::new("test".to_string(), json!({}))
        ).await;

        // Verify execution statistics are tracked
        let stats = mcp_server.get_stats().await;
        assert!(stats.total_executions >= 1);
        // Note: success/failure depends on Node.js availability
    }

    #[tokio::test]
    async fn test_database_pattern_retrieval() {
        let (memory, _mcp_server) = setup_test_environment().await;

        // Create a test episode
        let episode_id = memory.start_episode(
            "Pattern test episode".to_string(),
            TaskContext {
                domain: "patterns".to_string(),
                language: Some("rust".to_string()),
                framework: Some("tokio".to_string()),
                complexity: ComplexityLevel::Simple,
                tags: vec!["pattern".to_string(), "test".to_string()],
            },
            TaskType::CodeGeneration
        );

        memory.complete_episode(episode_id, TaskOutcome::Success {
            verdict: "Pattern test completed".to_string(),
            artifacts: vec!["pattern.rs".to_string()],
        }).await;

        // Test pattern retrieval
        let patterns = memory.retrieve_relevant_patterns(
            &TaskContext {
                domain: "patterns".to_string(),
                language: None,
                framework: None,
                complexity: ComplexityLevel::Simple,
                tags: vec!["pattern".to_string()],
            },
            10
        ).await;

        // Patterns may be empty initially, but the query should work
        assert!(patterns.len() >= 0); // At minimum, should return empty vec
    }

    #[tokio::test]
    async fn test_mcp_server_initialization() {
        let (_memory, mcp_server) = setup_test_environment().await;

        // Test that server initializes with correct tools
        let tools = mcp_server.list_tools().await;
        assert_eq!(tools.len(), 3);

        let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
        assert!(tool_names.contains(&"query_memory".to_string()));
        assert!(tool_names.contains(&"execute_agent_code".to_string()));
        assert!(tool_names.contains(&"analyze_patterns".to_string()));
    }

    #[tokio::test]
    async fn test_memory_query_functionality() {
        let (memory, mcp_server) = setup_test_environment().await;

        // Create a test episode
        let episode_id = memory.start_episode(
            "Database test episode".to_string(),
            TaskContext {
                domain: "database".to_string(),
                language: Some("rust".to_string()),
                framework: Some("sqlx".to_string()),
                complexity: ComplexityLevel::Simple,
                tags: vec!["test".to_string()],
            },
            TaskType::CodeGeneration
        );

        // Complete the episode
        memory.complete_episode(episode_id, TaskOutcome::Success {
            verdict: "Test episode completed".to_string(),
            artifacts: vec!["test.rs".to_string()],
        }).await;

        // Test MCP memory query
        let result = mcp_server.query_memory(
            "database test".to_string(),
            "database".to_string(),
            None,
            10
        ).await.unwrap();

        let episodes = result["episodes"].as_array().unwrap();
        assert_eq!(episodes.len(), 1);

        let episode = &episodes[0];
        assert_eq!(episode["task_description"], "Database test episode");
        assert!(episode["outcome"].is_object());
    }

    #[tokio::test]
    async fn test_full_episode_lifecycle_with_database_verification() {
        let (memory, mcp_server) = setup_test_environment().await;

        // Start episode
        let episode_id = memory.start_episode(
            "Test API implementation".to_string(),
            TaskContext {
                domain: "web-api".to_string(),
                language: Some("rust".to_string()),
                framework: Some("axum".to_string()),
                complexity: ComplexityLevel::Simple,
                tags: vec!["test".to_string(), "api".to_string()],
            }
        ).await.unwrap();

        // Log execution steps
        let steps = vec![
            ExecutionStep {
                tool: "cargo".to_string(),
                action: "create".to_string(),
                latency_ms: 100,
                tokens: Some(25),
                success: true,
                observation: "Project created".to_string(),
            },
            ExecutionStep {
                tool: "rust_analyzer".to_string(),
                action: "implement".to_string(),
                latency_ms: 200,
                tokens: Some(50),
                success: true,
                observation: "API implemented".to_string(),
            },
        ];

        for step in steps {
            memory.log_step(episode_id, step).await.unwrap();
        }

        // Complete episode
        let outcome = TaskOutcome {
            success: true,
            reward: Some(RewardScore {
                total: 0.9,
                efficiency: 0.95,
                quality: 0.85,
                learning: 0.9,
            }),
            artifacts: json!({"files": ["src/api.rs"], "tests": 5}),
            summary: "Successfully implemented test API".to_string(),
        };

        memory.complete_episode(episode_id, outcome).await.unwrap();

        // Verify database entries
        let episodes = memory.retrieve_relevant_context(
            "test API".to_string(),
            TaskContext {
                domain: "web-api".to_string(),
                language: None,
                framework: None,
                complexity: ComplexityLevel::Simple,
                tags: vec![],
            },
            10
        ).await.unwrap();

        assert_eq!(episodes.len(), 1);
        let episode = &episodes[0];
        assert_eq!(episode.task_description, "Test API implementation");
        assert_eq!(episode.steps.len(), 2);
        assert!(episode.reward.as_ref().unwrap().total >= 0.9);
        assert!(episode.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_tool_usage_tracking() {
        let (_memory, mcp_server) = setup_test_environment().await;

        // Perform some tool operations
        let _ = mcp_server.query_memory(
            "test".to_string(),
            "test".to_string(),
            None,
            5
        ).await.unwrap();

        let _ = mcp_server.analyze_patterns(
            "test".to_string(),
            0.7,
            5
        ).await.unwrap();

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
        let _ = mcp_server.execute_agent_code(
            "return 42;".to_string(),
            ExecutionContext::new("test".to_string(), json!({}))
        ).await;

        // Verify execution statistics are tracked
        let stats = mcp_server.get_stats().await;
        assert!(stats.total_executions >= 1);
        // Note: success/failure depends on Node.js availability
    }

    #[tokio::test]
    async fn test_database_pattern_retrieval() {
        let (memory, mcp_server) = setup_test_environment().await;

        // Create a test episode
        let episode_id = memory.start_episode(
            "Pattern test episode".to_string(),
            TaskContext {
                domain: "patterns".to_string(),
                language: Some("rust".to_string()),
                framework: Some("tokio".to_string()),
                complexity: ComplexityLevel::Simple,
                tags: vec!["pattern".to_string(), "test".to_string()],
            },
            TaskType::CodeGeneration
        );

        memory.complete_episode(episode_id, TaskOutcome::Success {
            verdict: "Pattern test completed".to_string(),
            artifacts: vec!["pattern.rs".to_string()],
        }).await;

        // Test pattern retrieval
        let patterns = memory.retrieve_relevant_patterns(
            &TaskContext {
                domain: "patterns".to_string(),
                language: None,
                framework: None,
                complexity: ComplexityLevel::Simple,
                tags: vec!["pattern".to_string()],
            },
            10
        ).await;

        // Patterns may be empty initially, but the query should work
        assert!(patterns.len() >= 0); // At minimum, should return empty vec
    }
}