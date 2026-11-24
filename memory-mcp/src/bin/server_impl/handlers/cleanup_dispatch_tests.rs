use super::{handle_batch_execute, handle_call_tool};
use do_memory_core::SelfLearningMemory;
use do_memory_mcp::MemoryMCPServer;
use do_memory_mcp::SandboxConfig;
use do_memory_mcp::jsonrpc::JsonRpcRequest;
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::Mutex;

async fn test_server() -> Arc<Mutex<MemoryMCPServer>> {
    let memory = Arc::new(SelfLearningMemory::new());
    let server = MemoryMCPServer::new(SandboxConfig::restrictive(), memory)
        .await
        .unwrap();
    Arc::new(Mutex::new(server))
}

fn request(method: &str, params: Value) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: Some("2.0".to_string()),
        id: Some(json!(1)),
        method: method.to_string(),
        params: Some(params),
    }
}

#[tokio::test]
async fn call_tool_dispatches_cleanup_requests() {
    let server = test_server().await;
    let response = handle_call_tool(
        request(
            "tools/call",
            json!({
                "name": "cleanup_episodes",
                "arguments": {"criterion": "keep_all", "count_only": true}
            }),
        ),
        &server,
    )
    .await
    .unwrap();

    assert!(response.error.is_none());
    let result = response.result.unwrap();
    let text = result["content"][0]["text"].as_str().unwrap();
    let cleanup: Value = serde_json::from_str(text).unwrap();
    assert_eq!(cleanup["candidate_count"], 0);
}

#[tokio::test]
async fn batch_dispatches_cleanup_requests() {
    let server = test_server().await;
    let response = handle_batch_execute(
        request(
            "batch/execute",
            json!({
                "operations": [{
                    "id": "cleanup",
                    "tool": "cleanup_episodes",
                    "arguments": {"criterion": "keep_all", "count_only": true}
                }]
            }),
        ),
        &server,
    )
    .await
    .unwrap();

    assert!(response.error.is_none());
    let result = response.result.unwrap();
    assert_eq!(result["success_count"], 1);
    assert_eq!(result["failure_count"], 0);
    assert_eq!(result["results"][0]["success"], true);
}
