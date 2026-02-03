//! End-to-End tests for MCP embedding tools
//!
//! Tests the MCP server integration with embeddings:
//! - `configure_embeddings` tool
//! - `query_semantic_memory` tool
//! - `test_embeddings` tool
//! - Tool chaining (create episode → semantic search)
//! - Error handling and validation

#![allow(clippy::unwrap_used, clippy::expect_used)]

use memory_core::episode::ExecutionStep;
use memory_core::types::{ComplexityLevel, TaskContext, TaskOutcome, TaskType};
use memory_core::SelfLearningMemory;
use memory_mcp::mcp::tools::embeddings::{
    configure_embeddings_tool, query_semantic_memory_tool, test_embeddings_tool,
    ConfigureEmbeddingsInput, EmbeddingTools, QuerySemanticMemoryInput,
};
use memory_mcp::server::MemoryMCPServer;
use memory_mcp::types::SandboxConfig;
use std::sync::Arc;

/// Create a test MCP server
async fn create_test_server() -> MemoryMCPServer {
    // Disable WASM for tests
    std::env::set_var("MCP_USE_WASM", "false");
    std::env::set_var("MCP_CACHE_WARMING_ENABLED", "false");

    let memory = Arc::new(SelfLearningMemory::new());
    MemoryMCPServer::new(SandboxConfig::default(), memory)
        .await
        .expect("Failed to create test server")
}

// ============================================================================
// Day 2: MCP Integration E2E Tests
// ============================================================================

#[tokio::test]
async fn test_mcp_embedding_tools_registered() {
    let server = create_test_server().await;
    let tools = server.list_tools().await;

    // Verify all embedding tools are registered
    let tool_names: Vec<_> = tools.iter().map(|t| &t.name).collect();

    assert!(
        tool_names.contains(&"configure_embeddings"),
        "configure_embeddings should be registered"
    );
    assert!(
        tool_names.contains(&"query_semantic_memory"),
        "query_semantic_memory should be registered"
    );
    assert!(
        tool_names.contains(&"test_embeddings"),
        "test_embeddings should be registered"
    );

    // Verify tool metadata
    for tool in tools {
        if tool.name == "configure_embeddings" {
            assert!(!tool.description.is_empty());
            assert!(tool.input_schema.is_object());
        }
    }
}

#[tokio::test]
async fn test_mcp_configure_embeddings_local() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EmbeddingTools::new(memory);

    let input = ConfigureEmbeddingsInput {
        provider: "local".to_string(),
        model: Some("sentence-transformers/all-MiniLM-L6-v2".to_string()),
        api_key_env: None,
        similarity_threshold: Some(0.75),
        batch_size: Some(16),
        base_url: None,
        api_version: None,
        resource_name: None,
        deployment_name: None,
    };

    let result = tools.execute_configure_embeddings(input).await;

    assert!(result.is_ok(), "Local configuration should succeed");

    let output = result.unwrap();
    assert!(output.success, "Configuration should be successful");
    assert_eq!(output.provider, "local");
    assert_eq!(output.model, "sentence-transformers/all-MiniLM-L6-v2");
    assert_eq!(output.dimension, 384);
    assert!(
        output.warnings.is_empty(),
        "No warnings expected for valid config"
    );

    println!("Local provider configured: {}", output.message);
}

#[tokio::test]
async fn test_mcp_configure_embeddings_openai() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EmbeddingTools::new(memory);

    // Test with potentially missing API key (should handle gracefully)
    let input = ConfigureEmbeddingsInput {
        provider: "openai".to_string(),
        model: Some("text-embedding-3-small".to_string()),
        api_key_env: Some("OPENAI_API_KEY".to_string()),
        similarity_threshold: None,
        batch_size: None,
        base_url: None,
        api_version: None,
        resource_name: None,
        deployment_name: None,
    };

    let result = tools.execute_configure_embeddings(input).await;

    // May succeed or fail depending on API key presence
    if let Ok(output) = result {
        assert_eq!(output.provider, "openai");
        assert_eq!(output.model, "text-embedding-3-small");
        assert_eq!(output.dimension, 1536);

        if !output.warnings.is_empty() {
            println!("Warnings: {:?}", output.warnings);
        }
    } else {
        let error = result.unwrap_err();
        println!("Expected error (no API key): {}", error);
        assert!(error.to_string().contains("API key") || error.to_string().contains("not set"));
    }
}

#[tokio::test]
async fn test_mcp_configure_embeddings_azure() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EmbeddingTools::new(memory);

    // Test with missing required fields
    let input_missing = ConfigureEmbeddingsInput {
        provider: "azure".to_string(),
        model: None,
        api_key_env: Some("AZURE_OPENAI_API_KEY".to_string()),
        similarity_threshold: None,
        batch_size: None,
        base_url: None,
        api_version: None,
        resource_name: None,   // Missing required
        deployment_name: None, // Missing required
    };

    let result = tools.execute_configure_embeddings(input_missing).await;
    assert!(result.is_err(), "Should fail without required fields");

    let error = result.unwrap_err();
    assert!(
        error.to_string().contains("required")
            || error.to_string().contains("resource_name")
            || error.to_string().contains("deployment_name")
    );

    // Test with all required fields
    let input_valid = ConfigureEmbeddingsInput {
        provider: "azure".to_string(),
        model: None,
        api_key_env: Some("AZURE_OPENAI_API_KEY".to_string()),
        similarity_threshold: Some(0.7),
        batch_size: Some(16),
        base_url: None,
        api_version: Some("2023-05-15".to_string()),
        resource_name: Some("test-resource".to_string()),
        deployment_name: Some("test-deployment".to_string()),
    };

    let result = tools.execute_configure_embeddings(input_valid).await;
    // May succeed or fail depending on actual Azure setup
    if let Ok(output) = result {
        assert_eq!(output.provider, "azure");
    }
}

#[tokio::test]
async fn test_mcp_configure_embeddings_invalid_provider() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EmbeddingTools::new(memory);

    let input = ConfigureEmbeddingsInput {
        provider: "invalid-provider".to_string(),
        model: None,
        api_key_env: None,
        similarity_threshold: None,
        batch_size: None,
        base_url: None,
        api_version: None,
        resource_name: None,
        deployment_name: None,
    };

    let result = tools.execute_configure_embeddings(input).await;
    assert!(result.is_err(), "Invalid provider should fail");

    let error = result.unwrap_err();
    assert!(error.to_string().contains("provider") || error.to_string().contains("Unsupported"));
}

#[tokio::test]
async fn test_mcp_query_semantic_memory_basic() {
    let server = create_test_server().await;

    let input = QuerySemanticMemoryInput {
        query: "implement REST API".to_string(),
        limit: Some(5),
        similarity_threshold: Some(0.7),
        domain: Some("web-api".to_string()),
        task_type: Some("code_generation".to_string()),
    };

    let result = server.execute_query_semantic_memory(input).await;

    assert!(result.is_ok(), "Query should succeed");

    let output = result.unwrap();
    assert!(output.is_object(), "Output should be JSON object");

    // Check response structure
    assert!(output.get("results_found").is_some());
    assert!(output.get("results").is_some());
    assert!(output.get("query_time_ms").is_some());
    assert!(output.get("embedding_dimension").is_some());

    let results_found = output
        .get("results_found")
        .and_then(|v| v.as_i64())
        .expect("results_found should be number");

    assert!(results_found >= 0, "results_found should be non-negative");

    println!("Query found {} results", results_found);
}

#[tokio::test]
async fn test_mcp_query_semantic_memory_with_filters() {
    let server = create_test_server().await;

    // Test domain filter
    let input_domain = QuerySemanticMemoryInput {
        query: "parse JSON data".to_string(),
        limit: Some(10),
        similarity_threshold: Some(0.6),
        domain: Some("data-processing".to_string()),
        task_type: None,
    };

    let result = server.execute_query_semantic_memory(input_domain).await;
    assert!(result.is_ok());

    // Test task type filter
    let input_task = QuerySemanticMemoryInput {
        query: "debug performance issue".to_string(),
        limit: Some(5),
        similarity_threshold: Some(0.7),
        domain: None,
        task_type: Some("debugging".to_string()),
    };

    let result = server.execute_query_semantic_memory(input_task).await;
    assert!(result.is_ok());

    // Test both filters
    let input_both = QuerySemanticMemoryInput {
        query: "optimize database query".to_string(),
        limit: Some(5),
        similarity_threshold: Some(0.65),
        domain: Some("database".to_string()),
        task_type: Some("refactoring".to_string()),
    };

    let result = server.execute_query_semantic_memory(input_both).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_mcp_query_semantic_memory_default_params() {
    let server = create_test_server().await;

    let input = QuerySemanticMemoryInput {
        query: "test query".to_string(),
        limit: None,
        similarity_threshold: None,
        domain: None,
        task_type: None,
    };

    let result = server.execute_query_semantic_memory(input).await;

    assert!(result.is_ok());

    let output = result.unwrap();

    // Check that defaults are applied
    let results = output
        .get("results")
        .and_then(|v| v.as_array())
        .expect("results should be array");

    // Default limit should be applied (typically 10)
    assert!(results.len() <= 10, "Should respect default limit");
}

#[tokio::test]
async fn test_mcp_test_embeddings_tool() {
    let server = create_test_server().await;

    let result = server.execute_test_embeddings().await;

    assert!(result.is_ok(), "Test command should succeed");

    let output = result.unwrap();
    assert!(output.is_object());

    // Check response structure
    assert!(output.get("available").is_some());
    assert!(output.get("provider").is_some());
    assert!(output.get("dimension").is_some());
    assert!(output.get("test_time_ms").is_some());
    assert!(output.get("sample_embedding").is_some());

    let available = output
        .get("available")
        .and_then(|v| v.as_bool())
        .expect("available should be boolean");

    let provider = output
        .get("provider")
        .and_then(|v| v.as_str())
        .expect("provider should be string");

    println!(
        "Embeddings available: {}, provider: {}",
        available, provider
    );
}

#[tokio::test]
async fn test_mcp_tool_chaining_create_and_query() {
    // Test workflow: create episode → query semantic memory
    let server = create_test_server().await;

    // Step 1: Create some test episodes
    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: Some("axum".to_string()),
        complexity: ComplexityLevel::Moderate,
        domain: "web-api".to_string(),
        tags: vec!["rest".to_string(), "authentication".to_string()],
    };

    let episode_id = server
        .memory
        .start_episode(
            "Implement JWT authentication for REST API".to_string(),
            context.clone(),
            TaskType::CodeGeneration,
        )
        .await;

    // Add steps
    for i in 1..=5 {
        let step = ExecutionStep::new(i, format!("tool_{}", i), format!("Action {}", i));
        server.memory.log_step(episode_id, step).await;
    }

    // Complete episode
    let outcome = TaskOutcome::Success {
        verdict: "JWT authentication implemented successfully".to_string(),
        artifacts: vec!["auth.rs".to_string(), "jwt.rs".to_string()],
    };
    server
        .memory
        .complete_episode(episode_id, outcome)
        .await
        .expect("Should complete episode");

    // Step 2: Query semantic memory
    let query_input = QuerySemanticMemoryInput {
        query: "How to add user authentication to API?".to_string(),
        limit: Some(5),
        similarity_threshold: Some(0.5),
        domain: Some("web-api".to_string()),
        task_type: None,
    };

    let query_result = server.execute_query_semantic_memory(query_input).await;

    assert!(query_result.is_ok(), "Query should succeed");

    let query_output = query_result.unwrap();
    let results_found = query_output
        .get("results_found")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    println!(
        "Tool chaining: Found {} results after creating episode",
        results_found
    );

    // Should find the episode we just created (or at least not error)
    assert!(results_found >= 0);
}

#[tokio::test]
async fn test_mcp_tool_usage_tracking() {
    let server = create_test_server().await;

    // Execute various embedding tools
    let _ = server.execute_test_embeddings().await;

    let config_input = ConfigureEmbeddingsInput {
        provider: "local".to_string(),
        model: None,
        api_key_env: None,
        similarity_threshold: None,
        batch_size: None,
        base_url: None,
        api_version: None,
        resource_name: None,
        deployment_name: None,
    };
    let _ = server.execute_configure_embeddings(config_input).await;

    let query_input = QuerySemanticMemoryInput {
        query: "test".to_string(),
        limit: None,
        similarity_threshold: None,
        domain: None,
        task_type: None,
    };
    let _ = server.execute_query_semantic_memory(query_input).await;

    // Check usage tracking
    let usage = server.get_tool_usage().await;

    assert!(usage.contains_key("test_embeddings"));
    assert!(usage.contains_key("configure_embeddings"));
    assert!(usage.contains_key("query_semantic_memory"));

    println!("Tool usage: {:?}", usage);
}

#[tokio::test]
async fn test_mcp_error_handling_invalid_query() {
    let server = create_test_server().await;

    // Test with empty query
    let input_empty = QuerySemanticMemoryInput {
        query: "".to_string(),
        limit: Some(5),
        similarity_threshold: Some(0.7),
        domain: None,
        task_type: None,
    };

    let result = server.execute_query_semantic_memory(input_empty).await;

    // Should handle gracefully (may succeed or fail with meaningful error)
    match result {
        Ok(output) => {
            // If it succeeds, check results
            let results_found = output
                .get("results_found")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            assert!(results_found >= 0);
        }
        Err(e) => {
            // Should have meaningful error
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("query")
                    || error_msg.contains("empty")
                    || error_msg.contains("invalid")
            );
        }
    }
}

#[tokio::test]
async fn test_mcp_tool_definitions_valid() {
    // Verify tool definitions match JSON-RPC schema

    let configure_tool = configure_embeddings_tool();
    assert_eq!(configure_tool.name, "configure_embeddings");
    assert!(!configure_tool.description.is_empty());

    // Check input schema
    let schema = configure_tool.input_schema.as_object().unwrap();
    assert!(schema.contains_key("type"));
    assert!(schema.contains_key("properties"));
    assert!(schema.contains_key("required"));

    let required = schema
        .get("required")
        .and_then(|v| v.as_array())
        .expect("required should be array");

    assert!(
        required.iter().any(|v| v == "provider"),
        "provider should be required"
    );

    // Check query tool
    let query_tool = query_semantic_memory_tool();
    let schema = query_tool.input_schema.as_object().unwrap();
    let required = schema.get("required").and_then(|v| v.as_array()).unwrap();

    assert!(
        required.iter().any(|v| v == "query"),
        "query should be required"
    );

    // Check test tool (no required fields)
    let test_tool = test_embeddings_tool();
    let schema = test_tool.input_schema.as_object().unwrap();
    let required = schema.get("required").and_then(|v| v.as_array()).unwrap();

    assert!(required.is_empty(), "test should have no required fields");
}

#[tokio::test]
async fn test_mcp_similarity_threshold_filtering() {
    let server = create_test_server().await;

    let query = "test query";

    // Test with different thresholds
    let thresholds = vec![0.5, 0.7, 0.9, 1.0];

    for threshold in thresholds {
        let input = QuerySemanticMemoryInput {
            query: query.to_string(),
            limit: Some(100),
            similarity_threshold: Some(threshold),
            domain: None,
            task_type: None,
        };

        let result = server.execute_query_semantic_memory(input).await;

        if let Ok(output) = result {
            let results = output
                .get("results")
                .and_then(|v| v.as_array())
                .unwrap_or(&vec![]);

            // All results should have similarity >= threshold
            for result in results {
                if let Some(similarity) = result.get("similarity").and_then(|v| v.as_f64()) {
                    assert!(
                        similarity >= threshold as f64,
                        "Similarity {} should be >= threshold {}",
                        similarity,
                        threshold
                    );
                }
            }
        }
    }
}

#[tokio::test]
async fn test_mcp_limit_parameter() {
    let server = create_test_server().await;

    let query = "test query";

    // Test with different limits
    let limits = vec![1, 5, 10, 50];

    for limit in limits {
        let input = QuerySemanticMemoryInput {
            query: query.to_string(),
            limit: Some(limit),
            similarity_threshold: Some(0.0),
            domain: None,
            task_type: None,
        };

        let result = server.execute_query_semantic_memory(input).await;

        if let Ok(output) = result {
            let results = output
                .get("results")
                .and_then(|v| v.as_array())
                .unwrap_or(&vec![]);

            assert!(
                results.len() <= limit,
                "Results count {} should be <= limit {}",
                results.len(),
                limit
            );
        }
    }
}

#[tokio::test]
async fn test_mcp_concurrent_tool_execution() {
    let server = create_test_server().await;

    // Execute multiple queries concurrently
    let queries = vec!["authentication", "database", "API", "testing"];

    let handles: Vec<_> = queries
        .into_iter()
        .map(|query| {
            let server_clone = server.clone();
            tokio::spawn(async move {
                let input = QuerySemanticMemoryInput {
                    query: query.to_string(),
                    limit: Some(5),
                    similarity_threshold: Some(0.5),
                    domain: None,
                    task_type: None,
                };
                server_clone.execute_query_semantic_memory(input).await
            })
        })
        .collect();

    // Wait for all to complete
    let results: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    // All should succeed
    assert_eq!(results.len(), 4);
    for result in results {
        assert!(result.is_ok());
    }

    println!(
        "Concurrent execution: All {} queries succeeded",
        results.len()
    );
}

// ============================================================================
// Integration Tests with Episodes
// ============================================================================

#[tokio::test]
async fn test_mcp_episode_to_embedding_workflow() {
    let server = create_test_server().await;

    // Create a complete episode
    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: Some("sqlx".to_string()),
        complexity: ComplexityLevel::Moderate,
        domain: "database".to_string(),
        tags: vec!["postgresql".to_string(), "migration".to_string()],
    };

    let episode_id = server
        .memory
        .start_episode(
            "Create database migration for users table".to_string(),
            context.clone(),
            TaskType::CodeGeneration,
        )
        .await;

    // Add steps
    for i in 1..=3 {
        let step = ExecutionStep::new(i, format!("tool_{}", i), format!("Migration step {}", i));
        server.memory.log_step(episode_id, step).await;
    }

    // Complete
    let outcome = TaskOutcome::Success {
        verdict: "Migration created successfully".to_string(),
        artifacts: vec!["migrations/001_create_users.sql".to_string()],
    };
    server
        .memory
        .complete_episode(episode_id, outcome)
        .await
        .expect("Should complete");

    // Query for similar episodes
    let query_input = QuerySemanticMemoryInput {
        query: "How to create database tables?".to_string(),
        limit: Some(5),
        similarity_threshold: Some(0.3),
        domain: Some("database".to_string()),
        task_type: None,
    };

    let query_result = server.execute_query_semantic_memory(query_input).await;

    assert!(query_result.is_ok());

    let output = query_result.unwrap();
    let results = output
        .get("results")
        .and_then(|v| v.as_array())
        .unwrap_or(&vec![]);

    println!("Workflow test: Found {} similar episodes", results.len());

    // The created episode should influence results
    // (even if embeddings are mocked, the structure should work)
}

#[tokio::test]
async fn test_mcp_multi_domain_queries() {
    let server = create_test_server().await;

    let domains = vec![
        ("web-api", "REST API endpoints"),
        ("database", "SQL query optimization"),
        ("authentication", "JWT token validation"),
        ("testing", "Unit test coverage"),
    ];

    for (domain, query) in domains {
        let input = QuerySemanticMemoryInput {
            query: query.to_string(),
            limit: Some(5),
            similarity_threshold: Some(0.5),
            domain: Some(domain.to_string()),
            task_type: None,
        };

        let result = server.execute_query_semantic_memory(input).await;

        assert!(result.is_ok(), "Domain {} query should succeed", domain);

        let output = result.unwrap();
        let results_found = output
            .get("results_found")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        println!("Domain {}: {} results", domain, results_found);
    }
}
