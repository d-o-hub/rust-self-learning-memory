//! # Transport Layer
//!
//! Network transport utilities for Turso storage including:
//! - Compression for network bandwidth optimization
//! - Async I/O utilities
//! - Connection handling
//!
//! ## Components
//!
//! - `compression`: Async compression for transport operations

#[cfg(feature = "compression")]
pub mod compression;

#[cfg(feature = "compression")]
pub use compression::{
    compress_transport, decompress_transport, AsyncCompressor, CompressionStreamResult,
    TransportCompressionConfig, TransportCompressionError, TransportCompressionStats,
};
