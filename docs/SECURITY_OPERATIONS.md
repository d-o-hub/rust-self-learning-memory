# Security Operations Guide

**Version**: v0.1.13  
**Last Updated**: 2026-02-01  
**Audience**: Security Teams, DevOps, System Administrators  
**Classification**: Internal Use  

---

## Table of Contents

- [Overview](#overview)
- [Security Architecture](#security-architecture)
- [Incident Response Procedures](#incident-response-procedures)
- [Security Monitoring](#security-monitoring)
- [Audit Logging](#audit-logging)
- [Vulnerability Disclosure](#vulnerability-disclosure)
- [Security Checklists](#security-checklists)
- [Compliance](#compliance)

---

## Overview

This guide provides comprehensive security operations procedures for the Rust Self-Learning Memory System in production environments. It covers incident response, monitoring, audit logging, and vulnerability management.

### Security Principles

1. **Defense in Depth** - Multiple security layers protect against single points of failure
2. **Zero Trust** - Never trust, always verify
3. **Least Privilege** - Minimal access required for operations
4. **Assume Breach** - Design for detection and containment

---

## Security Architecture

### Security Layers

```
┌─────────────────────────────────────────────────────────────────┐
│ Layer 6: Application Security                                    │
│ - Input validation, output sanitization, rate limiting          │
├─────────────────────────────────────────────────────────────────┤
│ Layer 5: API Security                                            │
│ - OAuth 2.0, rate limiting, request validation                  │
├─────────────────────────────────────────────────────────────────┤
│ Layer 4: Sandbox Security                                        │
│ - WASM isolation, resource limits, network controls             │
├─────────────────────────────────────────────────────────────────┤
│ Layer 3: Storage Security                                        │
│ - Encrypted connections, parameterized queries, access control  │
├─────────────────────────────────────────────────────────────────┤
│ Layer 2: Infrastructure Security                                 │
│ - TLS encryption, network segmentation, firewall rules          │
├─────────────────────────────────────────────────────────────────┤
│ Layer 1: Supply Chain Security                                   │
│ - Dependency auditing, license compliance, vulnerability scans  │
└─────────────────────────────────────────────────────────────────┘
```

### Security Components

| Component | Purpose | Implementation |
|-----------|---------|----------------|
| Rate Limiter | DoS prevention | Token bucket algorithm |
| Audit Logger | Security event tracking | Structured JSON logging |
| WASM Sandbox | Code execution isolation | Wasmtime with resource limits |
| Input Validator | Injection prevention | Size limits, type checking |
| Secret Manager | Credential protection | Environment variables only |

---

## Incident Response Procedures

### Incident Severity Levels

| Level | Description | Response Time | Examples |
|-------|-------------|---------------|----------|
| **P0 - Critical** | System compromise, data breach | 15 minutes | Unauthorized admin access, mass data exfiltration |
| **P1 - High** | Service disruption, security vulnerability | 1 hour | DoS attack, privilege escalation attempt |
| **P2 - Medium** | Policy violation, suspicious activity | 4 hours | Unusual access patterns, failed auth spikes |
| **P3 - Low** | Minor issues, informational | 24 hours | Single failed login, configuration drift |

### Incident Response Playbook

#### Phase 1: Detection (0-15 minutes)

**Automated Detection**:
```bash
# Monitor for security violations in real-time
tail -f /var/log/memory-mcp/audit.log | jq 'select(.operation == "security_violation")'

# Alert on rate limit violations
watch -n 5 'grep "rate_limit_violation" /var/log/memory-mcp/audit.log | tail -20'

# Check for failed authentication spikes
grep "authentication" /var/log/memory-mcp/audit.log | \
  jq 'select(.result == "failure")' | \
  jq -s 'group_by(.client_id) | map({client: .[0].client_id, count: length}) | .[] | select(.count > 10)'
```

**Manual Detection Indicators**:
- Unusual CPU/memory spikes
- Unexpected network connections
- Failed authentication attempts
- Rate limit violations
- Audit log anomalies

#### Phase 2: Containment (15-60 minutes)

**Immediate Actions**:

1. **Isolate Affected Components**:
```bash
# Stop the MCP server
sudo systemctl stop memory-mcp

# Or disable specific tools
export MCP_DISABLE_TOOLS="execute_agent_code,configure_embeddings"
```

2. **Enable Emergency Rate Limiting**:
```bash
# Aggressive rate limits during incident
export MCP_RATE_LIMIT_READ_RPS=10
export MCP_RATE_LIMIT_WRITE_RPS=1
export MCP_RATE_LIMIT_ENABLED=true
```

3. **Increase Audit Verbosity**:
```bash
export AUDIT_LOG_LEVEL=debug
export AUDIT_LOG_DESTINATION=both
```

4. **Block Suspicious Clients**:
```bash
# Add to firewall (example with ufw)
sudo ufw deny from 192.168.1.100

# Or use application-level blocking
export MCP_BLOCKED_CLIENTS="192.168.1.100,malicious-client-id"
```

#### Phase 3: Investigation (1-4 hours)

**Evidence Collection**:

```bash
# 1. Collect audit logs for incident timeframe
START_TIME="2026-02-01T10:00:00Z"
END_TIME="2026-02-01T12:00:00Z"

grep -E "$START_TIME|$END_TIME" /var/log/memory-mcp/audit.log > /tmp/incident_audit.log

# 2. Extract all operations by suspicious client
SUSPICIOUS_CLIENT="attacker-client-id"
grep "$SUSPICIOUS_CLIENT" /var/log/memory-mcp/audit.log > /tmp/suspicious_client.log

# 3. Get failed authentication attempts
grep '"operation":"authentication"' /var/log/memory-mcp/audit.log | \
  jq 'select(.result == "failure")' > /tmp/failed_auths.log

# 4. Check for data exfiltration
grep -E '(bulk_episodes|batch_query)' /var/log/memory-mcp/audit.log | \
  jq 'select(.metadata.limit > 100)' > /tmp/large_queries.log

# 5. System state snapshot
sudo systemctl status memory-mcp > /tmp/service_status.txt
sudo netstat -tulpn | grep memory-mcp > /tmp/network_connections.txt
sudo ps aux | grep memory-mcp > /tmp/process_info.txt
df -h > /tmp/disk_usage.txt
free -h > /tmp/memory_usage.txt
```

**Analysis Queries**:

```bash
# Timeline of events
cat /tmp/incident_audit.log | jq -r '[.timestamp, .operation, .client_id, .result] | @tsv' | sort

# Most active clients during incident
cat /tmp/incident_audit.log | jq -r '.client_id' | sort | uniq -c | sort -rn | head -10

# Operations by type
cat /tmp/incident_audit.log | jq -r '.operation' | sort | uniq -c | sort -rn

# Failed operations
cat /tmp/incident_audit.log | jq 'select(.result == "failure") | .operation' | sort | uniq -c
```

#### Phase 4: Eradication (4-24 hours)

**Remediation Steps**:

1. **Patch Vulnerabilities**:
```bash
# Update dependencies
cargo update

# Run security audit
cargo audit

# Rebuild with fixes
cargo build --release
```

2. **Rotate Compromised Credentials**:
```bash
# Generate new Turso token
turso db tokens create prod-memory-db --expiration 24h

# Update environment
export TURSO_AUTH_TOKEN="new-token-here"

# Restart service
sudo systemctl restart memory-mcp
```

3. **Clean Up Malicious Data** (if applicable):
```bash
# Delete suspicious episodes (if created by attacker)
# Use memory-cli or direct database access
memory-cli episode delete --id <suspicious-episode-id> --confirm
```

#### Phase 5: Recovery (24-48 hours)

**Service Restoration**:

```bash
# 1. Verify fixes
cargo test --all
./scripts/quality-gates.sh

# 2. Gradual rollout
# Start with limited traffic
export MCP_RATE_LIMIT_READ_RPS=50
export MCP_RATE_LIMIT_WRITE_RPS=10

# 3. Monitor closely
sudo journalctl -u memory-mcp -f | grep -E "(error|warning|security)"

# 4. Restore normal rate limits after 24h
export MCP_RATE_LIMIT_READ_RPS=100
export MCP_RATE_LIMIT_WRITE_RPS=20
```

#### Phase 6: Post-Incident (48+ hours)

**Documentation Requirements**:

1. **Incident Report**:
   - Timeline of events
   - Root cause analysis
   - Impact assessment
   - Lessons learned

2. **Update Runbooks**:
   - Document new attack vectors
   - Update detection rules
   - Improve response procedures

---

## Security Monitoring

### Key Security Metrics

| Metric | Target | Alert Threshold | Description |
|--------|--------|-----------------|-------------|
| Failed Auth Rate | < 1% | > 5% | Authentication failures per minute |
| Rate Limit Violations | < 10/hour | > 50/hour | Clients hitting rate limits |
| Security Violations | 0 | > 0 | Policy violations detected |
| Unusual Query Patterns | Baseline | 3σ deviation | Abnormal query behavior |
| Sandbox Escapes | 0 | > 0 | WASM sandbox breaches |

### Monitoring Dashboard

```bash
# Real-time security dashboard script
#!/bin/bash
# security-dashboard.sh

echo "=== Memory MCP Security Dashboard ==="
echo "Last updated: $(date)"
echo ""

echo "=== Failed Authentications (Last Hour) ==="
grep '"operation":"authentication"' /var/log/memory-mcp/audit.log | \
  jq 'select(.timestamp > "'$(date -u -d '1 hour ago' +%Y-%m-%dT%H:%M)'")' | \
  jq 'select(.result == "failure")' | \
  jq -s 'group_by(.client_id) | map({client: .[0].client_id, failures: length}) | sort_by(.failures) | reverse | .[0:5]'

echo ""
echo "=== Rate Limit Violations (Last Hour) ==="
grep '"operation":"rate_limit_violation"' /var/log/memory-mcp/audit.log | \
  jq 'select(.timestamp > "'$(date -u -d '1 hour ago' +%Y-%m-%dT%H:%M)'")' | \
  jq -s 'length'

echo ""
echo "=== Top Clients by Activity (Last Hour) ==="
grep -E '"timestamp":"'$(date -u -d '1 hour ago' +%Y-%m-%dT)'.*Z"' /var/log/memory-mcp/audit.log | \
  jq -r '.client_id' | sort | uniq -c | sort -rn | head -5

echo ""
echo "=== Security Violations (24h) ==="
grep '"operation":"security_violation"' /var/log/memory-mcp/audit.log | \
  jq -s 'group_by(.metadata.violation_type) | map({type: .[0].metadata.violation_type, count: length})'

echo ""
echo "=== Suspicious Activity ==="
# Episodes deleted (potential data destruction)
grep '"operation":"delete_episode"' /var/log/memory-mcp/audit.log | \
  jq 'select(.timestamp > "'$(date -u -d '1 hour ago' +%Y-%m-%dT%H:%M)'")' | \
  jq -s 'length'
```

### Automated Alerting

#### Prometheus Alert Rules

```yaml
# security-alerts.yml
groups:
  - name: memory_mcp_security
    rules:
      - alert: HighFailedAuthRate
        expr: rate(audit_authentication_failures_total[5m]) > 0.05
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "High authentication failure rate"
          description: "{{ $value }}% auth failures in last 5 minutes"

      - alert: RateLimitViolationsSpike
        expr: rate(audit_rate_limit_violations_total[15m]) > 10
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Rate limit violations spike"
          description: "{{ $value }} violations per minute"

      - alert: SecurityViolationDetected
        expr: audit_security_violations_total > 0
        for: 0m
        labels:
          severity: critical
        annotations:
          summary: "Security violation detected"
          description: "Check audit logs immediately"

      - alert: UnusualBulkQuery
        expr: rate(audit_bulk_operations_total[10m]) > 100
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Unusual bulk query activity"
          description: "{{ $value }} bulk ops per minute"
```

#### Slack Integration

```bash
# alert-slack.sh
#!/bin/bash
WEBHOOK_URL="https://hooks.slack.com/services/YOUR/WEBHOOK/URL"

send_alert() {
  local severity="$1"
  local message="$2"
  local color="danger"
  
  [[ "$severity" == "warning" ]] && color="warning"
  [[ "$severity" == "info" ]] && color="good"
  
  curl -X POST -H 'Content-type: application/json' \
    --data "{
      \"attachments\": [{
        \"color\": \"$color\",
        \"title\": \"Memory MCP Security Alert\",
        \"text\": \"$message\",
        \"footer\": \"Security Monitoring\",
        \"ts\": $(date +%s)
      }]
    }" \
    "$WEBHOOK_URL"
}

# Usage
send_alert "critical" "High failed authentication rate detected"
```

---

## Audit Logging

### Logged Security Events

| Event Type | Level | Description |
|------------|-------|-------------|
| `authentication` | info/warn | Login attempts (success/failure) |
| `rate_limit_violation` | warn | Rate limit exceeded |
| `security_violation` | error | Policy violation detected |
| `config_change` | info | Configuration modifications |
| `delete_episode` | warn | Episode deletion |
| `execute_agent_code` | info | Code execution in sandbox |

### Audit Log Analysis

#### Daily Security Report

```bash
#!/bin/bash
# daily-security-report.sh

LOG_FILE="/var/log/memory-mcp/audit.log"
YESTERDAY=$(date -d "yesterday" +%Y-%m-%d)
REPORT_FILE="/var/log/memory-mcp/reports/security-${YESTERDAY}.json"

mkdir -p "$(dirname "$REPORT_FILE")"

cat > "$REPORT_FILE" <<EOF
{
  "date": "$YESTERDAY",
  "generated_at": "$(date -Iseconds)",
  "summary": {
    "total_operations": $(grep "$YESTERDAY" "$LOG_FILE" | wc -l),
    "failed_operations": $(grep "$YESTERDAY" "$LOG_FILE" | grep '"result":"failure"' | wc -l),
    "security_violations": $(grep "$YESTERDAY" "$LOG_FILE" | grep '"operation":"security_violation"' | wc -l),
    "rate_limit_violations": $(grep "$YESTERDAY" "$LOG_FILE" | grep '"operation":"rate_limit_violation"' | wc -l),
    "failed_authentications": $(grep "$YESTERDAY" "$LOG_FILE" | grep '"operation":"authentication"' | grep '"result":"failure"' | wc -l)
  },
  "top_clients": $(grep "$YESTERDAY" "$LOG_FILE" | jq -r '.client_id' | sort | uniq -c | sort -rn | head -10 | jq -R -s 'split("\n") | map(select(length > 0) | split(" ") | {count: .[0], client: .[-1]})'),
  "operation_breakdown": $(grep "$YESTERDAY" "$LOG_FILE" | jq -r '.operation' | sort | uniq -c | jq -R -s 'split("\n") | map(select(length > 0) | split(" ") | {count: .[0], operation: .[-1]})')
}
EOF

echo "Security report generated: $REPORT_FILE"
```

#### SIEM Integration

```yaml
# filebeat-security.yml
filebeat.inputs:
- type: log
  enabled: true
  paths:
    - /var/log/memory-mcp/audit.log
  json.keys_under_root: true
  json.add_error_key: true
  fields:
    service: memory-mcp
    log_type: security
  fields_under_root: true

processors:
- add_host_metadata:
    when.not.contains.tags: forwarded
- add_cloud_metadata: ~
- add_docker_metadata: ~

output.elasticsearch:
  hosts: ["elasticsearch:9200"]
  index: "memory-mcp-security-%{+yyyy.MM.dd}"
  
template.enabled: true
template.name: "memory-mcp-security"
template.pattern: "memory-mcp-security-*"
```

---

## Vulnerability Disclosure

### Reporting Security Vulnerabilities

**DO NOT** open public issues for security vulnerabilities.

**Contact**: security@example.com

**Required Information**:
1. Description of vulnerability
2. Steps to reproduce
3. Potential impact
4. Suggested fix (if any)
5. Your contact information for coordination

### Disclosure Timeline

| Phase | Timeline | Action |
|-------|----------|--------|
| **Initial Report** | Day 0 | Reporter submits vulnerability |
| **Acknowledgment** | Day 1-2 | Team acknowledges receipt |
| **Assessment** | Day 3-7 | Validate and assess severity |
| **Fix Development** | Day 8-60 | Develop and test fix |
| **Patch Release** | Day 61-90 | Release patched version |
| **Public Disclosure** | Day 90+ | Publish security advisory |

### Security Advisory Template

```markdown
# Security Advisory: [SA-YYYY-NNN]

**Severity**: [Critical/High/Medium/Low]  
**Affected Versions**: [v0.1.0 - v0.1.5]  
**Fixed Version**: [v0.1.6]  
**CVE ID**: [CVE-YYYY-NNNNN] (if assigned)  

## Summary
Brief description of the vulnerability.

## Impact
What could an attacker accomplish?

## Affected Components
- Component 1
- Component 2

## Mitigation
How to protect yourself before upgrading.

## Upgrade Instructions
```bash
cargo update -p memory-mcp
cargo build --release
sudo systemctl restart memory-mcp
```

## Credit
Discovered by [Researcher Name] ([Organization]).

## References
- Commit: [hash]
- PR: [link]
```

---

## Security Checklists

### Pre-Deployment Security Checklist

- [ ] All dependencies audited (`cargo audit`)
- [ ] No hardcoded secrets in code
- [ ] Rate limiting enabled
- [ ] Audit logging enabled
- [ ] TLS configured for Turso connections
- [ ] Input validation configured
- [ ] Sandbox resource limits set
- [ ] OAuth enabled (if applicable)
- [ ] Security headers configured
- [ ] Log rotation configured
- [ ] Monitoring alerts configured
- [ ] Incident response plan documented
- [ ] Backup/recovery tested

### Daily Security Operations

- [ ] Review security dashboard
- [ ] Check for failed authentication spikes
- [ ] Verify audit logs are being generated
- [ ] Review rate limit violations
- [ ] Check for security violations
- [ ] Verify log shipping to SIEM
- [ ] Review automated alerts

### Weekly Security Review

- [ ] Run `cargo audit` for new vulnerabilities
- [ ] Review access logs for anomalies
- [ ] Check for unauthorized configuration changes
- [ ] Verify backup integrity
- [ ] Review and update firewall rules
- [ ] Test incident response procedures
- [ ] Update threat intelligence feeds

### Monthly Security Assessment

- [ ] Full penetration test
- [ ] Review and update security policies
- [ ] Audit user access permissions
- [ ] Review third-party dependencies
- [ ] Update security documentation
- [ ] Conduct security training
- [ ] Review incident response plan

---

## Compliance

### Standards Mapping

| Standard | Requirements | Implementation |
|----------|--------------|----------------|
| **OWASP Top 10** | Injection, Broken Auth, etc. | Input validation, OAuth, rate limiting |
| **SOC 2** | Access controls, monitoring | Audit logging, RBAC, monitoring |
| **GDPR** | Data protection, right to deletion | Episode deletion, data export |
| **HIPAA** | PHI protection | Encryption, access logs |

### Audit Evidence

```bash
# Generate compliance evidence package
#!/bin/bash
# compliance-evidence.sh

OUTPUT_DIR="/tmp/compliance-$(date +%Y%m%d)"
mkdir -p "$OUTPUT_DIR"

# 1. Dependency audit
cargo audit --json > "$OUTPUT_DIR/dependency-audit.json"

# 2. License compliance
cargo deny list > "$OUTPUT_DIR/licenses.txt"

# 3. Security configuration
echo "Rate Limiting: $MCP_RATE_LIMIT_ENABLED" > "$OUTPUT_DIR/security-config.txt"
echo "Audit Logging: $AUDIT_LOG_ENABLED" >> "$OUTPUT_DIR/security-config.txt"

# 4. Recent audit logs (last 30 days)
find /var/log/memory-mcp -name "audit.log*" -mtime -30 -exec cp {} "$OUTPUT_DIR/" \;

# 5. Create tarball
tar -czf "$OUTPUT_DIR.tar.gz" "$OUTPUT_DIR"
echo "Compliance evidence: $OUTPUT_DIR.tar.gz"
```

---

## See Also

- [API Reference](./API_REFERENCE.md)
- [Deployment Security Guide](./DEPLOYMENT_SECURITY.md)
- [Security Features Guide](./SECURITY_FEATURES_GUIDE.md)
- [Troubleshooting Guide](./TROUBLESHOOTING.md)

---

**Document Version**: 1.0  
**Last Updated**: 2026-02-01  
**Maintained By**: Security Team  
**Next Review**: 2026-03-01
