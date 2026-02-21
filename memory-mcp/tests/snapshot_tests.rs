//! Snapshot tests for MCP tool responses and schemas
//!
//! These tests verify that tool definitions, responses, and error formats
//! remain consistent across changes. Part of ADR-033 Phase 6.

use insta::assert_json_snapshot;
use memory_mcp::types::{ErrorType, ExecutionContext, ExecutionResult, ExecutionStats, Tool};
use serde_json::json;

/// Test that Tool struct serialization produces consistent output
#[test]
fn test_tool_definition_serialization() {
    let tool = Tool::new(
        "query_memory".to_string(),
        "Query episodic memory for relevant past experiences".to_string(),
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query describing the task"
                },
                "domain": {
                    "type": "string",
                    "description": "Task domain"
                }
            },
            "required": ["query", "domain"]
        }),
    );

    assert_json_snapshot!(tool);
}

/// Test execution result success format
#[test]
fn test_execution_result_success() {
    let result = ExecutionResult::Success {
        output: r#"{"sum": 42, "message": "Hello from sandbox"}"#.to_string(),
        stdout: "Processing...\nDone!".to_string(),
        stderr: "".to_string(),
        execution_time_ms: 150,
    };

    assert_json_snapshot!(result);
}

/// Test execution result error format
#[test]
fn test_execution_result_error() {
    let result = ExecutionResult::Error {
        message: "ReferenceError: undefined_variable is not defined".to_string(),
        error_type: ErrorType::Runtime,
        stdout: "".to_string(),
        stderr: "Error at line 5: undefined_variable".to_string(),
    };

    assert_json_snapshot!(result);
}

/// Test execution result timeout format
#[test]
fn test_execution_result_timeout() {
    let result = ExecutionResult::Timeout {
        elapsed_ms: 5000,
        partial_output: Some("Processing step 1...".to_string()),
    };

    assert_json_snapshot!(result);
}

/// Test execution result security violation format
#[test]
fn test_execution_result_security_violation() {
    use memory_mcp::types::SecurityViolationType;

    let result = ExecutionResult::SecurityViolation {
        reason: "Attempted filesystem access to /etc/passwd".to_string(),
        violation_type: SecurityViolationType::FileSystemAccess,
    };

    assert_json_snapshot!(result);
}

/// Test execution stats serialization
#[test]
fn test_execution_stats_serialization() {
    let stats = ExecutionStats {
        total_executions: 150,
        successful_executions: 142,
        failed_executions: 8,
        timeout_count: 3,
        security_violations: 1,
        avg_execution_time_ms: 245.5,
    };

    assert_json_snapshot!(stats);
}

/// Test execution context serialization
#[test]
fn test_execution_context_serialization() {
    let mut ctx = ExecutionContext::new(
        "Calculate sum of array".to_string(),
        json!({
            "numbers": [1, 2, 3, 4, 5],
            "operation": "sum"
        }),
    );

    // Add some environment variables
    ctx.env.insert("DEBUG".to_string(), "true".to_string());
    ctx.env.insert("NODE_ENV".to_string(), "test".to_string());

    // Serialize and verify structure (avoid direct snapshot due to HashMap ordering)
    let json_str = serde_json::to_string_pretty(&ctx).unwrap();
    assert!(json_str.contains("\"task\": \"Calculate sum of array\""));
    assert!(json_str.contains("\"DEBUG\": \"true\""));
    assert!(json_str.contains("\"NODE_ENV\": \"test\""));
    assert!(json_str.contains("\"numbers\": ["));
    assert!(json_str.contains("\"operation\": \"sum\""));
}

/// Test multiple tool definitions in a collection
#[test]
fn test_tool_definitions_collection() {
    let tools = vec![
        Tool::new(
            "query_memory".to_string(),
            "Query episodic memory".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string"},
                    "limit": {"type": "integer", "default": 10}
                },
                "required": ["query"]
            }),
        ),
        Tool::new(
            "analyze_patterns".to_string(),
            "Analyze patterns from episodes".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "task_type": {"type": "string"},
                    "min_success_rate": {"type": "number", "default": 0.7}
                },
                "required": ["task_type"]
            }),
        ),
        Tool::new(
            "health_check".to_string(),
            "Check server health".to_string(),
            json!({"type": "object", "properties": {}}),
        ),
    ];

    assert_json_snapshot!(tools);
}
