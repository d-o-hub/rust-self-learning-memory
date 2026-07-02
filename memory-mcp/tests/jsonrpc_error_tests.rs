//! Unit tests for JSON-RPC header error handling

use do_memory_mcp::jsonrpc::read_next_message;
use std::io::{self, Cursor};

#[test]
fn test_read_next_message_malformed_content_length_abc() {
    let payload = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"test\"}";
    let header = "Content-Length: abc\r\n\r\n";
    let mut data = header.as_bytes().to_vec();
    data.extend_from_slice(payload.as_bytes());
    let mut cursor = Cursor::new(data);

    let res = read_next_message(&mut cursor);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    assert!(err.to_string().contains("Malformed Content-Length"));
}

#[test]
fn test_read_next_message_malformed_content_length_negative() {
    let payload = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"test\"}";
    let header = "Content-Length: -1\r\n\r\n";
    let mut data = header.as_bytes().to_vec();
    data.extend_from_slice(payload.as_bytes());
    let mut cursor = Cursor::new(data);

    let res = read_next_message(&mut cursor);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    assert!(err.to_string().contains("Malformed Content-Length"));
}

#[test]
fn test_read_next_message_zero_content_length() {
    let header = "Content-Length: 0\r\n\r\n";
    let mut cursor = Cursor::new(header.as_bytes().to_vec());

    let res = read_next_message(&mut cursor);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    assert!(
        err.to_string()
            .contains("Content-Length header value is zero")
    );
}

#[test]
fn test_read_next_message_with_content_type() {
    let payload = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"test\"}";
    let header = format!(
        "Content-Length: {}\r\nContent-Type: application/vscode-jsonrpc; charset=utf-8\r\n\r\n",
        payload.len()
    );
    let mut data = header.into_bytes();
    data.extend_from_slice(payload.as_bytes());
    let mut cursor = Cursor::new(data);

    let out = read_next_message(&mut cursor).unwrap();
    assert!(out.is_some());
    let (msg, is_lsp) = out.unwrap();
    assert!(is_lsp);
    assert_eq!(msg, payload);
}
