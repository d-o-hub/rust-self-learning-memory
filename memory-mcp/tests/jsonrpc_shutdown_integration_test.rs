//! End-to-end JSON-RPC `shutdown` integration test
//!
//! Locks in the behavior introduced by PR #708: Content-Length framed `shutdown`
//! requests must reach the dispatch layer and produce the contract response
//! (`{"jsonrpc":"2.0","id":<id>,"result":null}`), while notification-style
//! shutdown messages (no `id`) must return no response.
//!
//! This covers the full path:
//!   stdin bytes → `read_next_message` (Content-Length header) → `JsonRpcRequest`
//!   → `protocol::handle_shutdown` → `JsonRpcResponse` → `write_response_with_length`.
//!
//! References: PR #708 (Strict JSON-RPC Content-Length validation), issue #697.

#![allow(deprecated)] // Calls `protocol::handle_shutdown` which is `#[deprecated]`.
#![allow(missing_docs)]

use do_memory_mcp::jsonrpc::{JsonRpcRequest, read_next_message, write_response_with_length};
use do_memory_mcp::protocol::handle_shutdown;
use serde_json::{Value, json};
use std::io::Cursor;

/// The contract payload that a `shutdown` request sends.
const SHUTDOWN_REQUEST: &str = r#"{"jsonrpc":"2.0","id":42,"method":"shutdown"}"#;

/// Frame a JSON-RPC payload with a Content-Length header (LSP-style).
fn frame_with_content_length(payload: &str) -> Vec<u8> {
    let mut out = format!(
        "Content-Length: {}\r\nContent-Type: application/vscode-jsonrpc; charset=utf-8\r\n\r\n",
        payload.len()
    )
    .into_bytes();
    out.extend_from_slice(payload.as_bytes());
    out
}

#[tokio::test]
async fn shutdown_via_content_length_returns_null_result() {
    // Arrange: a Content-Length framed shutdown request
    let framed = frame_with_content_length(SHUTDOWN_REQUEST);
    let mut cursor = Cursor::new(framed);

    // Act: parse the framed bytes into a JsonRpcRequest via the real reader,
    // then dispatch through the real (deprecated) handler.
    let (raw, is_lsp) = read_next_message(&mut cursor)
        .expect("read should not fail for a well-formed frame")
        .expect("a frame was provided");
    assert!(
        is_lsp,
        "shutdown frame must be detected as LSP/Content-Length framed"
    );

    let request: JsonRpcRequest =
        serde_json::from_str(&raw).expect("frame must deserialize into a JsonRpcRequest");
    assert_eq!(request.method, "shutdown");
    assert_eq!(request.id, Some(json!(42)));

    let response = handle_shutdown(request)
        .await
        .expect("shutdown with id must produce a response");

    // Assert: response contract per the JSON-RPC shutdown spec
    assert_eq!(response.jsonrpc, "2.0");
    assert_eq!(response.id, Some(json!(42)));
    assert_eq!(
        response.result,
        Some(Value::Null),
        "shutdown result must be JSON null"
    );
    assert!(
        response.error.is_none(),
        "shutdown must not produce an error"
    );
}

#[tokio::test]
async fn shutdown_notification_returns_no_response() {
    // No `id` in the request → notification → no response (per JSON-RPC 2.0 §4.1).
    let request = JsonRpcRequest {
        jsonrpc: Some("2.0".to_string()),
        id: None,
        method: "shutdown".to_string(),
        params: None,
    };

    let response = handle_shutdown(request).await;
    assert!(
        response.is_none(),
        "notification shutdown must not produce a response"
    );
}

#[tokio::test]
async fn shutdown_response_roundtrips_via_content_length() {
    // Dispatch the real handler, then write the response back through the
    // Content-Length writer so the LSP-style client can parse it.
    let request = serde_json::from_str::<JsonRpcRequest>(SHUTDOWN_REQUEST).unwrap();
    let response = handle_shutdown(request).await.expect("response present");

    let body = serde_json::to_string(&response).expect("serialize response");
    let mut out: Vec<u8> = Vec::new();
    write_response_with_length(&mut out, &body).expect("write framing succeeds");

    let out_str = std::str::from_utf8(&out).expect("utf-8 framing");
    let lower = out_str.to_ascii_lowercase();
    assert!(
        lower.starts_with("content-length:"),
        "shutdown response must use Content-Length framing for LSP clients, got: {out_str:?}",
    );

    let split: Vec<&str> = out_str.splitn(2, "\r\n\r\n").collect();
    assert_eq!(
        split.len(),
        2,
        "must have header + body separated by blank line"
    );
    let header_len: usize = split[0]
        .split(':')
        .nth(1)
        .unwrap()
        .trim()
        .parse()
        .expect("Content-Length must be a valid usize");
    assert_eq!(
        split[1].trim().len(),
        header_len,
        "Content-Length header must exactly match body length",
    );

    // The body itself must round-trip to the contract shape.
    let parsed: Value = serde_json::from_str(split[1].trim()).expect("valid JSON body");
    assert_eq!(parsed["jsonrpc"], "2.0");
    assert_eq!(parsed["id"], json!(42));
    assert_eq!(parsed["result"], Value::Null);
    // `JsonRpcResponse::error` has `skip_serializing_if = "Option::is_none"`, so an
    // absent field is the only observable shape — never `null`.
    assert!(parsed.get("error").is_none());
}
