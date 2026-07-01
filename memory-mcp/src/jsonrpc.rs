use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, BufRead, Read, Write};

/// JSON-RPC request structure
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    #[serde(default)]
    pub jsonrpc: Option<String>,
    #[serde(default)]
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

/// JSON-RPC response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    // Always include result, even when null, to satisfy strict clients
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC error structure
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Read a message from a reader supporting both line-delimited JSON and LSP Content-Length framing.
/// Returns (message, is_content_length) where is_content_length indicates whether the message
/// came in with a Content-Length header (LSP-style)
pub fn read_next_message<R: BufRead + Read>(reader: &mut R) -> io::Result<Option<(String, bool)>> {
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            // EOF
            return Ok(None);
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // If the line looks like JSON directly, return it (not LSP framed)
        if trimmed.starts_with('{') {
            return Ok(Some((trimmed.to_string(), false)));
        }

        // If it's a Content-Length header, parse it and read the payload
        let low = trimmed.to_ascii_lowercase();
        if low.starts_with("content-length:") {
            // Parse length
            let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
            let len_str = parts.get(1).map(|s| s.trim()).unwrap_or("");
            let len: usize = len_str.parse().map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Malformed Content-Length header value '{}': {}", len_str, e),
                )
            })?;

            if len == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Content-Length header value is zero",
                ));
            }

            // Consume remaining header lines until we reach an empty line
            let mut has_content_type = false;
            loop {
                let mut hline = String::new();
                let hn = reader.read_line(&mut hline)?;
                if hn == 0 {
                    break;
                }
                let htrimmed = hline.trim();
                if htrimmed.is_empty() {
                    break;
                }

                if htrimmed.to_ascii_lowercase().starts_with("content-type:") {
                    has_content_type = true;
                }
            }

            if !has_content_type {
                tracing::warn!("LSP message missing Content-Type header");
            }

            // Read exact number of bytes for the content
            let mut buf = vec![0u8; len];
            reader.read_exact(&mut buf)?;
            return Ok(Some((String::from_utf8_lossy(&buf).to_string(), true)));
        }

        // Otherwise, skip the line (e.g., logs accidentally printed to stdout) and continue
        continue;
    }
}

/// Write a JSON-RPC response using Content-Length framing to support LSP-style clients.
pub fn write_response_with_length<W: Write>(writer: &mut W, body: &str) -> io::Result<()> {
    let bytes = body.as_bytes();
    let header = format!("Content-Length: {}\r\n\r\n", bytes.len());
    writer.write_all(header.as_bytes())?;
    writer.write_all(bytes)?;
    writer.flush()?;
    Ok(())
}
