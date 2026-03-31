//! Fuzz and property-based tests for MCP JSON-RPC message handling (ACT-032)
//!
//! Tests robustness of `read_next_message` against malformed input and verifies
//! JSON-RPC response construction invariants via proptest.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use std::io::Cursor;

use do_memory_mcp::jsonrpc::{JsonRpcError, JsonRpcResponse, read_next_message};
use proptest::prelude::*;
use serde_json::json;

// ============================================================================
// Malformed JSON-RPC Messages
// ============================================================================

#[test]
fn empty_input_returns_none() {
    let mut cursor = Cursor::new(Vec::<u8>::new());
    let result = read_next_message(&mut cursor).unwrap();
    assert!(result.is_none());
}

#[test]
fn truncated_json_is_returned_as_raw_line() {
    // Partial JSON starting with '{' is returned as-is (line-delimited mode)
    let input = b"{\"jsonrpc\":\"2.0\",\"id\":1\n";
    let mut cursor = Cursor::new(input.to_vec());
    let result = read_next_message(&mut cursor).unwrap();
    assert!(result.is_some());
    let (msg, is_lsp) = result.unwrap();
    assert!(!is_lsp);
    // The raw truncated JSON is still returned — caller is responsible for parsing
    assert!(msg.starts_with('{'));
}

#[test]
fn missing_jsonrpc_field_still_readable() {
    let input = b"{\"id\":1,\"method\":\"test\"}\n";
    let mut cursor = Cursor::new(input.to_vec());
    let (msg, _) = read_next_message(&mut cursor).unwrap().unwrap();
    // Can be parsed as JsonRpcRequest (jsonrpc is Option)
    let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
    assert!(parsed.get("jsonrpc").is_none());
}

#[test]
fn invalid_jsonrpc_version_still_readable() {
    let input = b"{\"jsonrpc\":\"1.0\",\"id\":1,\"method\":\"test\"}\n";
    let mut cursor = Cursor::new(input.to_vec());
    let (msg, _) = read_next_message(&mut cursor).unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
    assert_eq!(parsed["jsonrpc"], "1.0");
}

#[test]
fn content_length_larger_than_body_returns_error() {
    // Content-Length says 1000 but body is only 10 bytes
    let header = "Content-Length: 1000\r\n\r\n";
    let body = "0123456789";
    let mut data = header.as_bytes().to_vec();
    data.extend_from_slice(body.as_bytes());
    let mut cursor = Cursor::new(data);
    // read_exact should fail with UnexpectedEof
    let result = read_next_message(&mut cursor);
    assert!(result.is_err());
}

#[test]
fn binary_non_utf8_in_content_length_body() {
    let body: &[u8] = &[0xFF, 0xFE, 0xFD, 0xFC, 0xFB];
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    let mut data = header.into_bytes();
    data.extend_from_slice(body);
    let mut cursor = Cursor::new(data);
    // from_utf8_lossy is used, so this should succeed with replacement chars
    let result = read_next_message(&mut cursor).unwrap();
    assert!(result.is_some());
    let (msg, is_lsp) = result.unwrap();
    assert!(is_lsp);
    assert!(msg.contains('\u{FFFD}'));
}

#[test]
fn null_bytes_in_line_json() {
    let input = b"{\"method\":\"\x00test\"}\n";
    let mut cursor = Cursor::new(input.to_vec());
    let result = read_next_message(&mut cursor).unwrap();
    assert!(result.is_some());
    let (msg, _) = result.unwrap();
    assert!(msg.contains('\0'));
}

#[test]
fn deeply_nested_json_object() {
    // Build a deeply nested JSON: {"a":{"a":{"a":...}}}
    let depth = 100;
    let mut json_str = String::new();
    for _ in 0..depth {
        json_str.push_str("{\"a\":");
    }
    json_str.push_str("null");
    for _ in 0..depth {
        json_str.push('}');
    }
    json_str.push('\n');
    let mut cursor = Cursor::new(json_str.into_bytes());
    let result = read_next_message(&mut cursor).unwrap();
    assert!(result.is_some());
}

// ============================================================================
// Content-Length Header Fuzzing
// ============================================================================

#[test]
fn content_length_zero_skips_to_next() {
    // Content-Length: 0 should be skipped, then read the next line
    let input = "Content-Length: 0\r\n\r\n{\"id\":1,\"method\":\"ping\"}\n";
    let mut cursor = Cursor::new(input.as_bytes().to_vec());
    let result = read_next_message(&mut cursor).unwrap();
    assert!(result.is_some());
    let (msg, is_lsp) = result.unwrap();
    assert!(!is_lsp); // The JSON line is read as line-delimited, not LSP
    assert!(msg.contains("\"method\":\"ping\""));
}

#[test]
fn content_length_negative_treated_as_zero() {
    // Negative value should parse to error -> unwrap_or(0) -> skipped
    let input = "Content-Length: -5\r\n\r\n{\"id\":1,\"method\":\"ping\"}\n";
    let mut cursor = Cursor::new(input.as_bytes().to_vec());
    let result = read_next_message(&mut cursor).unwrap();
    assert!(result.is_some());
    let (msg, _) = result.unwrap();
    assert!(msg.contains("\"method\":\"ping\""));
}

#[test]
fn content_length_much_larger_than_body() {
    let payload = "{\"id\":1}";
    let header = format!("Content-Length: {}\r\n\r\n", payload.len() + 10000);
    let mut data = header.into_bytes();
    data.extend_from_slice(payload.as_bytes());
    let mut cursor = Cursor::new(data);
    let result = read_next_message(&mut cursor);
    assert!(result.is_err());
}

#[test]
fn missing_crlf_separator_after_header() {
    // No blank line between header and body — body line starts with '{' eventually
    let input = "Content-Length: 10\r\n{\"id\":1}\n";
    let mut cursor = Cursor::new(input.as_bytes().to_vec());
    // The '{' line is consumed as a header line (not empty), so it reads the body
    // after consuming that line. This tests the fallthrough behavior.
    let result = read_next_message(&mut cursor);
    // Should not panic regardless of outcome
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn multiple_content_length_headers() {
    // Second header line is consumed in the inner header loop
    let payload = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"test\"}";
    let header = format!(
        "Content-Length: {}\r\nContent-Type: application/json\r\n\r\n",
        payload.len()
    );
    let mut data = header.into_bytes();
    data.extend_from_slice(payload.as_bytes());
    let mut cursor = Cursor::new(data);
    let result = read_next_message(&mut cursor).unwrap();
    assert!(result.is_some());
    let (msg, is_lsp) = result.unwrap();
    assert!(is_lsp);
    assert_eq!(msg, payload);
}

// ============================================================================
// JSON-RPC Response Construction Invariants (proptest)
// ============================================================================

proptest! {
    #[test]
    fn response_has_result_xor_error(
        id in 1i64..10000i64,
        is_success in proptest::bool::ANY,
        code in -32700i32..-32000i32,
        message in "[a-zA-Z ]{5,50}",
    ) {
        let response = if is_success {
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: Some(json!(id)),
                result: Some(json!({"ok": true})),
                error: None,
            }
        } else {
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: Some(json!(id)),
                result: None,
                error: Some(JsonRpcError {
                    code,
                    message,
                    data: None,
                }),
            }
        };

        let json_str = serde_json::to_string(&response).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        // "error" key is absent from JSON when None (skip_serializing_if)
        let has_result = parsed.get("result").is_some_and(|v| !v.is_null());
        let has_error = parsed.get("error").is_some();
        prop_assert!(has_result ^ has_error, "Must have result XOR error");
    }

    #[test]
    fn id_roundtrips_for_various_types(
        id_type in 0u8..3u8,
        num_id in 0i64..100000i64,
        str_id in "[a-zA-Z0-9-]{1,30}",
    ) {
        let id_value = match id_type {
            0 => json!(num_id),
            1 => json!(str_id),
            // Note: json!(null) in Option<Value> serializes to null,
            // but deserializes back to None (not Some(Null)) due to serde's
            // handling of Option<Value>. This is expected behavior.
            _ => json!(null),
        };

        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(id_value.clone()),
            result: Some(json!("ok")),
            error: None,
        };

        let json_str = serde_json::to_string(&response).unwrap();
        let deserialized: JsonRpcResponse = serde_json::from_str(&json_str).unwrap();

        // For null values, serde deserializes Option<Value> null to None
        if id_type >= 2 {
            prop_assert!(deserialized.id.is_none() || deserialized.id == Some(json!(null)));
        } else {
            prop_assert_eq!(deserialized.id, Some(id_value));
        }
    }

    #[test]
    fn error_code_preserved_exactly(
        code in prop::num::i32::ANY,
        message in "[a-zA-Z0-9 ]{3,40}",
    ) {
        let error = JsonRpcError {
            code,
            message,
            data: None,
        };

        let json_str = serde_json::to_string(&error).unwrap();
        let deserialized: JsonRpcError = serde_json::from_str(&json_str).unwrap();

        prop_assert_eq!(deserialized.code, code);
    }

    #[test]
    fn jsonrpc_field_always_2_0(
        id in 1i64..10000i64,
    ) {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(id)),
            result: Some(json!(null)),
            error: None,
        };

        let json_str = serde_json::to_string(&response).unwrap();
        let deserialized: JsonRpcResponse = serde_json::from_str(&json_str).unwrap();

        prop_assert_eq!(deserialized.jsonrpc, "2.0");
    }
}

// ============================================================================
// Boundary Value Tests
// ============================================================================

#[test]
fn empty_method_string() {
    let input = b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"\"}\n";
    let mut cursor = Cursor::new(input.to_vec());
    let (msg, _) = read_next_message(&mut cursor).unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
    assert_eq!(parsed["method"], "");
}

#[test]
fn very_long_method_string() {
    let long_method = "a".repeat(2000);
    let input = format!("{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"{long_method}\"}}\n");
    let mut cursor = Cursor::new(input.into_bytes());
    let (msg, _) = read_next_message(&mut cursor).unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
    assert_eq!(parsed["method"].as_str().unwrap().len(), 2000);
}

#[test]
fn special_characters_in_method_name() {
    let methods = [
        "foo/bar",
        "foo.bar.baz",
        "método_ñ",
        "日本語メソッド",
        "method with spaces",
        "method\twith\ttabs",
    ];
    for method in methods {
        let input = json!({"jsonrpc": "2.0", "id": 1, "method": method});
        let line = format!("{input}\n");
        let mut cursor = Cursor::new(line.into_bytes());
        let (msg, _) = read_next_message(&mut cursor).unwrap().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(parsed["method"].as_str().unwrap(), method);
    }
}

#[test]
fn integer_overflow_in_id_field() {
    let large_id = i64::MAX;
    let input = format!("{{\"jsonrpc\":\"2.0\",\"id\":{large_id},\"method\":\"test\"}}\n");
    let mut cursor = Cursor::new(input.into_bytes());
    let (msg, _) = read_next_message(&mut cursor).unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
    assert_eq!(parsed["id"].as_i64().unwrap(), i64::MAX);
}

#[test]
fn whitespace_only_lines_are_skipped() {
    let input = b"   \n\t\n  \n{\"id\":1,\"method\":\"test\"}\n";
    let mut cursor = Cursor::new(input.to_vec());
    let result = read_next_message(&mut cursor).unwrap();
    assert!(result.is_some());
    let (msg, _) = result.unwrap();
    assert!(msg.contains("\"method\":\"test\""));
}
