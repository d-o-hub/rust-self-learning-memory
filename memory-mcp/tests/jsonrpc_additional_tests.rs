use memory_mcp::jsonrpc::{JsonRpcResponse, read_next_message, write_response_with_length};
use serde_json::json;
use std::io::Cursor;

#[test]
fn test_parse_failure_returns_none_and_does_not_panic() {
    // Malformed JSON line should be ignored; next valid message should be returned
    let input = b"not-json\n{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\"}\n";
    let mut cursor = Cursor::new(input.to_vec());
    let out = read_next_message(&mut cursor).unwrap();
    assert!(out.is_some());
    let (msg, is_lsp) = out.unwrap();
    assert!(!is_lsp);
    assert!(msg.contains("\"method\":\"initialize\""));
}

#[test]
fn test_write_response_with_content_length() {
    let mut buf: Vec<u8> = Vec::new();
    let body = serde_json::to_string(&JsonRpcResponse {
        jsonrpc: "2.0".into(),
        id: Some(json!(1)),
        result: Some(json!({"ok":true})),
        error: None,
    })
    .unwrap();
    write_response_with_length(&mut buf, &body).unwrap();

    // Validate header
    let s = String::from_utf8(buf.clone()).unwrap();
    let parts: Vec<&str> = s.splitn(2, "\r\n\r\n").collect();
    assert!(parts[0].to_ascii_lowercase().starts_with("content-length:"));

    // Validate payload length matches header
    let header_len: usize = parts[0].split(':').nth(1).unwrap().trim().parse().unwrap();
    let payload = parts[1].trim();
    assert_eq!(payload.len(), header_len);
}
