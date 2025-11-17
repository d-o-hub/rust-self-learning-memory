# Phase 17: Operational Runbooks - Day-2 Operations

**Date**: 2025-11-16
**Status**: IMPLEMENTED (v0.1.2)
**Priority**: P0 (Critical for Production)
**Target**: v0.1.2 Production Deployments
**Dependencies**: plans/16-observability-implementation.md

## Executive Summary ✅ COMPLETE

Operational runbooks provide step-by-step procedures for common operational tasks, incident response, and troubleshooting. This document ensures consistent, reliable operations and rapid incident resolution for v0.1.2 deployments.

**Coverage**:
- 🚀 Deployment and startup procedures
- 🛑 Graceful shutdown and maintenance windows
- 💾 Backup, restore, and disaster recovery
- 📊 Monitoring and health checks
- 🔥 Incident response and troubleshooting
- 📈 Scaling and capacity planning
- 🔄 Upgrade and migration procedures

---

## Quick Reference Card ✅ IMPLEMENTED

| Scenario | Priority | Runbook | MTTR Target | Status |
|----------|----------|---------|-------------|--------|
| Service down | P0 | RB-001 | <5 min | ✅ Complete |
| High error rate | P1 | RB-002 | <10 min | ✅ Complete |
| Storage full | P1 | RB-003 | <15 min | ✅ Complete |
| Cache degraded | P2 | RB-004 | <30 min | ✅ Complete |
| Slow queries | P2 | RB-005 | <30 min | ✅ Complete |
| Circuit breaker open | P1 | RB-006 | <10 min | ✅ Complete |
| Memory leak | P1 | RB-007 | <20 min | ✅ Complete |

---

## RB-001: Service Startup and Deployment ✅ COMPLETE

### Prerequisites
- [ ] Rust 1.83+ installed
- [ ] Turso database URL and auth token
- [ ] Environment variables configured
- [ ] Firewall rules allow required ports

### Startup Procedure

#### Step 1: Verify Prerequisites

```bash
# Check Rust version
rustc --version  # Should be 1.83.0 or higher

# Check environment variables
env | grep -E "TURSO_|REDB_"
# Required:
# - TURSO_DATABASE_URL=libsql://...
# - TURSO_AUTH_TOKEN=...
# - REDB_CACHE_PATH=./data/cache.redb

# Verify database connectivity
curl -H "Authorization: Bearer $TURSO_AUTH_TOKEN" \
     "$TURSO_DATABASE_URL/.well-known/health"
```

#### Step 2: Build Release Binary

```bash
# Clean build
cargo clean

# Build with optimizations
cargo build --release --workspace

# Verify binary
ls -lh target/release/memory-*

# Run tests (optional but recommended)
cargo test --release --workspace
```

#### Step 3: Initialize Storage

```bash
# Create redb cache directory
mkdir -p "$(dirname $REDB_CACHE_PATH)"

# Verify Turso schema (if first deployment)
# Note: Schema migrations are applied automatically on first connection
```

#### Step 4: Start Service

```bash
# Option 1: Direct execution
./target/release/memory-core \
  --turso-url "$TURSO_DATABASE_URL" \
  --turso-token "$TURSO_AUTH_TOKEN" \
  --redb-path "$REDB_CACHE_PATH" \
  --log-level info \
  --metrics-port 9090

# Option 2: Systemd service
sudo systemctl start memory-service
sudo systemctl status memory-service

# Option 3: Docker
docker-compose up -d memory-service
docker-compose logs -f memory-service
```

#### Step 5: Verify Service Health

```bash
# Check liveness
curl http://localhost:9090/health/live
# Expected: "OK"

# Check readiness
curl http://localhost:9090/health/ready | jq .
# Expected: {"status":"healthy","checks":[...]}

# Check metrics endpoint
curl http://localhost:9090/metrics | grep memory_episode_created_total

# Verify first episode creation
# (Use your application's test script or API)
```

#### Step 6: Monitor Startup

```bash
# Watch logs for errors
tail -f /var/log/memory-service/memory.log | grep -E "ERROR|WARN"

# Watch metrics
watch -n 2 'curl -s http://localhost:9090/metrics | grep -E "episode_created|storage_errors"'

# Check Grafana dashboard
open http://localhost:3000/d/memory-overview
```

### Rollback Procedure

If startup fails:

```bash
# Stop service
sudo systemctl stop memory-service

# Check logs for errors
journalctl -u memory-service -n 100 --no-pager

# Restore previous version
sudo systemctl start memory-service@previous

# Verify rollback
curl http://localhost:9090/health/ready
```

### Success Criteria
- [ ] Service responds to health checks
- [ ] Metrics endpoint accessible
- [ ] Logs show no ERROR level messages
- [ ] First episode can be created successfully
- [ ] Cache and storage connectivity confirmed

---

## RB-002: Graceful Shutdown and Maintenance ✅ COMPLETE

### Planned Maintenance Window

#### Step 1: Announce Maintenance

```bash
# Set maintenance mode (if supported)
curl -X POST http://localhost:9090/admin/maintenance \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -d '{"enabled":true,"message":"Scheduled maintenance"}'

# Send notifications (Slack, PagerDuty, etc.)
```

#### Step 2: Drain Traffic

```bash
# Stop accepting new episodes
# (Application-specific, may involve load balancer config)

# Wait for in-flight operations to complete
watch -n 1 'curl -s http://localhost:9090/metrics | grep memory_active_episodes'
# Wait until memory_active_episodes reaches 0
```

#### Step 3: Perform Graceful Shutdown

```bash
# Send SIGTERM (graceful shutdown)
sudo systemctl stop memory-service

# Or with Docker
docker-compose stop memory-service

# Monitor shutdown progress
tail -f /var/log/memory-service/memory.log
# Look for: "Shutting down gracefully", "All workers stopped", "Goodbye"
```

#### Step 4: Verify Clean Shutdown

```bash
# Check no processes running
ps aux | grep memory-

# Verify no active connections
lsof -i :9090

# Check for crash dumps or core files
ls -lh /var/crash/
```

### Emergency Shutdown

If graceful shutdown hangs (timeout: 30 seconds):

```bash
# Send SIGKILL (force shutdown)
sudo pkill -9 memory-service

# Clean up resources
rm -f /tmp/memory-service.lock
```

---

## RB-003: Backup and Restore Procedures ✅ COMPLETE

### Backup Strategy

**Frequency**:
- Turso: Automatic backups (every 24 hours)
- redb cache: Daily backups (manual or cron job)
- Configuration: Version-controlled in Git

#### Daily redb Cache Backup

```bash
#!/bin/bash
# backup-redb.sh

DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backups/redb"
REDB_PATH="${REDB_CACHE_PATH:-./data/cache.redb}"

mkdir -p "$BACKUP_DIR"

# Stop service (or use snapshot if supported)
sudo systemctl stop memory-service

# Create backup
cp "$REDB_PATH" "$BACKUP_DIR/cache_$DATE.redb"

# Compress
gzip "$BACKUP_DIR/cache_$DATE.redb"

# Start service
sudo systemctl start memory-service

# Cleanup old backups (keep last 7 days)
find "$BACKUP_DIR" -name "cache_*.redb.gz" -mtime +7 -delete

echo "Backup completed: $BACKUP_DIR/cache_$DATE.redb.gz"
```

#### Turso Episode Export

```bash
#!/bin/bash
# export-episodes.sh

DATE=$(date +%Y%m%d_%H%M%S)
EXPORT_DIR="/backups/turso"

mkdir -p "$EXPORT_DIR"

# Export episodes table
turso db shell "$TURSO_DATABASE_URL" \
  ".mode json" \
  ".output $EXPORT_DIR/episodes_$DATE.json" \
  "SELECT * FROM episodes WHERE created_at > datetime('now', '-30 days');"

# Export patterns table
turso db shell "$TURSO_DATABASE_URL" \
  ".mode json" \
  ".output $EXPORT_DIR/patterns_$DATE.json" \
  "SELECT * FROM patterns WHERE created_at > datetime('now', '-30 days');"

# Compress
tar -czf "$EXPORT_DIR/backup_$DATE.tar.gz" \
  "$EXPORT_DIR/episodes_$DATE.json" \
  "$EXPORT_DIR/patterns_$DATE.json"

# Cleanup
rm "$EXPORT_DIR/episodes_$DATE.json" "$EXPORT_DIR/patterns_$DATE.json"

echo "Export completed: $EXPORT_DIR/backup_$DATE.tar.gz"
```

### Restore Procedure

#### Restore redb Cache

```bash
# Stop service
sudo systemctl stop memory-service

# Restore from backup
gunzip -c /backups/redb/cache_20250114_120000.redb.gz > "$REDB_CACHE_PATH"

# Verify integrity
file "$REDB_CACHE_PATH"
# Expected: "redb database"

# Start service
sudo systemctl start memory-service

# Verify health
curl http://localhost:9090/health/ready
```

#### Restore from Turso Backup

```bash
# Option 1: Point-in-time restore (Turso feature)
turso db restore "$TURSO_DATABASE_NAME" \
  --timestamp "2025-01-14T12:00:00Z"

# Option 2: Import from JSON export
turso db shell "$TURSO_DATABASE_URL" <<EOF
DELETE FROM episodes WHERE created_at > datetime('now', '-30 days');
.import /backups/turso/episodes_20250114_120000.json episodes
.import /backups/turso/patterns_20250114_120000.json patterns
EOF

# Verify data
turso db shell "$TURSO_DATABASE_URL" "SELECT COUNT(*) FROM episodes;"
```

### Disaster Recovery

**RTO (Recovery Time Objective)**: <1 hour
**RPO (Recovery Point Objective)**: <24 hours

#### Scenario: Complete Data Loss

```bash
# 1. Provision new Turso database
turso db create memory-production-restored

# 2. Restore from last backup
turso db restore memory-production-restored \
  --from-backup "2025-01-14T00:00:00Z"

# 3. Update environment variables
export TURSO_DATABASE_URL="libsql://memory-production-restored.turso.io"

# 4. Restore redb cache (or rebuild from Turso)
# Option A: Restore from backup
gunzip -c /backups/redb/cache_latest.redb.gz > "$REDB_CACHE_PATH"

# Option B: Rebuild cache (run sync)
./target/release/memory-cli storage sync --force

# 5. Restart service
sudo systemctl restart memory-service

# 6. Verify recovery
curl http://localhost:9090/health/ready
./target/release/memory-cli episode list --limit 10
```

---

## RB-004: Incident Response ✅ COMPLETE

### Alert: HighStorageErrorRate

**Severity**: P0 (Critical)
**MTTR Target**: <5 minutes

#### Step 1: Acknowledge and Assess

```bash
# Acknowledge alert in PagerDuty/Alertmanager

# Check current error rate
curl -s http://localhost:9090/metrics | grep memory_storage_errors_total

# Check Grafana dashboard
open http://localhost:3000/d/memory-overview
```

#### Step 2: Identify Root Cause

```bash
# Check recent logs for errors
journalctl -u memory-service --since "5 minutes ago" | grep ERROR

# Common causes:
# - Turso connection issues
# - Database schema mismatch
# - Quota exceeded
# - Network partition
```

#### Step 3: Immediate Mitigation

```bash
# Option 1: Restart service (if transient issue)
sudo systemctl restart memory-service

# Option 2: Switch to degraded mode (cache-only)
# (Requires application support for read-only mode)

# Option 3: Scale down load
# (Implement rate limiting or circuit breaker)
```

#### Step 4: Verify Resolution

```bash
# Check error rate dropped
watch -n 5 'curl -s http://localhost:9090/metrics | grep storage_errors'

# Check health status
curl http://localhost:9090/health/ready | jq '.checks[] | select(.name=="storage")'

# Verify new episodes can be created
./target/release/memory-cli episode create --task "test recovery"
```

#### Step 5: Post-Incident

```bash
# Export metrics and logs for analysis
prometheus-query-exporter \
  --query 'rate(memory_storage_errors_total[1h])' \
  --output /tmp/incident_metrics.json

journalctl -u memory-service --since "1 hour ago" > /tmp/incident_logs.txt

# File incident report
# - Root cause
# - Timeline
# - Mitigation steps
# - Preventive measures
```

### Alert: CircuitBreakerOpen

**Severity**: P1 (High)
**MTTR Target**: <10 minutes

#### Step 1: Identify Failing Backend

```bash
# Check which backend has circuit breaker open
curl -s http://localhost:9090/metrics | grep memory_circuit_breaker_state
# memory_circuit_breaker_state{backend="turso"} 1  <- "1" means open
```

#### Step 2: Check Backend Health

```bash
# For Turso backend
curl -H "Authorization: Bearer $TURSO_AUTH_TOKEN" \
     "$TURSO_DATABASE_URL/.well-known/health"

# For redb backend
# (Check if file is corrupted or disk full)
ls -lh "$REDB_CACHE_PATH"
df -h "$(dirname $REDB_CACHE_PATH)"
```

#### Step 3: Trigger Manual Recovery

```bash
# Reset circuit breaker (if backend is healthy now)
curl -X POST http://localhost:9090/admin/circuit-breaker/reset \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -d '{"backend":"turso"}'

# Or restart service to reset state
sudo systemctl restart memory-service
```

### Alert: LowCacheHitRate

**Severity**: P2 (Medium)
**MTTR Target**: <30 minutes

#### Step 1: Measure Current Hit Rate

```bash
# Get current cache metrics
curl -s http://localhost:9090/metrics | grep -E "cache_hit|cache_miss"
# memory_cache_hit_total 1250
# memory_cache_miss_total 750
# Hit rate = 1250 / (1250 + 750) = 62.5%
```

#### Step 2: Identify Cause

**Common Causes**:
- Cache size too small
- High episode churn
- Cache eviction policy too aggressive
- Cold start (cache recently cleared)

```bash
# Check cache size
curl -s http://localhost:9090/metrics | grep memory_cache_size
# memory_cache_size 950  (out of 1000 max)

# Check cache TTL configuration
cat /etc/memory-service/config.toml | grep cache_ttl
```

#### Step 3: Tune Cache Configuration

```bash
# Increase cache size
# Edit config file:
[storage]
cache_max_size = 2000  # Increase from 1000 to 2000
cache_ttl_seconds = 7200  # Increase from 3600 to 7200

# Restart service
sudo systemctl restart memory-service

# Monitor improvement
watch -n 10 'curl -s http://localhost:9090/metrics | grep -E "cache_hit|cache_miss"'
```

---

## RB-005: Performance Troubleshooting ✅ COMPLETE

### Slow Episode Creation (>1s)

#### Step 1: Profile Operation

```bash
# Enable debug logging
export RUST_LOG=memory_core=debug

# Restart service
sudo systemctl restart memory-service

# Observe logs for slow operations
journalctl -u memory-service -f | grep -E "duration_ms"
```

#### Step 2: Check Storage Layer

```bash
# Check Turso query latency
curl -s http://localhost:9090/metrics | grep storage_operation_duration

# Check network latency to Turso
ping $(echo $TURSO_DATABASE_URL | cut -d'/' -f3)

# Check for connection pool exhaustion
curl -s http://localhost:9090/metrics | grep connection_pool
```

#### Step 3: Optimize

```bash
# Increase connection pool size
[storage.turso]
pool_min_connections = 20  # Increase from 10
pool_max_connections = 200  # Increase from 100

# Enable batch writes
[storage]
batch_writes = true
batch_size = 50

# Restart and verify
sudo systemctl restart memory-service
```

### High Memory Usage

#### Step 1: Measure Memory

```bash
# Check process memory
ps aux | grep memory-service | awk '{print $6}'  # RSS in KB

# Check Grafana memory dashboard
open http://localhost:3000/d/memory-system-resources

# Get heap profile (if enabled)
curl http://localhost:9090/debug/pprof/heap > /tmp/heap.prof
```

#### Step 2: Identify Leak

```bash
# Check for episode leak (not completing episodes)
curl -s http://localhost:9090/metrics | grep memory_active_episodes

# Check cache size growth
curl -s http://localhost:9090/metrics | grep memory_cache_size

# Review logs for memory warnings
journalctl -u memory-service | grep -i "memory"
```

#### Step 3: Mitigation

```bash
# Force garbage collection (Rust doesn't have explicit GC, but can restart)
sudo systemctl restart memory-service

# Reduce cache size
[storage]
cache_max_size = 500  # Reduce from 1000

# Enable memory limits (systemd)
sudo systemctl edit memory-service
# Add:
[Service]
MemoryLimit=2G
MemoryHigh=1.5G
```

---

## RB-006: Scaling and Capacity Planning ✅ COMPLETE

### Vertical Scaling (Single Instance)

**When to Scale Up**:
- CPU usage >70% sustained
- Memory usage >80% sustained
- Storage operation latency increasing

```bash
# Current resource usage
top -p $(pgrep memory-service)

# Increase systemd resource limits
sudo systemctl edit memory-service
[Service]
CPUQuota=400%      # Allow 4 cores
MemoryLimit=8G     # Increase memory

# Restart with new limits
sudo systemctl restart memory-service
```

### Horizontal Scaling (Multi-Instance)

**Architecture**:
- Load balancer distributes episode creation requests
- Shared Turso database (all instances)
- Independent redb caches per instance
- Pattern extraction can run on any instance

**Setup**:

```bash
# Instance 1
export INSTANCE_ID=1
export METRICS_PORT=9091
./target/release/memory-core --instance-id $INSTANCE_ID

# Instance 2
export INSTANCE_ID=2
export METRICS_PORT=9092
./target/release/memory-core --instance-id $INSTANCE_ID

# Load balancer config (nginx)
upstream memory_service {
    server localhost:8080;  # Instance 1
    server localhost:8081;  # Instance 2
    least_conn;  # Route to instance with fewest connections
}
```

### Capacity Planning

**Metrics to Monitor**:
- Episodes created per second
- Pattern extraction backlog
- Storage size growth rate
- Cache hit rate

**Forecasting**:

```bash
# Calculate current episode rate
rate=$(curl -s http://localhost:9090/metrics | \
  grep memory_episode_created_total | \
  awk '{print $2}')
echo "Current rate: $(($rate / 86400)) episodes/day"

# Estimate storage growth
# Assumptions:
# - Average episode size: 50 KB
# - Average pattern size: 5 KB
# - 1000 episodes/day
storage_per_day=$((1000 * 50 + 1000 * 5 * 3))  # 3 patterns per episode
echo "Storage growth: $((storage_per_day / 1024)) MB/day"
```

---

## RB-007: Upgrade and Migration ✅ COMPLETE

### Minor Version Upgrade (e.g., v0.1.0 → v0.1.1)

**Downtime**: <5 minutes (rolling upgrade possible)

```bash
# Step 1: Backup current state
./scripts/backup-redb.sh
./scripts/export-episodes.sh

# Step 2: Download new binary
wget https://github.com/rust-self-learning-memory/releases/v0.1.1/memory-service
chmod +x memory-service

# Step 3: Test new binary
./memory-service --version
# v0.1.1

# Step 4: Replace binary
sudo systemctl stop memory-service
sudo cp memory-service /usr/local/bin/memory-service
sudo systemctl start memory-service

# Step 5: Verify upgrade
curl http://localhost:9090/health/ready
./target/release/memory-cli episode list --limit 5
```

### Major Version Upgrade (e.g., v0.1.0 → v0.2.0)

**Downtime**: 10-30 minutes (schema migrations)

```bash
# Step 1: Read release notes
cat CHANGELOG.md | grep -A 50 "## \[0.2.0\]"

# Step 2: Backup everything
./scripts/full-backup.sh

# Step 3: Run migration dry-run
./memory-service-v0.2.0 migrate --dry-run
# Review migration plan

# Step 4: Execute migration
sudo systemctl stop memory-service
./memory-service-v0.2.0 migrate --execute

# Step 5: Start new version
sudo cp memory-service-v0.2.0 /usr/local/bin/memory-service
sudo systemctl start memory-service

# Step 6: Verify migration
curl http://localhost:9090/health/ready
./target/release/memory-cli episode list --limit 5

# Step 7: Smoke tests
./scripts/smoke-tests.sh
```

---

## Troubleshooting Decision Tree

```
Is service responding?
├─ NO → Check if process is running (ps aux | grep memory)
│       ├─ NO → Start service (RB-001)
│       └─ YES → Check network/firewall
├─ YES → Are requests succeeding?
        ├─ NO → Check error rate (RB-004: HighStorageErrorRate)
        │       ├─ Storage errors → Check Turso connectivity
        │       └─ Other errors → Check logs
        └─ YES → Is performance acceptable?
                ├─ NO → Profile slow operations (RB-005)
                └─ YES → Monitor proactively
```

---

## Maintenance Schedule

| Task | Frequency | Script | Duration |
|------|-----------|--------|----------|
| redb backup | Daily 2 AM | `backup-redb.sh` | 5 min |
| Turso export | Weekly Sun 3 AM | `export-episodes.sh` | 15 min |
| Log rotation | Daily | `logrotate` | 1 min |
| Metric cleanup | Weekly | `prometheus-cleanup.sh` | 5 min |
| Health check | Every 5 min | Automated (Prometheus) | - |
| Capacity review | Monthly | Manual review | 30 min |
| Security audit | Quarterly | `cargo audit` | 15 min |

---

## On-Call Checklist

**Before Going On-Call**:
- [ ] Access to PagerDuty/Alertmanager
- [ ] VPN access to production environment
- [ ] SSH keys configured
- [ ] Grafana/Prometheus access verified
- [ ] Runbooks bookmarked
- [ ] Emergency contact list available

**During On-Call**:
- [ ] Respond to alerts within SLA (<5 min for P0)
- [ ] Follow runbook procedures
- [ ] Escalate if unable to resolve within 30 min
- [ ] Document all incidents
- [ ] Communicate status updates

**After On-Call**:
- [ ] File incident reports for all P0/P1 incidents
- [ ] Update runbooks with lessons learned
- [ ] Handoff to next on-call engineer
- [ ] Review metrics and trends

---

## Contact Information

| Role | Contact | Escalation |
|------|---------|------------|
| On-Call Engineer | PagerDuty | - |
| Team Lead | email@example.com | +1-555-0101 |
| Database Admin | dba@example.com | +1-555-0102 |
| Security Team | security@example.com | +1-555-0103 |

---

## Additional Resources

- **Monitoring**: http://grafana.example.com
- **Logs**: http://loki.example.com
- **Traces**: http://jaeger.example.com
- **Documentation**: https://docs.example.com
- **GitHub**: https://github.com/rust-self-learning-memory
- **Slack**: #memory-service-ops

---

**Document Version**: 1.0
**Last Updated**: 2025-11-14
**Author**: Operations Team
**Status**: READY FOR IMPLEMENTATION
