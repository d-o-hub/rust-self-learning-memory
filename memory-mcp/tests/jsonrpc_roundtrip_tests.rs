use memory_mcp::jsonrpc::{JsonRpcResponse, read_next_message, write_response_with_length};
use serde_json::json;
use std::io::Cursor;

#[test]
fn test_roundtrip_preserves_content_length_style() {
    // Prepare a Content-Length framed request
    let payload = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\"}";
    let header = format!("Content-Length: {}\r\n\r\n", payload.len());
    let mut data = header.into_bytes();
    data.extend_from_slice(payload.as_bytes());

    // Read request
    let mut cursor = Cursor::new(data);
    let (msg, is_lsp) = read_next_message(&mut cursor).unwrap().expect("message");
    assert!(is_lsp, "Input should be detected as LSP framed");
    assert_eq!(msg, payload);

    // Write response with the same framing
    let response = JsonRpcResponse {
        jsonrpc: "2.0".into(),
        id: Some(json!(1)),
        result: Some(json!({"ok":true})),
        error: None,
    };
    let resp_json = serde_json::to_string(&response).unwrap();
    let mut out: Vec<u8> = Vec::new();
    write_response_with_length(&mut out, &resp_json).unwrap();

    let out_str = String::from_utf8(out).unwrap();
    assert!(out_str.to_ascii_lowercase().starts_with("content-length:"));
    let parts: Vec<&str> = out_str.splitn(2, "\r\n\r\n").collect();
    assert_eq!(parts.len(), 2);
    let header_len: usize = parts[0].split(':').nth(1).unwrap().trim().parse().unwrap();
    assert_eq!(parts[1].trim().len(), header_len);
}
