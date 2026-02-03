# Security Monitoring Guide

**Version**: 1.0
**Last Updated**: 2026-02-01
**Status**: Production Ready

## Overview

This guide provides comprehensive instructions for setting up security monitoring and alerting for the rust-self-learning-memory system. It covers metrics, dashboards, log aggregation, and alert configuration.

## Table of Contents

- [Quick Start](#quick-start)
- [Key Metrics to Monitor](#key-metrics-to-monitor)
- [Prometheus Integration](#prometheus-integration)
- [Grafana Dashboards](#grafana-dashboards)
- [Log Aggregation](#log-aggregation)
- [Alert Configuration](#alert-configuration)
- [SIEM Integration](#siem-integration)
- [Monitoring Best Practices](#monitoring-best-practices)

## Quick Start

### Enable Metrics Endpoint

The memory service exposes metrics on the `/metrics` endpoint (Prometheus format):

```bash
# Check metrics are available
curl http://localhost:8080/metrics

# Example output:
# HELP rate_limit_denied_total Total number of rate limit denials
# TYPE rate_limit_denied_total counter
rate_limit_denied_total{operation="read"} 123
```

### Basic Prometheus Setup

Create `prometheus.yml`:

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'memory-service'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'
```

Run Prometheus:

```bash
docker run -d \
  -p 9090:9090 \
  -v $(pwd)/prometheus.yml:/etc/prometheus/prometheus.yml \
  prom/prometheus
```

Access Prometheus UI: http://localhost:9090

## Key Metrics to Monitor

### Security Metrics

#### 1. Rate Limit Violations

```promql
# Rate limit denial rate (per second)
rate(rate_limit_denied_total[5m])

# Rate limit hit rate (percentage)
sum(rate(rate_limit_denied_total[5m])) /
sum(rate(rate_limit_allowed_total[5m])) * 100

# Top rate-limited clients
topk(10, sum by (client_id) (rate(rate_limit_denied_total[5m])))
```

**Alert Thresholds**:
- Warning: > 10 denials/second
- Critical: > 50 denials/second

#### 2. Authentication Failures

```promql
# Authentication failure rate
rate(authentication_failures_total[5m])

# Authentication failure rate by IP
sum by (ip_address) (rate(authentication_failures_total[5m]))

# Authentication success rate
rate(authentication_successes_total[5m]) /
(rate(authentication_successes_total[5m]) + rate(authentication_failures_total[5m]))
```

**Alert Thresholds**:
- Warning: > 5 failures/second
- Critical: > 20 failures/second

#### 3. Security Violations

```promql
# Security violation rate by severity
sum by (severity) (rate(security_violations_total[5m]))

# Critical security violations (should be 0)
sum(rate(security_violations_total{severity="critical"}[5m]))

# Security violations by type
sum by (violation_type) (rate(security_violations_total[5m]))
```

**Alert Thresholds**:
- Warning: > 1 violation/hour
- Critical: > 5 violations/hour OR any critical violation

#### 4. Access Denials

```promql
# Access denial rate
rate(access_denied_total[5m])

# Access denials by resource
sum by (resource_type) (rate(access_denied_total[5m]))

# Access denials by user
topk(10, sum by (actor) (rate(access_denied_total[5m])))
```

**Alert Thresholds**:
- Warning: > 10 denials/hour
- Critical: > 50 denials/hour

### Operational Metrics

#### 5. Episode Operations

```promql
# Episode creation rate
rate(episode_created_total[5m])

# Episode completion rate
rate(episode_completed_total[5m])

# Episode failure rate
rate(episode_failed_total[5m]) /
rate(episode_completed_total[5m]) * 100
```

**Alert Thresholds**:
- Warning: > 10% failure rate
- Critical: > 25% failure rate

#### 6. Database Operations

```promql
# Database operation latency (P95)
histogram_quantile(0.95, rate(storage_operation_duration_seconds_bucket[5m]))

# Database connection pool utilization
storage_pool_connections_available / storage_pool_connections_max * 100

# Database operation error rate
rate(storage_operation_errors_total[5m]) /
rate(storage_operations_total[5m]) * 100
```

**Alert Thresholds**:
- Warning: P95 > 100ms
- Critical: P95 > 500ms OR pool utilization > 90%

#### 7. Cache Performance

```promql
# Cache hit rate
rate(cache_hits_total[5m]) /
(rate(cache_hits_total[5m]) + rate(cache_misses_total[5m])) * 100

# Cache size
cache_size_bytes

# Cache eviction rate
rate(cache_evictions_total[5m])
```

**Alert Thresholds**:
- Warning: Hit rate < 70%
- Critical: Hit rate < 50%

## Prometheus Integration

### Advanced Prometheus Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s
  external_labels:
    cluster: 'production'
    env: 'prod'

# Alertmanager configuration
alerting:
  alertmanagers:
    - static_configs:
        - targets: ['localhost:9093']

# Rule files
rule_files:
  - 'alerts.yml'

# Scrape configurations
scrape_configs:
  - job_name: 'memory-service'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'
    scrape_interval: 10s
    scrape_timeout: 5s

  - job_name: 'node-exporter'
    static_configs:
      - targets: ['localhost:9100']
```

### Recording Rules

Create `recording_rules.yml`:

```yaml
groups:
  - name: security_recording_rules
    interval: 30s
    rules:
      # Rate limit hit rate
      - record: job:rate_limit_hit_rate:5m
        expr: |
          sum(rate(rate_limit_denied_total[5m])) /
          sum(rate(rate_limit_allowed_total[5m]))

      # Authentication failure rate
      - record: job:auth_failure_rate:5m
        expr: |
          sum(rate(authentication_failures_total[5m]))

      # Security violation rate by severity
      - record: job:security_violations_by_severity:5m
        expr: |
          sum by (severity) (rate(security_violations_total[5m]))

      # Cache hit rate
      - record: job:cache_hit_rate:5m
        expr: |
          sum(rate(cache_hits_total[5m])) /
          sum(rate(cache_hits_total[5m]) + rate(cache_misses_total[5m]))
```

### Query Examples

#### Trend Analysis

```promql
# Rate limit violations over time (1 week)
rate(rate_limit_denied_total[5m])[1w:1h]

# Compare today vs yesterday
# Current day
rate(rate_limit_denied_total[5m]) > 0

# Same time yesterday
rate(rate_limit_denied_total[5m] offset 24h) > 0

# Anomaly detection (current vs average)
rate(rate_limit_denied_total[5m]) >
  avg_over_time(rate(rate_limit_denied_total[5m])[1w:]) * 2
```

#### Top-N Analysis

```promql
# Top 10 clients by rate limit denials
topk(10, sum by (client_id) (rate(rate_limit_denied_total[5m])))

# Top 10 resources by access denials
topk(10, sum by (resource_id) (rate(access_denied_total[5m])))

# Bottom 10 cache hit rates (by client)
bottomk(10, sum by (client_id) (
  rate(cache_hits_total[5m]) /
  (rate(cache_hits_total[5m]) + rate(cache_misses_total[5m]))
))
```

## Grafana Dashboards

### Dashboard JSON

Create a comprehensive security dashboard:

```json
{
  "dashboard": {
    "title": "Memory Service Security Monitor",
    "panels": [
      {
        "title": "Rate Limit Violations (Rate)",
        "targets": [
          {
            "expr": "sum(rate(rate_limit_denied_total[5m]))"
          }
        ],
        "type": "graph"
      },
      {
        "title": "Authentication Failures (Rate)",
        "targets": [
          {
            "expr": "sum(rate(authentication_failures_total[5m]))"
          }
        ],
        "type": "graph"
      },
      {
        "title": "Security Violations (by Severity)",
        "targets": [
          {
            "expr": "sum by (severity) (rate(security_violations_total[5m]))"
          }
        ],
        "type": "graph"
      },
      {
        "title": "Cache Hit Rate (%)",
        "targets": [
          {
            "expr": "job:cache_hit_rate:5m * 100"
          }
        ],
        "type": "gauge"
      },
      {
        "title": "Database Latency (P95)",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(storage_operation_duration_seconds_bucket[5m]))"
          }
        ],
        "type": "graph"
      },
      {
        "title": "Top 10 Rate-Limited Clients",
        "targets": [
          {
            "expr": "topk(10, sum by (client_id) (rate(rate_limit_denied_total[5m])))"
          }
        ],
        "type": "table"
      }
    ]
  }
}
```

### Panel Types

#### Single Stat Panel

```json
{
  "type": "stat",
  "title": "Critical Security Violations",
  "targets": [
    {
      "expr": "sum(rate(security_violations_total{severity=\"critical\"}[5m]))"
    }
  ],
  "options": {
    "graphMode": "area",
    "colorMode": "value"
  }
}
```

#### Time Series Graph

```json
{
  "type": "timeseries",
  "title": "Rate Limit Hit Rate Over Time",
  "targets": [
    {
      "expr": "job:rate_limit_hit_rate:5m * 100",
      "legendFormat": "Hit Rate %"
    }
  ],
  "fieldConfig": {
    "defaults": {
      "unit": "percent",
      "min": 0,
      "max": 100
    }
  }
}
```

#### Table Panel

```json
{
  "type": "table",
  "title": "Top Security Events",
  "targets": [
    {
      "expr": "topk(20, sum by (event_type, severity) (rate(audit_events_total[5m])))"
    }
  ],
  "transformations": [
    {
      "id": "organize"
    }
  ]
}
```

## Log Aggregation

### ELK Stack Integration

#### Filebeat Configuration

Create `filebeat.yml`:

```yaml
filebeat.inputs:
  - type: log
    enabled: true
    paths:
      - /var/log/memory/audit.log
    json.keys_under_root: true
    json.add_error_key: true
    fields:
      service: memory-service
      environment: production

output.elasticsearch:
  hosts: ["localhost:9200"]
  index: "memory-audit-%{+yyyy.MM.dd}"

setup.kibana:
  host: "localhost:5601"

processors:
  - drop_event:
      when:
        not:
          or:
            - equals:
                event_type: "access_denied"
            - equals:
                event_type: "security_violation"
            - equals:
                event_type: "auth_failure"
```

#### Kibana Index Pattern

```json
{
  "index_pattern": {
    "title": "memory-audit-*"
  },
  "field_attrs": {
    "timestamp": {
      "format": "yyyy-MM-dd HH:mm:ss.SSS"
    }
  }
}
```

#### Kibana Queries

**KQL Examples**:

```kql
# All critical events
event_type: "access_denied" AND level: "CRITICAL"

# Authentication failures
event_type: "auth_failure"

# Rate limit violations
event_type: "rate_limit_violation"

# Specific client
actor: "user:alice@example.com"

# Time range
@timestamp >= "2026-02-01" AND @timestamp < "2026-02-02"

# Complex query
(event_type: "security_violation" OR event_type: "access_denied")
  AND level: ("CRITICAL" OR "ERROR")
  AND @timestamp >= now-24h
```

### Loki Integration

#### Promtail Configuration

Create `promtail-config.yml`:

```yaml
server:
  http_listen_port: 9080

positions:
  filename: /tmp/positions.yaml

clients:
  - url: http://localhost:3100/loki/api/v1/push

scrape_configs:
  - job_name: memory-audit
    static_configs:
      - targets:
          - localhost
        labels:
          job: memory-service
          env: production
          __path__: /var/log/memory/audit.log

    pipeline_stages:
      - json:
          expressions:
            timestamp: timestamp
            event_type: event_type
            level: level
            actor: actor
            resource_id: resource_id

      - labels:
          level:
          event_type:

      - output:
          source: output
```

#### LogQL Queries

```logql
# All critical events
{job="memory-service", level="CRITICAL"}

# Rate limit violations
{job="memory-service"} |= "rate_limit_violation"

# Authentication failures
{job="memory-service"} |= "auth_failure"

# Rate of events
rate({job="memory-service"}[5m])

# Count by event type
count by (event_type) ({job="memory-service"})
```

## Alert Configuration

### Prometheus Alerts

Create `alerts.yml`:

```yaml
groups:
  - name: security_alerts
    interval: 30s
    rules:
      # Critical security violations
      - alert: CriticalSecurityViolation
        expr: |
          sum(rate(security_violations_total{severity="critical"}[5m])) > 0
        for: 0m
        labels:
          severity: critical
          priority: P0
        annotations:
          summary: "Critical security violation detected"
          description: "{{ $value }} critical security violations in last 5 minutes"

      # High rate of rate limit violations
      - alert: HighRateLimitViolationRate
        expr: |
          sum(rate(rate_limit_denied_total[5m])) > 50
        for: 5m
        labels:
          severity: critical
          priority: P1
        annotations:
          summary: "High rate of rate limit violations"
          description: "{{ $value }} rate limit denials per second"

      # High rate of authentication failures
      - alert: HighAuthFailureRate
        expr: |
          sum(rate(authentication_failures_total[5m])) > 20
        for: 5m
        labels:
          severity: warning
          priority: P1
        annotations:
          summary: "High rate of authentication failures"
          description: "{{ $value }} auth failures per second"

      # Security violations detected
      - alert: SecurityViolationsDetected
        expr: |
          sum(rate(security_violations_total{severity!~"debug|info"}[5m])) > 1
        for: 10m
        labels:
          severity: warning
          priority: P2
        annotations:
          summary: "Security violations detected"
          description: "{{ $value }} security violations per second"

      # Access denied rate
      - alert: HighAccessDeniedRate
        expr: |
          sum(rate(access_denied_total[1h])) > 50
        for: 10m
        labels:
          severity: warning
          priority: P2
        annotations:
          summary: "High rate of access denials"
          description: "{{ $value }} access denials per hour"

      # Cache hit rate low
      - alert: LowCacheHitRate
        expr: |
          job:cache_hit_rate:5m < 0.5
        for: 15m
        labels:
          severity: info
          priority: P3
        annotations:
          summary: "Low cache hit rate"
          description: "Cache hit rate is {{ $value | humanizePercentage }}"

      # Database operation latency high
      - alert: HighDatabaseLatency
        expr: |
          histogram_quantile(0.95, rate(storage_operation_duration_seconds_bucket[5m])) > 0.5
        for: 10m
        labels:
          severity: warning
          priority: P2
        annotations:
          summary: "High database latency"
          description: "P95 latency is {{ $value }}s"
```

### Alertmanager Configuration

Create `alertmanager.yml`:

```yaml
global:
  resolve_timeout: 5m
  slack_api_url: 'https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK'

route:
  group_by: ['alertname', 'priority']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 12h
  receiver: 'default'

  routes:
    # Critical alerts (P0) - immediate notification
    - match:
        priority: P0
      receiver: 'critical-alerts'
      continue: true

    # High priority alerts (P1) - immediate notification
    - match:
        priority: P1
      receiver: 'high-priority-alerts'
      continue: true

    # Medium priority alerts (P2) - batched
    - match:
        priority: P2
      receiver: 'medium-priority-alerts'

    # Low priority alerts (P3) - daily digest
    - match:
        priority: P3
      receiver: 'low-priority-alerts'

receivers:
  - name: 'default'
    slack_configs:
      - channel: '#memory-service-alerts'
        title: '{{ .GroupLabels.alertname }}'
        text: '{{ range .Alerts }}{{ .Annotations.description }}{{ end }}'

  - name: 'critical-alerts'
    slack_configs:
      - channel: '#security-critical'
        color: 'danger'
        title: '🔴 CRITICAL: {{ .GroupLabels.alertname }}'
        text: |
          *Summary:* {{ .CommonAnnotations.summary }}
          *Description:* {{ .CommonAnnotations.description }}
          *Priority:* {{ .GroupLabels.priority }}
    # PagerDuty integration
    pagerduty_configs:
      - service_key: 'YOUR_PAGERDUTY_KEY'
        severity: 'critical'

  - name: 'high-priority-alerts'
    slack_configs:
      - channel: '#security-alerts'
        color: 'warning'
        title: '⚠️ WARNING: {{ .GroupLabels.alertname }}'
        text: |
          *Summary:* {{ .CommonAnnotations.summary }}
          *Description:* {{ .CommonAnnotations.description }}

  - name: 'medium-priority-alerts'
    slack_configs:
      - channel: '#memory-service-ops'
        color: 'warning'
        title: '⚠️ Alert: {{ .GroupLabels.alertname }}'
        text: '{{ .CommonAnnotations.description }}'

  - name: 'low-priority-alerts'
    email_configs:
      - to: 'ops-team@example.com'
        headers:
          Subject: '📊 Daily Alert Digest: {{ .GroupLabels.alertname }}'
        html: '{{ template "email.default.html" . }}'
```

## SIEM Integration

### Splunk Integration

#### Splunk HTTP Event Collector

```bash
# Configure HEC in Splunk
# Settings > Data Inputs > HTTP Event Collector
# Create new token with source=memory-service

# Send audit logs to Splunk
curl -k https://splunk-server:8088/services/collector/event \
  -H "Authorization: Splunk YOUR_TOKEN" \
  -d '{
    "event": {
      "timestamp": "2026-02-01T12:34:56.789Z",
      "event_type": "episode_created",
      "level": "INFO",
      "actor": "user:alice@example.com",
      "resource_id": "ep-123"
    },
    "sourcetype": "_json",
    "source": "memory-service",
    "index": "security"
  }'
```

#### Splunk Queries

```spl
# All critical events
index=security source=memory-service level=CRITICAL

# Rate limit violations
index=security source=memory-service event_type=rate_limit_violation

# Authentication failures
index=security source=memory-service event_type=auth_failure

# Timechart of security violations
index=security source=memory-service
| timechart count by severity

# Top actors by event count
index=security source=memory-service
| stats count by actor
| sort - count
| head 10
```

### SentinelOne Integration

```yaml
# sentinelone_integration.py
import requests
import json

def send_event_to_sentinelone(event):
    url = "https://your-tenant.sentinelone.net/api/v2/events"
    headers = {
        "Authorization": "Bearer YOUR_TOKEN",
        "Content-Type": "application/json"
    }

    payload = {
        "event_type": event.get("event_type"),
        "description": f"Memory service security event: {event.get('event_type')}",
        "severity": map_severity(event.get("level")),
        "source": "memory-service",
        "timestamp": event.get("timestamp"),
        "data": event
    }

    response = requests.post(url, headers=headers, json=payload)
    return response.status_code == 200

def map_severity(level):
    mapping = {
        "CRITICAL": "critical",
        "ERROR": "high",
        "WARN": "medium",
        "INFO": "low",
        "DEBUG": "info"
    }
    return mapping.get(level, "low")
```

## Monitoring Best Practices

### Alert Tuning

1. **Start with high thresholds** to avoid alert fatigue
2. **Gradually lower thresholds** as you understand baseline metrics
3. **Use multiple severity levels** (P0, P1, P2, P3)
4. **Group related alerts** to reduce noise
5. **Set appropriate for durations** to avoid transient alerts

### Metric Retention

```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

# Configure retention
# storage.tsdb.retention.time: 15d
# storage.tsdb.retention.size: 50GB
```

**Recommended Retention**:
- Raw metrics: 15 days
- Aggregated metrics: 1 year (via recording rules)
- Audit logs: 90 days (compliance requirement)

### Dashboard Organization

1. **Overview Dashboard**: High-level health indicators
2. **Security Dashboard**: Security metrics and events
3. **Performance Dashboard**: Latency, throughput, cache
4. **Incident Response Dashboard**: Real-time incident monitoring

### Query Optimization

```promql
# BAD: Expensive query
rate(rate_limit_denied_total[5m])[1w:1h]

# GOOD: Use recording rules
rate(job:rate_limit_denial_rate:5m[1w:1h]

# BAD: High cardinality
sum by (client_id) (rate(rate_limit_denied_total[5m]))

# GOOD: Use topk
topk(100, sum by (client_id) (rate(rate_limit_denied_total[5m])))
```

### Testing Alerts

```bash
#!/bin/bash
# test_alerts.sh

echo "=== Testing Alert Configuration ==="

# 1. Validate Prometheus configuration
docker exec prometheus promtool check config /etc/prometheus/prometheus.yml

# 2. Validate alert rules
docker exec prometheus promtool check rules /etc/prometheus/alerts.yml

# 3. Check Alertmanager configuration
docker exec alertmanager amtool check-config /etc/alertmanager/alertmanager.yml

# 4. Test alert firing
echo "Generating test events..."
curl -X POST http://localhost:8080/test/security_violation

# 5. Verify alerts in Prometheus
curl -s http://localhost:9090/api/v1/alerts | jq '.data.alerts[] | select(.state=="firing")'

echo "Alert testing complete"
```

## Troubleshooting

### Issue: No Metrics Appearing

**Diagnosis**:
```bash
# Check metrics endpoint
curl http://localhost:8080/metrics

# Check Prometheus targets
curl http://localhost:9090/api/v1/targets

# Check Prometheus configuration
docker logs prometheus | grep error
```

**Solutions**:
1. Verify service is running
2. Check metrics endpoint is accessible
3. Verify Prometheus scrape configuration
4. Check firewall rules

### Issue: Alerts Not Firing

**Diagnosis**:
```bash
# Check alert rules
curl http://localhost:9090/api/v1/rules | jq '.data.groups[].rules[] | select(.name=="HighRateLimitViolationRate")'

# Check Alertmanager alerts
curl http://localhost:9093/api/v1/alerts | jq '.'

# Check Alertmanager configuration
docker logs alertmanager | grep error
```

**Solutions**:
1. Verify alert rule syntax
2. Check alert evaluation interval
3. Verify Alertmanager configuration
4. Check notification channels

### Issue: High Memory Usage by Prometheus

**Diagnosis**:
```bash
# Check Prometheus memory
docker stats prometheus

# Check number of active series
curl http://localhost:9090/api/v1/label/__name__/values | jq '.length'

# Check retention settings
curl http://localhost:9090/api/v1/status/config | jq '.data.yml.storage.tsdb'
```

**Solutions**:
1. Reduce metric retention period
2. Implement metric filtering
3. Use recording rules for aggregations
4. Reduce scrape interval for less critical targets

## Related Documentation

- [SECURITY_OPERATIONS_GUIDE.md](./SECURITY_OPERATIONS_GUIDE.md) - Main security operations guide
- [AUDIT_LOGGING_SETUP.md](./AUDIT_LOGGING_SETUP.md) - Audit logging configuration
- [RATE_LIMITING_TUNING.md](./RATE_LIMITING_TUNING.md) - Rate limiting configuration
- [INCIDENT_RESPONSE.md](./INCIDENT_RESPONSE.md) - Incident response procedures

---

**Document Version**: 1.0
**Last Updated**: 2026-02-01
**Next Review**: 2026-05-01
