# Audit Logging Setup Guide

**Version**: 1.0
**Last Updated**: 2026-02-01
**Status**: Production Ready

## Overview

This guide provides detailed instructions for configuring and managing audit logging in the rust-self-learning-memory system. Audit logging tracks all security-relevant events for compliance and incident investigation.

## Table of Contents

- [Quick Start](#quick-start)
- [Configuration Options](#configuration-options)
- [Environment Variables](#environment-variables)
- [Log Levels](#log-levels)
- [Output Destinations](#output-destinations)
- [Log Rotation & Retention](#log-rotation--retention)
- [Audit Log Entries](#audit-log-entries)
- [Performance Considerations](#performance-considerations)
- [Troubleshooting](#troubleshooting)

## Quick Start

### Enable Audit Logging

```bash
# Minimum configuration
export MEMORY_AUDIT_ENABLED=true
export MEMORY_AUDIT_LEVEL=info
export MEMORY_AUDIT_OUTPUT=stdout

# Recommended production configuration
export MEMORY_AUDIT_ENABLED=true
export MEMORY_AUDIT_LEVEL=info
export MEMORY_AUDIT_OUTPUT=file
export MEMORY_AUDIT_FILE=/var/log/memory/audit.log
export MEMORY_AUDIT_RETENTION_DAYS=90
```

### Verify Audit Logging

```bash
# Start the service
sudo systemctl restart memory-service

# Check logs
tail -f /var/log/memory/audit.log

# Verify audit events are being logged
grep "audit" /var/log/memory/audit.log | tail -10
```

## Configuration Options

### Basic Configuration

```rust
use memory_core::security::audit::{AuditConfig, AuditLogger, AuditOutput, AuditLogLevel};

// Manual configuration
let config = AuditConfig {
    enabled: true,
    log_level: AuditLogLevel::Info,
    output_destination: AuditOutput::File("/var/log/memory/audit.log".to_string()),
    retention_days: 90,
    include_state_changes: true,
    include_ip_address: true,
    include_session_info: true,
    buffer_size: 1000,
};

let logger = AuditLogger::new(config);
```

### Environment Variable Configuration

All audit logging settings can be configured via environment variables:

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `MEMORY_AUDIT_ENABLED` | boolean | `false` | Enable/disable audit logging |
| `MEMORY_AUDIT_LEVEL` | string | `info` | Minimum log level to record |
| `MEMORY_AUDIT_OUTPUT` | string | `stdout` | Output destination |
| `MEMORY_AUDIT_FILE` | path | `audit.log` | File path when output=file |
| `MEMORY_AUDIT_RETENTION_DAYS` | integer | `90` | Log retention period in days |

```bash
# Example .env file
MEMORY_AUDIT_ENABLED=true
MEMORY_AUDIT_LEVEL=info
MEMORY_AUDIT_OUTPUT=file
MEMORY_AUDIT_FILE=/var/log/memory/audit.log
MEMORY_AUDIT_RETENTION_DAYS=90
```

## Environment Variables

### MEMORY_AUDIT_ENABLED

Enable or disable audit logging.

**Values**: `true`, `false`, `1`, `0`, `yes`, `no`, `on`, `off`

**Examples**:
```bash
# Enable audit logging
export MEMORY_AUDIT_ENABLED=true

# Disable audit logging
export MEMORY_AUDIT_ENABLED=false
```

**Recommendation**: Enable in production environments for compliance and security monitoring.

### MEMORY_AUDIT_LEVEL

Set the minimum log level to record.

**Values**: `debug`, `info`, `warn`, `error`, `critical`

**Examples**:
```bash
# Production: Info level and above
export MEMORY_AUDIT_LEVEL=info

# Development: Debug level for detailed logs
export MEMORY_AUDIT_LEVEL=debug

# Security-focused: Only errors and critical events
export MEMORY_AUDIT_LEVEL=error
```

**Log Level Hierarchy**:
```
DEBUG < INFO < WARN < ERROR < CRITICAL
```

When set to `INFO`, the following are logged:
- ✅ Info events
- ✅ Warning events
- ✅ Error events
- ✅ Critical events
- ❌ Debug events (filtered out)

### MEMORY_AUDIT_OUTPUT

Specify where audit logs should be written.

**Values**: `stdout`, `stderr`, `file`, `none`, `disabled`

**Examples**:
```bash
# Write to standard output (useful for containerized environments)
export MEMORY_AUDIT_OUTPUT=stdout

# Write to standard error (separates audit from application logs)
export MEMORY_AUDIT_OUTPUT=stderr

# Write to a file
export MEMORY_AUDIT_OUTPUT=file
export MEMORY_AUDIT_FILE=/var/log/memory/audit.log

# Disable audit output (for testing)
export MEMORY_AUDIT_OUTPUT=none
```

### MEMORY_AUDIT_FILE

Specify the file path when `MEMORY_AUDIT_OUTPUT=file`.

**Default**: `audit.log`

**Examples**:
```bash
# Linux/Unix
export MEMORY_AUDIT_FILE=/var/log/memory/audit.log

# Container path
export MEMORY_AUDIT_FILE=/logs/audit.log

# Daily rotated files
export MEMORY_AUDIT_FILE=/var/log/memory/audit-$(date +%Y%m%d).log
```

**Best Practices**:
- Use a dedicated log directory
- Ensure the directory exists before starting the service
- Set appropriate file permissions (640 or 600)
- Use log rotation to prevent disk space exhaustion

### MEMORY_AUDIT_RETENTION_DAYS

Set the retention period for audit logs in days.

**Default**: `90`

**Examples**:
```bash
# 90 days (SOC 2 compliance minimum)
export MEMORY_AUDIT_RETENTION_DAYS=90

# 1 year for extended compliance
export MEMORY_AUDIT_RETENTION_DAYS=365

# 30 days for reduced storage
export MEMORY_AUDIT_RETENTION_DAYS=30
```

**Note**: This setting is for documentation purposes. Actual log retention should be implemented via log rotation policies (see [Log Rotation & Retention](#log-rotation--retention)).

## Log Levels

### Debug

**Purpose**: Detailed diagnostic information for troubleshooting.

**Use Cases**:
- Development and debugging
- Performance analysis
- Detailed request/response logging

**Example Events**:
- Routine operations
- Cache hits/misses
- Internal function calls

**Configuration**:
```bash
export MEMORY_AUDIT_LEVEL=debug
```

**Caution**: Debug logging can generate significant volume and should only be used temporarily for troubleshooting.

### Info

**Purpose**: General informational events about normal operations.

**Use Cases**:
- Production monitoring
- Operational awareness
- Routine activity tracking

**Example Events**:
- Episode created/completed
- Relationships added/removed
- Tags modified
- Patterns extracted

**Configuration**:
```bash
export MEMORY_AUDIT_LEVEL=info
```

**Recommendation**: This is the recommended level for production environments.

### Warn

**Purpose**: Warning conditions that may indicate issues.

**Use Cases**:
- Anomaly detection
- Configuration changes
- Resource depletion warnings

**Example Events**:
- Episode deleted
- Configuration changed
- Rate limit violations
- Access denied

**Configuration**:
```bash
export MEMORY_AUDIT_LEVEL=warn
```

### Error

**Purpose**: Error conditions that prevent operations from completing.

**Use Cases**:
- Error tracking
- Failure analysis
- System health monitoring

**Example Events**:
- Storage operation failures
- Database connection errors
- Serialization errors

**Configuration**:
```bash
export MEMORY_AUDIT_LEVEL=error
```

### Critical

**Purpose**: Critical security events that require immediate attention.

**Use Cases**:
- Security incident detection
- Forensic investigation
- Compliance monitoring

**Example Events**:
- Access denied
- Authentication failures
- Security violations

**Configuration**:
```bash
export MEMORY_AUDIT_LEVEL=critical
```

**Recommendation**: Always log critical events, regardless of the configured log level.

## Output Destinations

### Standard Output (stdout)

**Use Case**: Containerized environments, log aggregation systems.

**Configuration**:
```bash
export MEMORY_AUDIT_OUTPUT=stdout
```

**Integration with Docker**:
```yaml
# docker-compose.yml
version: '3.8'
services:
  memory-service:
    image: memory-service:latest
    environment:
      - MEMORY_AUDIT_OUTPUT=stdout
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

**Integration with Kubernetes**:
```yaml
# deployment.yaml
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: memory-service
    image: memory-service:latest
    env:
    - name: MEMORY_AUDIT_OUTPUT
      value: "stdout"
```

### Standard Error (stderr)

**Use Case**: Separating audit logs from application logs.

**Configuration**:
```bash
export MEMORY_AUDIT_OUTPUT=stderr
```

**Benefit**: Allows separate handling of audit logs and application logs in logging systems.

### File Output

**Use Case**: Traditional logging to filesystem.

**Configuration**:
```bash
export MEMORY_AUDIT_OUTPUT=file
export MEMORY_AUDIT_FILE=/var/log/memory/audit.log
```

**Systemd Configuration**:
```ini
[Service]
Environment="MEMORY_AUDIT_OUTPUT=file"
Environment="MEMORY_AUDIT_FILE=/var/log/memory/audit.log"
Environment="MEMORY_AUDIT_RETENTION_DAYS=90"

# Log rotation via logrotate
[Install]
WantedBy=multi-user.target
```

**Create log directory**:
```bash
sudo mkdir -p /var/log/memory
sudo chown memory:memory /var/log/memory
sudo chmod 750 /var/log/memory
```

### Multiple Destinations

**Use Case**: Write to multiple outputs simultaneously.

**Configuration** (programmatic):
```rust
use memory_core::security::audit::{AuditConfig, AuditOutput};

let config = AuditConfig {
    enabled: true,
    output_destination: AuditOutput::Multiple(vec![
        AuditOutput::Stdout,
        AuditOutput::File("/var/log/memory/audit.log".to_string()),
    ]),
    ..Default::default()
};
```

## Log Rotation & Retention

### Logrotate Configuration

Create `/etc/logrotate.d/memory-service`:

```conf
/var/log/memory/audit.log {
    daily
    rotate 90
    missingok
    notifempty
    compress
    delaycompress
    copytruncate
    create 0640 memory memory
    dateext
    dateformat -%Y%m%d
    maxage 90
}
```

**Explanation**:
- `daily`: Rotate logs daily
- `rotate 90`: Keep 90 days of rotated logs
- `compress`: Compress rotated logs with gzip
- `delaycompress`: Compress the next day (not immediately)
- `copytruncate`: Copy log file and truncate original (allows continuous logging)
- `maxage 90`: Delete logs older than 90 days
- `dateext`: Use date extension for rotated files

**Test logrotate**:
```bash
# Dry run (no changes made)
sudo logrotate -d /etc/logrotate.d/memory-service

# Force rotation
sudo logrotate -f /etc/logrotate.d/memory-service
```

### Log Retention by Compliance Requirements

| Compliance | Retention Period | Rotation Frequency |
|------------|------------------|-------------------|
| SOC 2 | 90 days minimum | Daily |
| PCI DSS | 1 year | Daily |
| HIPAA | 6 years | Daily |
| GDPR | Not specified (reasonable period) | Daily |

### Manual Log Archival

```bash
#!/bin/bash
# Archive audit logs older than 30 days

LOG_DIR="/var/log/memory"
ARCHIVE_DIR="/var/log/memory/archive"
DATE=$(date +%Y%m)

# Create archive directory
mkdir -p "$ARCHIVE_DIR"

# Find and compress logs older than 30 days
find "$LOG_DIR" -name "audit.log.*" -mtime +30 -exec gzip {} \;

# Move compressed logs to archive
find "$LOG_DIR" -name "audit.log.*.gz" -exec mv {} "$ARCHIVE_DIR/" \;

# Delete archives older than retention period
find "$ARCHIVE_DIR" -name "audit.log.*.gz" -mtime +365 -delete

echo "Log archival completed: $(date)"
```

## Audit Log Entries

### Entry Structure

Every audit log entry is a JSON object with the following structure:

```json
{
  "entry_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2026-02-01T12:34:56.789Z",
  "event_type": "episode_created",
  "level": "INFO",
  "actor": "user:alice@example.com",
  "resource_id": "ep-12345678-1234-1234-1234-123456789abc",
  "details": {
    "task_description": "Process customer data",
    "task_type": "data_processing"
  },
  "before_state": null,
  "after_state": null,
  "result": "Success",
  "ip_address": "192.168.1.100",
  "session_id": "sess-abc123",
  "metadata": {
    "request_id": "req-xyz789",
    "user_agent": "memory-cli/1.0.0"
  }
}
```

### Event Types

#### Episode Lifecycle Events

```json
{
  "event_type": "episode_created",
  "level": "INFO",
  "resource_id": "ep-12345678-1234-1234-1234-123456789abc",
  "details": {
    "task_description": "Process customer data",
    "task_type": "data_processing",
    "complexity": "moderate"
  }
}
```

```json
{
  "event_type": "episode_completed",
  "level": "INFO",
  "resource_id": "ep-12345678-1234-1234-1234-123456789abc",
  "details": {
    "outcome": "Successfully processed 1000 records",
    "success": true,
    "duration_ms": 1234
  }
}
```

```json
{
  "event_type": "episode_deleted",
  "level": "WARN",
  "resource_id": "ep-12345678-1234-1234-1234-123456789abc",
  "details": {
    "reason": "User requested deletion",
    "deleted_at": "2026-02-01T12:34:56.789Z"
  }
}
```

#### Security Events

```json
{
  "event_type": "access_denied",
  "level": "CRITICAL",
  "resource_id": "ep-12345678-1234-1234-1234-123456789abc",
  "details": {
    "action": "delete",
    "reason": "insufficient_permissions"
  },
  "result": "Denied"
}
```

```json
{
  "event_type": "auth_failure",
  "level": "CRITICAL",
  "details": {
    "auth_type": "token",
    "error": "Invalid token",
    "ip_address": "192.168.1.100"
  },
  "result": "Failure"
}
```

```json
{
  "event_type": "security_violation",
  "level": "ERROR",
  "details": {
    "violation_type": "rate_limit_exceeded",
    "limit": 100,
    "actual": 150
  }
}
```

#### Configuration Events

```json
{
  "event_type": "config_changed",
  "level": "WARN",
  "resource_id": "max_episodes",
  "details": {
    "config_key": "max_episodes",
    "old_value": "1000",
    "new_value": "2000"
  }
}
```

## Performance Considerations

### Performance Impact

| Metric | Impact (Audit Disabled) | Impact (Audit Enabled) |
|--------|------------------------|------------------------|
| Episode Creation | ~2.5 µs | ~3.0 µs (+20%) |
| Step Logging | ~1.1 µs | ~1.3 µs (+18%) |
| Memory Overhead | 0 MB | ~5 MB (buffer) |

**Key Points**:
- Audit logging uses async buffering to minimize performance impact
- Typical overhead is <5% when enabled
- Buffer size can be tuned based on load

### Buffer Size Tuning

```rust
let config = AuditConfig {
    buffer_size: 1000,  // Default: 1000 entries
    ..Default::default()
};
```

**Guidelines**:
- **Low load** (< 10 eps): `buffer_size = 500`
- **Medium load** (10-100 eps): `buffer_size = 1000`
- **High load** (100-1000 eps): `buffer_size = 5000`

### Disk Space Planning

| Log Level | Avg Entry Size | Entries/Day (est.) | Daily Size | 90-Day Size |
|-----------|---------------|-------------------|-----------|-------------|
| CRITICAL | 500 B | 10 | 5 KB | 450 KB |
| ERROR | 500 B | 100 | 50 KB | 4.5 MB |
| WARN | 500 B | 1,000 | 500 KB | 45 MB |
| INFO | 500 B | 10,000 | 5 MB | 450 MB |
| DEBUG | 1 KB | 100,000 | 100 MB | 9 GB |

**Recommendation**: Plan for 500 MB - 1 GB for 90 days at INFO level.

## Troubleshooting

### Issue: Audit Logs Not Being Written

**Symptoms**:
- No log entries in expected location
- Service running but no audit events recorded

**Diagnosis**:
```bash
# Check if audit logging is enabled
echo $MEMORY_AUDIT_ENABLED

# Check log file path
echo $MEMORY_AUDIT_FILE

# Check file permissions
ls -l /var/log/memory/audit.log

# Check service logs for errors
journalctl -u memory-service -n 50
```

**Solutions**:

1. **Enable audit logging**:
   ```bash
   export MEMORY_AUDIT_ENABLED=true
   sudo systemctl restart memory-service
   ```

2. **Fix file permissions**:
   ```bash
   sudo touch /var/log/memory/audit.log
   sudo chown memory:memory /var/log/memory/audit.log
   sudo chmod 640 /var/log/memory/audit.log
   ```

3. **Create log directory**:
   ```bash
   sudo mkdir -p /var/log/memory
   sudo chown memory:memory /var/log/memory
   sudo chmod 750 /var/log/memory
   ```

4. **Check disk space**:
   ```bash
   df -h /var/log/memory
   ```

### Issue: Logs Not Rotating

**Symptoms**:
- Single log file growing indefinitely
- No rotated log files

**Diagnosis**:
```bash
# Check logrotate configuration
cat /etc/logrotate.d/memory-service

# Test logrotate
sudo logrotate -d /etc/logrotate.d/memory-service

# Check logrotate status
cat /var/lib/logrotate/status
```

**Solutions**:

1. **Ensure logrotate is configured**:
   ```bash
   sudo cat > /etc/logrotate.d/memory-service << 'EOF'
   /var/log/memory/audit.log {
       daily
       rotate 90
       compress
       delaycompress
       copytruncate
       missingok
       notifempty
       create 0640 memory memory
   }
   EOF
   ```

2. **Run logrotate manually**:
   ```bash
   sudo logrotate -f /etc/logrotate.d/memory-service
   ```

3. **Verify logrotate cron job**:
   ```bash
   ls -l /etc/cron.daily/logrotate
   ```

### Issue: High Disk Usage from Logs

**Symptoms**:
- Log directory consuming too much disk space
- Disk space warnings

**Diagnosis**:
```bash
# Check log directory size
du -sh /var/log/memory

# Find largest log files
du -h /var/log/memory/* | sort -rh | head -10

# Check compression status
ls -lh /var/log/memory/
```

**Solutions**:

1. **Compress old logs**:
   ```bash
   find /var/log/memory -name "audit.log.*" -mtime +1 ! -name "*.gz" -exec gzip {} \;
   ```

2. **Reduce retention period**:
   ```bash
   export MEMORY_AUDIT_RETENTION_DAYS=30
   ```

3. **Adjust logrotate configuration**:
   ```conf
   /var/log/memory/audit.log {
       daily
       rotate 30  # Reduced from 90
       compress
       delaycompress
       ...
   }
   ```

4. **Raise log level**:
   ```bash
   # From INFO to WARN (reduces volume by ~90%)
   export MEMORY_AUDIT_LEVEL=warn
   ```

### Issue: Missing Events in Logs

**Symptoms**:
- Expected events not appearing in logs
- Gaps in audit trail

**Diagnosis**:
```bash
# Check log level
echo $MEMORY_AUDIT_LEVEL

# Search for specific events
grep "episode_created" /var/log/memory/audit.log | wc -l

# Check for filtered events
grep "DEBUG" /var/log/memory/audit.log | tail -10
```

**Solutions**:

1. **Lower log level**:
   ```bash
   # From WARN to INFO
   export MEMORY_AUDIT_LEVEL=info
   ```

2. **Check for filtering**:
   ```bash
   # Ensure no grep/filtering in logging pipeline
   journalctl -u memory-service | grep audit
   ```

3. **Verify async buffer is flushing**:
   ```bash
   # Restart service to flush buffer
   sudo systemctl restart memory-service
   ```

## Best Practices

1. **Always enable audit logging in production** for compliance and security monitoring
2. **Use INFO level** for production to balance detail and volume
3. **Implement log rotation** to prevent disk space exhaustion
4. **Ship logs to SIEM** for centralized analysis and alerting
5. **Secure log files** with appropriate permissions (640 or 600)
6. **Monitor log volume** and adjust retention policies as needed
7. **Regularly test log rotation** to ensure it works correctly
8. **Archive logs periodically** for long-term compliance requirements
9. **Use structured JSON logging** for machine parsing and analysis
10. **Include context** (IP, session, user) in all audit entries

## Related Documentation

- [SECURITY_OPERATIONS_GUIDE.md](./SECURITY_OPERATIONS_GUIDE.md) - Main security operations guide
- [RATE_LIMITING_TUNING.md](./RATE_LIMITING_TUNING.md) - Rate limiting configuration
- [INCIDENT_RESPONSE.md](./INCIDENT_RESPONSE.md) - Incident response procedures
- [SECURITY_MONITORING.md](./SECURITY_MONITORING.md) - Monitoring and alerting setup

---

**Document Version**: 1.0
**Last Updated**: 2026-02-01
