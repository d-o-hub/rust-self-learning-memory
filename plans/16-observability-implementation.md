# Phase 16: Observability Implementation - Production Monitoring

**Date**: 2025-11-14
**Status**: PLANNING
**Priority**: P0 (Critical for Production)
**Target**: v0.2.0 Release
**Dependencies**: None

## Executive Summary

Production systems require comprehensive observability to ensure reliability, performance, and rapid incident response. This plan details the implementation of monitoring, metrics, tracing, logging, and alerting for the self-learning memory system.

**Three Pillars of Observability**:
1. **Metrics**: Quantitative measurements (latency, throughput, error rates)
2. **Logs**: Event records with context (errors, warnings, debug info)
3. **Traces**: Request flow through distributed system components

---

## Goals and Success Criteria

### Primary Goals
- Real-time visibility into system health and performance
- Rapid incident detection and diagnosis (MTTR <5 minutes)
- Performance regression detection before production impact
- Capacity planning with historical trend analysis

### Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Metric Collection Latency | <1ms P95 | Overhead on operations |
| Metrics Cardinality | <10K unique series | Prometheus scrape size |
| Log Volume | <100 MB/day (default) | Storage requirements |
| Trace Sampling Rate | 1-10% configurable | Overhead vs. coverage |
| Alert False Positive Rate | <5% | Alert quality |
| Dashboard Load Time | <2s | Operational efficiency |
| MTTR (Incident Detection) | <5 min | Time to first alert |

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Self-Learning Memory                      │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │ Episode  │  │ Pattern  │  │ Storage  │  │   MCP    │   │
│  │Lifecycle │  │Extraction│  │  Layer   │  │ Sandbox  │   │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘   │
│       │             │             │             │          │
│       └─────────────┴─────────────┴─────────────┘          │
│                         │                                   │
│                         ▼                                   │
│              ┌─────────────────────┐                        │
│              │  Observability SDK  │                        │
│              │  (tracing crate)    │                        │
│              └─────────────────────┘                        │
│                         │                                   │
└─────────────────────────┼───────────────────────────────────┘
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
        ▼                 ▼                 ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│  Prometheus  │  │OpenTelemetry │  │     Logs     │
│   Metrics    │  │   Tracing    │  │(stdout/file) │
└──────────────┘  └──────────────┘  └──────────────┘
        │                 │                 │
        ▼                 ▼                 ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│   Grafana    │  │    Jaeger    │  │     Loki     │
│  Dashboards  │  │  Trace UI    │  │ Log Aggreg.  │
└──────────────┘  └──────────────┘  └──────────────┘
        │                 │                 │
        └─────────────────┴─────────────────┘
                          │
                          ▼
                   ┌──────────────┐
                   │  Alertmanager │
                   │   (Alerts)    │
                   └──────────────┘
```

---

## Implementation Plan

### Phase 1: Metrics Collection (Week 1-2)

#### 1.1 Prometheus Metrics Exporter

**File**: `memory-core/src/observability/metrics.rs`

**Dependencies**:
```toml
[dependencies]
prometheus = "0.13"
lazy_static = "1.4"
```

**Core Metrics Registry**:

```rust
use prometheus::{
    Counter, Histogram, Gauge, IntCounter, IntGauge,
    HistogramVec, CounterVec, GaugeVec,
    register_counter, register_histogram_vec, register_int_gauge,
};
use lazy_static::lazy_static;

lazy_static! {
    // Episode lifecycle metrics
    pub static ref EPISODE_CREATED_TOTAL: IntCounter =
        register_int_counter!(
            "memory_episode_created_total",
            "Total number of episodes created"
        ).unwrap();

    pub static ref EPISODE_COMPLETED_TOTAL: IntCounter =
        register_int_counter!(
            "memory_episode_completed_total",
            "Total number of episodes completed"
        ).unwrap();

    pub static ref EPISODE_DURATION_SECONDS: HistogramVec =
        register_histogram_vec!(
            "memory_episode_duration_seconds",
            "Episode duration in seconds",
            &["task_type", "verdict"],
            vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0]
        ).unwrap();

    // Pattern extraction metrics
    pub static ref PATTERN_EXTRACTED_TOTAL: CounterVec =
        register_counter_vec!(
            "memory_pattern_extracted_total",
            "Total patterns extracted by type",
            &["pattern_type"]
        ).unwrap();

    pub static ref PATTERN_EXTRACTION_DURATION_SECONDS: Histogram =
        register_histogram!(
            "memory_pattern_extraction_duration_seconds",
            "Pattern extraction latency",
            vec![0.00001, 0.0001, 0.001, 0.01, 0.1, 1.0]
        ).unwrap();

    // Storage metrics
    pub static ref STORAGE_OPERATION_DURATION_SECONDS: HistogramVec =
        register_histogram_vec!(
            "memory_storage_operation_duration_seconds",
            "Storage operation latency",
            &["operation", "backend"],  // operation: write/read, backend: turso/redb
            vec![0.0001, 0.001, 0.01, 0.1, 1.0]
        ).unwrap();

    pub static ref STORAGE_ERRORS_TOTAL: CounterVec =
        register_counter_vec!(
            "memory_storage_errors_total",
            "Storage operation errors",
            &["operation", "backend", "error_type"]
        ).unwrap();

    pub static ref CACHE_HIT_TOTAL: IntCounter =
        register_int_counter!(
            "memory_cache_hit_total",
            "Cache hits"
        ).unwrap();

    pub static ref CACHE_MISS_TOTAL: IntCounter =
        register_int_counter!(
            "memory_cache_miss_total",
            "Cache misses"
        ).unwrap();

    // Circuit breaker metrics
    pub static ref CIRCUIT_BREAKER_STATE: GaugeVec =
        register_gauge_vec!(
            "memory_circuit_breaker_state",
            "Circuit breaker state (0=closed, 1=open, 2=half_open)",
            &["backend"]
        ).unwrap();

    pub static ref CIRCUIT_BREAKER_FAILURES_TOTAL: CounterVec =
        register_counter_vec!(
            "memory_circuit_breaker_failures_total",
            "Circuit breaker failure count",
            &["backend"]
        ).unwrap();

    // Heuristic metrics
    pub static ref HEURISTIC_MATCH_TOTAL: IntCounter =
        register_int_counter!(
            "memory_heuristic_match_total",
            "Total heuristic matches"
        ).unwrap();

    pub static ref HEURISTIC_ACCURACY: Gauge =
        register_gauge!(
            "memory_heuristic_accuracy",
            "Heuristic match accuracy (0.0 to 1.0)"
        ).unwrap();

    // System metrics
    pub static ref ACTIVE_EPISODES: IntGauge =
        register_int_gauge!(
            "memory_active_episodes",
            "Number of active (incomplete) episodes"
        ).unwrap();

    pub static ref TOTAL_PATTERNS: IntGauge =
        register_int_gauge!(
            "memory_total_patterns",
            "Total number of patterns in storage"
        ).unwrap();
}
```

**Instrumentation Points**:

```rust
// In memory-core/src/memory/episode.rs
pub async fn start_episode(&self, task: String, context: TaskContext) -> Result<String> {
    EPISODE_CREATED_TOTAL.inc();
    ACTIVE_EPISODES.inc();

    let timer = STORAGE_OPERATION_DURATION_SECONDS
        .with_label_values(&["write", "turso"])
        .start_timer();

    let result = self.storage.create_episode(...).await;

    timer.observe_duration();

    if let Err(e) = &result {
        STORAGE_ERRORS_TOTAL
            .with_label_values(&["write", "turso", error_type(e)])
            .inc();
    }

    result
}

pub async fn complete_episode(&self, episode_id: &str, outcome: TaskOutcome) -> Result<()> {
    let episode = self.storage.get_episode(episode_id).await?;
    let duration = episode.duration_seconds();

    EPISODE_COMPLETED_TOTAL.inc();
    ACTIVE_EPISODES.dec();
    EPISODE_DURATION_SECONDS
        .with_label_values(&[&episode.task_type, outcome.verdict.as_str()])
        .observe(duration);

    // ... rest of completion logic
}
```

#### 1.2 HTTP Metrics Endpoint

**File**: `memory-core/src/observability/http.rs`

```rust
use prometheus::{Encoder, TextEncoder};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use axum::{Router, routing::get, response::IntoResponse};

pub async fn serve_metrics(addr: SocketAddr) -> Result<()> {
    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/health", get(health_handler));

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn metrics_handler() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    buffer
}

async fn health_handler() -> &'static str {
    "OK"
}
```

**Usage**:
```rust
// In main.rs or server initialization
tokio::spawn(async {
    observability::http::serve_metrics("0.0.0.0:9090".parse().unwrap())
        .await
        .expect("Failed to start metrics server");
});
```

#### 1.3 Prometheus Configuration

**File**: `deploy/prometheus.yml`

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'memory-service'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: '/metrics'

  - job_name: 'memory-mcp'
    static_configs:
      - targets: ['localhost:9091']
    metrics_path: '/metrics'
```

---

### Phase 2: Distributed Tracing (Week 3-4)

#### 2.1 OpenTelemetry Integration

**Dependencies**:
```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-opentelemetry = "0.22"
opentelemetry = { version = "0.21", features = ["trace", "metrics"] }
opentelemetry-jaeger = { version = "0.20", features = ["rt-tokio"] }
opentelemetry-otlp = { version = "0.14", features = ["trace", "tokio"] }
```

**File**: `memory-core/src/observability/tracing.rs`

```rust
use tracing_subscriber::{Layer, Registry, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use tracing_opentelemetry::OpenTelemetryLayer;
use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::trace::{self, RandomIdGenerator, Sampler};
use opentelemetry_sdk::Resource;
use opentelemetry_otlp::WithExportConfig;

pub fn init_tracing(service_name: &str, sampling_rate: f64) -> Result<()> {
    // Configure OpenTelemetry tracer
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317")
        )
        .with_trace_config(
            trace::config()
                .with_sampler(Sampler::TraceIdRatioBased(sampling_rate))
                .with_id_generator(RandomIdGenerator::default())
                .with_resource(Resource::new(vec![
                    opentelemetry::KeyValue::new("service.name", service_name),
                    opentelemetry::KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                ]))
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    // Configure tracing subscriber with multiple layers
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env()
            .add_directive(tracing::Level::INFO.into()))
        .with(tracing_subscriber::fmt::layer()
            .with_target(false)
            .with_thread_ids(true)
            .json())
        .with(OpenTelemetryLayer::new(tracer))
        .init();

    Ok(())
}

pub fn shutdown_tracing() {
    opentelemetry::global::shutdown_tracer_provider();
}
```

#### 2.2 Span Instrumentation

**File**: `memory-core/src/memory/episode.rs`

```rust
use tracing::{info, warn, error, instrument, Span};
use opentelemetry::trace::{TraceContextExt, Status};

#[instrument(
    name = "episode.start",
    skip(self),
    fields(
        episode.id = tracing::field::Empty,
        episode.task_type = %context.task_type,
        episode.domain = %context.domain,
    )
)]
pub async fn start_episode(&self, task: String, context: TaskContext) -> Result<String> {
    let episode_id = uuid::Uuid::new_v4().to_string();

    // Record episode ID in span
    Span::current().record("episode.id", &episode_id);

    info!("Starting episode: {}", task);

    let result = self.storage.create_episode(...).await;

    match &result {
        Ok(_) => {
            Span::current().set_status(Status::Ok);
            info!("Episode created successfully");
        }
        Err(e) => {
            Span::current().set_status(Status::error(e.to_string()));
            error!("Failed to create episode: {}", e);
        }
    }

    result
}

#[instrument(
    name = "episode.complete",
    skip(self, outcome),
    fields(
        episode.id = %episode_id,
        episode.verdict = tracing::field::Empty,
        episode.reward = tracing::field::Empty,
    )
)]
pub async fn complete_episode(&self, episode_id: &str, outcome: TaskOutcome) -> Result<()> {
    let verdict = outcome.verdict.clone();
    let reward = outcome.reward_score.total;

    Span::current().record("episode.verdict", verdict.as_str());
    Span::current().record("episode.reward", reward);

    // Nested span for pattern extraction
    let _guard = tracing::info_span!("pattern.extract", episode.id = %episode_id).entered();

    let patterns = self.extract_patterns(...).await?;

    info!("Extracted {} patterns", patterns.len());

    Ok(())
}
```

#### 2.3 Trace Context Propagation

**File**: `memory-core/src/observability/context.rs`

```rust
use opentelemetry::propagation::{Injector, Extractor};
use std::collections::HashMap;

pub struct TraceContextInjector<'a> {
    headers: &'a mut HashMap<String, String>,
}

impl<'a> Injector for TraceContextInjector<'a> {
    fn set(&mut self, key: &str, value: String) {
        self.headers.insert(key.to_string(), value);
    }
}

pub fn inject_trace_context(headers: &mut HashMap<String, String>) {
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject(&mut TraceContextInjector { headers });
    });
}

pub fn extract_trace_context(headers: &HashMap<String, String>) -> opentelemetry::Context {
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor { headers })
    })
}
```

---

### Phase 3: Structured Logging (Week 5)

#### 3.1 Log Configuration

**File**: `memory-core/src/observability/logging.rs`

```rust
use tracing_subscriber::fmt::format::FmtSpan;

pub fn init_logging(config: LogConfig) -> Result<()> {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(config.include_targets)
        .with_thread_ids(config.include_thread_ids)
        .with_thread_names(config.include_thread_names)
        .with_line_number(config.include_line_numbers)
        .with_file(config.include_file_paths)
        .with_span_events(FmtSpan::CLOSE);

    let fmt_layer = match config.format {
        LogFormat::Json => fmt_layer.json().boxed(),
        LogFormat::Pretty => fmt_layer.pretty().boxed(),
        LogFormat::Compact => fmt_layer.compact().boxed(),
    };

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt_layer)
        .init();

    Ok(())
}

pub struct LogConfig {
    pub format: LogFormat,
    pub include_targets: bool,
    pub include_thread_ids: bool,
    pub include_thread_names: bool,
    pub include_line_numbers: bool,
    pub include_file_paths: bool,
}

pub enum LogFormat {
    Json,
    Pretty,
    Compact,
}
```

#### 3.2 Logging Best Practices

**Levels**:
- `ERROR`: System failures requiring immediate attention
- `WARN`: Degraded state or potential issues
- `INFO`: Key business events (episode created, pattern extracted)
- `DEBUG`: Detailed diagnostic information
- `TRACE`: Very verbose, used for deep debugging

**Examples**:
```rust
use tracing::{error, warn, info, debug, trace};

// ERROR: Critical failures
error!(
    episode.id = %episode_id,
    error = %e,
    "Failed to persist episode to storage"
);

// WARN: Degraded state
warn!(
    cache.hit_rate = %hit_rate,
    cache.size = cache_size,
    "Cache hit rate below threshold (70%)"
);

// INFO: Key events
info!(
    episode.id = %episode_id,
    episode.task_type = %task_type,
    episode.duration_ms = duration_ms,
    "Episode completed successfully"
);

// DEBUG: Diagnostic info
debug!(
    pattern.type = %pattern_type,
    pattern.confidence = confidence,
    "Pattern extracted from episode"
);

// TRACE: Very verbose
trace!(
    step.index = step_idx,
    step.tool = %tool,
    "Processing execution step"
);
```

---

### Phase 4: Health Checks (Week 6)

#### 4.1 Health Check Endpoints

**File**: `memory-core/src/observability/health.rs`

```rust
use axum::{Json, response::IntoResponse, http::StatusCode};
use serde::{Serialize, Deserialize};
use std::time::SystemTime;

#[derive(Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub timestamp: u64,
    pub checks: Vec<HealthCheck>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Serialize, Deserialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub duration_ms: u64,
}

pub async fn liveness_handler() -> impl IntoResponse {
    // Simple check: is the service running?
    (StatusCode::OK, "OK")
}

pub async fn readiness_handler(state: AppState) -> impl IntoResponse {
    // Check if service is ready to accept requests
    let checks = vec![
        check_storage_connectivity(&state.storage).await,
        check_cache_health(&state.cache).await,
    ];

    let status = if checks.iter().all(|c| matches!(c.status, HealthStatus::Healthy)) {
        HealthStatus::Healthy
    } else if checks.iter().any(|c| matches!(c.status, HealthStatus::Unhealthy)) {
        HealthStatus::Unhealthy
    } else {
        HealthStatus::Degraded
    };

    let response = HealthResponse {
        status,
        timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        checks,
    };

    let status_code = match response.status {
        HealthStatus::Healthy => StatusCode::OK,
        HealthStatus::Degraded => StatusCode::OK,
        HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };

    (status_code, Json(response))
}

async fn check_storage_connectivity(storage: &TursoStorage) -> HealthCheck {
    let start = std::time::Instant::now();

    let result = storage.health_check().await;
    let duration_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(_) => HealthCheck {
            name: "storage".to_string(),
            status: HealthStatus::Healthy,
            message: None,
            duration_ms,
        },
        Err(e) => HealthCheck {
            name: "storage".to_string(),
            status: HealthStatus::Unhealthy,
            message: Some(e.to_string()),
            duration_ms,
        },
    }
}
```

---

### Phase 5: Alerting (Week 7)

#### 5.1 Prometheus Alerting Rules

**File**: `deploy/alert_rules.yml`

```yaml
groups:
  - name: memory_system_alerts
    interval: 30s
    rules:
      # High error rate
      - alert: HighStorageErrorRate
        expr: rate(memory_storage_errors_total[5m]) > 0.1
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High storage error rate detected"
          description: "Storage error rate is {{ $value }} errors/sec (threshold: 0.1)"

      # Circuit breaker open
      - alert: CircuitBreakerOpen
        expr: memory_circuit_breaker_state{state="open"} == 1
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "Circuit breaker {{ $labels.backend }} is open"
          description: "Circuit breaker for {{ $labels.backend }} has been open for 2+ minutes"

      # Low cache hit rate
      - alert: LowCacheHitRate
        expr: |
          rate(memory_cache_hit_total[10m]) /
          (rate(memory_cache_hit_total[10m]) + rate(memory_cache_miss_total[10m]))
          < 0.5
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "Cache hit rate is below 50%"
          description: "Cache hit rate: {{ $value | humanizePercentage }}"

      # High P95 latency
      - alert: HighEpisodeCreationLatency
        expr: |
          histogram_quantile(0.95,
            rate(memory_storage_operation_duration_seconds_bucket{operation="write"}[5m])
          ) > 1.0
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High episode creation latency"
          description: "P95 latency: {{ $value }}s (threshold: 1s)"

      # Service down
      - alert: MemoryServiceDown
        expr: up{job="memory-service"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Memory service is down"
          description: "Memory service has been down for 1+ minute"
```

#### 5.2 Alertmanager Configuration

**File**: `deploy/alertmanager.yml`

```yaml
global:
  resolve_timeout: 5m

route:
  group_by: ['alertname', 'cluster']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 12h
  receiver: 'default'
  routes:
    - match:
        severity: critical
      receiver: 'pagerduty'
      continue: true
    - match:
        severity: warning
      receiver: 'slack'

receivers:
  - name: 'default'
    # Default: log to file

  - name: 'slack'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/xxx'
        channel: '#memory-alerts'
        title: 'Memory System Alert'
        text: '{{ range .Alerts }}{{ .Annotations.description }}{{ end }}'

  - name: 'pagerduty'
    pagerduty_configs:
      - service_key: 'xxx'
        description: '{{ range .Alerts }}{{ .Annotations.summary }}{{ end }}'
```

---

### Phase 6: Dashboards (Week 8)

#### 6.1 Grafana Dashboard JSON

**File**: `deploy/grafana_dashboards/memory_overview.json`

**Panels**:
1. **Episode Creation Rate** (Graph)
   - Query: `rate(memory_episode_created_total[5m])`
2. **Episode Completion Rate** (Graph)
   - Query: `rate(memory_episode_completed_total[5m])`
3. **Active Episodes** (Gauge)
   - Query: `memory_active_episodes`
4. **Storage Latency (P50, P95, P99)** (Graph)
   - Query: `histogram_quantile(0.95, rate(memory_storage_operation_duration_seconds_bucket[5m]))`
5. **Cache Hit Rate** (Graph)
   - Query: `rate(memory_cache_hit_total[5m]) / (rate(memory_cache_hit_total[5m]) + rate(memory_cache_miss_total[5m]))`
6. **Error Rate by Type** (Graph)
   - Query: `rate(memory_storage_errors_total[5m])`
7. **Circuit Breaker State** (State Timeline)
   - Query: `memory_circuit_breaker_state`
8. **Pattern Extraction Rate** (Graph)
   - Query: `rate(memory_pattern_extracted_total[5m])`

---

## Configuration

**File**: `memory-core/observability.toml`

```toml
[metrics]
enabled = true
port = 9090
path = "/metrics"

[tracing]
enabled = true
endpoint = "http://localhost:4317"
sampling_rate = 0.1  # 10% of requests
service_name = "memory-core"

[logging]
level = "info"
format = "json"  # json, pretty, compact
include_thread_ids = true
include_file_paths = false

[health]
enabled = true
liveness_path = "/health/live"
readiness_path = "/health/ready"
storage_timeout_ms = 1000
```

---

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_registration() {
        // Verify metrics are registered correctly
        let families = prometheus::gather();
        assert!(families.iter().any(|f| f.get_name() == "memory_episode_created_total"));
    }

    #[tokio::test]
    async fn test_health_check() {
        let health = check_storage_connectivity(&mock_storage()).await;
        assert!(matches!(health.status, HealthStatus::Healthy));
    }
}
```

### Integration Tests
- End-to-end trace propagation
- Metrics collection under load
- Alert triggering thresholds

---

## Deployment

### Docker Compose

**File**: `deploy/docker-compose.observability.yml`

```yaml
version: '3.8'

services:
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - ./alert_rules.yml:/etc/prometheus/alert_rules.yml
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    volumes:
      - grafana-data:/var/lib/grafana
      - ./grafana_dashboards:/etc/grafana/provisioning/dashboards
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin

  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686"  # UI
      - "4317:4317"    # OTLP gRPC
      - "4318:4318"    # OTLP HTTP
    environment:
      - COLLECTOR_OTLP_ENABLED=true

  loki:
    image: grafana/loki:latest
    ports:
      - "3100:3100"
    volumes:
      - ./loki-config.yml:/etc/loki/config.yml

  alertmanager:
    image: prom/alertmanager:latest
    ports:
      - "9093:9093"
    volumes:
      - ./alertmanager.yml:/etc/alertmanager/alertmanager.yml

volumes:
  prometheus-data:
  grafana-data:
```

---

## Runbook

See **plans/17-operational-runbooks.md** for detailed operational procedures including:
- Responding to alerts
- Debugging with traces
- Performance troubleshooting
- Capacity planning

---

**Document Version**: 1.0
**Last Updated**: 2025-11-14
**Author**: Observability Team
**Status**: READY FOR IMPLEMENTATION
