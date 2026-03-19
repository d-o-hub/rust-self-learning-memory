//! Decompression logic for CompressedTransport.
//!
//! Extracted from `wrapper.rs` to maintain the ≤500 LOC invariant.

use super::CompressedTransport;
use crate::transport::compression::TransportCompressionError;
use anyhow::Result;

impl CompressedTransport {
    /// Decompress zstd data
    pub(super) async fn decompress_zstd(&self, data: &[u8]) -> Result<Vec<u8>> {
        #[cfg(feature = "compression-zstd")]
        {
            // Use spawn_blocking for CPU-intensive decompression
            let data = data.to_vec();
            tokio::task::spawn_blocking(move || {
                zstd::stream::decode_all(&data[..]).map_err(|e| {
                    anyhow::anyhow!(TransportCompressionError::DecompressionFailed(
                        e.to_string()
                    ))
                })
            })
            .await
            .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?
        }

        #[cfg(not(feature = "compression-zstd"))]
        {
            let _ = data;
            Err(anyhow::anyhow!(
                "Zstd decompression not available (enable compression-zstd feature)"
            ))
        }
    }

    /// Decompress LZ4 data
    pub(super) async fn decompress_lz4(&self, data: &[u8]) -> Result<Vec<u8>> {
        #[cfg(feature = "compression-lz4")]
        {
            // LZ4 decompression is fast, but we still use spawn_blocking for consistency
            let data = data.to_vec();
            // Lock guard is dropped before await (scope block)
            let original_size = {
                let stats = self.stats.lock().map_err(|e| {
                    anyhow::anyhow!(TransportCompressionError::DecompressionFailed(format!(
                        "Failed to acquire stats lock: {}",
                        e
                    )))
                })?;
                stats.base.total_original_bytes as usize
            }; // Lock released here

            tokio::task::spawn_blocking(move || {
                lz4_flex::decompress(&data, original_size).map_err(|e| {
                    anyhow::anyhow!(TransportCompressionError::DecompressionFailed(
                        e.to_string()
                    ))
                })
            })
            .await
            .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?
        }

        #[cfg(not(feature = "compression-lz4"))]
        {
            let _ = data;
            Err(anyhow::anyhow!(
                "LZ4 decompression not available (enable compression-lz4 feature)"
            ))
        }
    }

    /// Decompress gzip data
    pub(super) async fn decompress_gzip(&self, data: &[u8]) -> Result<Vec<u8>> {
        #[cfg(feature = "compression-gzip")]
        {
            let data = data.to_vec();
            tokio::task::spawn_blocking(move || {
                use flate2::bufread::GzDecoder;
                use std::io::Read;

                let mut decoder = GzDecoder::new(&data[..]);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed).map_err(|e| {
                    anyhow::anyhow!(TransportCompressionError::DecompressionFailed(
                        e.to_string()
                    ))
                })?;
                Ok(decompressed)
            })
            .await
            .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?
        }

        #[cfg(not(feature = "compression-gzip"))]
        {
            let _ = data;
            Err(anyhow::anyhow!(
                "Gzip decompression not available (enable compression-gzip feature)"
            ))
        }
    }
}
