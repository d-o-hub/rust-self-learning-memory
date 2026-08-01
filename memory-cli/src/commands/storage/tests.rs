//! Unit tests for storage telemetry provenance (PTA-A2).
//!
//! Verifies that `MetricValue`/`MetricProvenance` behave correctly, that JSON
//! output carries provenance, and that human output labels estimates and
//! unavailable metrics truthfully.

use super::provenance::{MetricProvenance, MetricValue};
use super::types::{ConnectionInfo, ConnectionStatus, StorageStats, StorageStatsData};
use crate::output::Output;

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

fn sample_stats() -> StorageStats {
    StorageStats {
        episodes: StorageStatsData {
            total_count: 5,
            completed_count: MetricValue::measured(3),
            average_size_bytes: MetricValue::estimated(2048),
        },
        patterns: StorageStatsData {
            total_count: 2,
            completed_count: MetricValue::unavailable(),
            average_size_bytes: MetricValue::estimated(1024),
        },
        storage_size_bytes: MetricValue::estimated(5 * 2048 + 2 * 1024),
        cache_hit_rate: MetricValue::unavailable(),
        last_sync: MetricValue::unavailable(),
    }
}

fn unavailable_connections() -> ConnectionStatus {
    ConnectionStatus {
        turso: ConnectionInfo {
            active_connections: MetricValue::unavailable(),
            pool_size: MetricValue::unavailable(),
            queue_depth: MetricValue::unavailable(),
            last_activity: MetricValue::unavailable(),
        },
        redb: ConnectionInfo {
            active_connections: MetricValue::unavailable(),
            pool_size: MetricValue::unavailable(),
            queue_depth: MetricValue::unavailable(),
            last_activity: MetricValue::unavailable(),
        },
    }
}

// ---------------------------------------------------------------------------
// MetricValue constructors and provenance
// ---------------------------------------------------------------------------

#[test]
fn metric_value_measured_sets_value_and_provenance() {
    // Arrange
    let metric = MetricValue::measured(42usize);

    // Act
    let value = metric.value;
    let provenance = metric.provenance;

    // Assert
    assert_eq!(value, Some(42));
    assert_eq!(provenance, MetricProvenance::Measured);
    assert!(!metric.is_unavailable());
}

#[test]
fn metric_value_estimated_sets_value_and_provenance() {
    // Arrange
    let metric = MetricValue::estimated(2048u64);

    // Act
    let value = metric.value;
    let provenance = metric.provenance;

    // Assert
    assert_eq!(value, Some(2048));
    assert_eq!(provenance, MetricProvenance::Estimated);
    assert!(!metric.is_unavailable());
}

#[test]
fn metric_value_unavailable_has_no_value() {
    // Arrange
    let metric = MetricValue::<usize>::unavailable();

    // Act
    let value = metric.value;
    let provenance = metric.provenance;

    // Assert
    assert_eq!(value, None);
    assert_eq!(provenance, MetricProvenance::Unavailable);
    assert!(metric.is_unavailable());
}

#[test]
fn metric_provenance_serializes_snake_case() {
    // Arrange
    let cases = [
        (MetricProvenance::Measured, "measured"),
        (MetricProvenance::Estimated, "estimated"),
        (MetricProvenance::Unavailable, "unavailable"),
    ];

    for (provenance, expected) in cases {
        // Act
        let json = serde_json::to_string(&provenance).unwrap();

        // Assert
        assert_eq!(json, format!("\"{expected}\""));
    }
}

#[test]
fn metric_value_render_labels_estimate_and_unavailable() {
    // Arrange
    let measured = MetricValue::measured(3usize);
    let estimated = MetricValue::estimated(2048u64);
    let unavailable = MetricValue::<usize>::unavailable();

    // Act
    let measured_render = measured.render();
    let estimated_render = estimated.render_with_suffix(" bytes");
    let unavailable_render = unavailable.render();

    // Assert
    assert_eq!(measured_render, "3");
    assert_eq!(estimated_render, "2048 bytes (estimate)");
    assert_eq!(unavailable_render, "unavailable");
}

// ---------------------------------------------------------------------------
// JSON serialization carries provenance
// ---------------------------------------------------------------------------

#[test]
fn storage_stats_json_includes_provenance() {
    // Arrange
    let stats = sample_stats();

    // Act
    let json = serde_json::to_value(&stats).unwrap();

    // Assert
    assert_eq!(json["episodes"]["total_count"], 5);
    assert_eq!(
        json["episodes"]["completed_count"],
        serde_json::json!({ "value": 3, "provenance": "measured" })
    );
    assert_eq!(
        json["episodes"]["average_size_bytes"],
        serde_json::json!({ "value": 2048, "provenance": "estimated" })
    );
    // Patterns have no completed concept: unavailable, never 0 measured.
    assert_eq!(
        json["patterns"]["completed_count"],
        serde_json::json!({ "value": null, "provenance": "unavailable" })
    );
    assert_eq!(
        json["storage_size_bytes"],
        serde_json::json!({ "value": 12288, "provenance": "estimated" })
    );
    assert_eq!(
        json["cache_hit_rate"],
        serde_json::json!({ "value": null, "provenance": "unavailable" })
    );
    assert_eq!(
        json["last_sync"],
        serde_json::json!({ "value": null, "provenance": "unavailable" })
    );
}

#[test]
fn connection_status_json_includes_provenance() {
    // Arrange
    let status = unavailable_connections();

    // Act
    let json = serde_json::to_value(&status).unwrap();

    // Assert
    for backend in ["turso", "redb"] {
        for metric in [
            "active_connections",
            "pool_size",
            "queue_depth",
            "last_activity",
        ] {
            assert_eq!(
                json[backend][metric],
                serde_json::json!({ "value": null, "provenance": "unavailable" }),
                "unexpected provenance for {backend}.{metric}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Human output labels estimates and unavailable metrics
// ---------------------------------------------------------------------------

#[test]
fn storage_stats_human_output_labels_estimates_and_unavailable() {
    // Arrange
    let stats = sample_stats();

    // Act
    let mut buffer = Vec::new();
    stats.write_human(&mut buffer).unwrap();
    let output = String::from_utf8(buffer).unwrap();

    // Assert
    assert!(
        output.contains("Completed: 3"),
        "episodes completed must be shown"
    );
    assert!(
        output.contains("Completed: unavailable"),
        "patterns completed must be unavailable, not 0"
    );
    assert!(
        output.contains("Avg Size: 2048 bytes (estimate)"),
        "episode average size must be labeled as an estimate"
    );
    assert!(
        output.contains("Total Size: 0.01 MB (estimate)"),
        "storage size must be labeled as an estimate"
    );
    assert!(
        output.contains("Cache Hit Rate: unavailable"),
        "cache hit rate must be unavailable, never 0.0%"
    );
    assert!(
        !output.contains("Last Sync"),
        "unavailable last sync line should be omitted"
    );
}

#[test]
fn connection_status_human_output_marks_metrics_unavailable() {
    // Arrange
    let status = unavailable_connections();

    // Act
    let mut buffer = Vec::new();
    status.write_human(&mut buffer).unwrap();
    let output = String::from_utf8(buffer).unwrap();

    // Assert
    assert!(output.contains("Connection Status"));
    assert!(output.contains("Turso:"));
    assert!(output.contains("redb:"));
    // No fabricated connection-pool values; every metric is unavailable.
    assert_eq!(
        output.matches("unavailable").count(),
        8,
        "all four metrics for both backends must be unavailable"
    );
    assert!(!output.contains("Active: 1"), "must not fabricate active=1");
    assert!(
        !output.contains("Pool Size: 10"),
        "must not fabricate pool=10"
    );
}
