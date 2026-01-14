//! Batch Operations Demo
//!
//! This example demonstrates how to use batch operations to execute multiple
//! MCP tools efficiently with dependency management and parallel execution.
//!
//! Run with: `cargo run --example batch_operations_demo`

use memory_core::SelfLearningMemory;
use memory_mcp::{
    BatchExecutor, BatchMode, BatchOperation, BatchRequest, MemoryMCPServer, SandboxConfig,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    println!("=== MCP Batch Operations Demo ===\n");

    // Create memory system and MCP server
    let memory = Arc::new(SelfLearningMemory::new());
    let _server = MemoryMCPServer::new(SandboxConfig::restrictive(), memory.clone())
        .await
        .expect("Failed to create MCP server");

    // Demo 1: Parallel independent operations
    println!("Demo 1: Parallel Independent Operations");
    println!("----------------------------------------");
    demo_parallel_operations().await?;

    // Demo 2: Operations with dependencies
    println!("\nDemo 2: Operations with Dependencies");
    println!("----------------------------------------");
    demo_dependency_chain().await?;

    // Demo 3: Partial failure handling
    println!("\nDemo 3: Partial Failure Handling");
    println!("----------------------------------------");
    demo_partial_failure().await?;

    // Demo 4: Complex workflow
    println!("\nDemo 4: Complex Workflow with DAG");
    println!("----------------------------------------");
    demo_complex_workflow().await?;

    println!("\n=== Demo Complete ===");
    Ok(())
}

/// Demo 1: Execute multiple independent operations in parallel
async fn demo_parallel_operations() -> anyhow::Result<()> {
    let executor = BatchExecutor::new();

    // Create batch request with 3 independent operations
    let request = BatchRequest {
        operations: vec![
            BatchOperation {
                id: "health_check".to_string(),
                tool: "mock_health_check".to_string(),
                arguments: json!({}),
                depends_on: vec![],
            },
            BatchOperation {
                id: "get_stats".to_string(),
                tool: "mock_get_stats".to_string(),
                arguments: json!({}),
                depends_on: vec![],
            },
            BatchOperation {
                id: "query_patterns".to_string(),
                tool: "mock_query_patterns".to_string(),
                arguments: json!({"domain": "web-api"}),
                depends_on: vec![],
            },
        ],
        mode: BatchMode::Parallel,
        max_parallel: 10,
    };

    // Mock executor function
    let executor_fn = |tool_name: String, _args: serde_json::Value| async move {
        // Simulate some work
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        Ok(json!({
            "tool": tool_name,
            "status": "success",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    };

    let start = std::time::Instant::now();
    let response = executor
        .execute(request, executor_fn)
        .await
        .map_err(|e| anyhow::anyhow!("Batch failed: {}", e))?;
    let duration = start.elapsed();

    println!(
        "✓ Executed {} operations in {}ms",
        response.results.len(),
        duration.as_millis()
    );
    println!(
        "  Success: {}, Failed: {}",
        response.success_count, response.failure_count
    );
    println!("  Parallel executed: {}", response.stats.parallel_executed);
    println!(
        "  Average operation time: {:.1}ms",
        response.stats.avg_duration_ms
    );

    Ok(())
}

/// Demo 2: Execute operations with dependencies
async fn demo_dependency_chain() -> anyhow::Result<()> {
    let executor = BatchExecutor::new();

    // Create a pipeline: fetch → process → analyze
    let request = BatchRequest {
        operations: vec![
            BatchOperation {
                id: "fetch_data".to_string(),
                tool: "mock_fetch".to_string(),
                arguments: json!({"source": "database"}),
                depends_on: vec![],
            },
            BatchOperation {
                id: "process_data".to_string(),
                tool: "mock_process".to_string(),
                arguments: json!({"algorithm": "transform"}),
                depends_on: vec!["fetch_data".to_string()],
            },
            BatchOperation {
                id: "analyze_results".to_string(),
                tool: "mock_analyze".to_string(),
                arguments: json!({"metric": "accuracy"}),
                depends_on: vec!["process_data".to_string()],
            },
        ],
        mode: BatchMode::Parallel,
        max_parallel: 10,
    };

    let executor_fn = |tool_name: String, args: serde_json::Value| async move {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        Ok(json!({
            "tool": tool_name,
            "input": args,
            "status": "completed"
        }))
    };

    let response = executor
        .execute(request, executor_fn)
        .await
        .map_err(|e| anyhow::anyhow!("Batch failed: {}", e))?;

    println!("✓ Pipeline executed successfully");
    println!("  Total time: {}ms", response.total_duration_ms);

    for result in &response.results {
        println!(
            "  {} → {} ({}ms)",
            result.id,
            if result.success { "✓" } else { "✗" },
            result.duration_ms
        );
    }

    Ok(())
}

/// Demo 3: Handle partial failures gracefully
async fn demo_partial_failure() -> anyhow::Result<()> {
    let executor = BatchExecutor::new();

    let request = BatchRequest {
        operations: vec![
            BatchOperation {
                id: "op1_success".to_string(),
                tool: "mock_tool".to_string(),
                arguments: json!({"fail": false, "value": 1}),
                depends_on: vec![],
            },
            BatchOperation {
                id: "op2_failure".to_string(),
                tool: "mock_tool".to_string(),
                arguments: json!({"fail": true}),
                depends_on: vec![],
            },
            BatchOperation {
                id: "op3_success".to_string(),
                tool: "mock_tool".to_string(),
                arguments: json!({"fail": false, "value": 3}),
                depends_on: vec![],
            },
        ],
        mode: BatchMode::Parallel,
        max_parallel: 10,
    };

    let executor_fn = |_tool: String, args: serde_json::Value| async move {
        if args.get("fail").and_then(|v| v.as_bool()).unwrap_or(false) {
            Err((-32000, "Simulated failure".to_string()))
        } else {
            Ok(json!({"status": "success", "data": args}))
        }
    };

    let response = executor
        .execute(request, executor_fn)
        .await
        .map_err(|e| anyhow::anyhow!("Batch failed: {}", e))?;

    println!("✓ Batch completed with partial results");
    println!(
        "  Total: {}, Success: {}, Failed: {}",
        response.results.len(),
        response.success_count,
        response.failure_count
    );

    for result in &response.results {
        if result.success {
            println!("  {} → Success", result.id);
        } else {
            let error = result.error.as_ref().unwrap();
            println!("  {} → Failed: {}", result.id, error.message);
        }
    }

    Ok(())
}

/// Demo 4: Complex workflow with multiple parallel branches
async fn demo_complex_workflow() -> anyhow::Result<()> {
    let executor = BatchExecutor::new();

    // Build a complex DAG:
    //           init
    //          /    \
    //      fetch1  fetch2
    //          \    /
    //          merge
    //            |
    //         analyze
    let request = BatchRequest {
        operations: vec![
            BatchOperation {
                id: "init".to_string(),
                tool: "initialize".to_string(),
                arguments: json!({"config": "prod"}),
                depends_on: vec![],
            },
            BatchOperation {
                id: "fetch1".to_string(),
                tool: "fetch_source_1".to_string(),
                arguments: json!({"endpoint": "/api/data1"}),
                depends_on: vec!["init".to_string()],
            },
            BatchOperation {
                id: "fetch2".to_string(),
                tool: "fetch_source_2".to_string(),
                arguments: json!({"endpoint": "/api/data2"}),
                depends_on: vec!["init".to_string()],
            },
            BatchOperation {
                id: "merge".to_string(),
                tool: "merge_data".to_string(),
                arguments: json!({"strategy": "union"}),
                depends_on: vec!["fetch1".to_string(), "fetch2".to_string()],
            },
            BatchOperation {
                id: "analyze".to_string(),
                tool: "analyze_merged".to_string(),
                arguments: json!({"algorithm": "ml_insights"}),
                depends_on: vec!["merge".to_string()],
            },
        ],
        mode: BatchMode::Parallel,
        max_parallel: 10,
    };

    let executor_fn = |tool_name: String, _args: serde_json::Value| async move {
        // Simulate varying execution times
        let delay = match tool_name.as_str() {
            "initialize" => 20,
            "fetch_source_1" | "fetch_source_2" => 80,
            "merge_data" => 40,
            "analyze_merged" => 60,
            _ => 30,
        };

        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;

        Ok(json!({
            "tool": tool_name,
            "status": "completed"
        }))
    };

    let start = std::time::Instant::now();
    let response = executor
        .execute(request, executor_fn)
        .await
        .map_err(|e| anyhow::anyhow!("Batch failed: {}", e))?;
    let duration = start.elapsed();

    println!("✓ Complex workflow completed");
    println!(
        "  Total time: {}ms (sequential would be ~{}ms)",
        duration.as_millis(),
        20 + 80 + 80 + 40 + 60 // Sum of all operations
    );
    println!("  Operations: {}", response.results.len());
    println!("  Speedup: ~{:.1}x", 280.0 / duration.as_millis() as f64);

    // Show execution timeline
    println!("\n  Execution Timeline:");
    let mut sorted_results = response.results.clone();
    sorted_results.sort_by_key(|r| response.results.iter().position(|x| x.id == r.id).unwrap());

    for result in sorted_results {
        println!("    {} ({}ms)", result.id, result.duration_ms);
    }

    Ok(())
}
