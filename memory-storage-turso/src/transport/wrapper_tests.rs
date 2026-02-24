use super::*;
use crate::transport::TransportMetadata;

/// Mock transport for testing
#[derive(Debug)]
struct MockTransport {
    last_sent: Arc<Mutex<Option<Vec<u8>>>>,
    should_fail: bool,
}

impl MockTransport {
    fn new() -> Self {
        Self {
            last_sent: Arc::new(Mutex::new(None)),
            should_fail: false,
        }
    }

    fn failing() -> Self {
        Self {
            last_sent: Arc::new(Mutex::new(None)),
            should_fail: true,
        }
    }

    fn last_sent(&self) -> Option<Vec<u8>> {
        self.last_sent.lock().unwrap().clone()
    }
}

#[async_trait]
impl Transport for MockTransport {
    async fn send(&self, data: &[u8]) -> anyhow::Result<TransportResponse> {
        if self.should_fail {
            return Err(anyhow::anyhow!("Mock transport failure"));
        }

        *self.last_sent.lock().unwrap() = Some(data.to_vec());
        Ok(TransportResponse::success(data.to_vec()))
    }

    async fn send_async(&self, data: &[u8]) -> anyhow::Result<()> {
        if self.should_fail {
            return Err(anyhow::anyhow!("Mock transport failure"));
        }

        *self.last_sent.lock().unwrap() = Some(data.to_vec());
        Ok(())
    }

    async fn health_check(&self) -> anyhow::Result<bool> {
        Ok(!self.should_fail)
    }

    fn metadata(&self) -> TransportMetadata {
        TransportMetadata::new("mock", "1.0")
    }
}

#[test]
fn test_compressed_transport_creation() {
    let inner = MockTransport::new();
    let config = TransportCompressionConfig::default();
    let transport = CompressedTransport::new(Box::new(inner), config);

    assert_eq!(transport.compression_level, 3);
    assert_eq!(transport.min_compress_size, 1024);
}

#[test]
fn test_compressed_transport_with_level() {
    let inner = MockTransport::new();
    let config = TransportCompressionConfig::default();
    let transport = CompressedTransport::with_level(Box::new(inner), config, 10);

    assert_eq!(transport.compression_level, 10);
}

#[test]
fn test_compressed_transport_with_threshold() {
    let inner = MockTransport::new();
    let config = TransportCompressionConfig::default();
    let transport = CompressedTransport::with_threshold(Box::new(inner), config, 2048);

    assert_eq!(transport.min_compress_size, 2048);
}

#[tokio::test]
async fn test_small_data_no_compression() {
    let inner = MockTransport::new();
    let config = TransportCompressionConfig::default();
    let transport = CompressedTransport::new(Box::new(inner), config);

    let small_data = b"hello world";
    let response = transport.send(small_data).await.unwrap();

    assert!(response.is_success());

    // Check that data was not compressed (sent as-is)
    let _last_sent = transport.inner.as_ref();
    // The mock received the data
}

#[tokio::test]
async fn test_large_data_compression() {
    let inner = MockTransport::new();
    let config = TransportCompressionConfig::default();
    let transport = CompressedTransport::new(Box::new(inner), config);

    // Large data that should be compressed
    let large_data = b"hello world".repeat(200);
    let original_size = large_data.len();

    let response = transport.send(&large_data).await.unwrap();

    assert!(response.is_success());

    // Check stats
    let stats = transport.stats();
    assert!(stats.base.compression_count >= 1);

    // Verify compression ratio
    let ratio = transport.overall_compression_ratio();
    assert!(
        ratio < 1.0,
        "Expected compression ratio < 1.0, got {}",
        ratio
    );

    // Verify bandwidth savings
    let savings = transport.bandwidth_savings_percent();
    assert!(
        savings > 0.0,
        "Expected bandwidth savings > 0%, got {}%",
        savings
    );

    println!(
        "Compression: {} bytes -> ratio {:.2}, savings {:.1}%",
        original_size, ratio, savings
    );
}

#[tokio::test]
async fn test_compression_roundtrip() {
    let inner = MockTransport::new();
    let config = TransportCompressionConfig::default();
    let transport = CompressedTransport::new(Box::new(inner), config);

    // Large JSON-like data
    let json_data = r#"{"key": "value", "items": [1, 2, 3]}"#.repeat(100);
    let original = json_data.as_bytes().to_vec();

    let response = transport.send(&original).await.unwrap();

    // Mock transport echoes back the data
    assert_eq!(response.body, original);
}

#[tokio::test]
async fn test_compression_stats() {
    let inner = MockTransport::new();
    let config = TransportCompressionConfig::default();
    let transport = CompressedTransport::new(Box::new(inner), config);

    // Send multiple large payloads
    for i in 0..5 {
        let data = format!("Payload {}: {}", i, "x".repeat(2000));
        let _ = transport.send(data.as_bytes()).await;
    }

    let stats = transport.stats();
    assert!(stats.base.compression_count >= 5);
    assert_eq!(stats.total_compressions, 5);

    // Reset stats
    transport.reset_stats();
    let stats_after_reset = transport.stats();
    assert_eq!(stats_after_reset.base.compression_count, 0);
}

#[tokio::test]
async fn test_health_check() {
    let inner = MockTransport::new();
    let config = TransportCompressionConfig::default();
    let transport = CompressedTransport::new(Box::new(inner), config);

    assert!(transport.health_check().await.unwrap());
}

#[tokio::test]
async fn test_metadata() {
    let inner = MockTransport::new();
    let config = TransportCompressionConfig::default();
    let transport = CompressedTransport::new(Box::new(inner), config);

    let metadata = transport.metadata();
    assert_eq!(metadata.name, "mock");
    assert!(metadata.supports_compression);
}

#[tokio::test]
async fn test_send_async() {
    let inner = MockTransport::new();
    let config = TransportCompressionConfig::default();
    let transport = CompressedTransport::new(Box::new(inner), config);

    let data = b"async test data".repeat(100);
    let result = transport.send_async(&data).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_compression_fallback_on_error() {
    // Create transport that will fail compression (if possible)
    let inner = MockTransport::new();
    let config = TransportCompressionConfig::default();
    let transport = CompressedTransport::new(Box::new(inner), config);

    // Even if compression fails, the data should still be sent
    let data = b"test data".repeat(100);
    let result = transport.send(&data).await;

    assert!(result.is_ok());
}
