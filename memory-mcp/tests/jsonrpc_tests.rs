use std::io::Cursor;

use memory_mcp::jsonrpc::read_next_message;

#[test]
fn test_read_next_message_line_json() {
    let input = b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\"}\n";
    let mut cursor = Cursor::new(input.to_vec());
    let out = read_next_message(&mut cursor).unwrap();
    assert!(out.is_some());
    let (msg, is_lsp) = out.unwrap();
    assert!(!is_lsp);
    assert!(msg.contains("\"method\":\"initialize\""));
}

#[test]
fn test_read_next_message_content_length() {
    let payload = "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"initialize\"}";
    let header = format!("Content-Length: {}\r\n\r\n", payload.len());
    let mut data = header.into_bytes();
    data.extend_from_slice(payload.as_bytes());
    let mut cursor = Cursor::new(data);
    let out = read_next_message(&mut cursor).unwrap();
    assert!(out.is_some());
    let (msg, is_lsp) = out.unwrap();
    assert!(is_lsp);
    assert_eq!(msg, payload);
}

#[test]
fn test_read_next_message_skips_garbage() {
    let input =
        b"Some log line\nAnother: line\n{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"initialize\"}\n";
    let mut cursor = Cursor::new(input.to_vec());
    let out = read_next_message(&mut cursor).unwrap();
    assert!(out.is_some());
    let (msg, is_lsp) = out.unwrap();
    assert!(!is_lsp);
    assert!(msg.contains("\"method\":\"initialize\""));
}
