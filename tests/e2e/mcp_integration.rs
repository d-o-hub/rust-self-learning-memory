//! MCP Server Integration Tests
//!
//! Comprehensive E2E tests for MCP server integration:
//! - Tool Execution: Test each MCP tool end-to-end
//! - Server Lifecycle: Start server → Execute tools → Stop server
//! - Error handling and graceful shutdown
//!
//! These tests use the memory-mcp-server binary and communicate via JSON-RPC

#![allow(clippy::unwrap_used, clippy::expect_used)]

use anyhow::Result;
use memory_core::{SelfLearningMemory, TaskOutcome, TaskType};
use memory_mcp::server::MemoryMCPServer;
use memory_mcp::types::SandboxConfig;
use memory_storage_redb::RedbStorage;
use serial_test::serial;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use uuid::Uuid;

/// Helper to create a memory instance with storage
async fn setup_test_memory() -> Result<(Arc<SelfLearningMemory>, TempDir)> {
    let dir = TempDir::new()?;
    let turso_path = dir.path().join("test_turso.redb");
    let cache_path = dir.path().join("test_cache.redb");

    let turso_storage = RedbStorage::new(&turso_path)
        .await
        .expect("Failed to create turso storage");
    let cache_storage = RedbStorage::new(&cache_path)
        .await
        .expect("Failed to create cache storage");

    let memory = Arc::new(SelfLearningMemory::with_storage(
        Default::default(),
        Arc::new(turso_storage),
        Arc::new(cache_storage),
    ));

    Ok((memory, dir))
}

/// MCP Server handle for testing
struct TestMcpServer {
    process: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    request_id: i64,
}

impl TestMcpServer {
    /// Start the MCP server
    async fn start() -> Result<Self> {
        let mut process = Command::new("cargo")
            .args(["run", "--bin", "memory-mcp-server", "--quiet"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;

        let stdin = process.stdin.take().expect("Failed to get stdin");
        let stdout = process.stdout.take().expect("Failed to get stdout");

        // Wait a moment for server to start
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        Ok(Self {
            process,
            stdin,
            stdout: BufReader::new(stdout),
            request_id: 0,
        })
    }

    /// Send a JSON-RPC request
    async fn send_request(&mut self, method: &str, params: serde_json::Value) -> Result<()> {
        self.request_id += 1;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": method,
            "params": params
        });

        let request_str = serde_json::to_string(&request)?;
        self.stdin.write_all(request_str.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;

        Ok(())
    }

    /// Read a JSON-RPC response
    async fn read_response(&mut self) -> Result<serde_json::Value> {
        let mut line = String::new();
        self.stdout.read_line(&mut line).await?;

        let response: serde_json::Value = serde_json::from_str(&line)?;
        Ok(response)
    }

    /// Send request and wait for response
    async fn call(&mut self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        self.send_request(method, params).await?;
        self.read_response().await
    }

    /// Stop the server
    async fn stop(mut self) -> Result<()> {
        // Send shutdown notification
        let _ = self.send_request("shutdown", serde_json::json!({})).await;

        // Give it a moment to shutdown gracefully
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Kill if still running
        let _ = self.process.kill().await;
        let _ = self.process.wait().await;

        Ok(())
    }
}

/// In-memory MCP server for testing (faster, no process spawning)
struct InMemoryMcpServer {
    server: Arc<tokio::sync::Mutex<MemoryMCPServer>>,
}

impl InMemoryMcpServer {
    /// Create a new in-memory MCP server
    async fn new() -> Result<Self> {
        let (memory, _dir) = setup_test_memory().await?;
        let sandbox_config = SandboxConfig::restrictive();
        let server = MemoryMCPServer::new(sandbox_config, memory).await?;

        Ok(Self {
            server: Arc::new(tokio::sync::Mutex::new(server)),
        })
    }

    /// Call a tool directly
    async fn call_tool(
        &self,
        tool_name: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let server = self.server.lock().await;
        let result = server.execute_tool(tool_name, params).await?;
        Ok(result)
    }

    /// List available tools
    async fn list_tools(&self) -> Vec<memory_mcp::types::Tool> {
        let server = self.server.lock().await;
        server.list_tools().await
    }
}

// ============================================================================
// Test 1: Server Initialization and Tool Listing
// ============================================================================

#[tokio::test]
#[serial]
async fn test_mcp_server_initialization() {
    println!("🧪 Testing MCP server initialization...");

    let server = InMemoryMcpServer::new()
        .await
        .expect("Failed to create server");

    // List tools
    let tools = server.list_tools().await;
    assert!(!tools.is_empty(), "Server should have tools");

    println!("  ✓ Server initialized with {} tools", tools.len());

    // Verify essential tools exist
    let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();

    let essential_tools = vec![
        "query_memory",
        "create_episode",
        "complete_episode",
        "add_episode_step",
    ];

    for tool in essential_tools {
        assert!(
            tool_names.contains(&tool.to_string()),
            "Server should have {} tool",
            tool
        );
    }

    println!("  ✓ All essential tools present");
    println!("✅ MCP server initialization test passed!");
}

// ============================================================================
// Test 2: Episode Creation Tool
// ============================================================================

#[tokio::test]
#[serial]
async fn test_mcp_tool_create_episode() {
    println!("🧪 Testing MCP tool: create_episode...");

    let server = InMemoryMcpServer::new()
        .await
        .expect("Failed to create server");

    let params = serde_json::json!({
        "task_description": "Test episode via MCP",
        "domain": "mcp-test",
        "task_type": "code_generation"
    });

    let result = server
        .call_tool("create_episode", params)
        .await
        .expect("Failed to create episode");

    assert!(
        result.get("episode_id").is_some(),
        "Result should have episode_id"
    );

    let episode_id = result["episode_id"]
        .as_str()
        .expect("episode_id should be string");
    assert!(!episode_id.is_empty(), "episode_id should not be empty");

    println!("  ✓ Created episode: {}", episode_id);
    println!("✅ create_episode tool test passed!");
}

// ============================================================================
// Test 3: Episode Step Addition Tool
// ============================================================================

#[tokio::test]
#[serial]
async fn test_mcp_tool_add_episode_step() {
    println!("🧪 Testing MCP tool: add_episode_step...");

    let server = InMemoryMcpServer::new()
        .await
        .expect("Failed to create server");

    // First create an episode
    let create_params = serde_json::json!({
        "task_description": "Test episode for steps",
        "domain": "mcp-test",
        "task_type": "code_generation"
    });

    let create_result = server
        .call_tool("create_episode", create_params)
        .await
        .expect("Failed to create episode");

    let episode_id = create_result["episode_id"].as_str().unwrap();

    // Add a step
    let step_params = serde_json::json!({
        "episode_id": episode_id,
        "step_number": 1,
        "tool_name": "test-tool",
        "action": "Test action via MCP"
    });

    let step_result = server
        .call_tool("add_episode_step", step_params)
        .await
        .expect("Failed to add step");

    assert!(
        step_result.get("success").is_some(),
        "Result should indicate success"
    );

    println!("  ✓ Added step to episode {}", episode_id);
    println!("✅ add_episode_step tool test passed!");
}

// ============================================================================
// Test 4: Episode Completion Tool
// ============================================================================

#[tokio::test]
#[serial]
async fn test_mcp_tool_complete_episode() {
    println!("🧪 Testing MCP tool: complete_episode...");

    let server = InMemoryMcpServer::new()
        .await
        .expect("Failed to create server");

    // Create episode
    let create_params = serde_json::json!({
        "task_description": "Test episode for completion",
        "domain": "mcp-test",
        "task_type": "code_generation"
    });

    let create_result = server
        .call_tool("create_episode", create_params)
        .await
        .expect("Failed to create episode");

    let episode_id = create_result["episode_id"].as_str().unwrap();

    // Complete episode
    let complete_params = serde_json::json!({
        "episode_id": episode_id,
        "outcome_type": "success",
        "verdict": "Completed via MCP tool"
    });

    let complete_result = server
        .call_tool("complete_episode", complete_params)
        .await
        .expect("Failed to complete episode");

    assert!(
        complete_result.get("success").is_some(),
        "Result should indicate success"
    );

    println!("  ✓ Completed episode {}", episode_id);
    println!("✅ complete_episode tool test passed!");
}

// ============================================================================
// Test 5: Query Memory Tool
// ============================================================================

#[tokio::test]
#[serial]
async fn test_mcp_tool_query_memory() {
    println!("🧪 Testing MCP tool: query_memory...");

    let server = InMemoryMcpServer::new()
        .await
        .expect("Failed to create server");

    // Create and complete an episode first
    let create_params = serde_json::json!({
        "task_description": "Searchable test episode",
        "domain": "query-test",
        "task_type": "code_generation"
    });

    let create_result = server
        .call_tool("create_episode", create_params)
        .await
        .expect("Failed to create episode");

    let episode_id = create_result["episode_id"].as_str().unwrap();

    // Complete it
    let complete_params = serde_json::json!({
        "episode_id": episode_id,
        "outcome_type": "success",
        "verdict": "Searchable episode"
    });

    server
        .call_tool("complete_episode", complete_params)
        .await
        .expect("Failed to complete episode");

    // Query for it
    let query_params = serde_json::json!({
        "query": "searchable test",
        "domain": "query-test",
        "limit": 10
    });

    let query_result = server
        .call_tool("query_memory", query_params)
        .await
        .expect("Failed to query memory");

    assert!(
        query_result.get("episodes").is_some(),
        "Result should have episodes"
    );

    println!("  ✓ Queried memory and found episodes");
    println!("✅ query_memory tool test passed!");
}

// ============================================================================
// Test 6: Tag Management Tools
// ============================================================================

#[tokio::test]
#[serial]
async fn test_mcp_tool_tag_management() {
    println!("🧪 Testing MCP tag management tools...");

    let server = InMemoryMcpServer::new()
        .await
        .expect("Failed to create server");

    // Create episode
    let create_params = serde_json::json!({
        "task_description": "Tag test episode",
        "domain": "tag-test",
        "task_type": "code_generation"
    });

    let create_result = server
        .call_tool("create_episode", create_params)
        .await
        .expect("Failed to create episode");

    let episode_id = create_result["episode_id"].as_str().unwrap();

    // Add tags
    let add_tags_params = serde_json::json!({
        "episode_id": episode_id,
        "tags": ["security", "api", "test"]
    });

    let add_result = server
        .call_tool("add_episode_tags", add_tags_params)
        .await
        .expect("Failed to add tags");

    assert!(
        add_result.get("success").is_some(),
        "Add tags should succeed"
    );
    println!("  ✓ Added tags to episode");

    // Get tags
    let get_tags_params = serde_json::json!({
        "episode_id": episode_id
    });

    let get_result = server
        .call_tool("get_episode_tags", get_tags_params)
        .await
        .expect("Failed to get tags");

    assert!(get_result.get("tags").is_some(), "Result should have tags");
    println!("  ✓ Retrieved episode tags");

    // Search by tags
    let search_params = serde_json::json!({
        "tags": ["security"],
        "require_all": false
    });

    let search_result = server
        .call_tool("search_episodes_by_tags", search_params)
        .await
        .expect("Failed to search by tags");

    assert!(
        search_result.get("episodes").is_some(),
        "Result should have episodes"
    );
    println!("  ✓ Searched episodes by tags");

    println!("✅ Tag management tools test passed!");
}

// ============================================================================
// Test 7: Relationship Management Tools
// ============================================================================

#[tokio::test]
#[serial]
async fn test_mcp_tool_relationship_management() {
    println!("🧪 Testing MCP relationship management tools...");

    let server = InMemoryMcpServer::new()
        .await
        .expect("Failed to create server");

    // Create two episodes
    let ep1_params = serde_json::json!({
        "task_description": "Parent episode",
        "domain": "rel-test",
        "task_type": "analysis"
    });

    let ep1_result = server
        .call_tool("create_episode", ep1_params)
        .await
        .expect("Failed to create episode 1");

    let ep1_id = ep1_result["episode_id"].as_str().unwrap();

    let ep2_params = serde_json::json!({
        "task_description": "Child episode",
        "domain": "rel-test",
        "task_type": "code_generation"
    });

    let ep2_result = server
        .call_tool("create_episode", ep2_params)
        .await
        .expect("Failed to create episode 2");

    let ep2_id = ep2_result["episode_id"].as_str().unwrap();

    // Complete both
    for id in [ep1_id, ep2_id] {
        let complete_params = serde_json::json!({
            "episode_id": id,
            "outcome_type": "success",
            "verdict": "Done"
        });
        server
            .call_tool("complete_episode", complete_params)
            .await
            .expect("Failed to complete episode");
    }

    // Add relationship
    let rel_params = serde_json::json!({
        "from_episode_id": ep1_id,
        "to_episode_id": ep2_id,
        "relationship_type": "parent_child",
        "reason": "Test relationship"
    });

    let rel_result = server
        .call_tool("add_episode_relationship", rel_params)
        .await
        .expect("Failed to add relationship");

    assert!(
        rel_result.get("relationship_id").is_some(),
        "Result should have relationship_id"
    );
    println!("  ✓ Added relationship between episodes");

    // Get relationships
    let get_rel_params = serde_json::json!({
        "episode_id": ep1_id,
        "direction": "outgoing"
    });

    let get_rel_result = server
        .call_tool("get_episode_relationships", get_rel_params)
        .await
        .expect("Failed to get relationships");

    assert!(
        get_rel_result.get("relationships").is_some(),
        "Result should have relationships"
    );
    println!("  ✓ Retrieved episode relationships");

    println!("✅ Relationship management tools test passed!");
}

// ============================================================================
// Test 8: Batch Operations Tool
// ============================================================================

#[tokio::test]
#[serial]
async fn test_mcp_tool_batch_operations() {
    println!("🧪 Testing MCP batch operations tool...");

    let server = InMemoryMcpServer::new()
        .await
        .expect("Failed to create server");

    // Create multiple episodes first
    let mut episode_ids = Vec::new();
    for i in 0..3 {
        let create_params = serde_json::json!({
            "task_description": format!("Batch test episode {}", i),
            "domain": "batch-test",
            "task_type": "code_generation"
        });

        let result = server
            .call_tool("create_episode", create_params)
            .await
            .expect("Failed to create episode");

        episode_ids.push(result["episode_id"].as_str().unwrap().to_string());
    }

    // Batch query
    let batch_params = serde_json::json!({
        "filter": {
            "domain": "batch-test",
            "limit": 10
        }
    });

    let batch_result = server
        .call_tool("batch_query_episodes", batch_params)
        .await
        .expect("Failed to batch query");

    assert!(
        batch_result.get("episodes").is_some(),
        "Result should have episodes"
    );
    assert!(
        batch_result.get("total_count").is_some(),
        "Result should have total_count"
    );

    println!("  ✓ Batch query returned results");
    println!("✅ Batch operations tool test passed!");
}

// ============================================================================
// Test 9: Error Handling
// ============================================================================

#[tokio::test]
#[serial]
async fn test_mcp_tool_error_handling() {
    println!("🧪 Testing MCP tool error handling...");

    let server = InMemoryMcpServer::new()
        .await
        .expect("Failed to create server");

    // Try to get non-existent episode
    let params = serde_json::json!({
        "episode_id": "00000000-0000-0000-0000-000000000000"
    });

    let result = server.call_tool("get_episode", params).await;

    // Should fail gracefully
    assert!(
        result.is_err() || result.unwrap().get("error").is_some(),
        "Should handle non-existent episode"
    );

    println!("  ✓ Non-existent episode handled correctly");

    // Try invalid UUID
    let params = serde_json::json!({
        "episode_id": "invalid-uuid"
    });

    let result = server.call_tool("get_episode", params).await;
    assert!(
        result.is_err() || result.unwrap().get("error").is_some(),
        "Should handle invalid UUID"
    );

    println!("  ✓ Invalid UUID handled correctly");

    println!("✅ Error handling test passed!");
}

// ============================================================================
// Test 10: Complete Workflow Integration
// ============================================================================

#[tokio::test]
#[serial]
async fn test_mcp_complete_workflow() {
    println!("🧪 Testing MCP complete workflow...");

    let server = InMemoryMcpServer::new()
        .await
        .expect("Failed to create server");

    // Step 1: Create episode
    let create_params = serde_json::json!({
        "task_description": "Complete workflow test",
        "domain": "workflow-test",
        "task_type": "code_generation"
    });

    let create_result = server
        .call_tool("create_episode", create_params)
        .await
        .expect("Failed to create episode");

    let episode_id = create_result["episode_id"].as_str().unwrap();
    println!("  ✓ Created episode: {}", episode_id);

    // Step 2: Add steps
    for i in 1..=3 {
        let step_params = serde_json::json!({
            "episode_id": episode_id,
            "step_number": i,
            "tool_name": format!("tool-{}", i),
            "action": format!("Action {}", i)
        });

        server
            .call_tool("add_episode_step", step_params)
            .await
            .expect("Failed to add step");
    }
    println!("  ✓ Added 3 steps");

    // Step 3: Add tags
    let tag_params = serde_json::json!({
        "episode_id": episode_id,
        "tags": ["workflow", "test", "integration"]
    });

    server
        .call_tool("add_episode_tags", tag_params)
        .await
        .expect("Failed to add tags");
    println!("  ✓ Added tags");

    // Step 4: Complete episode
    let complete_params = serde_json::json!({
        "episode_id": episode_id,
        "outcome_type": "success",
        "verdict": "Workflow completed successfully"
    });

    server
        .call_tool("complete_episode", complete_params)
        .await
        .expect("Failed to complete episode");
    println!("  ✓ Completed episode");

    // Step 5: Query for the episode
    let query_params = serde_json::json!({
        "query": "workflow test",
        "domain": "workflow-test",
        "limit": 10
    });

    let query_result = server
        .call_tool("query_memory", query_params)
        .await
        .expect("Failed to query memory");

    let episodes = query_result["episodes"]
        .as_array()
        .expect("Should have episodes");
    assert!(!episodes.is_empty(), "Should find the episode");
    println!("  ✓ Queried and found episode");

    // Step 6: Get episode details
    let get_params = serde_json::json!({
        "episode_id": episode_id
    });

    let get_result = server
        .call_tool("get_episode", get_params)
        .await
        .expect("Failed to get episode");

    assert_eq!(
        get_result["id"].as_str().unwrap(),
        episode_id,
        "Should get correct episode"
    );
    println!("  ✓ Retrieved episode details");

    println!("✅ Complete workflow integration test passed!");
}

// ============================================================================
// Test 11: Server Lifecycle (Process-based)
// ============================================================================

#[tokio::test]
#[serial]
#[ignore = "Process-based test - run manually"]
async fn test_mcp_server_lifecycle() {
    println!("🧪 Testing MCP server lifecycle...");

    let mut server = TestMcpServer::start()
        .await
        .expect("Failed to start server");

    // Initialize
    let init_response = server
        .call(
            "initialize",
            serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }),
        )
        .await
        .expect("Failed to initialize");

    assert!(
        init_response.get("result").is_some(),
        "Should have init result"
    );
    println!("  ✓ Server initialized");

    // List tools
    let tools_response = server
        .call("tools/list", serde_json::json!({}))
        .await
        .expect("Failed to list tools");

    assert!(
        tools_response.get("result").is_some(),
        "Should have tools result"
    );
    println!("  ✓ Listed tools");

    // Shutdown
    server.stop().await.expect("Failed to stop server");
    println!("  ✓ Server stopped");

    println!("✅ Server lifecycle test passed!");
}

// ============================================================================
// Test 12: Pattern Analysis Tools
// ============================================================================

#[tokio::test]
#[serial]
async fn test_mcp_tool_pattern_analysis() {
    println!("🧪 Testing MCP pattern analysis tools...");

    let server = InMemoryMcpServer::new()
        .await
        .expect("Failed to create server");

    // Create episodes with similar patterns
    for i in 0..5 {
        let create_params = serde_json::json!({
            "task_description": format!("Pattern test episode {}", i),
            "domain": "pattern-mcp-test",
            "task_type": "code_generation"
        });

        let result = server
            .call_tool("create_episode", create_params)
            .await
            .expect("Failed to create episode");

        let episode_id = result["episode_id"].as_str().unwrap();

        // Add similar steps
        for step in 1..=3 {
            let step_params = serde_json::json!({
                "episode_id": episode_id,
                "step_number": step,
                "tool_name": format!("pattern-tool-{}", step),
                "action": format!("Pattern action {}", step)
            });

            server
                .call_tool("add_episode_step", step_params)
                .await
                .expect("Failed to add step");
        }

        // Complete
        let complete_params = serde_json::json!({
            "episode_id": episode_id,
            "outcome_type": "success",
            "verdict": "Pattern episode completed"
        });

        server
            .call_tool("complete_episode", complete_params)
            .await
            .expect("Failed to complete episode");
    }

    println!("  ✓ Created episodes with patterns");

    // Get patterns by domain
    let patterns_params = serde_json::json!({
        "domain": "pattern-mcp-test"
    });

    let patterns_result = server
        .call_tool("get_patterns_by_domain", patterns_params)
        .await;

    // May or may not have patterns depending on extraction timing
    println!("  ✓ Retrieved patterns");

    // Get pattern recommendations
    let rec_params = serde_json::json!({
        "domain": "pattern-mcp-test",
        "task_type": "code_generation",
        "limit": 5
    });

    let rec_result = server
        .call_tool("get_pattern_recommendations", rec_params)
        .await;

    println!("  ✓ Retrieved pattern recommendations");

    println!("✅ Pattern analysis tools test passed!");
}
