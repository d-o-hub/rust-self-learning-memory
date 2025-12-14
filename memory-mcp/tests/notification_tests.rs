use serde_json::json;

use memory_mcp::jsonrpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
#[derive(Debug, serde::Serialize)]
struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    protocol_version: String,
    capabilities: serde_json::Value,
    #[serde(rename = "serverInfo")]
    server_info: serde_json::Value,
}

async fn handle_initialize(request: JsonRpcRequest) -> Option<JsonRpcResponse> {
    if request.id.is_none() {
        return None;
    }
    let result = InitializeResult {
        protocol_version: "2024-11-05".to_string(),
        capabilities: serde_json::json!({"tools":{"listChanged":false}}),
        server_info: serde_json::json!({"name":"memory-mcp-server","version":"test"}),
    };
    let value = serde_json::to_value(result).unwrap();
    Some(JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id,
        result: Some(value),
        error: None,
    })
}

async fn handle_shutdown(request: JsonRpcRequest) -> Option<JsonRpcResponse> {
    if request.id.is_none() {
        return None;
    }
    Some(JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id,
        result: Some(serde_json::json!(null)),
        error: None,
    })
}

#[tokio::test]
async fn test_initialize_notification_no_response() {
    let req = JsonRpcRequest {
        id: None,
        method: "initialize".into(),
        params: None,
    };
    let resp = handle_initialize(req).await;
    assert!(resp.is_none());
}

#[tokio::test]
async fn test_initialize_with_id_response() {
    let req = JsonRpcRequest {
        id: Some(json!(1)),
        method: "initialize".into(),
        params: None,
    };
    let resp = handle_initialize(req).await;
    assert!(resp.is_some());
    let resp = resp.unwrap();
    assert_eq!(resp.jsonrpc, "2.0");
    assert!(resp.result.is_some());
    assert!(resp.error.is_none());
}

#[tokio::test]
async fn test_shutdown_notification_no_response() {
    let req = JsonRpcRequest {
        id: None,
        method: "shutdown".into(),
        params: None,
    };
    let resp = handle_shutdown(req).await;
    assert!(resp.is_none());
}
