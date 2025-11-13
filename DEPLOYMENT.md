# Production Deployment Guide

This document provides comprehensive guidance for deploying the rust-self-learning-memory system to production environments.

## Table of Contents

- [Overview](#overview)
- [Environment Configuration](#environment-configuration)
- [Production Deployment Steps](#production-deployment-steps)
- [Performance Tuning](#performance-tuning)
- [Monitoring & Observability](#monitoring--observability)
- [Backup & Disaster Recovery](#backup--disaster-recovery)
- [Troubleshooting](#troubleshooting)
- [Upgrade & Rollback](#upgrade--rollback)

## Overview

The rust-self-learning-memory system consists of:

- **Rust Binary**: Compiled application (memory-core, memory-storage-turso, memory-storage-redb, memory-mcp)
- **Turso Remote Database**: Durable SQL storage for episodes, patterns, and heuristics
- **redb Local Cache**: Hot key-value cache for fast retrieval

### Supported Platforms

- **Linux** (primary): x86_64, aarch64
- **macOS**: x86_64, Apple Silicon
- **Windows**: x86_64

### Minimum Requirements

- **CPU**: 2+ cores (4+ recommended for production)
- **RAM**: 4GB minimum (8GB+ recommended)
- **Disk**: 10GB minimum for cache and logs
- **Network**: Stable connection to Turso (latency <100ms recommended)
- **Rust**: 1.70+ (for building from source)

## Environment Configuration

### Required Environment Variables

#### Turso Database Connection

```bash
# REQUIRED: Turso database URL (must use libsql:// protocol)
export TURSO_DATABASE_URL="libsql://your-database-name.turso.io"

# REQUIRED: Turso authentication token
export TURSO_AUTH_TOKEN="eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9..."
```

**Security Note**: The `libsql://` protocol enforces TLS encryption. HTTP/HTTPS protocols are explicitly rejected. See [SECURITY.md](SECURITY.md) for details.

#### redb Cache Configuration (Optional)

```bash
# Path to redb cache file (default: ./memory.redb)
export REDB_CACHE_PATH="/var/lib/memory-cache/cache.redb"

# Maximum episodes to cache (default: 1000)
export REDB_MAX_CACHE_SIZE="1000"
```

### Optional Configuration

#### Logging

```bash
# Set log level (trace, debug, info, warn, error)
export RUST_LOG="info,memory_core=debug"
```

#### Turso Connection Pool

Configure connection pooling for high-throughput scenarios:

```rust
use memory_storage_turso::{TursoStorage, TursoConfig, PoolConfig};
use std::time::Duration;

let pool_config = PoolConfig {
    max_connections: 20,                           // Default: 10
    connection_timeout: Duration::from_secs(10),   // Default: 5s
    enable_health_check: true,                     // Default: true
    health_check_timeout: Duration::from_secs(2),  // Default: 2s
};

let turso_config = TursoConfig {
    max_retries: 3,              // Default: 3
    retry_base_delay_ms: 100,    // Default: 100ms
    retry_max_delay_ms: 5000,    // Default: 5000ms
    enable_pooling: true,        // Default: true
};

let storage = TursoStorage::new_with_pool_config(
    &std::env::var("TURSO_DATABASE_URL")?,
    &std::env::var("TURSO_AUTH_TOKEN")?,
    turso_config,
    pool_config,
).await?;
```

#### Step Batching

Configure batching for high-volume step logging:

```rust
use memory_core::{MemoryConfig, BatchConfig};

let config = MemoryConfig {
    batch_config: Some(BatchConfig {
        max_batch_size: 50,       // Default: 50 steps
        flush_interval_ms: 5000,  // Default: 5 seconds
        auto_flush: true,         // Default: true
    }),
    ..Default::default()
};

let memory = SelfLearningMemory::new(config).await?;
```

### Example Production Configuration File

Create a `.env` file (never commit to version control):

```env
# Turso Database
TURSO_DATABASE_URL=libsql://prod-memory-db.turso.io
TURSO_AUTH_TOKEN=your-production-token-here

# redb Cache
REDB_CACHE_PATH=/var/lib/memory-cache/cache.redb
REDB_MAX_CACHE_SIZE=1000

# Logging
RUST_LOG=info,memory_core=debug,memory_storage_turso=info

# Application
MEMORY_MAX_CONNECTIONS=20
MEMORY_CONNECTION_TIMEOUT=10
MEMORY_BATCH_SIZE=50
MEMORY_FLUSH_INTERVAL=5000
```

## Production Deployment Steps

### Prerequisites

1. **Rust Toolchain** (if building from source):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup override set stable
   ```

2. **Turso Account and Database**:
   ```bash
   # Install Turso CLI
   curl -sSfL https://get.tur.so/install.sh | bash

   # Login
   turso auth login

   # Create database
   turso db create prod-memory-db

   # Get connection URL and token
   turso db show prod-memory-db
   turso db tokens create prod-memory-db
   ```

### Step 1: Build Release Binary

```bash
# Clone repository
git clone https://github.com/d-o-hub/rust-self-learning-memory.git
cd rust-self-learning-memory

# Build optimized release binary
cargo build --release --workspace

# Verify build
ls -lh target/release/
```

**Build Output**:
- `target/release/memory-core` - Core library
- `target/release/memory-mcp` - MCP server binary (if applicable)

### Step 2: Database Setup

Initialize the Turso database schema:

```rust
use memory_storage_turso::TursoStorage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let storage = TursoStorage::new(
        &std::env::var("TURSO_DATABASE_URL")?,
        &std::env::var("TURSO_AUTH_TOKEN")?,
    ).await?;

    storage.initialize_schema().await?;
    println!("Database schema initialized successfully");
    Ok(())
}
```

Or using the Turso CLI:

```bash
# Connect to database
turso db shell prod-memory-db

# Run schema initialization (copy from memory-storage-turso/src/schema.rs)
-- See schema.rs for CREATE TABLE statements
```

### Step 3: Deploy as Systemd Service (Linux)

Create `/etc/systemd/system/memory-service.service`:

```ini
[Unit]
Description=Self-Learning Memory Service
After=network.target

[Service]
Type=simple
User=memory
Group=memory
WorkingDirectory=/opt/memory
Environment="TURSO_DATABASE_URL=libsql://prod-memory-db.turso.io"
Environment="TURSO_AUTH_TOKEN=your-token-here"
Environment="REDB_CACHE_PATH=/var/lib/memory-cache/cache.redb"
Environment="RUST_LOG=info,memory_core=debug"
ExecStart=/opt/memory/bin/memory-service
Restart=always
RestartSec=10

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/memory-cache

[Install]
WantedBy=multi-user.target
```

Deploy and start:

```bash
# Copy binary
sudo mkdir -p /opt/memory/bin
sudo cp target/release/memory-service /opt/memory/bin/

# Create cache directory
sudo mkdir -p /var/lib/memory-cache
sudo chown memory:memory /var/lib/memory-cache

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable memory-service
sudo systemctl start memory-service

# Check status
sudo systemctl status memory-service
```

### Step 4: Deploy with Docker (Alternative)

Create `Dockerfile`:

```dockerfile
FROM rust:1.83-slim as builder

WORKDIR /usr/src/app
COPY . .

# Build release binary
RUN cargo build --release --workspace

FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /usr/src/app/target/release/memory-service /usr/local/bin/

# Create cache directory
RUN mkdir -p /var/lib/memory-cache

# Set environment variables
ENV RUST_LOG=info,memory_core=debug

EXPOSE 8080

CMD ["memory-service"]
```

Build and run:

```bash
# Build image
docker build -t memory-service:latest .

# Run container
docker run -d \
  --name memory-service \
  -e TURSO_DATABASE_URL="libsql://prod-memory-db.turso.io" \
  -e TURSO_AUTH_TOKEN="your-token-here" \
  -e REDB_CACHE_PATH="/var/lib/memory-cache/cache.redb" \
  -v /var/lib/memory-cache:/var/lib/memory-cache \
  -p 8080:8080 \
  --restart unless-stopped \
  memory-service:latest

# Check logs
docker logs -f memory-service
```

### Step 5: Health Check

Verify deployment:

```bash
# Check Turso connectivity
turso db shell prod-memory-db --execute "SELECT COUNT(*) FROM episodes;"

# Check redb cache
ls -lh /var/lib/memory-cache/cache.redb

# Check service logs
journalctl -u memory-service -f  # systemd
docker logs -f memory-service    # Docker
```

## Performance Tuning

### Connection Pool Configuration

**Recommended Settings by Load**:

| Load Level | max_connections | connection_timeout | Use Case |
|------------|----------------|-------------------|----------|
| Low (< 10 req/s) | 10 | 5s | Development, small teams |
| Medium (10-100 req/s) | 20-50 | 10s | Production, moderate load |
| High (100-1000 req/s) | 50-100 | 30s | High-traffic production |

**Configuration**:

```rust
let pool_config = PoolConfig {
    max_connections: 50,                           // Adjust based on load
    connection_timeout: Duration::from_secs(10),
    enable_health_check: true,
    health_check_timeout: Duration::from_secs(2),
};
```

**Trade-offs**:
- **Higher max_connections**: More memory, better concurrency, higher Turso costs
- **Lower max_connections**: Less memory, potential queueing delays
- **Shorter timeout**: Fail fast, better for latency-sensitive apps
- **Longer timeout**: More resilient to temporary network issues

### redb Cache Size Optimization

**Recommended Settings**:

| Memory Available | max_cache_size | Cache Hit Rate Target |
|-----------------|---------------|----------------------|
| 4GB | 500 episodes | 70-80% |
| 8GB | 1000 episodes | 80-90% |
| 16GB+ | 2000+ episodes | 90%+ |

**Configuration**:

```rust
let config = MemoryConfig {
    max_cache_size: 1000,  // Adjust based on available RAM
    ..Default::default()
};
```

**Cache Hit Rate Formula**:
- Average episode size: ~50KB
- Cache size: 1000 episodes × 50KB = 50MB
- Monitor hit rate: `cache_hits / (cache_hits + cache_misses)`
- Target: >80% hit rate for optimal performance

### Step Batching Configuration

**Recommended Settings**:

| Scenario | max_batch_size | flush_interval_ms | Rationale |
|----------|---------------|------------------|-----------|
| Interactive | 10 | 1000 (1s) | Low latency, immediate feedback |
| Standard | 50 | 5000 (5s) | Balanced throughput and latency |
| Batch Processing | 100 | 10000 (10s) | Maximum throughput |
| Real-time Analytics | 25 | 2000 (2s) | Near real-time with efficiency |

**Configuration**:

```rust
let batch_config = BatchConfig {
    max_batch_size: 50,      // Flush after 50 steps
    flush_interval_ms: 5000, // Or flush every 5 seconds
    auto_flush: true,        // Enable automatic flushing
};
```

**Trade-offs**:
- **Larger batch_size**: Fewer database writes, higher latency, more memory
- **Smaller batch_size**: More database writes, lower latency, less memory
- **Longer flush_interval**: Better batching efficiency, higher latency risk
- **Shorter flush_interval**: Near real-time updates, more frequent writes

### Performance Targets

From [PERFORMANCE_BASELINES.md](PERFORMANCE_BASELINES.md):

| Operation | P95 Target | Actual Performance | Status |
|-----------|-----------|-------------------|--------|
| Episode Creation | < 50ms | 2.56 µs | 19,531x faster ✓ |
| Step Logging | < 20ms | 1.13 µs | 17,699x faster ✓ |
| Episode Completion | < 500ms | 3.82 µs | 130,890x faster ✓ |
| Pattern Extraction | < 1000ms | 10.43 µs | 95,880x faster ✓ |
| Memory Retrieval | < 100ms | 721 µs | 138x faster ✓ |
| Storage (Write) | < 50ms | 13.22 ms | 3.8x faster ✓ |

## Monitoring & Observability

### Key Metrics to Track

#### 1. Episode Metrics

```rust
// Episode creation rate (episodes/second)
// Target: Stable rate, no sudden drops
histogram!("memory.episode.creation_rate").record(rate);

// Episode completion latency (P95)
// Target: < 500ms
histogram!("memory.episode.completion_latency_ms").record(latency);
```

#### 2. Retrieval Metrics

```rust
// Retrieval latency (P95)
// Target: < 100ms
histogram!("memory.retrieval.latency_ms").record(latency);

// Cache hit rate (percentage)
// Target: > 80%
gauge!("memory.cache.hit_rate").set(hit_rate);
```

#### 3. Pattern Extraction Metrics

```rust
// Pattern extraction success rate
// Target: > 95%
counter!("memory.pattern.extraction.success").increment(1);
counter!("memory.pattern.extraction.failure").increment(1);

// Pattern extraction latency (P95)
// Target: < 1000ms
histogram!("memory.pattern.extraction_latency_ms").record(latency);
```

#### 4. Connection Pool Metrics

```rust
// Connection pool utilization
// Target: 50-80% under normal load
gauge!("memory.pool.utilization").set(utilization);

// Active connections
gauge!("memory.pool.active_connections").set(active);

// Pool checkout wait time (P95)
// Target: < 10ms
histogram!("memory.pool.checkout_wait_ms").record(wait_time);
```

### Logging Configuration

Set appropriate log levels:

```bash
# Production (recommended)
export RUST_LOG="info,memory_core=info,memory_storage_turso=warn"

# Debug (troubleshooting)
export RUST_LOG="debug,memory_core=debug,memory_storage_turso=debug"

# Trace (detailed investigation)
export RUST_LOG="trace"
```

### Integration with Monitoring Systems

#### Prometheus Metrics Export

The memory system uses the `tracing` framework. To export metrics to Prometheus:

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_telemetry() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();
}
```

#### Grafana Dashboard

Recommended dashboard panels:

1. **Episode Creation Rate**: Line graph, episodes/sec
2. **Retrieval Latency**: Histogram, P50/P95/P99
3. **Cache Hit Rate**: Gauge, percentage
4. **Pool Utilization**: Gauge, percentage
5. **Error Rate**: Counter, errors/sec by type
6. **Storage Latency**: Histogram, write/read latency

### Health Check Endpoints

Implement health checks for orchestration:

```rust
use memory_storage_turso::TursoStorage;

async fn health_check(storage: &TursoStorage) -> anyhow::Result<bool> {
    // Check Turso connectivity
    let healthy = storage.health_check().await?;

    // Check pool statistics
    if let Some(stats) = storage.pool_statistics().await {
        if stats.total_health_checks_failed > 10 {
            return Ok(false);
        }
    }

    Ok(healthy)
}
```

## Backup & Disaster Recovery

### Turso Backup Strategy

Turso provides built-in replication and point-in-time recovery:

```bash
# Create a manual backup (snapshot)
turso db export prod-memory-db backup-$(date +%Y%m%d).sql

# List available backups
turso db backups list prod-memory-db

# Restore from backup
turso db restore prod-memory-db <backup-id>
```

**Backup Schedule** (recommended):
- **Automated**: Turso handles continuous replication
- **Manual Snapshots**: Daily exports for long-term archival
- **Retention**: 30 days minimum

### redb Cache Regeneration

The redb cache is ephemeral and can be safely regenerated:

```rust
use memory_core::SelfLearningMemory;

async fn regenerate_cache(memory: &SelfLearningMemory) -> anyhow::Result<()> {
    // Clear existing cache
    std::fs::remove_file("/var/lib/memory-cache/cache.redb")?;

    // Sync from Turso
    memory.sync_memories().await?;

    println!("Cache regenerated successfully");
    Ok(())
}
```

**Cache Corruption Recovery**:

```bash
# Stop service
sudo systemctl stop memory-service

# Remove corrupted cache
sudo rm /var/lib/memory-cache/cache.redb

# Restart service (will regenerate cache)
sudo systemctl start memory-service
```

### Episode Export for Archival

Export episodes to JSONL format for long-term storage:

```rust
use memory_core::Episode;
use std::fs::File;
use std::io::{BufWriter, Write};

async fn export_episodes(
    storage: &TursoStorage,
    output_path: &str,
) -> anyhow::Result<()> {
    let episodes = storage.query_episodes_since(
        chrono::Utc::now() - chrono::Duration::days(30)
    ).await?;

    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);

    for episode in episodes {
        let json = serde_json::to_string(&episode)?;
        writeln!(writer, "{}", json)?;
    }

    writer.flush()?;
    Ok(())
}
```

**Archival Schedule** (recommended):
- **Monthly**: Export all episodes to JSONL
- **Storage**: S3, Google Cloud Storage, or local NAS
- **Format**: Compressed JSONL (gzip)
- **Retention**: 1 year minimum

### Recovery Procedures

#### Scenario 1: Turso Database Failure

```bash
# 1. Check Turso status
turso db status prod-memory-db

# 2. If database is corrupted, restore from latest backup
turso db restore prod-memory-db <backup-id>

# 3. Verify data integrity
turso db shell prod-memory-db --execute "SELECT COUNT(*) FROM episodes;"

# 4. Regenerate cache
sudo systemctl restart memory-service
```

#### Scenario 2: redb Cache Corruption

```bash
# 1. Stop service
sudo systemctl stop memory-service

# 2. Remove corrupted cache
sudo rm /var/lib/memory-cache/cache.redb

# 3. Restart service (auto-regenerates)
sudo systemctl start memory-service

# 4. Monitor cache rebuild
sudo journalctl -u memory-service -f
```

#### Scenario 3: Complete Data Loss

```bash
# 1. Restore Turso database from backup
turso db restore prod-memory-db <backup-id>

# 2. Import archived episodes (if available)
# Run custom import script

# 3. Regenerate cache
sudo rm /var/lib/memory-cache/cache.redb
sudo systemctl restart memory-service

# 4. Verify data integrity
# Run validation queries
```

## Troubleshooting

### Common Deployment Issues

#### Issue 1: Connection Failures to Turso

**Symptoms**:
- Error: "Failed to connect to Turso"
- Logs: "Connection refused" or "Timeout"

**Diagnosis**:

```bash
# Test connectivity
curl -I https://your-database.turso.io

# Verify credentials
echo $TURSO_DATABASE_URL
echo $TURSO_AUTH_TOKEN

# Check network
ping your-database.turso.io
```

**Solutions**:

1. **Invalid Credentials**:
   ```bash
   # Regenerate token
   turso db tokens create prod-memory-db

   # Update environment
   export TURSO_AUTH_TOKEN="new-token-here"
   sudo systemctl restart memory-service
   ```

2. **Network Firewall**:
   ```bash
   # Allow outbound HTTPS (port 443)
   sudo ufw allow out 443/tcp
   ```

3. **DNS Resolution**:
   ```bash
   # Check DNS
   nslookup your-database.turso.io

   # Use Google DNS if needed
   echo "nameserver 8.8.8.8" | sudo tee -a /etc/resolv.conf
   ```

#### Issue 2: redb File Corruption

**Symptoms**:
- Error: "Failed to open redb database"
- Logs: "Corrupted database file"

**Diagnosis**:

```bash
# Check file integrity
ls -lh /var/lib/memory-cache/cache.redb
file /var/lib/memory-cache/cache.redb

# Check disk space
df -h /var/lib/memory-cache
```

**Solutions**:

1. **Regenerate Cache**:
   ```bash
   sudo systemctl stop memory-service
   sudo rm /var/lib/memory-cache/cache.redb
   sudo systemctl start memory-service
   ```

2. **Fix Permissions**:
   ```bash
   sudo chown memory:memory /var/lib/memory-cache/cache.redb
   sudo chmod 600 /var/lib/memory-cache/cache.redb
   ```

#### Issue 3: Cache Sync Issues

**Symptoms**:
- Stale data in cache
- Cache hit rate drops
- Inconsistent retrieval results

**Diagnosis**:

```bash
# Check cache statistics
# (implement cache stats endpoint)

# Compare Turso vs redb counts
turso db shell prod-memory-db --execute "SELECT COUNT(*) FROM episodes;"
# vs cache count
```

**Solutions**:

1. **Manual Sync**:
   ```rust
   // Trigger sync programmatically
   memory.sync_memories().await?;
   ```

2. **Restart Service**:
   ```bash
   sudo systemctl restart memory-service
   ```

#### Issue 4: Performance Degradation

**Symptoms**:
- Slow retrieval (P95 > 100ms)
- High CPU usage
- Memory leaks

**Diagnosis**:

```bash
# Check resource usage
top -p $(pgrep memory-service)
htop

# Check pool statistics
# (implement pool stats endpoint)

# Profile application
cargo install flamegraph
sudo flamegraph -p $(pgrep memory-service)
```

**Solutions**:

1. **Tune Connection Pool**:
   - Increase `max_connections` if pool is saturated
   - Decrease if seeing high memory usage

2. **Optimize Cache Size**:
   - Reduce `max_cache_size` if memory constrained
   - Increase if hit rate is low (<80%)

3. **Adjust Batch Configuration**:
   - Increase `max_batch_size` for better throughput
   - Decrease `flush_interval_ms` for lower latency

### Diagnostic Commands

```bash
# Check service status
sudo systemctl status memory-service

# View recent logs
sudo journalctl -u memory-service -n 100

# Follow logs in real-time
sudo journalctl -u memory-service -f

# Check disk usage
du -sh /var/lib/memory-cache/

# Check open files
sudo lsof -p $(pgrep memory-service)

# Check network connections
sudo netstat -anp | grep memory-service

# Test Turso connectivity
turso db shell prod-memory-db --execute "SELECT 1;"
```

### Reference: AGENTS.md Troubleshooting Checklist

See [AGENTS.md](AGENTS.md) for additional troubleshooting guidance:

1. If retrieval returns few results: check embeddings are computed and cached
2. If redb is stale: run `sync_memories()` to reconcile with Turso
3. If pattern updates are slow: reduce batch size or limit concurrency
4. If tests fail intermittently: run with `RUST_LOG=debug cargo test`

## Upgrade & Rollback

### Rolling Upgrade Strategy

For zero-downtime upgrades:

1. **Deploy New Version to Staging**:
   ```bash
   # Build new version
   git pull origin main
   cargo build --release --workspace

   # Test in staging
   ./target/release/memory-service --config staging.env
   ```

2. **Run Database Migrations**:
   ```bash
   # Apply schema changes
   turso db shell prod-memory-db < migrations/v0.2.0.sql
   ```

3. **Blue-Green Deployment**:
   ```bash
   # Deploy new version (blue)
   sudo cp target/release/memory-service /opt/memory/bin/memory-service-new

   # Update systemd to use new binary
   sudo systemctl stop memory-service
   sudo mv /opt/memory/bin/memory-service /opt/memory/bin/memory-service-old
   sudo mv /opt/memory/bin/memory-service-new /opt/memory/bin/memory-service
   sudo systemctl start memory-service

   # Monitor for errors
   sudo journalctl -u memory-service -f
   ```

4. **Health Check**:
   ```bash
   # Verify new version works
   curl http://localhost:8080/health

   # Check metrics
   # Monitor for 5-10 minutes
   ```

### Schema Migration Procedures

Create migration files in `migrations/`:

```sql
-- migrations/v0.2.0.sql
-- Add new column to episodes table

ALTER TABLE episodes ADD COLUMN metadata TEXT DEFAULT '{}';

CREATE INDEX IF NOT EXISTS idx_episodes_metadata
ON episodes(json_extract(metadata, '$.priority'));
```

Apply migration:

```bash
# Backup first
turso db export prod-memory-db backup-pre-migration.sql

# Apply migration
turso db shell prod-memory-db < migrations/v0.2.0.sql

# Verify
turso db shell prod-memory-db --execute "PRAGMA table_info(episodes);"
```

### Backward Compatibility Notes

The system maintains backward compatibility:

- **Episode JSON**: New fields are nullable, old episodes still load
- **Pattern Types**: New pattern types are additive
- **API**: New endpoints don't break existing clients
- **Cache**: redb cache is version-agnostic, regenerates on mismatch

**Breaking Changes** (if any) will be documented in:
- `CHANGELOG.md`: Details of breaking changes
- `MIGRATION.md`: Step-by-step upgrade guide

### Rollback Procedures

If upgrade fails:

```bash
# 1. Stop new version
sudo systemctl stop memory-service

# 2. Restore old binary
sudo mv /opt/memory/bin/memory-service-old /opt/memory/bin/memory-service

# 3. Rollback database (if migration applied)
turso db restore prod-memory-db backup-pre-migration.sql

# 4. Clear cache (to avoid version mismatch)
sudo rm /var/lib/memory-cache/cache.redb

# 5. Start old version
sudo systemctl start memory-service

# 6. Verify rollback
sudo journalctl -u memory-service -f
curl http://localhost:8080/health
```

**Rollback Checklist**:
- [ ] Binary reverted to previous version
- [ ] Database restored from pre-migration backup
- [ ] Cache cleared and regenerated
- [ ] Service started and healthy
- [ ] Monitoring shows normal metrics
- [ ] No errors in logs

### Version Verification

```bash
# Check deployed version
/opt/memory/bin/memory-service --version

# Check database schema version
turso db shell prod-memory-db --execute "SELECT version FROM schema_version;"

# Check cache format version
# (implement version check in redb metadata)
```

---

## Additional Resources

- [README.md](README.md) - Project overview and quick start
- [SECURITY.md](SECURITY.md) - Security guidelines and threat model
- [AGENTS.md](AGENTS.md) - Agent responsibilities and operational guidance
- [PERFORMANCE_BASELINES.md](PERFORMANCE_BASELINES.md) - Performance targets and benchmarks
- [TESTING.md](TESTING.md) - Testing infrastructure and best practices

## Support

For deployment issues or questions:

1. Check this guide's [Troubleshooting](#troubleshooting) section
2. Review [AGENTS.md](AGENTS.md) troubleshooting checklist
3. Search GitHub Issues: https://github.com/d-o-hub/rust-self-learning-memory/issues
4. Open a new issue with deployment details

---

**Document Version**: 1.0
**Last Updated**: 2025-11-13
**Status**: Production Ready
