# Staging Deployment Plan - Turso Optimizations

**Date**: 2026-01-26
**Target**: Staging Environment
**Expected Impact**: 10-15x performance improvement
**Risk Level**: LOW (all features have fallback mechanisms)

---

## 🎯 Deployment Objectives

1. Enable all optimizations in staging environment
2. Validate 10-15x performance improvement
3. Identify any configuration issues before production
4. Establish performance baseline metrics
5. Train team on monitoring and troubleshooting

---

## 📋 Pre-Deployment Checklist

### Environment Setup
- [ ] Staging database URL and token available
- [ ] Rust 1.70+ installed on staging servers
- [ ] Sufficient memory allocated (recommended: 2GB+)
- [ ] Monitoring infrastructure ready
- [ ] Rollback plan documented

### Code Preparation
- [ ] Feature flags configured in Cargo.toml
- [ ] Environment variables set
- [ ] Configuration files updated
- [ ] Tests passing locally
- [ ] Dependencies up to date

### Team Readiness
- [ ] Team briefed on changes
- [ ] Monitoring dashboard access granted
- [ ] Troubleshooting guide shared
- [ ] Communication plan established

---

## 🚀 Deployment Steps

### Step 1: Update Cargo.toml (5 minutes)

**File**: `Cargo.toml` or your application's `Cargo.toml`

```toml
[dependencies]
memory-storage-turso = { 
    version = "0.1.12",
    features = [
        "turso",              # Turso database support
        "redb",               # Cache layer
        "keepalive-pool",     # Keep-alive optimization
        "compression",        # Enable compression
        "compression-zstd",   # Zstd algorithm (best balance)
    ]
}

# Optional: Enable all compression algorithms for testing
# memory-storage-turso = { 
#     version = "0.1.12",
#     features = ["full"]  # All features enabled
# }
```

**Validation**:
```bash
cargo check --features keepalive-pool,compression,compression-zstd
```

---

### Step 2: Create Staging Configuration File (10 minutes)

**File**: `config/staging.toml` (create if doesn't exist)

```toml
# Staging Environment Configuration
[database]
url = "libsql://your-staging-db.turso.io"
# Token should come from environment variable
# token = "${TURSO_AUTH_TOKEN}"

[connection_pool]
max_connections = 20
connection_timeout_secs = 10
enable_health_check = true
health_check_timeout_secs = 2

[keepalive]
enabled = true
interval_secs = 30
stale_threshold_secs = 60
enable_proactive_ping = true
ping_timeout_secs = 5

[compression]
enabled = true
threshold_bytes = 1024
compress_episodes = true
compress_patterns = true
compress_embeddings = true

[cache]
enabled = true
max_episodes = 10000
max_patterns = 5000
max_query_results = 1000
episode_ttl_secs = 1800  # 30 minutes
pattern_ttl_secs = 3600  # 1 hour
query_ttl_secs = 300     # 5 minutes

[monitoring]
enabled = true
metrics_interval_secs = 60
log_level = "info"
```

---

### Step 3: Update Storage Initialization Code (15 minutes)

**File**: Your application's storage initialization (e.g., `src/main.rs` or `src/storage.rs`)

```rust
use memory_storage_turso::{
    TursoStorage, TursoConfig, PoolConfig, KeepAliveConfig, CacheConfig,
};
use std::time::Duration;
use tracing::{info, warn};

/// Initialize optimized storage for staging
pub async fn init_staging_storage() -> anyhow::Result<impl StorageBackend> {
    info!("🚀 Initializing optimized Turso storage for STAGING");
    
    // Load configuration from environment
    let db_url = std::env::var("TURSO_DB_URL")
        .unwrap_or_else(|_| "libsql://staging-db.turso.io".to_string());
    let auth_token = std::env::var("TURSO_AUTH_TOKEN")
        .expect("TURSO_AUTH_TOKEN must be set");
    
    // 1. Base Turso configuration
    let turso_config = TursoConfig {
        max_retries: 3,
        retry_base_delay_ms: 100,
        retry_max_delay_ms: 5000,
        enable_pooling: true,
        enable_keepalive: true,
        keepalive_interval_secs: 30,
        stale_threshold_secs: 60,
        compression_threshold: 1024,
        compress_episodes: true,
        compress_patterns: true,
        compress_embeddings: true,
        cache_config: Some(CacheConfig::default()),
    };
    
    // 2. Connection pool configuration (staging: moderate load)
    let pool_config = PoolConfig {
        max_connections: 20,
        connection_timeout: Duration::from_secs(10),
        enable_health_check: true,
        health_check_timeout: Duration::from_secs(2),
    };
    
    // 3. Keep-alive configuration
    let keepalive_config = KeepAliveConfig {
        keep_alive_interval: Duration::from_secs(30),
        stale_threshold: Duration::from_secs(60),
        enable_proactive_ping: true,
        ping_timeout: Duration::from_secs(5),
    };
    
    // 4. Create storage with keep-alive pool
    info!("Creating storage with keep-alive pool...");
    let storage = TursoStorage::new_with_keepalive(
        &db_url,
        &auth_token,
        turso_config,
        pool_config,
        keepalive_config,
    ).await?;
    
    // 5. Initialize schema
    info!("Initializing database schema...");
    storage.initialize_schema().await?;
    
    // 6. Wrap with cache layer
    info!("Enabling cache layer...");
    let cache_config = CacheConfig {
        enable_episode_cache: true,
        enable_pattern_cache: true,
        enable_query_cache: true,
        max_episodes: 10_000,
        max_patterns: 5_000,
        max_query_results: 1_000,
        episode_ttl: Duration::from_secs(1800),
        pattern_ttl: Duration::from_secs(3600),
        query_ttl: Duration::from_secs(300),
        min_ttl: Duration::from_secs(60),
        max_ttl: Duration::from_secs(7200),
        hot_threshold: 10,
        cold_threshold: 2,
        adaptation_rate: 0.25,
        enable_background_cleanup: true,
        cleanup_interval_secs: 60,
    };
    let cached_storage = storage.with_cache(cache_config);
    
    // 7. Verify connectivity
    info!("Verifying database connectivity...");
    match cached_storage.storage().health_check().await {
        Ok(true) => info!("✅ Database health check passed"),
        Ok(false) => warn!("⚠️  Database health check failed"),
        Err(e) => warn!("⚠️  Health check error: {}", e),
    }
    
    info!("✅ Optimized storage initialized successfully!");
    info!("   - Keep-alive pool: ENABLED");
    info!("   - Compression: ENABLED (Zstd)");
    info!("   - Cache layer: ENABLED");
    info!("   - Batch operations: AVAILABLE");
    info!("   - Prepared statements: ENABLED");
    
    Ok(cached_storage)
}
```

---

### Step 4: Set Environment Variables (5 minutes)

**File**: `.env.staging` or deployment configuration

```bash
# Turso Database Configuration
TURSO_DB_URL=libsql://your-staging-db.turso.io
TURSO_AUTH_TOKEN=your_staging_auth_token_here

# Feature Flags
ENABLE_OPTIMIZATIONS=true
ENABLE_COMPRESSION=true
ENABLE_CACHE=true

# Monitoring
RUST_LOG=info,memory_storage_turso=debug
METRICS_ENABLED=true
METRICS_INTERVAL_SECS=60

# Performance Tuning (Staging)
MAX_CONNECTIONS=20
CACHE_SIZE_EPISODES=10000
CACHE_SIZE_PATTERNS=5000
```

**Set environment variables** (choose one):

```bash
# Option 1: Load from file
source .env.staging

# Option 2: Export directly
export TURSO_DB_URL="libsql://your-staging-db.turso.io"
export TURSO_AUTH_TOKEN="your_token_here"
export RUST_LOG="info,memory_storage_turso=debug"

# Option 3: Use in Docker
docker run --env-file .env.staging your-image
```

---

### Step 5: Deploy to Staging (20 minutes)

#### Build with optimizations

```bash
# Clean build
cargo clean

# Build with all features
cargo build --release \
    --features keepalive-pool,compression,compression-zstd

# Verify binary size
ls -lh target/release/your-binary

# Run tests
cargo test --release --all
```

#### Deploy

```bash
# Option 1: Direct deployment
./target/release/your-binary

# Option 2: Docker deployment
docker build -t your-app:staging-optimized .
docker push your-app:staging-optimized
kubectl rollout restart deployment/your-app -n staging

# Option 3: systemd service
sudo systemctl restart your-app-staging
```

---

### Step 6: Verify Deployment (10 minutes)

**Immediate Checks**:

```bash
# 1. Check service is running
curl http://staging-api/health

# 2. Check logs for optimization messages
tail -f /var/log/your-app/app.log | grep -i "optimized\|cache\|compression"

# Expected log entries:
# ✅ Optimized storage initialized successfully!
# ✅ Keep-alive pool: ENABLED
# ✅ Compression: ENABLED (Zstd)
# ✅ Cache layer: ENABLED
```

**Database Connectivity**:

```bash
# Test database connection
curl http://staging-api/db-health

# Expected response:
# {"status": "healthy", "optimizations": "enabled"}
```

---

### Step 7: Run Performance Validation (30 minutes)

**Create validation script**: `scripts/validate_staging.sh`

```bash
#!/bin/bash

echo "🔬 Validating Turso Optimizations in Staging"
echo "=============================================="
echo ""

# Test 1: Single Read Performance
echo "Test 1: Single Read Performance"
START=$(date +%s%N)
curl -s http://staging-api/episodes/test-id-1 > /dev/null
END=$(date +%s%N)
LATENCY=$(( ($END - $START) / 1000000 ))
echo "  Latency: ${LATENCY}ms (target: < 30ms)"
echo ""

# Test 2: Cached Read Performance
echo "Test 2: Cached Read (should be faster)"
START=$(date +%s%N)
curl -s http://staging-api/episodes/test-id-1 > /dev/null
END=$(date +%s%N)
CACHED_LATENCY=$(( ($END - $START) / 1000000 ))
echo "  Cached Latency: ${CACHED_LATENCY}ms (target: < 5ms)"
echo "  Speedup: $(echo "scale=1; $LATENCY / $CACHED_LATENCY" | bc)x"
echo ""

# Test 3: Batch Operations
echo "Test 3: Batch Store (10 episodes)"
START=$(date +%s%N)
curl -s -X POST http://staging-api/episodes/batch \
  -H "Content-Type: application/json" \
  -d @test-data/batch-10.json > /dev/null
END=$(date +%s%N)
BATCH_LATENCY=$(( ($END - $START) / 1000000 ))
echo "  Batch Latency: ${BATCH_LATENCY}ms (target: < 200ms)"
echo ""

# Test 4: Check Metrics
echo "Test 4: Optimization Metrics"
METRICS=$(curl -s http://staging-api/metrics)
echo "  Cache Hit Rate: $(echo $METRICS | jq -r '.cache.hit_rate')%"
echo "  Compression Ratio: $(echo $METRICS | jq -r '.compression.ratio')"
echo "  Pool Utilization: $(echo $METRICS | jq -r '.pool.utilization')%"
echo ""

echo "✅ Validation Complete!"
```

**Run validation**:

```bash
chmod +x scripts/validate_staging.sh
./scripts/validate_staging.sh
```

---

### Step 8: Enable Monitoring Dashboard (15 minutes)

**Create monitoring endpoint**: `src/monitoring.rs`

```rust
use axum::{Json, Router, routing::get};
use serde_json::{json, Value};

pub fn monitoring_routes(storage: Arc<CachedTursoStorage>) -> Router {
    Router::new()
        .route("/metrics", get(get_metrics))
        .route("/health", get(health_check))
        .with_state(storage)
}

async fn get_metrics(
    State(storage): State<Arc<CachedTursoStorage>>
) -> Json<Value> {
    let cache_stats = storage.stats();
    let pool_stats = storage.storage().pool_statistics().await;
    let ka_stats = storage.storage().keepalive_statistics();
    let prep_stats = storage.storage().prepared_cache_stats();
    
    Json(json!({
        "cache": {
            "hit_rate": cache_stats.hit_rate(),
            "episode_hit_rate": cache_stats.episode_hit_rate(),
            "pattern_hit_rate": cache_stats.pattern_hit_rate(),
            "episode_hits": cache_stats.episode_hits,
            "episode_misses": cache_stats.episode_misses,
        },
        "connection_pool": pool_stats.map(|s| json!({
            "active": s.active_connections,
            "idle": s.idle_connections,
            "max": s.max_connections,
            "utilization": s.utilization,
        })),
        "keepalive": ka_stats.map(|s| json!({
            "connections_created": s.total_connections_created,
            "connections_refreshed": s.total_connections_refreshed,
            "stale_detected": s.total_stale_detected,
            "avg_time_saved_ms": s.avg_time_saved_ms,
        })),
        "prepared_statements": {
            "hit_rate": prep_stats.hit_rate(),
            "hits": prep_stats.hits,
            "misses": prep_stats.misses,
            "avg_prep_time_us": prep_stats.avg_preparation_time_us,
        },
    }))
}

async fn health_check(
    State(storage): State<Arc<CachedTursoStorage>>
) -> Json<Value> {
    let healthy = storage.storage().health_check().await.unwrap_or(false);
    
    Json(json!({
        "status": if healthy { "healthy" } else { "unhealthy" },
        "optimizations": "enabled",
        "features": {
            "keepalive": true,
            "compression": true,
            "cache": true,
            "batch_operations": true,
        }
    }))
}
```

**Access monitoring dashboard**:

```bash
# View metrics
curl http://staging-api/metrics | jq

# Check health
curl http://staging-api/health | jq
```

---

## 📊 Success Criteria

### Performance Metrics

| Metric | Target | Validation |
|--------|--------|-----------|
| **Cache Hit Rate** | > 80% | Check `/metrics` endpoint |
| **Single Read Latency** | < 30ms | Run validation script |
| **Cached Read Latency** | < 5ms | Run validation script |
| **Batch Store (10)** | < 200ms | Run validation script |
| **Connection Stale Rate** | < 5% | Check keepalive stats |
| **Compression Ratio** | > 0.4 (60% savings) | Check compression stats |

### Operational Metrics

- [ ] No connection pool exhaustion errors
- [ ] No memory leaks (monitor for 24h)
- [ ] No increased error rates
- [ ] Logs show optimization features active
- [ ] Health check endpoint returning healthy

---

## 🔍 Monitoring Schedule

### First 2 Hours (Critical Period)
- Monitor every 15 minutes
- Check error logs continuously
- Validate performance metrics
- Watch for memory growth

### Next 6 Hours
- Monitor every hour
- Review cache hit rates
- Check pool utilization
- Validate compression ratios

### Next 24 Hours
- Monitor every 4 hours
- Review daily metrics
- Check for any anomalies
- Prepare production readiness report

---

## ⚠️ Rollback Plan

### Trigger Conditions
- Error rate > 5% increase
- Response time > 2x slower
- Memory usage > 3GB
- Cache hit rate < 30%
- Connection pool exhaustion

### Rollback Steps

```bash
# 1. Stop current deployment
kubectl rollout undo deployment/your-app -n staging

# OR for systemd
sudo systemctl stop your-app-staging
sudo systemctl start your-app-staging-backup

# 2. Verify rollback
curl http://staging-api/health

# 3. Check logs
tail -f /var/log/your-app/app.log

# 4. Notify team
echo "Rolled back staging deployment" | slack-notify #team-channel
```

### Rollback Configuration

Keep old binary available:
```bash
# Before deploying
cp target/release/your-binary target/release/your-binary.backup

# Rollback command
sudo systemctl stop your-app-staging
cp target/release/your-binary.backup target/release/your-binary
sudo systemctl start your-app-staging
```

---

## 📝 Post-Deployment Tasks

### Immediate (Day 1)
- [ ] Validate all success criteria met
- [ ] Document any issues encountered
- [ ] Share metrics with team
- [ ] Update runbook with learnings

### Week 1
- [ ] Review 7-day performance trends
- [ ] Fine-tune cache sizes if needed
- [ ] Adjust connection pool settings
- [ ] Prepare production deployment plan

### Before Production
- [ ] Complete staging validation (7-14 days)
- [ ] Document optimal configuration
- [ ] Train operations team
- [ ] Create production runbook
- [ ] Schedule production deployment

---

## 📞 Communication Plan

### Stakeholders
- Development Team
- DevOps/SRE Team
- QA Team
- Product Management

### Status Updates

**Before Deployment**:
```
Subject: Staging Deployment - Turso Optimizations

We're deploying database optimizations to staging today at [TIME].

Expected Impact: 10-15x performance improvement
Risk Level: LOW (all features have fallback)
Duration: ~90 minutes
Monitoring: First 24 hours critical

Rollback plan is ready if needed.
```

**After Deployment**:
```
Subject: ✅ Staging Deployment Complete - Initial Results

Deployment completed successfully at [TIME].

Initial metrics (first 2 hours):
- Cache hit rate: XX%
- Avg read latency: XXms (target: <30ms)
- No errors detected

Continuing to monitor for 24h before production consideration.
```

---

## 🎓 Training Materials

### Quick Reference Card

```
TURSO OPTIMIZATIONS - QUICK REFERENCE

Monitoring:
  Metrics: curl http://staging-api/metrics | jq
  Health:  curl http://staging-api/health

Success Indicators:
  ✅ Cache hit rate > 80%
  ✅ Read latency < 30ms
  ✅ No connection errors

Troubleshooting:
  Low cache hit rate → Increase cache size
  Connection timeouts → Increase max_connections
  High memory → Reduce cache sizes

Emergency Rollback:
  kubectl rollout undo deployment/your-app -n staging
```

---

## ✅ Deployment Checklist

**Pre-Deployment** (30 minutes before)
- [ ] Cargo.toml updated with features
- [ ] Environment variables set
- [ ] Configuration file created
- [ ] Storage initialization code updated
- [ ] Tests passing
- [ ] Team notified
- [ ] Monitoring dashboard ready
- [ ] Rollback plan tested

**During Deployment** (90 minutes)
- [ ] Step 1: Update Cargo.toml ✓
- [ ] Step 2: Create config file ✓
- [ ] Step 3: Update initialization ✓
- [ ] Step 4: Set environment variables ✓
- [ ] Step 5: Deploy to staging ✓
- [ ] Step 6: Verify deployment ✓
- [ ] Step 7: Run validation ✓
- [ ] Step 8: Enable monitoring ✓

**Post-Deployment** (First 24 hours)
- [ ] All success criteria met
- [ ] No errors detected
- [ ] Performance validated
- [ ] Metrics collected
- [ ] Team updated
- [ ] Documentation updated

---

**Deployment Lead**: _______________  
**Date/Time**: _______________  
**Sign-off**: _______________

---

**Questions or Issues?** Refer to:
- `plans/PRODUCTION_ENABLEMENT_GUIDE.md` - Complete configuration guide
- `plans/PHASE1_OPTIMIZATION_COMPLETE.md` - Technical details
- Troubleshooting section above
