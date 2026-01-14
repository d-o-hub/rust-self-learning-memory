//! Comprehensive tests for batch operations
//!
//! Tests cover:
//! - Basic parallel execution
//! - Dependency management
//! - Error handling and partial results
//! - Sequential and fail-fast modes
//! - Performance characteristics

use memory_core::SelfLearningMemory;
use memory_mcp::{
    BatchExecutor, BatchMode, BatchOperation, BatchRequest, MemoryMCPServer, SandboxConfig,
};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

/// Helper to create test server
async fn create_test_server() -> MemoryMCPServer {
    let memory = Arc::new(SelfLearningMemory::new());
    MemoryMCPServer::new(SandboxConfig::restrictive(), memory)
        .await
        .expect("Failed to create test server")
}

#[tokio::test]
async fn test_batch_parallel_independent_operations() {
    let executor = BatchExecutor::new();

    let request = BatchRequest {
        operations: vec![
            BatchOperation {
                id: "op1".to_string(),
                tool: "test_tool".to_string(),
                arguments: json!({"delay_ms": 50, "value": 1}),
                depends_on: vec![],
            },
            BatchOperation {
                id: "op2".to_string(),
                tool: "test_tool".to_string(),
                arguments: json!({"delay_ms": 50, "value": 2}),
                depends_on: vec![],
            },
            BatchOperation {
                id: "op3".to_string(),
                tool: "test_tool".to_string(),
                arguments: json!({"delay_ms": 50, "value": 3}),
                depends_on: vec![],
            },
        ],
        mode: BatchMode::Parallel,
        max_parallel: 10,
    };

    let start = std::time::Instant::now();

    let executor_fn = |_tool: String, args: serde_json::Value| async move {
        let delay = args["delay_ms"].as_u64().unwrap_or(0);
        tokio::time::sleep(Duration::from_millis(delay)).await;
        Ok(args)
    };

    let response = executor.execute(request, executor_fn).await.unwrap();

    let duration = start.elapsed();

    // Verify all operations succeeded
    assert_eq!(response.results.len(), 3);
    assert_eq!(response.success_count, 3);
    assert_eq!(response.failure_count, 0);

    // Verify parallel execution (should be ~50ms, not 150ms)
    assert!(
        duration.as_millis() < 100,
        "Parallel execution too slow: {}ms",
        duration.as_millis()
    );

    // Verify all operations executed in parallel
    assert_eq!(response.stats.parallel_executed, 3);
    assert_eq!(response.stats.sequential_executed, 0);
}

#[tokio::test]
async fn test_batch_with_dependencies() {
    let executor = BatchExecutor::new();

    let request = BatchRequest {
        operations: vec![
            BatchOperation {
                id: "fetch_data".to_string(),
                tool: "test_tool".to_string(),
                arguments: json!({"action": "fetch"}),
                depends_on: vec![],
            },
            BatchOperation {
                id: "process_data".to_string(),
                tool: "test_tool".to_string(),
                arguments: json!({"action": "process"}),
                depends_on: vec!["fetch_data".to_string()],
            },
            BatchOperation {
                id: "analyze_data".to_string(),
                tool: "test_tool".to_string(),
                arguments: json!({"action": "analyze"}),
                depends_on: vec!["process_data".to_string()],
            },
        ],
        mode: BatchMode::Parallel,
        max_parallel: 10,
    };

    let executor_fn = |_tool: String, args: serde_json::Value| async move {
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(args)
    };

    let response = executor.execute(request, executor_fn).await.unwrap();

    // Verify all operations completed successfully
    assert_eq!(response.results.len(), 3);
    assert_eq!(response.success_count, 3);
    assert_eq!(response.failure_count, 0);

    // Verify operations executed in correct order
    let fetch_idx = response
        .results
        .iter()
        .position(|r| r.id == "fetch_data")
        .unwrap();
    let process_idx = response
        .results
        .iter()
        .position(|r| r.id == "process_data")
        .unwrap();
    let analyze_idx = response
        .results
        .iter()
        .position(|r| r.id == "analyze_data")
        .unwrap();

    assert!(
        fetch_idx < process_idx,
        "fetch should complete before process"
    );
    assert!(
        process_idx < analyze_idx,
        "process should complete before analyze"
    );
}

#[tokio::test]
async fn test_batch_partial_failure() {
    let executor = BatchExecutor::new();

    let request = BatchRequest {
        operations: vec![
            BatchOperation {
                id: "success1".to_string(),
                tool: "test_tool".to_string(),
                arguments: json!({"fail": false, "value": 1}),
                depends_on: vec![],
            },
            BatchOperation {
                id: "failure".to_string(),
                tool: "test_tool".to_string(),
                arguments: json!({"fail": true}),
                depends_on: vec![],
            },
            BatchOperation {
                id: "success2".to_string(),
                tool: "test_tool".to_string(),
                arguments: json!({"fail": false, "value": 2}),
                depends_on: vec![],
            },
        ],
        mode: BatchMode::Parallel,
        max_parallel: 10,
    };

    let executor_fn = |_tool: String, args: serde_json::Value| async move {
        if args.get("fail").and_then(|v| v.as_bool()).unwrap_or(false) {
            Err((-32000, "Intentional failure".to_string()))
        } else {
            Ok(args)
        }
    };

    let response = executor.execute(request, executor_fn).await.unwrap();

    // Verify partial results
    assert_eq!(response.results.len(), 3);
    assert_eq!(response.success_count, 2);
    assert_eq!(response.failure_count, 1);

    // Verify successful operations have results
    let successful: Vec<_> = response.results.iter().filter(|r| r.success).collect();
    assert_eq!(successful.len(), 2);

    // Verify failed operation has error
    let failed = response.results.iter().find(|r| r.id == "failure").unwrap();
    assert!(!failed.success);
    assert!(failed.error.is_some());
    assert_eq!(failed.error.as_ref().unwrap().code, -32000);
}

#[tokio::test]
async fn test_batch_sequential_mode() {
    let executor = BatchExecutor::new();

    let request = BatchRequest {
        operations: vec![
            BatchOperation {
                id: "op1".to_string(),
                tool: "test_tool".to_string(),
                arguments: json!({"value": 1}),
                depends_on: vec![],
            },
            BatchOperation {
                id: "op2".to_string(),
                tool: "test_tool".to_string(),
                arguments: json!({"value": 2}),
                depends_on: vec![],
            },
        ],
        mode: BatchMode::Sequential,
        max_parallel: 10,
    };

    let executor_fn = |_tool: String, args: serde_json::Value| async move {
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(args)
    };

    let response = executor.execute(request, executor_fn).await.unwrap();

    assert_eq!(response.results.len(), 2);
    assert_eq!(response.success_count, 2);
    assert_eq!(response.stats.sequential_executed, 2);
    assert_eq!(response.stats.parallel_executed, 0);
}

#[tokio::test]
async fn test_batch_fail_fast_mode() {
    let executor = BatchExecutor::new();

    let request = BatchRequest {
        operations: vec![
            BatchOperation {
                id: "op1".to_string(),
                tool: "test_tool".to_string(),
                arguments: json!({"fail": false}),
                depends_on: vec![],
            },
            BatchOperation {
                id: "op2_fails".to_string(),
                tool: "test_tool".to_string(),
                arguments: json!({"fail": true}),
                depends_on: vec![],
            },
            BatchOperation {
                id: "op3_skipped".to_string(),
                tool: "test_tool".to_string(),
                arguments: json!({"fail": false}),
                depends_on: vec![],
            },
        ],
        mode: BatchMode::FailFast,
        max_parallel: 10,
    };

    let executor_fn = |_tool: String, args: serde_json::Value| async move {
        if args.get("fail").and_then(|v| v.as_bool()).unwrap_or(false) {
            Err((-32000, "Intentional failure".to_string()))
        } else {
            Ok(args)
        }
    };

    let response = executor.execute(request, executor_fn).await.unwrap();

    // Should stop after encountering a failure
    // Note: HashMap iteration order is not guaranteed, so the failing operation
    // might be executed at different positions. We just verify that execution
    // stopped when a failure occurred.
    assert!(
        response.failure_count >= 1,
        "Should have at least one failure"
    );
    assert!(
        response.results.len() < 3,
        "Should stop before executing all operations in fail-fast mode, got {} results",
        response.results.len()
    );
}

#[tokio::test]
async fn test_batch_circular_dependency_detection() {
    use memory_mcp::DependencyGraph;

    let operations = vec![
        BatchOperation {
            id: "op1".to_string(),
            tool: "test".to_string(),
            arguments: json!({}),
            depends_on: vec!["op2".to_string()],
        },
        BatchOperation {
            id: "op2".to_string(),
            tool: "test".to_string(),
            arguments: json!({}),
            depends_on: vec!["op1".to_string()],
        },
    ];

    let result = DependencyGraph::new(operations);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("Circular dependency"));
}

#[tokio::test]
async fn test_batch_missing_dependency() {
    use memory_mcp::DependencyGraph;

    let operations = vec![BatchOperation {
        id: "op1".to_string(),
        tool: "test".to_string(),
        arguments: json!({}),
        depends_on: vec!["nonexistent".to_string()],
    }];

    let result = DependencyGraph::new(operations);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("unknown operation"));
}

#[tokio::test]
async fn test_batch_max_parallel_limit() {
    let executor = BatchExecutor::new();

    // Create 10 operations
    let operations: Vec<_> = (0..10)
        .map(|i| BatchOperation {
            id: format!("op{}", i),
            tool: "test_tool".to_string(),
            arguments: json!({"value": i, "delay_ms": 100}),
            depends_on: vec![],
        })
        .collect();

    let request = BatchRequest {
        operations,
        mode: BatchMode::Parallel,
        max_parallel: 3, // Limit to 3 concurrent operations
    };

    let start = std::time::Instant::now();

    let executor_fn = |_tool: String, args: serde_json::Value| async move {
        let delay = args["delay_ms"].as_u64().unwrap_or(0);
        tokio::time::sleep(Duration::from_millis(delay)).await;
        Ok(args)
    };

    let response = executor.execute(request, executor_fn).await.unwrap();

    let duration = start.elapsed();

    // Verify all operations completed
    assert_eq!(response.results.len(), 10);
    assert_eq!(response.success_count, 10);

    // With max_parallel=3 and 100ms per operation, should take at least 400ms
    // (10 operations / 3 concurrent = 4 batches, but the last batch has only 1 operation)
    // So: 100ms * 4 rounds = 400ms minimum
    assert!(
        duration.as_millis() >= 300,
        "Should respect max_parallel limit, took {}ms",
        duration.as_millis()
    );
}

#[tokio::test]
async fn test_batch_complex_dag() {
    let executor = BatchExecutor::new();

    // Build a complex DAG:
    //     op1
    //    /   \
    //  op2   op3
    //    \   /
    //     op4
    let request = BatchRequest {
        operations: vec![
            BatchOperation {
                id: "op1".to_string(),
                tool: "test".to_string(),
                arguments: json!({"value": 1}),
                depends_on: vec![],
            },
            BatchOperation {
                id: "op2".to_string(),
                tool: "test".to_string(),
                arguments: json!({"value": 2}),
                depends_on: vec!["op1".to_string()],
            },
            BatchOperation {
                id: "op3".to_string(),
                tool: "test".to_string(),
                arguments: json!({"value": 3}),
                depends_on: vec!["op1".to_string()],
            },
            BatchOperation {
                id: "op4".to_string(),
                tool: "test".to_string(),
                arguments: json!({"value": 4}),
                depends_on: vec!["op2".to_string(), "op3".to_string()],
            },
        ],
        mode: BatchMode::Parallel,
        max_parallel: 10,
    };

    let executor_fn = |_tool: String, args: serde_json::Value| async move {
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(args)
    };

    let response = executor.execute(request, executor_fn).await.unwrap();

    // Verify all operations completed
    assert_eq!(response.results.len(), 4);
    assert_eq!(response.success_count, 4);

    // Verify op2 and op3 can run in parallel (both depend only on op1)
    // Both should complete before op4
    let op1_idx = response.results.iter().position(|r| r.id == "op1").unwrap();
    let op2_idx = response.results.iter().position(|r| r.id == "op2").unwrap();
    let op3_idx = response.results.iter().position(|r| r.id == "op3").unwrap();
    let op4_idx = response.results.iter().position(|r| r.id == "op4").unwrap();

    assert!(op1_idx < op2_idx && op1_idx < op3_idx);
    assert!(op2_idx < op4_idx && op3_idx < op4_idx);
}

#[tokio::test]
async fn test_batch_operations_all_complete() {
    let executor = BatchExecutor::new();

    let request = BatchRequest {
        operations: vec![
            BatchOperation {
                id: "first".to_string(),
                tool: "test".to_string(),
                arguments: json!({"value": 1}),
                depends_on: vec![],
            },
            BatchOperation {
                id: "second".to_string(),
                tool: "test".to_string(),
                arguments: json!({"value": 2}),
                depends_on: vec![],
            },
            BatchOperation {
                id: "third".to_string(),
                tool: "test".to_string(),
                arguments: json!({"value": 3}),
                depends_on: vec![],
            },
        ],
        mode: BatchMode::Sequential,
        max_parallel: 10,
    };

    let executor_fn = |_tool: String, args: serde_json::Value| async move { Ok(args) };

    let response = executor.execute(request, executor_fn).await.unwrap();

    // All operations should complete
    assert_eq!(response.results.len(), 3);
    assert_eq!(response.success_count, 3);

    // Verify all expected IDs are present
    let ids: Vec<_> = response.results.iter().map(|r| r.id.as_str()).collect();
    assert!(ids.contains(&"first"));
    assert!(ids.contains(&"second"));
    assert!(ids.contains(&"third"));
}

#[tokio::test]
async fn test_batch_statistics() {
    let executor = BatchExecutor::new();

    let request = BatchRequest {
        operations: vec![
            BatchOperation {
                id: "op1".to_string(),
                tool: "test".to_string(),
                arguments: json!({"delay_ms": 50}),
                depends_on: vec![],
            },
            BatchOperation {
                id: "op2".to_string(),
                tool: "test".to_string(),
                arguments: json!({"delay_ms": 100}),
                depends_on: vec![],
            },
        ],
        mode: BatchMode::Parallel,
        max_parallel: 10,
    };

    let executor_fn = |_tool: String, args: serde_json::Value| async move {
        let delay = args["delay_ms"].as_u64().unwrap_or(0);
        tokio::time::sleep(Duration::from_millis(delay)).await;
        Ok(args)
    };

    let response = executor.execute(request, executor_fn).await.unwrap();

    // Verify statistics
    assert_eq!(response.stats.total_operations, 2);
    assert_eq!(response.stats.parallel_executed, 2);
    assert!(response.stats.avg_duration_ms > 0.0);
    assert!(response.total_duration_ms >= 100); // Should take at least 100ms (longest operation)
}
