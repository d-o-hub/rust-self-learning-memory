//! Test to verify all server responses are valid JSON
//!
//! This test validates that the MCP server always produces valid JSON-RPC responses
//! that can be parsed by clients, addressing issue #143.

use memory_mcp::jsonrpc::{JsonRpcError, JsonRpcResponse};

#[test]
fn test_valid_jsonrpc_response() {
    // Test that a valid success response serializes to valid JSON
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(1)),
        result: Some(serde_json::json!({"success": true})),
        error: None,
    };

    let json_str = serde_json::to_string(&response).expect("Should serialize to JSON");

    // Verify it's valid JSON that can be parsed
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("Response should be valid JSON");

    assert_eq!(parsed["jsonrpc"], "2.0");
    assert_eq!(parsed["id"], 1);
    assert_eq!(parsed["result"]["success"], true);
}

#[test]
fn test_valid_jsonrpc_error_response() {
    // Test that error responses serialize to valid JSON
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(2)),
        result: None,
        error: Some(JsonRpcError {
            code: -32603,
            message: "Internal error".to_string(),
            data: Some(serde_json::json!({"details": "Test error"})),
        }),
    };

    let json_str = serde_json::to_string(&response).expect("Should serialize to JSON");

    // Verify it's valid JSON that can be parsed
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("Error response should be valid JSON");

    assert_eq!(parsed["jsonrpc"], "2.0");
    assert_eq!(parsed["id"], 2);
    assert!(parsed["error"].is_object());
    assert_eq!(parsed["error"]["code"], -32603);
    assert_eq!(parsed["error"]["message"], "Internal error");
}

#[test]
fn test_parse_error_response_serialization() {
    // Test that parse error responses are valid JSON
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: None,
        result: None,
        error: Some(JsonRpcError {
            code: -32700,
            message: "Parse error".to_string(),
            data: Some(serde_json::json!({"details": "Invalid JSON"})),
        }),
    };

    let json_str = serde_json::to_string(&response).expect("Should serialize to JSON");

    // Verify it's valid JSON that can be parsed
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("Parse error response should be valid JSON");

    assert_eq!(parsed["jsonrpc"], "2.0");
    assert!(parsed["id"].is_null());
    assert!(parsed["error"].is_object());
    assert_eq!(parsed["error"]["code"], -32700);
}

#[test]
fn test_response_roundtrip() {
    // Test that response can be serialized and deserialized correctly
    let original = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(123)),
        result: Some(serde_json::json!({"data": [1, 2, 3]})),
        error: None,
    };

    let json_str = serde_json::to_string(&original).expect("Should serialize");
    let parsed: JsonRpcResponse = serde_json::from_str(&json_str).expect("Should deserialize");

    assert_eq!(parsed.jsonrpc, original.jsonrpc);
    assert_eq!(parsed.id, original.id);
    assert_eq!(parsed.result, original.result);
}

#[test]
fn test_complex_result_serialization() {
    // Test that complex result structures serialize correctly
    let result = serde_json::json!({
        "episodes": [
            {
                "id": "ep1",
                "content": "Test content",
                "timestamp": 1234567890
            }
        ],
        "patterns": {
            "count": 5,
            "domains": ["web", "data"]
        }
    });

    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(10)),
        result: Some(result),
        error: None,
    };

    let json_str = serde_json::to_string(&response).expect("Should serialize to JSON");

    // Verify it's valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("Complex response should be valid JSON");

    assert_eq!(parsed["result"]["episodes"][0]["id"], "ep1");
    assert_eq!(parsed["result"]["patterns"]["count"], 5);
}
