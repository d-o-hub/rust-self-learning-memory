//! # Episode Compression Helpers
//!
//! Helpers for compressing and decompressing JSON fields in episodes.

/// Compress JSON data if compression is enabled and data is large enough
#[cfg(feature = "compression")]
pub fn compress_json_field(data: &[u8], threshold: usize) -> memory_core::Result<Vec<u8>> {
    use crate::compression::CompressedPayload;

    let compressed = CompressedPayload::compress(data, threshold)?;
    if compressed.algorithm == crate::CompressionAlgorithm::None {
        // No compression applied, return original data
        Ok(data.to_vec())
    } else {
        // Store as base64-encoded compressed data with algorithm prefix
        let payload = format!(
            "__compressed__:{}:{}\n{}",
            compressed.algorithm,
            compressed.original_size,
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &compressed.data)
        );
        Ok(payload.into_bytes())
    }
}

/// Compress JSON data (no-op when compression feature is disabled)
#[cfg(not(feature = "compression"))]
pub fn compress_json_field(data: &[u8], _threshold: usize) -> memory_core::Result<Vec<u8>> {
    Ok(data.to_vec())
}

/// Decompress JSON data if it's compressed
#[cfg(feature = "compression")]
pub fn decompress_json_field(data: &str) -> memory_core::Result<Vec<u8>> {
    use memory_core::Error;
    if let Some(remainder) = data.strip_prefix("__compressed__:") {
        // Parse the compressed format: __compressed__:<algorithm>:<original_size>\n<base64_data>
        let newline_pos = remainder.find('\n').ok_or_else(|| {
            Error::Storage("Invalid compressed data format: missing newline".to_string())
        })?;
        let header = &remainder[..newline_pos];
        let encoded_data = &remainder[newline_pos + 1..];

        // Parse header: <algorithm>:<original_size>
        let colon_pos = header
            .find(':')
            .ok_or_else(|| Error::Storage("Invalid compressed header format".to_string()))?;
        let algorithm_str = &header[..colon_pos];
        let original_size: usize = header[colon_pos + 1..].parse().map_err(|_| {
            Error::Storage("Invalid original size in compressed header".to_string())
        })?;

        let algorithm = match algorithm_str {
            "lz4" => crate::CompressionAlgorithm::Lz4,
            "zstd" => crate::CompressionAlgorithm::Zstd,
            "gzip" => crate::CompressionAlgorithm::Gzip,
            _ => {
                return Err(Error::Storage(format!(
                    "Unknown compression algorithm: {}",
                    algorithm_str
                )));
            }
        };

        let compressed_data =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded_data)
                .map_err(|e| {
                    Error::Storage(format!("Failed to decode base64 compressed data: {}", e))
                })?;

        let payload = crate::CompressedPayload {
            original_size,
            compressed_size: compressed_data.len(),
            compression_ratio: compressed_data.len() as f64 / original_size as f64,
            data: compressed_data,
            algorithm,
        };

        payload.decompress()
    } else {
        // Not compressed, return as-is
        Ok(data.as_bytes().to_vec())
    }
}

/// Decompress JSON data (no-op when compression feature is disabled)
#[cfg(not(feature = "compression"))]
pub fn decompress_json_field(data: &str) -> memory_core::Result<Vec<u8>> {
    Ok(data.as_bytes().to_vec())
}
