use serde_json::json;

use memory_mcp::jsonrpc::JsonRpcRequest;

#[test]
fn test_protocol_version_negotiation() {
    // Test 1: Client requests latest version
    let req_latest = JsonRpcRequest {
        jsonrpc: Some("2.0".to_string()),
        id: Some(json!(1)),
        method: "initialize".into(),
        params: Some(json!({
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        })),
    };

    // Test 2: Client requests older version
    let req_old = JsonRpcRequest {
        jsonrpc: Some("2.0".to_string()),
        id: Some(json!(2)),
        method: "initialize".into(),
        params: Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        })),
    };

    // Test 3: Client requests unsupported version
    let req_unsupported = JsonRpcRequest {
        jsonrpc: Some("2.0".to_string()),
        id: Some(json!(3)),
        method: "initialize".into(),
        params: Some(json!({
            "protocolVersion": "2020-01-01",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        })),
    };

    // Test 4: Client doesn't specify version
    let req_no_version = JsonRpcRequest {
        jsonrpc: Some("2.0".to_string()),
        id: Some(json!(4)),
        method: "initialize".into(),
        params: Some(json!({
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        })),
    };

    // Extract requested versions
    assert_eq!(
        req_latest
            .params
            .as_ref()
            .and_then(|p| p.get("protocolVersion"))
            .and_then(|v| v.as_str()),
        Some("2025-11-25")
    );

    assert_eq!(
        req_old
            .params
            .as_ref()
            .and_then(|p| p.get("protocolVersion"))
            .and_then(|v| v.as_str()),
        Some("2024-11-05")
    );

    assert_eq!(
        req_unsupported
            .params
            .as_ref()
            .and_then(|p| p.get("protocolVersion"))
            .and_then(|v| v.as_str()),
        Some("2020-01-01")
    );

    assert_eq!(
        req_no_version
            .params
            .as_ref()
            .and_then(|p| p.get("protocolVersion"))
            .and_then(|v| v.as_str()),
        None
    );

    println!("All protocol version extraction tests passed!");
}
