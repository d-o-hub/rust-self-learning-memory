#[tokio::test]
async fn test_initialize_protocol_version_negotiation() {
    use memory_mcp::jsonrpc::JsonRpcRequest;
    use memory_mcp::protocol::OAuthConfig;
    use memory_mcp::protocol::handle_initialize;
    use serde_json::json;

    let oauth_config = OAuthConfig::default();

    // Test 1: Client requests latest version (2025-11-25)
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

    let resp = handle_initialize(req_latest, &oauth_config).await;
    assert!(resp.is_some());
    let resp = resp.unwrap();
    assert!(resp.result.is_some());

    let result = resp.result.unwrap();
    let protocol_version = result.get("protocolVersion").and_then(|v| v.as_str());
    assert_eq!(protocol_version, Some("2025-11-25"));

    // Test 2: Client requests older version (2024-11-05)
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

    let resp = handle_initialize(req_old, &oauth_config).await;
    assert!(resp.is_some());
    let resp = resp.unwrap();
    assert!(resp.result.is_some());

    let result = resp.result.unwrap();
    let protocol_version = result.get("protocolVersion").and_then(|v| v.as_str());
    assert_eq!(protocol_version, Some("2024-11-05"));

    // Test 3: Client requests unsupported version (should return latest)
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

    let resp = handle_initialize(req_unsupported, &oauth_config).await;
    assert!(resp.is_some());
    let resp = resp.unwrap();
    assert!(resp.result.is_some());

    let result = resp.result.unwrap();
    let protocol_version = result.get("protocolVersion").and_then(|v| v.as_str());
    // Should return latest supported version
    assert_eq!(protocol_version, Some("2025-11-25"));

    // Test 4: Client doesn't specify version (should return latest)
    let req_no_version = JsonRpcRequest {
        jsonrpc: Some("2.0".to_string()),
        id: Some(json!(4)),
        method: "initialize".into(),
        params: Some(json!({
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        })),
    };

    let resp = handle_initialize(req_no_version, &oauth_config).await;
    assert!(resp.is_some());
    let resp = resp.unwrap();
    assert!(resp.result.is_some());

    let result = resp.result.unwrap();
    let protocol_version = result.get("protocolVersion").and_then(|v| v.as_str());
    assert_eq!(protocol_version, Some("2025-11-25"));

    println!("All protocol version negotiation tests passed!");
}
