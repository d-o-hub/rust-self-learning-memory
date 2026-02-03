# Security Operations Guide

**Version**: 1.0
**Last Updated**: 2026-02-01
**Status**: Production Ready

## Overview

This guide provides comprehensive operational procedures for security management in the rust-self-learning-memory system. It covers audit logging, rate limiting, incident response, monitoring, and compliance considerations.

## Table of Contents

- [Quick Reference](#quick-reference)
- [Security Architecture](#security-architecture)
- [Core Security Features](#core-security-features)
- [Operational Procedures](#operational-procedures)
- [Related Documentation](#related-documentation)

## Quick Reference

### Enable Security Features

```bash
# Audit Logging
export MEMORY_AUDIT_ENABLED=true
export MEMORY_AUDIT_LEVEL=info
export MEMORY_AUDIT_OUTPUT=file
export MEMORY_AUDIT_FILE=/var/log/memory/audit.log
export MEMORY_AUDIT_RETENTION_DAYS=90

# Rate Limiting (enabled by default)
export MCP_RATE_LIMIT_ENABLED=true
export MCP_RATE_LIMIT_READ_RPS=100
export MCP_RATE_LIMIT_WRITE_RPS=20
```

### Key Commands

```bash
# Check audit log status
grep "audit" /var/log/memory/audit.log | tail -20

# Check rate limit statistics
curl http://localhost:8080/metrics | grep rate_limit

# Monitor security events
tail -f /var/log/memory/audit.log | grep -E "(CRITICAL|ERROR|security_violation)"

# Test rate limiting
ab -n 200 -c 10 http://localhost:8080/api/episodes
```

### Emergency Response

```bash
# Emergency: Disable rate limiting
export MCP_RATE_LIMIT_ENABLED=false
sudo systemctl restart memory-service

# Emergency: Enable critical audit logging
export MEMORY_AUDIT_LEVEL=critical
export MEMORY_AUDIT_OUTPUT=stderr
sudo systemctl restart memory-service

# Emergency: Block all writes
export MCP_RATE_LIMIT_WRITE_RPS=0
sudo systemctl restart memory-service
```

## Security Architecture

### Zero-Trust Model

The memory system implements a **zero-trust security architecture**:

```
┌─────────────────────────────────────────────────────────────┐
│                      Security Layers                         │
├─────────────────────────────────────────────────────────────┤
│  1. Input Validation    │ Size limits, type checking         │
│  2. Rate Limiting       │ Token bucket per client           │
│  3. Authentication      │ Client identification             │
│  4. Authorization       │ Access control checks             │
│  5. Audit Logging       │ All operations logged             │
│  6. Data Security       │ Encryption at rest (Turso)        │
│  7. Network Security    │ TLS-only (libsql://)              │
└─────────────────────────────────────────────────────────────┘
```

### Defense in Depth

| Layer | Mechanism | Protection |
|-------|-----------|------------|
| **Perimeter** | Rate limiting | DoS prevention |
| **Application** | Input validation | Injection attacks |
| **Authorization** | Access control | Unauthorized access |
| **Monitoring** | Audit logging | Forensic analysis |
| **Data** | Encryption | Data breach mitigation |

## Core Security Features

### 1. Audit Logging

**Purpose**: Track all security-relevant events for compliance and incident investigation.

**Key Features**:
- Structured JSON logging for machine parsing
- Millisecond-precision timestamps
- User/agent identification
- Before/after state tracking for modifications
- Configurable log levels and destinations

**Documentation**: [AUDIT_LOGGING_SETUP.md](./AUDIT_LOGGING_SETUP.md)

### 2. Rate Limiting

**Purpose**: Prevent DoS attacks and ensure fair resource allocation.

**Key Features**:
- Token bucket algorithm for smooth rate limiting
- Per-client rate limiting (by IP or client ID)
- Different limits for read vs write operations
- Configurable burst allowance
- Rate limit headers in responses

**Documentation**: [RATE_LIMITING_TUNING.md](./RATE_LIMITING_TUNING.md)

### 3. Input Validation

**Purpose**: Prevent injection attacks and resource exhaustion.

**Limits**:

| Input Type | Maximum Size | Rationale |
|------------|--------------|-----------|
| Task Description | 10KB | Prevent memory exhaustion |
| Step Observation | 10KB | Prevent log flooding |
| Step Parameters | 1MB | Allow complex parameters |
| Episode Artifacts | 1MB | Prevent storage abuse |
| Episode Steps | 1,000 | Prevent infinite episodes |

### 4. Access Control

**Authentication Methods**:
- Client ID (via `X-Client-ID` header)
- IP address tracking
- Session tokens (optional)

**Authorization Checks**:
- Resource ownership validation
- Relationship permissions
- Episode access control

### 5. Data Security

**At Rest**:
- Turso: Encrypted storage (managed by Turso)
- redb cache: Local filesystem permissions

**In Transit**:
- libsql:// protocol enforces TLS encryption
- HTTP/HTTPS explicitly rejected
- Certificate validation enforced

## Operational Procedures

### Daily Operations

#### Morning Checklist

```bash
# 1. Check service health
curl http://localhost:8080/health

# 2. Review overnight security events
grep "ERROR\|CRITICAL" /var/log/memory/audit.log | grep "$(date +%Y-%m-%d)"

# 3. Check rate limit statistics
curl http://localhost:8080/metrics | grep rate_limit

# 4. Verify audit log rotation
ls -lh /var/log/memory/audit.log*
```

#### During Day Monitoring

```bash
# Monitor rate limit violations in real-time
tail -f /var/log/memory/audit.log | grep "rate_limit_violation"

# Monitor authentication failures
tail -f /var/log/memory/audit.log | grep "auth_failure"

# Monitor security violations
tail -f /var/log/memory/audit.log | grep "security_violation"
```

### Weekly Operations

#### Security Review

```bash
# 1. Generate weekly security report
grep "CRITICAL\|ERROR" /var/log/memory/audit.log | \
  grep "$(date -d '7 days ago' +%Y-%m-%d)" | \
  awk '{print $5}' | sort | uniq -c | sort -rn

# 2. Top 10 rate-limited clients
grep "rate_limit_violation" /var/log/memory/audit.log | \
  awk '{print $7}' | sort | uniq -c | sort -rn | head -10

# 3. Audit log retention check
find /var/log/memory/ -name "audit.log.*" -mtime +90 -ls
```

#### Performance Review

```bash
# 1. Check rate limit impact
curl http://localhost:8080/metrics | grep -E "(rate_limit_allowed|rate_limit_denied)"

# 2. Review audit log volume
du -sh /var/log/memory/audit.log*

# 3. Check cache hit rate
curl http://localhost:8080/metrics | grep cache_hit_rate
```

### Monthly Operations

#### Compliance Audit

```bash
# 1. Export audit logs for archival
tar -czf audit-$(date +%Y%m).tar.gz /var/log/memory/audit.log.*
gpg --encrypt audit-$(date +%Y%m).tar.gz

# 2. Verify log retention compliance
find /var/log/memory/ -name "audit.log.*" -mtime +90 -delete

# 3. Generate compliance report
# (use custom scripts or SIEM integration)
```

#### Security Update Review

```bash
# 1. Check for security advisories
cargo audit

# 2. Review dependency updates
cargo outdated

# 3. Update dependencies if needed
cargo update
cargo test
cargo build --release
```

## Incident Response Procedures

### Severity Levels

| Severity | Description | Response Time |
|----------|-------------|---------------|
| **P0 - Critical** | System compromise, data breach | < 15 minutes |
| **P1 - High** | Service degradation, active attack | < 1 hour |
| **P2 - Medium** | Security violation, policy breach | < 4 hours |
| **P3 - Low** | Configuration issue, minor anomaly | < 1 day |

### Quick Reference: Incident Response

1. **Detect**: Monitor alerts and logs
2. **Triage**: Assess severity and impact
3. **Contain**: Isolate affected systems
4. **Eradicate**: Remove threat or vulnerability
5. **Recover**: Restore service and data
6. **Post-Mortem**: Document and improve

**Detailed Procedures**: [INCIDENT_RESPONSE.md](./INCIDENT_RESPONSE.md)

## Security Monitoring

### Key Metrics to Track

1. **Rate Limit Violations**: Per client, per operation
2. **Authentication Failures**: By client, IP, and method
3. **Security Violations**: By type and severity
4. **Access Denials**: By resource and user
5. **Configuration Changes**: By user and key

**Setup Guide**: [SECURITY_MONITORING.md](./SECURITY_MONITORING.md)

### Alert Thresholds

| Metric | Warning | Critical |
|--------|---------|----------|
| Rate limit violations (per minute) | > 10 | > 50 |
| Auth failures (per minute) | > 5 | > 20 |
| Security violations (per hour) | > 1 | > 5 |
| Access denied (per hour) | > 10 | > 50 |

## Compliance Considerations

### SOC 2 Compliance

**Audit Logging Requirements**:
- [ ] All access to customer data logged
- [ ] Logs include user, timestamp, action, resource
- [ ] Logs retained for 90 days minimum
- [ ] Log access controlled and audited

**Data Security**:
- [ ] Encryption at rest (Turso managed)
- [ ] Encryption in transit (TLS enforced)
- [ ] Access control based on least privilege
- [ ] Regular security assessments

**Incident Response**:
- [ ] Documented incident response procedures
- [ ] Incident response team identified
- [ ] Regular incident response testing
- [ ] Post-incident reviews conducted

### GDPR Considerations

**Data Protection**:
- [ ] Personal data access logged
- [ ] Data minimization practices
- [ ] Right to erasure supported
- [ ] Data portability enabled

**Audit Trail**:
- [ ] All data processing activities logged
- [ ] Consent tracking (if applicable)
- [ ] Data access requests tracked
- [ ] Breach notification procedures

## Best Practices

### Production Deployment

1. **Enable audit logging** in production environments
2. **Use appropriate log levels** (INFO for production, DEBUG for troubleshooting)
3. **Set log retention** based on compliance requirements (minimum 90 days)
4. **Monitor log volume** to prevent disk space exhaustion
5. **Rotate logs regularly** to prevent file size issues
6. **Secure log files** with appropriate permissions (600 or 640)
7. **Ship logs to SIEM** for centralized analysis and alerting

### Rate Limiting Configuration

1. **Set appropriate limits** based on your use case and capacity
2. **Monitor rate limit violations** to tune thresholds
3. **Use separate limits** for read vs write operations
4. **Consider burst allowance** for legitimate traffic spikes
5. **Implement backoff** for rate-limited clients
6. **Monitor impact** on legitimate users

### Secret Management

1. **Never hardcode secrets** in source code
2. **Use environment variables** for all secrets
3. **Rotate credentials regularly** (database tokens, API keys)
4. **Use secret management systems** (HashiCorp Vault, AWS Secrets Manager)
5. **Audit secret access** regularly
6. **Revoke compromised secrets immediately**

### Network Security

1. **Enforce TLS** for all database connections (libsql:// only)
2. **Use firewall rules** to restrict database access
3. **Implement network segmentation** for database tiers
4. **Use VPN or private endpoints** for administrative access
5. **Monitor network traffic** for anomalies

### Regular Security Audits

1. **Weekly**: Review security logs for anomalies
2. **Monthly**: Run `cargo audit` and `cargo deny check`
3. **Quarterly**: Conduct penetration testing
4. **Annually**: Complete SOC 2 audit (if applicable)

## Troubleshooting

### Audit Logging Issues

**Issue**: Audit logs not being written

**Diagnosis**:
```bash
# Check if audit logging is enabled
echo $MEMORY_AUDIT_ENABLED

# Check log file permissions
ls -l /var/log/memory/audit.log

# Check disk space
df -h /var/log/memory
```

**Solutions**:
1. Ensure `MEMORY_AUDIT_ENABLED=true`
2. Fix file permissions: `chmod 640 /var/log/memory/audit.log`
3. Free up disk space or adjust retention policy

### Rate Limiting Issues

**Issue**: Legitimate requests being rate-limited

**Diagnosis**:
```bash
# Check rate limit configuration
echo $MCP_RATE_LIMIT_READ_RPS
echo $MCP_RATE_LIMIT_WRITE_RPS

# Check rate limit statistics
curl http://localhost:8080/metrics | grep rate_limit

# Check which clients are being limited
grep "rate_limit_violation" /var/log/memory/audit.log | \
  awk '{print $7}' | sort | uniq -c | sort -rn
```

**Solutions**:
1. Increase rate limits for legitimate traffic
2. Implement client whitelisting for trusted services
3. Adjust burst allowance for traffic patterns
4. Review and tune rate limit thresholds

## Related Documentation

- [SECURITY.md](../SECURITY.md) - Security architecture and threat model
- [AUDIT_LOGGING_SETUP.md](./AUDIT_LOGGING_SETUP.md) - Audit logging configuration
- [RATE_LIMITING_TUNING.md](./RATE_LIMITING_TUNING.md) - Rate limiting configuration
- [INCIDENT_RESPONSE.md](./INCIDENT_RESPONSE.md) - Incident response procedures
- [SECURITY_MONITORING.md](./SECURITY_MONITORING.md) - Monitoring and alerting setup
- [DEPLOYMENT.md](../DEPLOYMENT.md) - Production deployment guide

## Support

For security issues or questions:

1. Check this guide's troubleshooting section
2. Review [SECURITY.md](../SECURITY.md) for architecture details
3. Search GitHub Issues: https://github.com/d-o-hub/rust-self-learning-memory/issues
4. For security vulnerabilities, follow the disclosure policy in [SECURITY.md](../SECURITY.md)

---

**Document Version**: 1.0
**Last Updated**: 2026-02-01
**Next Review**: 2026-05-01
