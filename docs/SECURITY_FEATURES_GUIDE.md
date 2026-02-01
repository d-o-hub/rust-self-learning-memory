# Security Features Guide - Audit Logging & Rate Limiting

**Version**: v0.1.13+  
**Last Updated**: 2026-02-01  
**Audience**: System Administrators, DevOps, Security Teams

---

## Overview

This guide covers two critical security features added in Phase 3:

1. **Audit Logging** - Comprehensive logging for security incident investigation
2. **Rate Limiting** - DoS prevention using token bucket algorithm

Both features are production-ready and designed for enterprise security requirements.

---

## Quick Start

### Enable Both Features

```bash
# Audit Logging
export AUDIT_LOG_ENABLED=true
export AUDIT_LOG_DESTINATION=file
export AUDIT_LOG_FILE_PATH=/var/log/memory-mcp/audit.log

# Rate Limiting
export MCP_RATE_LIMIT_ENABLED=true
export MCP_RATE_LIMIT_READ_RPS=100
export MCP_RATE_LIMIT_WRITE_RPS=20

# Start MCP server
memory-mcp server
```

---

## Part 1: Audit Logging

### What is Audit Logging?

Audit logging provides immutable, structured logs of all security-relevant operations
for incident investigation, compliance, and forensics.

### Key Features

- **Structured JSON format** - Machine-readable, easy to parse
- **25+ operation types** - Episodes, tags, patterns, relationships, config, auth
- **Automatic redaction** - Sensitive fields (passwords, tokens) automatically masked
- **Flexible destinations** - File, stdout, or both
- **Log rotation** - Automatic rotation based on size/age
- **Zero performance impact** - Async logging, non-blocking

### Logged Operations

#### Episode Operations
- `create_episode` - New episode creation
- `modify_episode` - Episode updates
- `delete_episode` - Episode deletion
- `add_episode_step` - Step additions
- `complete_episode` - Episode completion

#### Relationship Operations
- `add_relationship` - Creating episode relationships
- `remove_relationship` - Deleting relationships
- `modify_relationship` - Updating relationship metadata

#### Tag Operations
- `add_episode_tags` - Adding tags to episodes
- `remove_episode_tags` - Removing tags
- `set_episode_tags` - Replacing all tags
- `search_episodes_by_tags` - Tag-based searches

#### Pattern Operations
- `analyze_patterns` - Pattern analysis runs
- `advanced_pattern_analysis` - Advanced analysis
- `search_patterns` - Pattern searches
- `recommend_patterns` - Pattern recommendations

#### Configuration & Security
- `config_change` - Configuration modifications
- `embedding_config_change` - Embedding config updates
- `authentication` - Auth events (success/failure)
- `rate_limit_violation` - Rate limit exceeded events
- `security_violation` - Security policy violations

#### Query Operations
- `query_memory` - Memory queries
- `query_semantic_memory` - Semantic searches
- `bulk_episodes` - Bulk episode operations
- `batch_execute` - Batch execution events


### Configuration

#### Environment Variables

```bash
# Enable/disable audit logging
AUDIT_LOG_ENABLED=true

# Log destination: stdout, file, or both
AUDIT_LOG_DESTINATION=file

# Log file path (only if destination is file or both)
AUDIT_LOG_FILE_PATH=/var/log/memory-mcp/audit.log

# Enable log rotation
AUDIT_LOG_ENABLE_ROTATION=true

# Max file size before rotation (in bytes)
AUDIT_LOG_MAX_FILE_SIZE=10485760  # 10MB

# Number of rotated files to keep
AUDIT_LOG_MAX_ROTATED_FILES=10

# Minimum log level: debug, info, warn, error
AUDIT_LOG_LEVEL=info

# Comma-separated list of fields to redact
AUDIT_LOG_REDACT_FIELDS=password,token,secret,api_key,private_key
```

#### Programmatic Configuration

```rust
use memory_mcp::server::audit::{AuditConfig, AuditDestination, AuditLogLevel};

let config = AuditConfig {
    enabled: true,
    destination: AuditDestination::File,
    file_path: Some("/var/log/memory-mcp/audit.log".into()),
    enable_rotation: true,
    max_file_size_bytes: 10 * 1024 * 1024, // 10MB
    max_rotated_files: 10,
    level: AuditLogLevel::Info,
    redact_fields: vec![
        "password".to_string(),
        "token".to_string(),
        "api_key".to_string(),
    ],
};
```

### Log Format

Each audit log entry is a single-line JSON object:

```json
{
  "timestamp": "2026-02-01T10:30:45.123Z",
  "level": "info",
  "client_id": "client-abc123",
  "operation": "create_episode",
  "result": "success",
  "metadata": {
    "episode_id": "550e8400-e29b-41d4-a716-446655440000",
    "task_description": "Implement new feature",
    "duration_ms": 45
  }
}
```

#### Field Descriptions

- **timestamp**: ISO 8601 UTC timestamp
- **level**: Log level (debug, info, warn, error)
- **client_id**: Identifier for the client making the request
- **operation**: Type of operation (see list above)
- **result**: Operation result (success, failure, partial)
- **metadata**: Operation-specific details (varies by operation type)

### Usage Examples

#### Example 1: Query Recent Episode Operations

```bash
# Get all episode creations in the last hour
grep '"operation":"create_episode"' /var/log/memory-mcp/audit.log | \
  jq 'select(.timestamp > "2026-02-01T09:00:00Z")'
```

#### Example 2: Find Failed Authentication Attempts

```bash
# Find all failed auth attempts
grep '"operation":"authentication"' /var/log/memory-mcp/audit.log | \
  jq 'select(.result == "failure")' | \
  jq -s 'group_by(.client_id) | map({client: .[0].client_id, count: length})'
```

#### Example 3: Audit Trail for Specific Episode

```bash
# Get all operations for a specific episode
EPISODE_ID="550e8400-e29b-41d4-a716-446655440000"
grep "$EPISODE_ID" /var/log/memory-mcp/audit.log | jq .
```

#### Example 4: Security Violations Report

```bash
# Count security violations by client
grep '"operation":"security_violation"' /var/log/memory-mcp/audit.log | \
  jq -s 'group_by(.client_id) | 
         map({client: .[0].client_id, 
              violations: length, 
              operations: [.[].metadata.violation_type]})' | \
  jq 'sort_by(.violations) | reverse'
```

### Log Rotation

When rotation is enabled:
1. When log file reaches `AUDIT_LOG_MAX_FILE_SIZE`, it's rotated
2. Current file becomes `audit.log.1`
3. Previous `audit.log.1` becomes `audit.log.2`, etc.
4. Oldest file beyond `AUDIT_LOG_MAX_ROTATED_FILES` is deleted
5. New events go to fresh `audit.log`

### Best Practices

#### 1. Secure Log Storage
```bash
# Set restrictive permissions
chmod 600 /var/log/memory-mcp/audit.log
chown memory-mcp:memory-mcp /var/log/memory-mcp/audit.log

# Use dedicated partition to prevent disk exhaustion
mount /dev/sdb1 /var/log/memory-mcp
```

#### 2. Ship Logs to SIEM

```bash
# Example: Filebeat configuration for Elasticsearch
filebeat.inputs:
- type: log
  enabled: true
  paths:
    - /var/log/memory-mcp/audit.log
  json.keys_under_root: true
  json.add_error_key: true

output.elasticsearch:
  hosts: ["elasticsearch:9200"]
  index: "memory-mcp-audit-%{+yyyy.MM.dd}"
```

#### 3. Regular Log Analysis

```bash
# Daily audit report cron job
0 0 * * * /usr/local/bin/generate-audit-report.sh

# generate-audit-report.sh
#!/bin/bash
LOG_FILE="/var/log/memory-mcp/audit.log"
YESTERDAY=$(date -d "yesterday" +%Y-%m-%d)

echo "Audit Report for $YESTERDAY"
echo "============================"
echo ""
echo "Total Operations:"
grep "$YESTERDAY" $LOG_FILE | wc -l
echo ""
echo "Failed Operations:"
grep "$YESTERDAY" $LOG_FILE | grep '"result":"failure"' | wc -l
echo ""
echo "Top 10 Clients by Activity:"
grep "$YESTERDAY" $LOG_FILE | \
  jq -r .client_id | sort | uniq -c | sort -rn | head -10
```

#### 4. Alerting on Suspicious Activity

```bash
# Monitor for unusual patterns
while inotifywait -e modify /var/log/memory-mcp/audit.log; do
  # Alert if >10 failed auth from same client in 1 minute
  FAILED_AUTHS=$(tail -1000 audit.log | \
    grep '"operation":"authentication"' | \
    grep '"result":"failure"' | \
    jq -s 'group_by(.client_id) | map(select(length > 10))')
  
  if [ -n "$FAILED_AUTHS" ]; then
    echo "ALERT: Multiple failed auth attempts" | \
      mail -s "Security Alert" security@example.com
  fi
done
```

---

## Part 2: Rate Limiting

### What is Rate Limiting?

Rate limiting prevents DoS attacks by restricting the number of requests
a client can make in a given time period.

### Key Features

- **Token bucket algorithm** - Industry-standard, allows bursts
- **Per-client isolation** - Each client has independent limits
- **Read/write separation** - Different limits for read vs write operations
- **Automatic cleanup** - Inactive clients removed from memory
- **Standard headers** - `X-RateLimit-*` and `Retry-After` headers
- **Zero config** - Works with sensible defaults

### Configuration

#### Environment Variables

```bash
# Enable/disable rate limiting
MCP_RATE_LIMIT_ENABLED=true

# Read operations (queries, list, status checks)
MCP_RATE_LIMIT_READ_RPS=100        # Requests per second
MCP_RATE_LIMIT_READ_BURST=150      # Burst allowance

# Write operations (create, update, delete)
MCP_RATE_LIMIT_WRITE_RPS=20        # Requests per second
MCP_RATE_LIMIT_WRITE_BURST=30      # Burst allowance

# Cleanup interval (remove inactive clients)
MCP_RATE_LIMIT_CLEANUP_INTERVAL_SECS=60

# Custom client ID header (default: X-Client-ID)
MCP_RATE_LIMIT_CLIENT_ID_HEADER=X-Client-ID
```

### Default Limits

| Operation Type | RPS | Burst | Use Case |
|---------------|-----|-------|----------|
| **Read** | 100 | 150 | Queries, lists, health checks |
| **Write** | 20 | 30 | Creates, updates, deletes |

### Response Headers

When rate limiting is active, every response includes:

```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 87
X-RateLimit-Reset: 1706781045
```

When rate limit is exceeded:

```http
HTTP/1.1 429 Too Many Requests
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1706781050
Retry-After: 5

{
  "jsonrpc": "2.0",
  "id": 123,
  "error": {
    "code": -32000,
    "message": "Rate limit exceeded",
    "data": {
      "retry_after": 5,
      "limit": 100,
      "remaining": 0
    }
  }
}
```

### Operation Classification

#### Read Operations (Higher Limits)
- `initialize` - Server initialization
- `tools/list` - List available tools
- `resources/list` - List resources
- `prompts/list` - List prompts
- `health/check` - Health checks
- `health/status` - Status queries
- Memory queries and searches

#### Write Operations (Lower Limits)
- `tools/call` - Execute tools
- `batch/execute` - Batch operations
- Episode creation/modification/deletion
- Relationship management
- Tag operations
- Configuration changes

### Client Identification

Clients are identified by (in order of preference):
1. `X-Client-ID` header (if provided)
2. Source IP address
3. "unknown" (fallback)

```bash
# Example: Set custom client ID
curl -H "X-Client-ID: service-a" \
     http://localhost:3000/memory/query
```

### Tuning Recommendations

#### Light Load (Single User/Dev)
```bash
MCP_RATE_LIMIT_READ_RPS=1000
MCP_RATE_LIMIT_READ_BURST=1500
MCP_RATE_LIMIT_WRITE_RPS=200
MCP_RATE_LIMIT_WRITE_BURST=300
```

#### Moderate Load (Team/Internal)
```bash
MCP_RATE_LIMIT_READ_RPS=100   # Default
MCP_RATE_LIMIT_READ_BURST=150
MCP_RATE_LIMIT_WRITE_RPS=20
MCP_RATE_LIMIT_WRITE_BURST=30
```

#### Heavy Load (Public API)
```bash
MCP_RATE_LIMIT_READ_RPS=50
MCP_RATE_LIMIT_READ_BURST=75
MCP_RATE_LIMIT_WRITE_RPS=10
MCP_RATE_LIMIT_WRITE_BURST=15
```

#### Conservative (High Security)
```bash
MCP_RATE_LIMIT_READ_RPS=10
MCP_RATE_LIMIT_READ_BURST=15
MCP_RATE_LIMIT_WRITE_RPS=1
MCP_RATE_LIMIT_WRITE_BURST=2
```

### Monitoring Rate Limits

#### Check Client Rate Limit Status

The `X-RateLimit-*` headers show current status:

```bash
curl -v http://localhost:3000/health/check 2>&1 | grep X-RateLimit
# X-RateLimit-Limit: 100
# X-RateLimit-Remaining: 99
# X-RateLimit-Reset: 1706781100
```

#### Log Rate Limit Violations

Rate limit violations are automatically logged to audit logs:

```json
{
  "timestamp": "2026-02-01T10:35:22.456Z",
  "level": "warn",
  "client_id": "192.168.1.100",
  "operation": "rate_limit_violation",
  "result": "rejected",
  "metadata": {
    "operation_type": "write",
    "limit": 20,
    "attempted_request": "create_episode",
    "retry_after_seconds": 3
  }
}
```

### Best Practices

#### 1. Implement Client-Side Rate Limiting

```python
import time
import requests

class RateLimitedClient:
    def __init__(self, base_url, client_id):
        self.base_url = base_url
        self.headers = {"X-Client-ID": client_id}
    
    def request(self, method, path, **kwargs):
        while True:
            response = requests.request(
                method, f"{self.base_url}{path}",
                headers=self.headers, **kwargs
            )
            
            if response.status_code == 429:
                retry_after = int(response.headers.get("Retry-After", 5))
                time.sleep(retry_after)
                continue
            
            return response

client = RateLimitedClient("http://localhost:3000", "my-service")
response = client.request("POST", "/episode/create", json={...})
```

#### 2. Use Exponential Backoff

```javascript
async function requestWithBackoff(url, options, maxRetries = 3) {
  for (let i = 0; i < maxRetries; i++) {
    const response = await fetch(url, options);
    
    if (response.status !== 429) {
      return response;
    }
    
    const retryAfter = response.headers.get('Retry-After') || Math.pow(2, i);
    await sleep(retryAfter * 1000);
  }
  
  throw new Error('Rate limit exceeded after max retries');
}
```

#### 3. Monitor Rate Limit Metrics

```bash
# Daily rate limit report
grep '"operation":"rate_limit_violation"' /var/log/memory-mcp/audit.log | \
  jq -s 'group_by(.client_id) | 
         map({
           client: .[0].client_id, 
           violations: length,
           operations: [.[].metadata.attempted_request] | unique
         })' | \
  jq 'sort_by(.violations) | reverse'
```

#### 4. Alert on Excessive Rate Limiting

```bash
# Alert if any client exceeds 100 rate limit violations per hour
THRESHOLD=100
VIOLATIONS=$(grep '"operation":"rate_limit_violation"' audit.log | \
  grep "$(date -u +%Y-%m-%dT%H)" | \
  jq -s 'group_by(.client_id) | map(select(length > '$THRESHOLD'))')

if [ -n "$VIOLATIONS" ]; then
  echo "$VIOLATIONS" | mail -s "Rate Limit Alert" ops@example.com
fi
```

---

## Production Deployment

### Checklist

#### Audit Logging
- [ ] Enable audit logging in production
- [ ] Configure file-based logging (not stdout)
- [ ] Set up log rotation
- [ ] Secure log file permissions (600)
- [ ] Ship logs to SIEM/log aggregation
- [ ] Set up daily audit reports
- [ ] Configure alerts for security violations
- [ ] Test log parsing and queries

#### Rate Limiting
- [ ] Enable rate limiting in production
- [ ] Tune limits based on expected load
- [ ] Configure client identification (IP or custom header)
- [ ] Set up monitoring for rate limit violations
- [ ] Implement client-side retry logic
- [ ] Test rate limits under load
- [ ] Document limits for API consumers
- [ ] Configure alerts for excessive violations

### Example Production Configuration

```bash
# /etc/memory-mcp/security.env

# Audit Logging - Production Settings
AUDIT_LOG_ENABLED=true
AUDIT_LOG_DESTINATION=file
AUDIT_LOG_FILE_PATH=/var/log/memory-mcp/audit.log
AUDIT_LOG_ENABLE_ROTATION=true
AUDIT_LOG_MAX_FILE_SIZE=52428800        # 50MB
AUDIT_LOG_MAX_ROTATED_FILES=30          # 30 days retention
AUDIT_LOG_LEVEL=info
AUDIT_LOG_REDACT_FIELDS=password,token,secret,api_key,private_key,auth_token

# Rate Limiting - Production Settings
MCP_RATE_LIMIT_ENABLED=true
MCP_RATE_LIMIT_READ_RPS=100
MCP_RATE_LIMIT_READ_BURST=150
MCP_RATE_LIMIT_WRITE_RPS=20
MCP_RATE_LIMIT_WRITE_BURST=30
MCP_RATE_LIMIT_CLEANUP_INTERVAL_SECS=300
MCP_RATE_LIMIT_CLIENT_ID_HEADER=X-Client-ID
```

### Systemd Service Integration

```ini
# /etc/systemd/system/memory-mcp.service

[Unit]
Description=Memory MCP Server with Security Features
After=network.target

[Service]
Type=simple
User=memory-mcp
Group=memory-mcp
EnvironmentFile=/etc/memory-mcp/security.env
ExecStart=/usr/local/bin/memory-mcp server
Restart=always
RestartSec=10

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/memory-mcp /var/lib/memory-mcp

[Install]
WantedBy=multi-user.target
```

---

## Troubleshooting

### Audit Logging Issues

#### Problem: No audit logs generated
```bash
# Check if logging is enabled
grep AUDIT_LOG_ENABLED /etc/memory-mcp/security.env

# Check file permissions
ls -la /var/log/memory-mcp/audit.log

# Check disk space
df -h /var/log/memory-mcp

# Check server logs for errors
journalctl -u memory-mcp | grep audit
```

#### Problem: Log rotation not working
```bash
# Verify rotation config
echo $AUDIT_LOG_ENABLE_ROTATION
echo $AUDIT_LOG_MAX_FILE_SIZE

# Check for rotated files
ls -lh /var/log/memory-mcp/audit.log*

# Manual rotation test
kill -HUP $(pidof memory-mcp)
```

### Rate Limiting Issues

#### Problem: All requests being rate limited
```bash
# Check if limits are too restrictive
echo $MCP_RATE_LIMIT_READ_RPS
echo $MCP_RATE_LIMIT_WRITE_RPS

# Check for client ID issues
curl -v http://localhost:3000/health | grep X-Client

# Temporarily disable to test
MCP_RATE_LIMIT_ENABLED=false memory-mcp server
```

#### Problem: Rate limits not being enforced
```bash
# Verify rate limiting is enabled
grep MCP_RATE_LIMIT_ENABLED /etc/memory-mcp/security.env

# Check server logs
journalctl -u memory-mcp | grep "rate limit"

# Test with curl
for i in {1..150}; do 
  curl -w "%{http_code}\n" http://localhost:3000/health/check
done
# Should see 429 responses
```

---

## FAQ

### Q: Does audit logging impact performance?
**A**: Minimal impact (<1%). Logging is async and non-blocking.

### Q: Can I disable audit logging for specific operations?
**A**: Not currently. All security-relevant operations are logged for compliance.

### Q: How do I rotate logs manually?
**A**: Send SIGHUP to the process: `kill -HUP $(pidof memory-mcp)`

### Q: Can rate limits be different per client?
**A**: Not currently. All clients share the same limits. Use a reverse proxy
for per-client custom limits.

### Q: What happens when rate limit is hit?
**A**: Request is rejected with 429 status, client should retry after `Retry-After` seconds.

### Q: Can I exempt certain clients from rate limiting?
**A**: Not built-in. Use a reverse proxy to bypass rate limiting for trusted IPs.

### Q: Are rate limits persistent across restarts?
**A**: No. Rate limit state is in-memory and resets on restart.

---

## Support & Resources

### Documentation
- Implementation Summary: `plans/AUDIT_LOGGING.md`
- Rate Limiting Summary: `plans/rate_limiting_implementation_summary.md`
- Phase 4 Roadmap: `plans/PHASE4_IMPLEMENTATION_PLAN.md`

### Code References
- Audit Logger: `memory-mcp/src/server/audit/`
- Rate Limiter: `memory-mcp/src/server/rate_limiter.rs`
- Integration Tests: `memory-mcp/tests/audit_tests.rs`, `memory-mcp/tests/rate_limiter_integration.rs`

### Support Channels
- GitHub Issues: For bug reports and feature requests
- Documentation: For implementation guides and examples
- Security Issues: Report privately to security@example.com

---

**Document Version**: 1.0  
**Last Updated**: 2026-02-01  
**Maintained By**: Development Team
