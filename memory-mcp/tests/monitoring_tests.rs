//! Latency measurement for the MCP monitoring system (issue #1086).
//!
//! `end_request` used to derive latency from two `SystemTime` values truncated with
//! `as_secs()`, so any request finishing inside one wall-clock second was recorded as
//! 0ms and a request that happened to straddle a second boundary was recorded as a full
//! 1000ms. The module had no integration tests at all before this file.

use std::time::Duration;

use do_memory_mcp::monitoring::MonitoringSystem;
use do_memory_mcp::monitoring::types::{MonitoringConfig, PerformanceMetrics, ToolPerformance};

/// One request long enough to be a real latency sample, short enough to stay inside a
/// single wall-clock second most of the time.
const SLEEP: Duration = Duration::from_millis(30);

/// A sampled request must land in this band. The lower bound is what the bug broke (the
/// old code reported 0), the upper bound catches the other half of the bug (the old code
/// reported 1000 for a 30ms request that crossed a second boundary).
const MIN_REPORTED_MS: u64 = 20;
const MAX_REPORTED_MS: u64 = 200;

/// Same band for the `f64` averages; literals only, because numeric casts trip `cast_precision_loss`.
const MIN_REPORTED_F64: f64 = 20.0;
const MAX_REPORTED_F64: f64 = 200.0;

fn monitoring() -> MonitoringSystem {
    MonitoringSystem::new(MonitoringConfig {
        enabled: true,
        ..Default::default()
    })
}

async fn timed_request(monitoring: &MonitoringSystem, tool: &str) {
    let id = monitoring
        .start_request(format!("{tool}-req"), tool.to_string())
        .await;
    tokio::time::sleep(SLEEP).await;
    monitoring.end_request(&id, true, None).await;
}

fn tool_perf<'a>(perf: &'a PerformanceMetrics, tool: &str) -> &'a ToolPerformance {
    assert!(
        perf.tool_metrics.contains_key(tool),
        "no tool metrics recorded for '{tool}'"
    );
    &perf.tool_metrics[tool]
}

/// Five 30ms requests, none of which is allowed to be measured as 0ms or as a whole second.
#[tokio::test]
async fn sub_second_requests_report_millisecond_latency() {
    let monitoring = monitoring();
    for _ in 0..5 {
        timed_request(&monitoring, "memory_search").await;
    }

    let perf = monitoring.get_performance();
    let tool = tool_perf(&perf, "memory_search");

    assert_eq!(tool.total_calls, 5);
    assert!(
        tool.min_response_time_ms >= MIN_REPORTED_MS,
        "a 30ms request must not report {0}ms; second-truncated wall clock reported 0 \
         whenever the request stayed inside one second",
        tool.min_response_time_ms
    );
    assert!(
        tool.max_response_time_ms <= MAX_REPORTED_MS,
        "a 30ms request must not report {0}ms; a request straddling a second boundary was \
         rounded up to a full 1000ms",
        tool.max_response_time_ms
    );
    assert!(
        tool.avg_response_time_ms >= MIN_REPORTED_F64
            && tool.avg_response_time_ms <= MAX_REPORTED_F64,
        "average latency {:.1}ms is outside the plausible band",
        tool.avg_response_time_ms
    );
}

/// Aggregate stats see the same corrected value, so `avg_response_time_ms` is no longer
/// pinned to whole seconds.
#[tokio::test]
async fn stats_aggregate_the_monotonic_duration() {
    let monitoring = monitoring();
    timed_request(&monitoring, "record_episode").await;

    let stats = monitoring.get_stats();
    assert_eq!(stats.total_requests, 1);
    assert_eq!(stats.successful_requests, 1);
    assert!(
        stats.avg_response_time_ms >= MIN_REPORTED_F64,
        "stats recorded {:.1}ms for a {}ms request",
        stats.avg_response_time_ms,
        SLEEP.as_millis()
    );
    assert!(
        stats.avg_response_time_ms < 1000.0,
        "stats recorded {:.1}ms — still quantised to whole seconds",
        stats.avg_response_time_ms
    );
}

/// Latency comes from a monotonic clock, so it cannot go negative or vanish when the wall
/// clock moves; a request that never started is simply ignored.
#[tokio::test]
async fn latency_is_non_negative_and_unknown_requests_are_ignored() {
    let monitoring = monitoring();
    let id = monitoring
        .start_request("orphan".to_string(), "tool_a".to_string())
        .await;
    assert_eq!(monitoring.active_request_count().await, 1);

    monitoring.end_request("never-tracked", true, None).await;
    assert_eq!(
        monitoring.active_request_count().await,
        1,
        "ending an untracked request must not consume a live one"
    );

    monitoring
        .end_request(&id, false, Some("boom".to_string()))
        .await;
    assert_eq!(monitoring.active_request_count().await, 0);

    let perf = monitoring.get_performance();
    let tool = tool_perf(&perf, "tool_a");
    assert_eq!(tool.failed_calls, 1);
    assert_eq!(tool.successful_calls, 0);
    assert!(
        tool.min_response_time_ms <= tool.max_response_time_ms,
        "min {} must never exceed max {}",
        tool.min_response_time_ms,
        tool.max_response_time_ms
    );
}

/// The exported payload stays serializable and keeps the same metric keys — the fix must
/// not change label cardinality.
#[tokio::test]
async fn exports_remain_valid_and_key_stable() {
    let monitoring = monitoring();
    timed_request(&monitoring, "memory_search").await;
    timed_request(&monitoring, "record_episode").await;

    let stats_json = serde_json::to_string(&monitoring.get_stats()).unwrap();
    let perf_json = serde_json::to_string(&monitoring.get_performance()).unwrap();

    let stats: serde_json::Value = serde_json::from_str(&stats_json).unwrap();
    let perf: serde_json::Value = serde_json::from_str(&perf_json).unwrap();

    assert!(
        stats["avg_response_time_ms"].is_number(),
        "avg_response_time_ms must stay a number, got {stats}"
    );
    assert!(
        stats["avg_response_time_ms"].as_f64().unwrap_or_default() > 0.0,
        "exported JSON must carry the corrected sub-second value"
    );

    let tools = perf["tool_metrics"]
        .as_object()
        .expect("tool_metrics object");
    let mut keys: Vec<&str> = tools.keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        vec!["memory_search", "record_episode"],
        "one label per tool, no new or dropped metric keys"
    );

    for tool in tools.values() {
        assert!(
            tool["min_response_time_ms"].is_number() && tool["max_response_time_ms"].is_number(),
            "latency fields must stay numeric in the export, got {tool}"
        );
    }
}
