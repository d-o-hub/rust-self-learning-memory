# Rate Limiting Configuration Guide

**Version**: 1.0
**Last Updated**: 2026-02-01
**Status**: Production Ready

## Overview

This guide provides detailed instructions for configuring and tuning rate limiting in the rust-self-learning-memory system. Rate limiting prevents DoS attacks and ensures fair resource allocation using a token bucket algorithm.

## Table of Contents

- [Quick Start](#quick-start)
- [Configuration Options](#configuration-options)
- [Token Bucket Algorithm](#token-bucket-algorithm)
- [Rate Limit Strategies](#rate-limit-strategies)
- [Per-Client vs Per-IP Limits](#per-client-vs-per-ip-limits)
- [Burst Allowance](#burst-allowance)
- [Monitoring & Alerting](#monitoring--alerting)
- [Performance Tuning](#performance-tuning)
- [Troubleshooting](#troubleshooting)

## Quick Start

### Default Configuration

Rate limiting is **enabled by default** with sensible defaults:

```bash
# Rate limiting is ON by default
# No configuration needed for basic protection

# View current configuration
curl http://localhost:8080/metrics | grep rate_limit
```

### Customize Limits

```bash
# Production: Higher limits for legitimate traffic
export MCP_RATE_LIMIT_READ_RPS=100
export MCP_RATE_LIMIT_WRITE_RPS=20
export MCP_RATE_LIMIT_READ_BURST=150
export MCP_RATE_LIMIT_WRITE_BURST=30

# Development: Disable for testing
export MCP_RATE_LIMIT_ENABLED=false

# Strict: Lower limits for API services
export MCP_RATE_LIMIT_READ_RPS=10
export MCP_RATE_LIMIT_WRITE_RPS=2
```

### Verify Rate Limiting

```bash
# Restart service to apply changes
sudo systemctl restart memory-service

# Test rate limiting
curl -v http://localhost:8080/api/episodes

# Check rate limit headers
curl -I http://localhost:8080/api/episodes | grep -i rate
```

## Configuration Options

### Environment Variables

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `MCP_RATE_LIMIT_ENABLED` | boolean | `true` | Enable/disable rate limiting |
| `MCP_RATE_LIMIT_READ_RPS` | integer | `100` | Read requests per second |
| `MCP_RATE_LIMIT_READ_BURST` | integer | `150` | Read burst size |
| `MCP_RATE_LIMIT_WRITE_RPS` | integer | `20` | Write requests per second |
| `MCP_RATE_LIMIT_WRITE_BURST` | integer | `30` | Write burst size |
| `MCP_RATE_LIMIT_CLEANUP_INTERVAL_SECS` | integer | `60` | Cleanup interval for stale buckets |
| `MCP_RATE_LIMIT_CLIENT_ID_HEADER` | string | `X-Client-ID` | Header name for client identification |

### Basic Configuration Example

```bash
# .env file
MCP_RATE_LIMIT_ENABLED=true
MCP_RATE_LIMIT_READ_RPS=100
MCP_RATE_LIMIT_READ_BURST=150
MCP_RATE_LIMIT_WRITE_RPS=20
MCP_RATE_LIMIT_WRITE_BURST=30
MCP_RATE_LIMIT_CLEANUP_INTERVAL_SECS=60
MCP_RATE_LIMIT_CLIENT_ID_HEADER=X-Client-ID
```

### Programmatic Configuration

```rust
use memory_mcp::server::rate_limiter::{RateLimiter, RateLimitConfig};
use std::time::Duration;

let config = RateLimitConfig {
    enabled: true,
    read_requests_per_second: 100,
    read_burst_size: 150,
    write_requests_per_second: 20,
    write_burst_size: 30,
    cleanup_interval: Duration::from_secs(60),
    client_id_header: "X-Client-ID".to_string(),
};

let limiter = RateLimiter::new(config);
```

## Token Bucket Algorithm

### How It Works

The token bucket algorithm allows for smooth rate limiting with burst handling:

```
┌────────────────────────────────────────────────────────────┐
│                    Token Bucket                            │
├────────────────────────────────────────────────────────────┤
│  Capacity: 150 tokens (burst size)                         │
│  Refill Rate: 100 tokens/second                           │
│  Current Tokens: ████░░░░░░░░░░░ (40/150)                  │
└────────────────────────────────────────────────────────────┘
         │                              │
         │ 1 token consumed per request │
         ▼                              ▼
    Request Arrived              Next Refill
    (if tokens > 0)              (+100 tokens/sec)
```

### Algorithm Behavior

**Initial State**: Bucket starts full (150 tokens)

```
Time 0s:   [████████████████████████████] 150 tokens
           Request 1: 149 tokens remaining
           Request 2: 148 tokens remaining
           ...
           Request 150: 0 tokens remaining
           Request 151: RATE LIMITED
```

**Refill Over Time**: Tokens regenerate at 100/second

```
Time 0.5s: [████████████████] 50 tokens regenerated
           Total: 50 tokens

Time 1.0s: [████████████████████] 100 tokens regenerated
           Total: 100 tokens
```

**Burst Allowance**: Can handle temporary spikes

```
Normal load: 10 requests/second
Burst: 150 requests in 1 second (allowed)
Recovery: Takes 0.5 seconds to refill burst capacity
```

### Mathematical Model

**Tokens at time t**:

```
tokens(t) = min(
    capacity,
    tokens(t-1) + (refill_rate × elapsed_time)
)
```

**Rate limited when**:

```
tokens(t) < requested_tokens
```

**Time until next token**:

```
retry_after = (requested_tokens - current_tokens) / refill_rate
```

## Rate Limit Strategies

### Fixed Rate Limiting

**Description**: Constant rate limit regardless of system load.

**Use Case**: Predictable resource allocation, fair sharing.

**Configuration**:
```bash
export MCP_RATE_LIMIT_READ_RPS=100
export MCP_RATE_LIMIT_WRITE_RPS=20
```

**Pros**:
- Simple to understand and configure
- Predictable behavior
- Fair resource allocation

**Cons**:
- Doesn't adapt to load changes
- May be too restrictive during low load
- May be too permissive during high load

### Adaptive Rate Limiting

**Description**: Adjust limits based on system load and performance.

**Use Case**: Maximizing throughput while protecting system health.

**Implementation** (manual):

```rust
use std::sync::atomic::{AtomicU32, Ordering};

struct AdaptiveRateLimiter {
    base_rps: u32,
    current_rps: AtomicU32,
}

impl AdaptiveRateLimiter {
    fn adjust(&self, system_load: f64) {
        let new_rps = if system_load > 0.8 {
            // Reduce limits under high load
            (self.base_rps as f64 * 0.5) as u32
        } else if system_load < 0.3 {
            // Increase limits under low load
            (self.base_rps as f64 * 1.5) as u32
        } else {
            self.base_rps
        };

        self.current_rps.store(new_rps, Ordering::Relaxed);
    }
}
```

**Pros**:
- Maximizes throughput
- Adapts to changing conditions
- Protects system health

**Cons**:
- More complex to implement
- Less predictable behavior
- Requires careful tuning

### Tiered Rate Limiting

**Description**: Different limits for different client tiers.

**Use Case**: API products with different service levels.

**Implementation**:

```rust
use std::collections::HashMap;

struct TieredRateLimiter {
    free_tier: RateLimitConfig,
    pro_tier: RateLimitConfig,
    enterprise_tier: RateLimitConfig,
    client_tiers: HashMap<String, RateLimitConfig>,
}

impl TieredRateLimiter {
    fn get_config(&self, client_id: &str) -> &RateLimitConfig {
        self.client_tiers
            .get(client_id)
            .unwrap_or(&self.free_tier)
    }
}
```

**Configuration**:
```bash
# Free tier: 10 RPS read, 2 RPS write
# Pro tier: 100 RPS read, 20 RPS write
# Enterprise tier: 1000 RPS read, 200 RPS write
```

**Pros**:
- Monetization strategy
- Protects resources for high-value clients
- Flexible allocation

**Cons**:
- More complex configuration
- Requires client tier management
- Potential for abuse if tiers are misconfigured

## Per-Client vs Per-IP Limits

### Per-Client Limits (Default)

**Description**: Rate limiting based on client ID (from header).

**Configuration**:
```bash
export MCP_RATE_LIMIT_CLIENT_ID_HEADER=X-Client-ID
```

**How It Works**:
```
Client "alice" → 100 RPS
Client "bob"   → 100 RPS
Client "charlie" → 100 RPS
```

**Use Cases**:
- API services with authenticated clients
- Multi-tenant applications
- Per-user rate limiting

**Pros**:
- Fair allocation per user
- Works with NAT and proxies
- Bounded memory (one bucket per client)

**Cons**:
- Requires client identification
- Can be circumvented by creating multiple clients
- Requires cleanup of stale buckets

### Per-IP Limits

**Description**: Rate limiting based on IP address.

**Configuration**:
```rust
use memory_mcp::server::rate_limiter::ClientId;

// Extract IP from request
let client_id = ClientId::from_ip("192.168.1.100");
```

**How It Works**:
```
192.168.1.100 → 100 RPS
192.168.1.101 → 100 RPS
192.168.1.102 → 100 RPS
```

**Use Cases**:
- Preventing DoS attacks
- Unauthenticated API access
- Geographic rate limiting

**Pros**:
- Works without authentication
- Harder to circumvent
- Simple to implement

**Cons**:
- Doesn't work with NAT (many users, one IP)
- Can block legitimate users behind NAT
- IP spoofing vulnerability

### Hybrid Approach

**Description**: Combine per-client and per-IP limits.

**Configuration**:
```bash
# Per-user limits
export MCP_RATE_LIMIT_READ_RPS=100

# Per-IP limits (enforce separately)
export MCP_RATE_LIMIT_IP_READ_RPS=1000
```

**Implementation**:
```rust
fn check_rate_limit_hybrid(
    user_limiter: &RateLimiter,
    ip_limiter: &RateLimiter,
    user_id: &str,
    ip: &str,
) -> bool {
    // Check both user and IP limits
    user_limiter.check_rate_limit(&ClientId::from_string(user_id), OperationType::Read).allowed
        && ip_limiter.check_rate_limit(&ClientId::from_ip(ip), OperationType::Read).allowed
}
```

**Use Cases**:
- Public APIs with optional authentication
- Multi-tier rate limiting
- Defense in depth

## Burst Allowance

### What Is Burst?

**Burst** allows temporary traffic spikes above the sustained rate.

```
Sustained Rate: 100 RPS
Burst Size: 150 tokens

Scenario 1: Steady traffic
100 requests/second ✓ (within sustained rate)

Scenario 2: Traffic spike
150 requests in 1 second ✓ (within burst)
0 requests for 0.5 seconds (refilling)
100 requests/second ✓ (back to normal)
```

### Burst Size Configuration

**Guidelines**:

| Use Case | Sustained RPS | Burst Size | Burst Duration |
|----------|---------------|-----------|----------------|
| API Service | 100 | 150 (1.5x) | 1.5 seconds |
| High Traffic | 1000 | 1500 (1.5x) | 1.5 seconds |
| Low Traffic | 10 | 20 (2x) | 2 seconds |
| Real-time | 50 | 100 (2x) | 2 seconds |

**Formula**:
```
burst_size = sustained_rps × burst_multiplier

burst_duration = burst_size / sustained_rps
```

**Examples**:

```bash
# API service: 100 RPS sustained, 150 burst
export MCP_RATE_LIMIT_READ_RPS=100
export MCP_RATE_LIMIT_READ_BURST=150  # 1.5x burst

# High traffic: 1000 RPS sustained, 1500 burst
export MCP_RATE_LIMIT_READ_RPS=1000
export MCP_RATE_LIMIT_READ_BURST=1500  # 1.5x burst

# Conservative: 10 RPS sustained, 15 burst
export MCP_RATE_LIMIT_READ_RPS=10
export MCP_RATE_LIMIT_READ_BURST=15  # 1.5x burst
```

### Burst Recovery

**When burst is exhausted**:

```
Time 0.0s: Burst exhausted (0 tokens remaining)
Time 0.5s: 50 tokens regenerated (50%)
Time 1.0s: 100 tokens regenerated (100%)
Time 1.5s: 150 tokens regenerated (burst recovered)
```

**Configuration for fast recovery**:

```bash
# Increase sustained rate for faster burst recovery
export MCP_RATE_LIMIT_READ_RPS=200
export MCP_RATE_LIMIT_READ_BURST=150

# Now burst recovers in 0.75 seconds
```

## Monitoring & Alerting

### Metrics to Track

**Rate Limit Metrics**:

```bash
# View all rate limit metrics
curl http://localhost:8080/metrics | grep rate_limit

# Example output:
rate_limit_allowed_total{operation="read"} 12345
rate_limit_denied_total{operation="read"} 123
rate_limit_allowed_total{operation="write"} 2345
rate_limit_denied_total{operation="write"} 23
rate_limit_buckets_active 150
rate_limit_tokens_remaining{client_id="alice",operation="read"} 45
```

### Prometheus Integration

**Example Prometheus queries**:

```promql
# Rate limit denial rate (per second)
rate(rate_limit_denied_total[5m])

# Top 10 rate-limited clients
topk(10, sum by (client_id) (rate_limit_denied_total))

# Overall rate limit hit rate
sum(rate(rate_limit_denied_total[5m])) / sum(rate(rate_limit_allowed_total[5m]))

# Active bucket count
rate_limit_buckets_active
```

### Grafana Dashboard

**Recommended Panels**:

1. **Rate Limit Denial Rate**
   - Query: `sum(rate(rate_limit_denied_total[5m]))`
   - Alert if > 10 denials/second

2. **Top Rate-Limited Clients**
   - Query: `topk(10, sum by (client_id) (rate(rate_limit_denied_total[5m])))`
   - Identify abusive or misconfigured clients

3. **Rate Limit Hit Rate**
   - Query: `sum(rate(rate_limit_denied_total[5m])) / sum(rate(rate_limit_allowed_total[5m]))`
   - Alert if > 1% of requests are rate-limited

4. **Active Buckets**
   - Query: `rate_limit_buckets_active`
   - Monitor memory usage

### Alert Thresholds

| Metric | Warning | Critical | Action |
|--------|---------|----------|--------|
| Denial rate (per second) | > 10 | > 50 | Investigate abuse or adjust limits |
| Hit rate (percentage) | > 1% | > 5% | Review limits, add capacity |
| Active buckets | > 1000 | > 5000 | Monitor memory, adjust cleanup |
| Per-client denials | > 100/min | > 500/min | Block abusive client |

### Alert Example (Prometheus)

```yaml
# prometheus/alerts.yml
groups:
  - name: rate_limiting
    rules:
      - alert: HighRateLimitDenialRate
        expr: sum(rate(rate_limit_denied_total[5m])) > 50
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High rate of rate limit denials"
          description: "Rate limit denial rate is {{ $value }} per second"

      - alert: HighRateLimitHitRate
        expr: |
          sum(rate(rate_limit_denied_total[5m])) /
          sum(rate(rate_limit_allowed_total[5m])) > 0.05
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High rate limit hit rate"
          description: "Rate limit hit rate is {{ $value | humanizePercentage }}"
```

## Performance Tuning

### Memory Usage

**Per-bucket memory**: ~200 bytes

**Estimate memory usage**:
```
active_clients × 200 bytes = total_memory

Example:
1000 clients × 200 bytes = 200 KB
10000 clients × 200 bytes = 2 MB
100000 clients × 200 bytes = 20 MB
```

**Monitor memory usage**:
```bash
# Check active buckets
curl http://localhost:8080/metrics | grep rate_limit_buckets_active

# Monitor process memory
ps aux | grep memory-service | awk '{print $6}'
```

### Cleanup Interval

**Purpose**: Remove stale client buckets to free memory.

**Configuration**:
```bash
export MCP_RATE_LIMIT_CLEANUP_INTERVAL_SECS=60
```

**Guidelines**:

| Traffic Pattern | Cleanup Interval | Rationale |
|----------------|------------------|-----------|
| High traffic (1000+ clients) | 60 seconds | Frequent cleanup, keep memory low |
| Medium traffic (100-1000 clients) | 300 seconds (5 min) | Balanced |
| Low traffic (<100 clients) | 3600 seconds (1 hour) | Minimal cleanup needed |

**Trade-offs**:
- **Shorter interval**: More CPU overhead, lower memory
- **Longer interval**: Less CPU overhead, higher memory

### Rate Limit Header Performance

**Headers added to every response**:
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 30
```

**Performance impact**: Negligible (<1 µs per request)

**Disable for maximum performance** (not recommended):
```rust
// Omit headers for slightly better performance
let headers = vec![];  // Don't add rate limit headers
```

## Troubleshooting

### Issue: Legitimate Requests Being Rate-Limited

**Symptoms**:
- Legitimate users getting 429 (Too Many Requests) errors
- High rate limit denial rate
- User complaints

**Diagnosis**:
```bash
# Check rate limit configuration
echo $MCP_RATE_LIMIT_READ_RPS
echo $MCP_RATE_LIMIT_WRITE_RPS

# Check rate limit metrics
curl http://localhost:8080/metrics | grep rate_limit_denied

# Identify most rate-limited clients
curl http://localhost:8080/metrics | grep rate_limit_denied | \
  grep -oP 'client_id="\K[^"]+' | sort | uniq -c | sort -rn
```

**Solutions**:

1. **Increase rate limits**:
   ```bash
   export MCP_RATE_LIMIT_READ_RPS=200  # Increased from 100
   export MCP_RATE_LIMIT_WRITE_RPS=40  # Increased from 20
   ```

2. **Increase burst allowance**:
   ```bash
   export MCP_RATE_LIMIT_READ_BURST=300  # Increased from 150
   ```

3. **Implement client whitelisting**:
   ```rust
   // Skip rate limiting for trusted clients
   if trusted_clients.contains(client_id) {
       return RateLimitResult::allowed();
   }
   ```

4. **Use adaptive rate limiting**:
   ```rust
   // Adjust limits based on system load
   adjust_rate_limits(system_load);
   ```

### Issue: No Rate Limiting Happening

**Symptoms**:
- No 429 errors even under extreme load
- Rate limit metrics show zero denials
- System overwhelmed by requests

**Diagnosis**:
```bash
# Check if rate limiting is enabled
echo $MCP_RATE_LIMIT_ENABLED

# Check rate limit metrics
curl http://localhost:8080/metrics | grep rate_limit

# Check client ID header
curl -I http://localhost:8080/api/episodes | grep -i client
```

**Solutions**:

1. **Enable rate limiting**:
   ```bash
   export MCP_RATE_LIMIT_ENABLED=true
   sudo systemctl restart memory-service
   ```

2. **Ensure client ID is being sent**:
   ```bash
   curl -H "X-Client-ID: test-client" http://localhost:8080/api/episodes
   ```

3. **Check rate limit configuration**:
   ```bash
   # Verify limits are set correctly
   echo "Read RPS: $MCP_RATE_LIMIT_READ_RPS"
   echo "Write RPS: $MCP_RATE_LIMIT_WRITE_RPS"
   ```

### Issue: High Memory Usage

**Symptoms**:
- High memory consumption by rate limiter
- Many active buckets
- Memory warnings

**Diagnosis**:
```bash
# Check active bucket count
curl http://localhost:8080/metrics | grep rate_limit_buckets_active

# Estimate memory usage
# active_buckets × 200 bytes
```

**Solutions**:

1. **Reduce cleanup interval**:
   ```bash
   export MCP_RATE_LIMIT_CLEANUP_INTERVAL_SECS=60  # More frequent cleanup
   ```

2. **Implement bucket limit**:
   ```rust
   // Maximum number of active buckets
   const MAX_BUCKETS: usize = 10000;

   if buckets.len() >= MAX_BUCKETS {
       // Remove oldest bucket
       remove_oldest_bucket();
   }
   ```

3. **Use per-IP limits instead of per-client**:
   ```bash
   # Fewer unique IPs than unique client IDs
   export MCP_RATE_LIMIT_CLIENT_ID_HEADER=X-Forwarded-For
   ```

4. **Monitor and alert on bucket count**:
   ```bash
   # Alert if bucket count exceeds threshold
   if bucket_count > 10000 {
       alert!("Too many active rate limit buckets");
   }
   ```

## Best Practices

1. **Always enable rate limiting in production** to prevent DoS attacks
2. **Set appropriate limits** based on your capacity and use case
3. **Monitor rate limit metrics** to detect abuse and tuning needs
4. **Use separate read/write limits** to reflect operation costs
5. **Configure burst allowance** for legitimate traffic spikes
6. **Implement cleanup** to prevent memory exhaustion
7. **Add rate limit headers** to responses for client visibility
8. **Test rate limiting** before production deployment
9. **Document limits** in API documentation
10. **Provide backoff guidance** in rate limit error responses

## Example Configurations

### Development Environment

```bash
# Disable or set very high limits for development
export MCP_RATE_LIMIT_ENABLED=false
# OR
export MCP_RATE_LIMIT_READ_RPS=10000
export MCP_RATE_LIMIT_WRITE_RPS=1000
```

### Staging Environment

```bash
# Moderate limits to test rate limiting behavior
export MCP_RATE_LIMIT_ENABLED=true
export MCP_RATE_LIMIT_READ_RPS=50
export MCP_RATE_LIMIT_WRITE_RPS=10
```

### Production API

```bash
# Production limits based on capacity
export MCP_RATE_LIMIT_ENABLED=true
export MCP_RATE_LIMIT_READ_RPS=100
export MCP_RATE_LIMIT_WRITE_RPS=20
export MCP_RATE_LIMIT_READ_BURST=150
export MCP_RATE_LIMIT_WRITE_BURST=30
```

### High-Traffic Service

```bash
# High limits for high-traffic scenarios
export MCP_RATE_LIMIT_ENABLED=true
export MCP_RATE_LIMIT_READ_RPS=1000
export MCP_RATE_LIMIT_WRITE_RPS=200
export MCP_RATE_LIMIT_READ_BURST=1500
export MCP_RATE_LIMIT_WRITE_BURST=300
```

## Related Documentation

- [SECURITY_OPERATIONS_GUIDE.md](./SECURITY_OPERATIONS_GUIDE.md) - Main security operations guide
- [AUDIT_LOGGING_SETUP.md](./AUDIT_LOGGING_SETUP.md) - Audit logging configuration
- [INCIDENT_RESPONSE.md](./INCIDENT_RESPONSE.md) - Incident response procedures
- [SECURITY_MONITORING.md](./SECURITY_MONITORING.md) - Monitoring and alerting setup

---

**Document Version**: 1.0
**Last Updated**: 2026-02-01
