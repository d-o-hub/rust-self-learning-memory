//! Coverage improvement tests for memory-storage-turso (ACT-028)
//!
//! Focus areas:
//! - transport/ - Compression wrapper operations
//! - pool/keepalive/ - Connection lifecycle
//! - metrics/export/ - HTTP export, Prometheus formatting

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::panic)]
#![allow(clippy::unreadable_literal)]

// ============================================================================
// Transport Tests (requires compression feature)
// ============================================================================

#[cfg(feature = "compression")]
mod transport_tests {
    use do_memory_storage_turso::transport::{
        TransportCompressionConfig, TransportMetadata, TransportResponse,
    };

    #[test]
    fn test_transport_response_success() {
        let body = b"test data".to_vec();
        let response = TransportResponse::success(body.clone());

        assert!(response.is_success());
        assert_eq!(response.status, 200);
        assert_eq!(response.body, body);
        assert!(response.headers.is_empty());
    }

    #[test]
    fn test_transport_response_error() {
        let response = TransportResponse::error(404, "Not found");

        assert!(!response.is_success());
        assert_eq!(response.status, 404);
        assert_eq!(response.body, b"Not found".to_vec());
    }

    #[test]
    fn test_transport_response_status_codes() {
        // Test various status codes
        let success_200 = TransportResponse::new(200, vec![]);
        let success_201 = TransportResponse::new(201, vec![]);
        let success_299 = TransportResponse::new(299, vec![]);
        let redirect_300 = TransportResponse::new(300, vec![]);
        let client_error_400 = TransportResponse::new(400, vec![]);
        let server_error_500 = TransportResponse::new(500, vec![]);

        assert!(success_200.is_success());
        assert!(success_201.is_success());
        assert!(success_299.is_success());
        assert!(!redirect_300.is_success());
        assert!(!client_error_400.is_success());
        assert!(!server_error_500.is_success());
    }

    #[test]
    fn test_transport_metadata_builder() {
        let metadata = TransportMetadata::new("test_transport", "2.0")
            .with_compression(true)
            .with_max_payload(1024 * 1024);

        assert_eq!(metadata.name, "test_transport");
        assert_eq!(metadata.version, "2.0");
        assert!(metadata.supports_compression);
        assert_eq!(metadata.max_payload_size, 1024 * 1024);
    }

    #[test]
    fn test_transport_metadata_defaults() {
        let metadata = TransportMetadata::new("default_transport", "1.0");

        assert_eq!(metadata.name, "default_transport");
        assert_eq!(metadata.version, "1.0");
        assert!(!metadata.supports_compression);
        assert_eq!(metadata.max_payload_size, 10 * 1024 * 1024); // 10MB default
    }

    #[test]
    fn test_transport_response_with_headers() {
        let mut response = TransportResponse::success(b"body".to_vec());
        response
            .headers
            .push(("Content-Type".to_string(), "application/json".to_string()));
        response
            .headers
            .push(("X-Custom".to_string(), "value".to_string()));

        assert_eq!(response.headers.len(), 2);
        assert_eq!(response.headers[0].0, "Content-Type");
        assert_eq!(response.headers[1].1, "value");
    }

    #[test]
    fn test_transport_compression_config_defaults() {
        let config = TransportCompressionConfig::default();

        assert_eq!(config.compression_threshold, 1024);
        assert!(config.auto_algorithm_selection);
        assert_eq!(config.max_compressed_size, 10 * 1024 * 1024);
        assert!(config.enable_metrics);
        assert!((config.warning_ratio_threshold - 0.9).abs() < 0.01);
        assert!((config.min_acceptable_ratio - 0.5).abs() < 0.01);
    }
}

// ============================================================================
// Compression Tests (requires compression feature)
// ============================================================================

#[cfg(feature = "compression")]
mod compression_tests {
    use do_memory_storage_turso::compression::{CompressionAlgorithm, CompressionStatistics};
    use do_memory_storage_turso::transport::TransportCompressionStats;

    #[test]
    fn test_compression_statistics_new() {
        let stats = CompressionStatistics::new();
        assert_eq!(stats.compression_count, 0);
        assert_eq!(stats.total_original_bytes, 0);
        assert_eq!(stats.total_compressed_bytes, 0);
        assert_eq!(stats.skipped_count, 0);
        assert_eq!(stats.failed_count, 0);
    }

    #[test]
    fn test_compression_statistics_record_compression() {
        let mut stats = CompressionStatistics::new();

        stats.record_compression(1000, 400, 50);

        assert_eq!(stats.compression_count, 1);
        assert_eq!(stats.total_original_bytes, 1000);
        assert_eq!(stats.total_compressed_bytes, 400);
        assert_eq!(stats.compression_time_us, 50);
    }

    #[test]
    fn test_compression_statistics_compression_ratio() {
        let mut stats = CompressionStatistics::new();

        // No data yet - returns 1.0 (no compression/no savings)
        assert!((stats.compression_ratio() - 1.0).abs() < 0.01);

        // With data
        stats.record_compression(1000, 500, 10);
        assert!((stats.compression_ratio() - 0.5).abs() < 0.01);

        stats.record_compression(1000, 300, 10);
        // Average ratio: (500 + 300) / (1000 + 1000) = 800/2000 = 0.4
        assert!((stats.compression_ratio() - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_compression_statistics_bandwidth_savings() {
        let mut stats = CompressionStatistics::new();

        stats.record_compression(1000, 400, 10);

        // 60% savings
        assert!((stats.bandwidth_savings_percent() - 60.0).abs() < 0.1);
    }

    #[test]
    fn test_compression_algorithm_variants() {
        assert_eq!(CompressionAlgorithm::None.to_string(), "none");
        assert_eq!(CompressionAlgorithm::Lz4.to_string(), "lz4");
        assert_eq!(CompressionAlgorithm::Zstd.to_string(), "zstd");
        assert_eq!(CompressionAlgorithm::Gzip.to_string(), "gzip");
    }

    #[test]
    fn test_transport_compression_stats_methods() {
        let mut stats = TransportCompressionStats::new();

        // Test streaming compression recording
        stats.record_streaming_compression(1000, 500);
        assert_eq!(stats.streaming_compressions, 1);
        assert_eq!(stats.total_bytes_saved, 500);

        // Test streaming decompression recording
        stats.record_streaming_decompression();
        assert_eq!(stats.streaming_decompressions, 1);

        // Test warning threshold recording
        stats.record_warning_threshold();
        assert_eq!(stats.warning_threshold_triggers, 1);

        // Test algorithm fallback recording
        stats.record_algorithm_fallback();
        assert_eq!(stats.algorithm_fallbacks, 1);
    }

    #[test]
    fn test_transport_compression_stats_time_recording() {
        let mut stats = TransportCompressionStats::new();

        // Record compression times
        stats.record_compression_time(100);
        assert_eq!(stats.avg_compression_time_us, 100);
        assert_eq!(stats.total_compressions, 1);

        stats.record_compression_time(200);
        // Average: (100 + 200) / 2 = 150
        assert_eq!(stats.avg_compression_time_us, 150);
        assert_eq!(stats.total_compressions, 2);

        // Record decompression times
        stats.record_decompression_time(50);
        assert_eq!(stats.avg_decompression_time_us, 50);
        assert_eq!(stats.total_decompressions, 1);
    }

    #[test]
    fn test_transport_compression_stats_overall_ratio() {
        let mut stats = TransportCompressionStats::new();

        stats.base.record_compression(2000, 800, 100);

        let ratio = stats.overall_ratio();
        assert!((ratio - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_compression_statistics_record_skipped() {
        let mut stats = CompressionStatistics::new();

        stats.record_skipped();
        assert_eq!(stats.skipped_count, 1);

        stats.record_skipped();
        assert_eq!(stats.skipped_count, 2);
    }

    #[test]
    fn test_compression_statistics_record_failed() {
        let mut stats = CompressionStatistics::new();

        stats.record_failed();
        assert_eq!(stats.failed_count, 1);
    }

    #[test]
    fn test_compression_statistics_record_decompression() {
        let mut stats = CompressionStatistics::new();

        stats.record_decompression(100);
        assert_eq!(stats.decompression_time_us, 100);
    }
}

// ============================================================================
// Keep-Alive Pool Tests
// ============================================================================

mod keepalive_tests {
    use do_memory_storage_turso::pool::keepalive::{KeepAliveConfig, KeepAliveStatistics};
    use std::time::Duration;

    #[test]
    fn test_keepalive_config_defaults() {
        let config = KeepAliveConfig::default();

        assert_eq!(config.keep_alive_interval, Duration::from_secs(30));
        assert_eq!(config.stale_threshold, Duration::from_secs(60));
        assert!(config.enable_proactive_ping);
        assert_eq!(config.ping_timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_keepalive_statistics_defaults() {
        let stats = KeepAliveStatistics::default();

        assert_eq!(stats.total_connections_created, 0);
        assert_eq!(stats.total_connections_refreshed, 0);
        assert_eq!(stats.total_stale_detected, 0);
        assert_eq!(stats.total_proactive_pings, 0);
        assert_eq!(stats.total_ping_failures, 0);
        assert_eq!(stats.active_connections, 0);
        assert_eq!(stats.avg_time_saved_ms, 0);
    }

    #[test]
    fn test_keepalive_statistics_update_activity() {
        let mut stats = KeepAliveStatistics::default();

        let before = stats.last_activity;
        std::thread::sleep(Duration::from_millis(10));
        stats.update_activity();

        assert!(stats.last_activity > before);
    }

    #[test]
    fn test_keepalive_config_custom() {
        let config = KeepAliveConfig {
            keep_alive_interval: Duration::from_secs(15),
            stale_threshold: Duration::from_secs(30),
            enable_proactive_ping: false,
            ping_timeout: Duration::from_secs(3),
        };

        assert_eq!(config.keep_alive_interval, Duration::from_secs(15));
        assert_eq!(config.stale_threshold, Duration::from_secs(30));
        assert!(!config.enable_proactive_ping);
        assert_eq!(config.ping_timeout, Duration::from_secs(3));
    }
}

// ============================================================================
// Metrics Export Tests
// ============================================================================

mod metrics_export_tests {
    use do_memory_storage_turso::metrics::export::{
        ExportConfig, ExportFormat, ExportTarget, ExportedMetric, MetricType, MetricValue,
        PrometheusExporter,
    };
    use std::time::Duration;

    #[test]
    fn test_export_config_defaults() {
        let config = ExportConfig::default();

        assert_eq!(config.format, ExportFormat::Prometheus);
        assert!(config.include_operations);
        assert!(config.include_cache);
        assert!(config.include_pool);
        assert!(config.include_errors);
        assert_eq!(config.export_interval, Duration::from_secs(60));
        assert_eq!(config.max_operation_types, 100);
    }

    #[test]
    fn test_export_config_http() {
        let config = ExportConfig::http("0.0.0.0", 9090);

        match config.target {
            ExportTarget::Http {
                bind_address,
                port,
                path,
            } => {
                assert_eq!(bind_address, "0.0.0.0");
                assert_eq!(port, 9090);
                assert_eq!(path, "/metrics");
            }
            _ => panic!("Expected Http target"),
        }
    }

    #[test]
    fn test_export_config_file() {
        let config = ExportConfig::file("/tmp/metrics.prom", Duration::from_secs(300));

        match config.target {
            ExportTarget::File {
                path,
                rotation_interval,
            } => {
                assert_eq!(path, "/tmp/metrics.prom");
                assert_eq!(rotation_interval, Duration::from_secs(300));
            }
            _ => panic!("Expected File target"),
        }
    }

    #[test]
    fn test_export_config_stdout() {
        let config = ExportConfig::stdout();

        assert!(matches!(config.target, ExportTarget::Stdout));
    }

    #[test]
    fn test_export_config_collection_only() {
        let config = ExportConfig::collection_only();

        assert!(matches!(config.target, ExportTarget::None));
    }

    #[test]
    fn test_export_config_builder_methods() {
        let config = ExportConfig::default()
            .with_format(ExportFormat::Json)
            .with_interval(Duration::from_secs(30))
            .without_operations()
            .without_cache()
            .without_pool()
            .without_errors();

        assert_eq!(config.format, ExportFormat::Json);
        assert_eq!(config.export_interval, Duration::from_secs(30));
        assert!(!config.include_operations);
        assert!(!config.include_cache);
        assert!(!config.include_pool);
        assert!(!config.include_errors);
    }

    #[test]
    fn test_metric_type_display() {
        assert_eq!(MetricType::Counter.to_string(), "counter");
        assert_eq!(MetricType::Gauge.to_string(), "gauge");
        assert_eq!(MetricType::Histogram.to_string(), "histogram");
        assert_eq!(MetricType::Summary.to_string(), "summary");
    }

    #[test]
    fn test_metric_value_display() {
        assert_eq!(MetricValue::Integer(42).to_string(), "42");

        let float_val = MetricValue::Float(1.23456);
        let display = float_val.to_string();
        assert!(display.starts_with("1.234"));

        assert_eq!(MetricValue::DurationMicros(1500).to_string(), "1500");
    }

    #[test]
    fn test_exported_metric_creation() {
        let metric = ExportedMetric::new(
            "test_metric",
            MetricType::Counter,
            MetricValue::Integer(100),
        );

        assert_eq!(metric.name, "test_metric");
        assert_eq!(metric.metric_type, MetricType::Counter);
        assert_eq!(metric.value, MetricValue::Integer(100));
        assert!(metric.labels.is_empty());
        assert!(metric.help.is_empty());
        assert!(metric.timestamp.is_none());
    }

    #[test]
    fn test_exported_metric_with_labels() {
        let metric =
            ExportedMetric::new("requests", MetricType::Counter, MetricValue::Integer(500))
                .with_label("method", "GET")
                .with_label("status", "200");

        assert_eq!(metric.labels.len(), 2);
        assert_eq!(metric.labels[0], ("method".to_string(), "GET".to_string()));
        assert_eq!(metric.labels[1], ("status".to_string(), "200".to_string()));
    }

    #[test]
    fn test_exported_metric_with_help() {
        let metric = ExportedMetric::new(
            "cache_hits",
            MetricType::Counter,
            MetricValue::Integer(1000),
        )
        .with_help("Total number of cache hits");

        assert_eq!(metric.help, "Total number of cache hits");
    }

    #[test]
    fn test_exported_metric_with_timestamp() {
        let metric = ExportedMetric::new("temp", MetricType::Gauge, MetricValue::Float(23.5))
            .with_timestamp(1234567890);

        assert_eq!(metric.timestamp, Some(1234567890));
    }

    #[test]
    fn test_exported_metric_prometheus_format_simple() {
        let metric = ExportedMetric::new(
            "simple_counter",
            MetricType::Counter,
            MetricValue::Integer(42),
        );

        let output = metric.to_prometheus();

        assert!(output.contains("# TYPE simple_counter counter"));
        assert!(output.contains("simple_counter 42"));
    }

    #[test]
    fn test_exported_metric_prometheus_format_with_help() {
        let metric =
            ExportedMetric::new("api_calls", MetricType::Counter, MetricValue::Integer(100))
                .with_help("Total API calls made");

        let output = metric.to_prometheus();

        assert!(output.contains("# HELP api_calls Total API calls made"));
        assert!(output.contains("# TYPE api_calls counter"));
    }

    #[test]
    fn test_exported_metric_prometheus_format_with_labels() {
        let metric = ExportedMetric::new(
            "http_requests",
            MetricType::Counter,
            MetricValue::Integer(500),
        )
        .with_label("method", "POST")
        .with_label("endpoint", "/api/episodes");

        let output = metric.to_prometheus();

        assert!(output.contains("http_requests{method=\"POST\",endpoint=\"/api/episodes\"} 500"));
    }

    #[test]
    fn test_exported_metric_prometheus_format_with_timestamp() {
        let metric = ExportedMetric::new("gauge_ts", MetricType::Gauge, MetricValue::Float(1.5))
            .with_timestamp(999888777);

        let output = metric.to_prometheus();

        assert!(output.contains("gauge_ts 1.500000 999888777"));
    }

    #[test]
    fn test_prometheus_exporter_creation() {
        let exporter = PrometheusExporter::default();

        assert_eq!(exporter.export_stats().export_count, 0);
    }

    #[test]
    fn test_prometheus_exporter_record_query() {
        let exporter = PrometheusExporter::new(ExportConfig::default());

        exporter.record_query(
            "episode_create",
            Duration::from_micros(100),
            true,
            Some(1024),
        );
        exporter.record_query("episode_get", Duration::from_micros(50), true, None);
        exporter.record_query("episode_delete", Duration::from_millis(1), false, None);

        let output = exporter.export_metrics();

        assert!(output.contains("turso_operation_latency_microseconds"));
        assert!(output.contains("episode_create"));
        assert!(output.contains("episode_get"));
    }

    #[test]
    fn test_prometheus_exporter_cache_metrics() {
        let exporter = PrometheusExporter::default();

        exporter.record_cache_hit();
        exporter.record_cache_hit();
        exporter.record_cache_miss();
        exporter.record_cache_eviction();

        let output = exporter.export_metrics();

        assert!(output.contains("turso_cache_hits 2"));
        assert!(output.contains("turso_cache_misses 1"));
        assert!(output.contains("turso_cache_evictions 1"));
        assert!(output.contains("turso_cache_hit_rate"));
    }

    #[test]
    fn test_prometheus_exporter_connection_metrics() {
        let exporter = PrometheusExporter::default();

        exporter.record_connection(Duration::from_micros(500));
        exporter.record_connection(Duration::from_micros(300));

        let output = exporter.export_metrics();

        assert!(output.contains("turso_pool_active_connections"));
        assert!(output.contains("turso_pool_wait_count"));
    }

    #[test]
    fn test_prometheus_exporter_error_recording() {
        let exporter = PrometheusExporter::default();

        exporter.record_error("connection_timeout");
        exporter.record_error("query_failed");

        // Just verify it doesn't panic - error metrics are included in export
        let output = exporter.export_metrics();
        assert!(output.contains("turso_query_success_rate"));
    }

    #[test]
    fn test_prometheus_exporter_reset() {
        let exporter = PrometheusExporter::default();

        exporter.record_query("test", Duration::from_micros(100), true, None);
        exporter.export_metrics();

        assert_eq!(exporter.export_stats().export_count, 1);

        exporter.reset();

        // After reset, the operation should not appear
        let output = exporter.export_metrics();
        assert!(!output.contains("turso_operation_count{operation=\"test\"}"));
    }

    #[test]
    fn test_prometheus_exporter_export_count() {
        let exporter = PrometheusExporter::default();

        assert_eq!(exporter.export_stats().export_count, 0);

        exporter.export_metrics();
        assert_eq!(exporter.export_stats().export_count, 1);

        exporter.export_metrics();
        exporter.export_metrics();
        assert_eq!(exporter.export_stats().export_count, 3);
    }

    #[test]
    fn test_export_format_equality() {
        // Test that format variants can be compared
        assert_eq!(ExportFormat::Prometheus, ExportFormat::Prometheus);
        assert_eq!(ExportFormat::Json, ExportFormat::Json);
        assert_eq!(ExportFormat::OpenTelemetry, ExportFormat::OpenTelemetry);
        assert_ne!(ExportFormat::Prometheus, ExportFormat::Json);
    }

    #[test]
    fn test_exported_metric_float_precision() {
        let metric = ExportedMetric::new(
            "precision_test",
            MetricType::Gauge,
            MetricValue::Float(1.23456789012345),
        );

        let output = metric.to_prometheus();

        // Should have 6 decimal places (rounded)
        assert!(output.contains("1.234568"));
    }

    #[test]
    fn test_exported_metric_duration_value() {
        let metric = ExportedMetric::new(
            "latency_micros",
            MetricType::Histogram,
            MetricValue::DurationMicros(1500),
        );

        let output = metric.to_prometheus();
        assert!(output.contains("latency_micros 1500"));
    }
}

// ============================================================================
// Metrics Collector Tests
// ============================================================================

mod metrics_collector_tests {
    use do_memory_storage_turso::metrics::MetricsCollector;
    use std::time::Duration;

    #[test]
    fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new();

        // Initial state
        assert!(collector.all_operation_metrics().is_empty());
    }

    #[test]
    fn test_metrics_collector_record_query() {
        let collector = MetricsCollector::new();

        collector.record_query(
            "episode_create",
            Duration::from_micros(100),
            true,
            Some(1024),
        );
        collector.record_query(
            "episode_create",
            Duration::from_micros(200),
            true,
            Some(512),
        );
        collector.record_query("episode_get", Duration::from_micros(50), true, None);

        let all_ops = collector.all_operation_metrics();
        assert_eq!(all_ops.len(), 2);

        let create_metrics = collector.operation_metrics("episode_create").unwrap();
        assert_eq!(create_metrics.total_count, 2);
    }

    #[test]
    fn test_metrics_collector_latency_percentiles() {
        let collector = MetricsCollector::new();

        // Record multiple operations with varying latencies
        for i in 1..=100 {
            collector.record_query("test_op", Duration::from_micros(i), true, None);
        }

        let (p50, p95, p99) = collector.latency_percentiles("test_op").unwrap();

        // Percentiles should be ordered
        assert!(p50 <= p95);
        assert!(p95 <= p99);

        // Rough bounds check (percentiles should be in range 1-100)
        assert!((1..=100).contains(&p50));
        assert!((80..=100).contains(&p95));
        assert!((90..=100).contains(&p99));
    }

    #[test]
    fn test_metrics_collector_cache_stats() {
        let collector = MetricsCollector::new();

        collector.record_cache(true); // hit
        collector.record_cache(true); // hit
        collector.record_cache(false); // miss
        collector.record_cache_eviction();

        let stats = collector.cache_stats();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.evictions, 1);
    }

    #[test]
    fn test_metrics_collector_query_success_rate() {
        let collector = MetricsCollector::new();

        // 2 successes, 1 failure = 66.6%
        collector.record_query("test", Duration::from_micros(100), true, None);
        collector.record_query("test", Duration::from_micros(100), true, None);
        collector.record_query("test", Duration::from_micros(100), false, None);

        let rate = collector.query_success_rate();
        assert!((rate - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_metrics_collector_reset() {
        let collector = MetricsCollector::new();

        collector.record_query("test", Duration::from_micros(100), true, None);
        collector.record_cache(true);

        collector.reset();

        assert!(collector.all_operation_metrics().is_empty());
        let stats = collector.cache_stats();
        assert_eq!(stats.hits, 0);
    }

    #[test]
    fn test_metrics_collector_error_recording() {
        let collector = MetricsCollector::new();

        // record_error should work without panic
        collector.record_error("connection_timeout");
        collector.record_error("query_failed");

        // Verify we can still export metrics
        collector.record_query("test", Duration::from_micros(100), true, None);
    }

    #[test]
    fn test_metrics_collector_bytes_transferred() {
        let collector = MetricsCollector::new();

        collector.record_query("read", Duration::from_micros(100), true, Some(1024));
        collector.record_query("read", Duration::from_micros(100), true, Some(512));

        let metrics = collector.metrics();
        assert_eq!(metrics.bytes_read(), 1536);
    }
}

// ============================================================================
// Pool Config Tests
// ============================================================================

mod pool_config_tests {
    use do_memory_storage_turso::pool::PoolConfig;
    use std::time::Duration;

    #[test]
    fn test_pool_config_defaults() {
        let config = PoolConfig::default();

        assert_eq!(config.max_connections, 10);
        assert_eq!(config.connection_timeout, Duration::from_secs(5));
        assert!(config.enable_health_check);
        assert_eq!(config.health_check_timeout, Duration::from_secs(2));
    }

    #[test]
    fn test_pool_config_custom() {
        let config = PoolConfig {
            max_connections: 25,
            connection_timeout: Duration::from_secs(10),
            enable_health_check: false,
            health_check_timeout: Duration::from_secs(5),
        };

        assert_eq!(config.max_connections, 25);
        assert_eq!(config.connection_timeout, Duration::from_secs(10));
        assert!(!config.enable_health_check);
        assert_eq!(config.health_check_timeout, Duration::from_secs(5));
    }
}

// ============================================================================
// Pool Statistics Tests
// ============================================================================

mod pool_statistics_tests {
    use do_memory_storage_turso::pool::PoolStatistics;

    #[test]
    fn test_pool_statistics_defaults() {
        let stats = PoolStatistics::default();

        assert_eq!(stats.total_created, 0);
        assert_eq!(stats.total_health_checks_passed, 0);
        assert_eq!(stats.total_health_checks_failed, 0);
        assert_eq!(stats.active_connections, 0);
        assert_eq!(stats.total_wait_time_ms, 0);
        assert_eq!(stats.total_checkouts, 0);
        assert_eq!(stats.avg_wait_time_ms, 0);
    }

    #[test]
    fn test_pool_statistics_update_averages() {
        let mut stats = PoolStatistics {
            total_checkouts: 10,
            total_wait_time_ms: 500,
            ..Default::default()
        };

        stats.update_averages();

        assert_eq!(stats.avg_wait_time_ms, 50); // 500/10 = 50
    }

    #[test]
    fn test_pool_statistics_update_averages_zero_checkouts() {
        let mut stats = PoolStatistics::default();

        stats.update_averages();

        assert_eq!(stats.avg_wait_time_ms, 0); // Avoid divide by zero
    }
}

// ============================================================================
// Integration-style Tests
// ============================================================================

mod integration_tests {
    use do_memory_storage_turso::metrics::export::{
        ExportConfig, ExportedMetric, MetricType, MetricValue, PrometheusExporter,
    };
    use std::time::Duration;

    #[test]
    fn test_full_metrics_workflow() {
        // Create exporter
        let exporter = PrometheusExporter::new(ExportConfig::default());

        // Record various operations
        exporter.record_query(
            "episode_create",
            Duration::from_micros(150),
            true,
            Some(500),
        );
        exporter.record_query("episode_get", Duration::from_micros(50), true, None);
        exporter.record_query("episode_update", Duration::from_micros(200), false, None);
        exporter.record_cache_hit();
        exporter.record_cache_miss();
        exporter.record_connection(Duration::from_micros(100));

        // Export metrics
        let output = exporter.export_metrics();

        // Verify all sections are present
        assert!(output.contains("# Operation latency metrics"));
        assert!(output.contains("# Cache metrics"));
        assert!(output.contains("# Connection pool metrics"));
        assert!(output.contains("# Throughput metrics"));
        assert!(output.contains("# Error metrics"));

        // Verify specific metrics
        assert!(
            output.contains("turso_operation_latency_microseconds{operation=\"episode_create\"")
        );
        assert!(output.contains("turso_cache_hits 1"));
        assert!(output.contains("turso_cache_misses 1"));
        assert!(output.contains("turso_total_queries 3"));
        assert!(output.contains("turso_successful_queries 2"));
        assert!(output.contains("turso_failed_queries 1"));
    }

    #[test]
    fn test_exported_metric_complete() {
        let metric = ExportedMetric::new(
            "complete_metric",
            MetricType::Summary,
            MetricValue::Float(42.5),
        )
        .with_help("A complete metric example")
        .with_label("environment", "production")
        .with_label("region", "us-west-2")
        .with_timestamp(1700000000);

        let output = metric.to_prometheus();

        // Verify all components
        assert!(output.contains("# HELP complete_metric A complete metric example"));
        assert!(output.contains("# TYPE complete_metric summary"));
        assert!(output.contains(
            "complete_metric{environment=\"production\",region=\"us-west-2\"} 42.500000 1700000000"
        ));
    }

    #[test]
    fn test_multiple_exports_increments_count() {
        let exporter = PrometheusExporter::default();

        for i in 1..=5 {
            exporter.export_metrics();
            assert_eq!(exporter.export_stats().export_count, i);
        }
    }
}
