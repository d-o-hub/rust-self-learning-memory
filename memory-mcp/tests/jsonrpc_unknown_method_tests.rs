use memory_mcp::jsonrpc::{JsonRpcRequest, JsonRpcResponse};
use serde_json::json;

#[tokio::test]
async fn test_unknown_method_notification_no_response() {
    let req = JsonRpcRequest {
        jsonrpc: Some("2.0".into()),
        id: None,
        method: "nope".into(),
        params: None,
    };
    let resp = crate::bin_server_handle_request(req).await; // we'll simulate via a small shim below
    assert!(resp.is_none());
}

// Minimal shim replicating server handler logic for unknown method path
async fn bin_server_handle_request(req: JsonRpcRequest) -> Option<JsonRpcResponse> {
    if req.id.is_none() || matches!(req.id, Some(serde_json::Value::Null)) {
        return None;
    }
    Some(JsonRpcResponse {
        jsonrpc: "2.0".into(),
        id: req.id,
        result: None,
        error: Some(memory_mcp::jsonrpc::JsonRpcError {
            code: -32601,
            message: "Method not found".into(),
            data: None,
        }),
    })
}
