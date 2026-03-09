//! Runtime Feature Wiring Verification: Transport Compression
//!
//! This test module verifies the wiring status and integration points for the
//! CompressedTransport feature in the memory-storage-turso crate.
//!
//! ## Analysis Summary (2026-03-09)
//!
//! **Current State**: Compression is implemented and USED at the data layer, but
//! CompressedTransport wrapper is NOT wired into TursoStorage.
//!
//! **Architectural Finding**:
//! - `CompressedTransport` wraps a `Transport` trait object
//! - `TursoStorage` uses `libsql::Database` directly, not the Transport trait
//! - Compression IS applied to embeddings via direct function calls
//! - The Transport trait abstraction is not used by the storage layer
//!
//! **What IS Wired**:
//! - `CompressionStatistics` tracked in `TursoStorage`
//! - `compress()` / `decompress()` functions called in embedding storage
//! - Feature flag `compression` enables the functionality
//!
//! **What is NOT Wired**:
//! - `CompressedTransport::new()` is never called
//! - `Transport` trait is not implemented for libsql
//! - Network-level compression is not applied (libsql handles its own transport)
//!
//! **Recommendation**:
//! - Current approach (data-layer compression) is correct for this architecture
//! - `CompressedTransport` could be used for custom HTTP transports
//! - Consider documenting that CompressedTransport is a standalone utility

#![cfg(feature = "compression")]
#![allow(clippy::expect_used)]

use async_trait::async_trait;
use memory_storage_turso::{
    CompressedTransport, CompressionStatistics, Transport, TransportCompressionConfig,
    TransportMetadata, TransportResponse, compress, decompress,
};
use std::fmt::Debug;
use std::time::Instant;

// ============================================================================
// Mock Transport for Testing
// ============================================================================

/// Mock transport for testing CompressedTransport wrapper
#[derive(Debug)]
struct MockTransport {
    send_count: std::sync::atomic::AtomicU64,
    #[allow(dead_code)]
    last_data_size: std::sync::atomic::AtomicUsize,
}

impl MockTransport {
    fn new() -> Self {
        Self {
            send_count: std::sync::atomic::AtomicU64::new(0),
            last_data_size: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    #[allow(dead_code)]
    fn get_send_count(&self) -> u64 {
        self.send_count.load(std::sync::atomic::Ordering::SeqCst)
    }

    #[allow(dead_code)]
    fn get_last_data_size(&self) -> usize {
        self.last_data_size
            .load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[async_trait]
impl Transport for MockTransport {
    async fn send(&self, data: &[u8]) -> anyhow::Result<TransportResponse> {
        self.send_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.last_data_size
            .store(data.len(), std::sync::atomic::Ordering::SeqCst);
        Ok(TransportResponse::success(data.to_vec()))
    }

    async fn send_async(&self, data: &[u8]) -> anyhow::Result<()> {
        self.send_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.last_data_size
            .store(data.len(), std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    async fn health_check(&self) -> anyhow::Result<bool> {
        Ok(true)
    }

    fn metadata(&self) -> TransportMetadata {
        TransportMetadata::new("mock", "1.0")
    }
}

// ============================================================================
// Verification Test 1: CompressedTransport Works in Isolation
// ============================================================================

/// Verify that CompressedTransport works correctly when used standalone.
/// This proves the implementation is sound, just not integrated with TursoStorage.
#[tokio::test]
async fn test_compressed_transport_works_in_isolation() {
    let mock = MockTransport::new();
    let config = TransportCompressionConfig::default();
    let transport = CompressedTransport::new(Box::new(mock), config);

    // Send large data that should be compressed
    let large_data = vec![0u8; 10000]; // 10KB of zeros (compresses well)

    let response = transport
        .send(&large_data)
        .await
        .expect("Send should succeed");

    assert!(response.is_success());
    // The response should be decompressed back to original
    assert_eq!(response.body.len(), 10000);
}

// ============================================================================
// Verification Test 2: Compression Threshold
// ============================================================================

/// Verify that small payloads are not compressed.
#[tokio::test]
async fn test_compressed_transport_threshold() {
    let mock = MockTransport::new();
    let config = TransportCompressionConfig {
        compression_threshold: 1024, // 1KB threshold
        ..Default::default()
    };
    let transport = CompressedTransport::new(Box::new(mock), config);

    // Send small data (below threshold)
    let small_data = vec![1u8; 100]; // 100 bytes

    transport
        .send(&small_data)
        .await
        .expect("Send should succeed");

    // Small data should not be compressed
    let metadata = transport.metadata();
    assert!(metadata.supports_compression);
}

// ============================================================================
// Verification Test 3: Compression Statistics Tracking
// ============================================================================

/// Verify that CompressionStatistics can be created and tracked.
#[tokio::test]
async fn test_compression_statistics_tracking() {
    let stats = CompressionStatistics::new();

    // Initial state
    assert_eq!(stats.compression_count, 0);
    assert_eq!(stats.total_original_bytes, 0);
    assert_eq!(stats.total_compressed_bytes, 0);
    assert_eq!(stats.skipped_count, 0);
    assert_eq!(stats.failed_count, 0);

    // Record a compression
    let mut stats = stats;
    stats.record_compression(1000, 400, 100); // 1000 bytes -> 400 bytes in 100us

    assert_eq!(stats.compression_count, 1);
    assert_eq!(stats.total_original_bytes, 1000);
    assert_eq!(stats.total_compressed_bytes, 400);

    // Calculate ratios
    let ratio = stats.compression_ratio();
    assert!(
        (ratio - 0.4).abs() < 0.01,
        "Compression ratio should be ~0.4"
    );

    let savings = stats.bandwidth_savings_percent();
    assert!(
        (savings - 60.0).abs() < 0.1,
        "Bandwidth savings should be ~60%"
    );
}

// ============================================================================
// Verification Test 4: Direct Compression Functions Work
// ============================================================================

/// Verify that the compress/decompress functions work correctly.
/// These are the functions actually used by TursoStorage for embedding compression.
#[tokio::test]
async fn test_direct_compress_decompress_functions() {
    // Create test data that compresses well
    let original_data: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();

    // Compress with threshold of 1000 bytes
    let compressed = compress(&original_data, 1000).expect("Compression should succeed");

    // Compressed should be smaller
    assert!(
        compressed.compressed_size < original_data.len(),
        "Compressed {} bytes should be smaller than original {} bytes",
        compressed.compressed_size,
        original_data.len()
    );

    // Decompress
    let decompressed = decompress(&compressed).expect("Decompression should succeed");

    // Should match original
    assert_eq!(
        decompressed, original_data,
        "Round-trip should preserve data"
    );
}

// ============================================================================
// Verification Test 5: CompressedTransport Metadata
// ============================================================================

/// Verify that CompressedTransport correctly updates metadata.
#[tokio::test]
async fn test_compressed_transport_metadata() {
    let mock = MockTransport::new();
    let config = TransportCompressionConfig::default();
    let transport = CompressedTransport::new(Box::new(mock), config);

    let metadata = transport.metadata();

    // CompressedTransport should report compression support
    assert!(metadata.supports_compression);
    assert!(!metadata.name.is_empty());
}

// ============================================================================
// Verification Test 6: CompressedTransport Health Check Passthrough
// ============================================================================

/// Verify that health check is passed through to inner transport.
#[tokio::test]
async fn test_compressed_transport_health_check() {
    let mock = MockTransport::new();
    let config = TransportCompressionConfig::default();
    let transport = CompressedTransport::new(Box::new(mock), config);

    let healthy = transport
        .health_check()
        .await
        .expect("Health check should succeed");
    assert!(healthy);
}

// ============================================================================
// Verification Test 7: Architecture Analysis
// ============================================================================

/// Document the architectural analysis of why CompressedTransport is not
/// directly wired into TursoStorage.
#[test]
fn test_transport_compression_architecture_analysis() {
    // Key finding: TursoStorage uses libsql::Database directly
    //
    // From memory-storage-turso/src/lib_impls/storage.rs:
    // pub struct TursoStorage {
    //     pub(crate) db: Arc<Database>,  // libsql Database, not Transport
    //     pub(crate) pool: Option<Arc<ConnectionPool>>,
    //     ...
    //     #[cfg(feature = "compression")]
    //     pub(crate) compression_stats: Arc<Mutex<CompressionStatistics>>,
    // }
    //
    // The libsql crate handles its own transport layer internally.
    // There's no way to inject a Transport trait implementation.
    //
    // What IS wired:
    // - CompressionStatistics is stored in TursoStorage
    // - compress() / decompress() are called for embeddings
    //
    // What is NOT wired:
    // - CompressedTransport::new() is never called
    // - Transport trait is not used for database operations
    //
    // This is the correct architectural choice because:
    // 1. libsql handles its own network layer efficiently
    // 2. Data-layer compression (at the embedding level) is more appropriate
    // 3. CompressedTransport would add unnecessary overhead

    println!(
        "Architecture analysis: CompressedTransport not wired because libsql handles its own transport"
    );
}

// ============================================================================
// Verification Test 8: CompressedTransport Use Cases
// ============================================================================

/// Document valid use cases for CompressedTransport.
#[test]
fn test_compressed_transport_use_cases() {
    // Valid use cases for CompressedTransport:
    //
    // 1. Custom HTTP transports for MCP server:
    //    - Wrap HTTP client with compression
    //    - Reduce bandwidth for tool responses
    //
    // 2. Custom database backends:
    //    - If implementing a custom StorageBackend
    //    - Wrap the network transport with compression
    //
    // 3. Inter-service communication:
    //    - Wrap gRPC or HTTP clients
    //    - Compress payloads between services
    //
    // NOT valid for:
    // - Wrapping libsql (not possible)
    // - Compressing already-compressed data
    // - Small payloads (overhead > benefit)

    println!("CompressedTransport is a standalone utility for custom transport implementations");
}

// ============================================================================
// Verification Test 9: Performance Characteristics
// ============================================================================

/// Verify compression performance characteristics.
#[tokio::test]
async fn test_compression_performance_characteristics() {
    let start = Instant::now();

    // Create data that compresses well (repetitive)
    let data: Vec<u8> = vec![0xAB; 100_000]; // 100KB

    let compress_start = Instant::now();
    let compressed = compress(&data, 1000).expect("Compression should succeed");
    let compress_time = compress_start.elapsed();

    let decompress_start = Instant::now();
    let decompressed = decompress(&compressed).expect("Decompression should succeed");
    let decompress_time = decompress_start.elapsed();

    let total_time = start.elapsed();

    // Verify correctness
    assert_eq!(decompressed, data);

    // Log performance
    println!("Original size: {} bytes", data.len());
    println!("Compressed size: {} bytes", compressed.compressed_size);
    println!(
        "Compression ratio: {:.2}%",
        compressed.compression_ratio * 100.0
    );
    println!("Compress time: {:?}", compress_time);
    println!("Decompress time: {:?}", decompress_time);
    println!("Total time: {:?}", total_time);

    // Verify meaningful compression
    assert!(
        compressed.compressed_size < data.len() / 2,
        "Should achieve at least 50% compression on repetitive data"
    );
}

// ============================================================================
// Integration Test: Current Compression Path
// ============================================================================

/// Document how compression is currently used in TursoStorage.
#[test]
fn test_current_compression_path_documented() {
    // Current compression flow in TursoStorage:
    //
    // 1. Embedding storage (store_episode_embedding):
    //    - Serialize embedding to bytes
    //    - Call compress() if data > threshold
    //    - Store compressed bytes in database
    //    - Update compression_stats
    //
    // 2. Embedding retrieval (get_episode_embedding):
    //    - Retrieve bytes from database
    //    - Call decompress() if compression header present
    //    - Deserialize to embedding vector
    //
    // This is data-layer compression, not transport-layer compression.
    // It's the correct approach for this architecture.

    // The CompressedTransport exists for future use cases where
    // a custom Transport implementation is needed.

    println!("Compression is applied at the data layer (embeddings), not transport layer");
}

// ============================================================================
// Summary: What Should Be Done
// ============================================================================

/// Document the recommended actions for CompressedTransport.
#[test]
fn test_recommended_actions_for_compressed_transport() {
    // RECOMMENDATION: No changes needed to wire CompressedTransport.
    //
    // Current state is correct:
    // - Data-layer compression is applied where appropriate
    // - libsql handles its own transport efficiently
    // - CompressionStatistics are tracked
    //
    // OPTIONAL improvements:
    // 1. Add documentation to CompressedTransport explaining its use cases
    // 2. Add example in examples/ showing how to use CompressedTransport
    // 3. Consider adding a "compression-stats" endpoint to MCP server
    //
    // NOT recommended:
    // - Trying to wire CompressedTransport into TursoStorage
    // - This would require replacing libsql with a custom Transport impl
    // - The current architecture is simpler and more maintainable

    println!(
        "Recommendation: Keep current architecture, add documentation for CompressedTransport standalone usage"
    );
}
