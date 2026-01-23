//! Integration tests for episode lifecycle MCP tools
//!
//! Tests the new MCP tools for creating, updating, completing, and managing
//! episodes programmatically through the MCP interface.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use memory_core::{MemoryConfig, SelfLearningMemory};
use memory_mcp::{MemoryMCPServer, SandboxConfig};
use serde_json::json;
use std::sync::Arc;

/// Helper to create a test server with in-memory storage
async fn create_test_server() -> MemoryMCPServer {
    let memory = Arc::new(SelfLearningMemory::with_config(MemoryConfig {
        quality_threshold: 0.0, // Zero threshold for test episodes
        ..Default::default()
    }));

    let sandbox_config = SandboxConfig::restrictive();
    MemoryMCPServer::new(sandbox_config, memory).await.unwrap()
}

#[tokio::test]
async fn test_create_episode_tool() {
    let server = create_test_server().await;

    let args = json!({
        "task_description": "Implement user authentication",
        "domain": "web-api",
        "task_type": "code_generation",
        "language": "rust",
        "framework": "axum",
        "tags": ["auth", "api", "security"],
        "complexity": "complex"
    });

    let result = server.create_episode_tool(args).await.unwrap();

    assert_eq!(result["success"], true);
    assert!(result["episode_id"].is_string());
    assert_eq!(result["task_description"], "Implement user authentication");
    assert_eq!(result["domain"], "web-api");
    assert_eq!(result["task_type"], "code_generation");
}

#[tokio::test]
async fn test_create_episode_missing_required_field() {
    let server = create_test_server().await;

    let args = json!({
        "task_description": "Test task",
        "domain": "test"
        // Missing task_type
    });

    let result = server.create_episode_tool(args).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("task_type"));
}

#[tokio::test]
async fn test_create_episode_invalid_task_type() {
    let server = create_test_server().await;

    let args = json!({
        "task_description": "Test task",
        "domain": "test",
        "task_type": "invalid_type"
    });

    let result = server.create_episode_tool(args).await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid task_type"));
}

#[tokio::test]
async fn test_add_episode_step_tool() {
    let server = create_test_server().await;

    // First create an episode
    let create_args = json!({
        "task_description": "Test task",
        "domain": "test",
        "task_type": "testing"
    });
    let create_result = server.create_episode_tool(create_args).await.unwrap();
    let episode_id = create_result["episode_id"].as_str().unwrap();

    // Add a step
    let step_args = json!({
        "episode_id": episode_id,
        "step_number": 1,
        "tool": "test_runner",
        "action": "Running unit tests",
        "parameters": {"suite": "integration"},
        "result": {
            "type": "success",
            "output": "All tests passed"
        },
        "latency_ms": 150
    });

    let result = server.add_episode_step_tool(step_args).await.unwrap();

    assert_eq!(result["success"], true);
    assert_eq!(result["episode_id"], episode_id);
    assert_eq!(result["step_number"], 1);
}

#[tokio::test]
async fn test_add_episode_step_with_error_result() {
    let server = create_test_server().await;

    // Create episode
    let create_args = json!({
        "task_description": "Test task",
        "domain": "test",
        "task_type": "debugging"
    });
    let create_result = server.create_episode_tool(create_args).await.unwrap();
    let episode_id = create_result["episode_id"].as_str().unwrap();

    // Add a step with error
    let step_args = json!({
        "episode_id": episode_id,
        "step_number": 1,
        "tool": "compiler",
        "action": "Compiling code",
        "result": {
            "type": "error",
            "message": "Type mismatch in function signature"
        }
    });

    let result = server.add_episode_step_tool(step_args).await.unwrap();
    assert_eq!(result["success"], true);
}

#[tokio::test]
async fn test_complete_episode_success() {
    let server = create_test_server().await;

    // Create episode
    let create_args = json!({
        "task_description": "Build new feature",
        "domain": "feature",
        "task_type": "code_generation"
    });
    let create_result = server.create_episode_tool(create_args).await.unwrap();
    let episode_id = create_result["episode_id"].as_str().unwrap();

    // Complete with success
    let complete_args = json!({
        "episode_id": episode_id,
        "outcome_type": "success",
        "verdict": "Feature implemented successfully",
        "artifacts": ["user_auth.rs", "auth_tests.rs"]
    });

    let result = server.complete_episode_tool(complete_args).await.unwrap();

    assert_eq!(result["success"], true);
    assert_eq!(result["episode_id"], episode_id);
    assert_eq!(result["outcome_type"], "success");
}

#[tokio::test]
async fn test_complete_episode_partial_success() {
    let server = create_test_server().await;

    // Create episode
    let create_args = json!({
        "task_description": "Implement multiple features",
        "domain": "feature",
        "task_type": "code_generation"
    });
    let create_result = server.create_episode_tool(create_args).await.unwrap();
    let episode_id = create_result["episode_id"].as_str().unwrap();

    // Complete with partial success
    let complete_args = json!({
        "episode_id": episode_id,
        "outcome_type": "partial_success",
        "verdict": "Some features implemented",
        "completed": ["login", "logout"],
        "failed": ["password-reset", "2fa"]
    });

    let result = server.complete_episode_tool(complete_args).await.unwrap();

    assert_eq!(result["success"], true);
    assert_eq!(result["outcome_type"], "partial_success");
}

#[tokio::test]
async fn test_complete_episode_failure() {
    let server = create_test_server().await;

    // Create episode
    let create_args = json!({
        "task_description": "Fix critical bug",
        "domain": "bugfix",
        "task_type": "debugging"
    });
    let create_result = server.create_episode_tool(create_args).await.unwrap();
    let episode_id = create_result["episode_id"].as_str().unwrap();

    // Complete with failure
    let complete_args = json!({
        "episode_id": episode_id,
        "outcome_type": "failure",
        "reason": "Root cause not identified",
        "error_details": "Unable to reproduce the issue in test environment"
    });

    let result = server.complete_episode_tool(complete_args).await.unwrap();

    assert_eq!(result["success"], true);
    assert_eq!(result["outcome_type"], "failure");
}

#[tokio::test]
async fn test_get_episode_tool() {
    let server = create_test_server().await;

    // Create and complete an episode
    let create_args = json!({
        "task_description": "Test retrieval",
        "domain": "test",
        "task_type": "testing"
    });
    let create_result = server.create_episode_tool(create_args).await.unwrap();
    let episode_id = create_result["episode_id"].as_str().unwrap();

    // Add a step
    let step_args = json!({
        "episode_id": episode_id,
        "step_number": 1,
        "tool": "test",
        "action": "Testing"
    });
    server.add_episode_step_tool(step_args).await.unwrap();

    // Complete
    let complete_args = json!({
        "episode_id": episode_id,
        "outcome_type": "success",
        "verdict": "Test completed"
    });
    server.complete_episode_tool(complete_args).await.unwrap();

    // Get episode
    let get_args = json!({"episode_id": episode_id});
    let result = server.get_episode_tool(get_args).await.unwrap();

    assert_eq!(result["success"], true);
    assert!(result["episode"].is_object());
    let episode = &result["episode"];
    assert_eq!(episode["task_description"], "Test retrieval");
    assert!(episode["steps"].is_array());
}

#[tokio::test]
async fn test_get_episode_timeline_tool() {
    let server = create_test_server().await;

    // Create episode
    let create_args = json!({
        "task_description": "Multi-step task",
        "domain": "test",
        "task_type": "analysis"
    });
    let create_result = server.create_episode_tool(create_args).await.unwrap();
    let episode_id = create_result["episode_id"].as_str().unwrap();

    // Add multiple steps
    for i in 1..=3 {
        let step_args = json!({
            "episode_id": episode_id,
            "step_number": i,
            "tool": format!("tool_{}", i),
            "action": format!("Action {}", i),
            "result": {"type": "success", "output": "OK"},
            "latency_ms": i * 100
        });
        server.add_episode_step_tool(step_args).await.unwrap();
    }

    // Complete
    let complete_args = json!({
        "episode_id": episode_id,
        "outcome_type": "success",
        "verdict": "All steps completed"
    });
    server.complete_episode_tool(complete_args).await.unwrap();

    // Get timeline
    let timeline_args = json!({"episode_id": episode_id});
    let result = server
        .get_episode_timeline_tool(timeline_args)
        .await
        .unwrap();

    assert_eq!(result["success"], true);
    assert_eq!(result["task_description"], "Multi-step task");
    assert_eq!(result["step_count"], 3);
    assert_eq!(result["outcome"], "success");

    let timeline = result["timeline"].as_array().unwrap();
    assert_eq!(timeline.len(), 3);
    assert_eq!(timeline[0]["step_number"], 1);
    assert_eq!(timeline[0]["tool"], "tool_1");
    assert_eq!(timeline[0]["result_type"], "success");
}

#[tokio::test]
async fn test_delete_episode_tool() {
    let server = create_test_server().await;

    // Create episode
    let create_args = json!({
        "task_description": "To be deleted",
        "domain": "test",
        "task_type": "testing"
    });
    let create_result = server.create_episode_tool(create_args).await.unwrap();
    let episode_id = create_result["episode_id"].as_str().unwrap();

    // Delete without confirmation should fail
    let delete_args = json!({
        "episode_id": episode_id,
        "confirm": false
    });
    let result = server.delete_episode_tool(delete_args).await;
    assert!(result.is_err());

    // Delete with confirmation should succeed
    let delete_args = json!({
        "episode_id": episode_id,
        "confirm": true
    });
    let result = server.delete_episode_tool(delete_args).await.unwrap();
    assert_eq!(result["success"], true);

    // Getting deleted episode should fail
    let get_args = json!({"episode_id": episode_id});
    let result = server.get_episode_tool(get_args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_full_episode_lifecycle() {
    let server = create_test_server().await;

    // 1. Create episode
    let create_result = server
        .create_episode_tool(json!({
            "task_description": "Implement feature X",
            "domain": "backend",
            "task_type": "code_generation",
            "language": "rust",
            "complexity": "moderate",
            "tags": ["feature", "api"]
        }))
        .await
        .unwrap();

    let episode_id = create_result["episode_id"].as_str().unwrap().to_string();
    assert_eq!(create_result["success"], true);

    // 2. Add multiple steps
    server
        .add_episode_step_tool(json!({
            "episode_id": &episode_id,
            "step_number": 1,
            "tool": "planner",
            "action": "Planning implementation",
            "result": {"type": "success", "output": "Plan created"},
            "latency_ms": 50
        }))
        .await
        .unwrap();

    server
        .add_episode_step_tool(json!({
            "episode_id": &episode_id,
            "step_number": 2,
            "tool": "code_generator",
            "action": "Generating code",
            "result": {"type": "success", "output": "Code generated"},
            "latency_ms": 200
        }))
        .await
        .unwrap();

    server
        .add_episode_step_tool(json!({
            "episode_id": &episode_id,
            "step_number": 3,
            "tool": "test_runner",
            "action": "Running tests",
            "result": {"type": "success", "output": "All tests passed"},
            "latency_ms": 300
        }))
        .await
        .unwrap();

    // 3. Get timeline to verify steps
    let timeline_result = server
        .get_episode_timeline_tool(json!({"episode_id": &episode_id}))
        .await
        .unwrap();

    assert_eq!(timeline_result["step_count"], 3);

    // 4. Complete episode
    let complete_result = server
        .complete_episode_tool(json!({
            "episode_id": &episode_id,
            "outcome_type": "success",
            "verdict": "Feature implemented and tested",
            "artifacts": ["feature_x.rs", "feature_x_test.rs"]
        }))
        .await
        .unwrap();

    assert_eq!(complete_result["success"], true);

    // 5. Retrieve full episode
    let get_result = server
        .get_episode_tool(json!({"episode_id": &episode_id}))
        .await
        .unwrap();

    let episode = &get_result["episode"];
    assert_eq!(episode["task_description"], "Implement feature X");
    assert_eq!(episode["steps"].as_array().unwrap().len(), 3);
}
