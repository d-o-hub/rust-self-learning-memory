# Phase 3 Implementation Plan - Performance & Caching Optimization
**Date**: 2026-01-23
**Version**: v0.1.14
**Status**: Planning
**Effort Estimate**: 35-50 hours
**Timeline**: 2-3 weeks (Target completion: 2026-02-15)

---

## Executive Summary

Phase 3 focuses on caching optimization, query performance, and observability enhancements. Building on Phase 2's connection and compression improvements, this phase targets **additional 1.5-2x performance gains** through intelligent caching and query optimization.

### Key Discovery
**Adaptive cache already exists in `memory-storage-redb`!** Phase 3 will integrate it with Turso storage rather than building from scratch.

### Key Objectives
1. **Integrate existing adaptive cache** with Turso storage layer
2. **Implement query result caching** for common patterns
3. **Add prepared statement caching** to reduce parsing overhead
4. **Enhance observability** with detailed performance metrics
5. **Optimize batch operations** for bulk inserts/updates

### Expected Impact

| Metric | After Phase 2 | After Phase 3 | Additional Improvement |
|--------|---------------|---------------|----------------------|
| Cache hit rate | 70% | 85-90% | **+15-20%** |
| Query latency (cached) | 45ms | 5-10ms | **80-90% reduction** |
| Bulk insert throughput | 50/sec | 200-300/sec | **4-6x increase** |
| Query parsing overhead | ~5ms | <1ms | **80% reduction** |

---

## Phase 3 Architecture

```
┌────────────────────────────────────────────────────────────────┐
│                     Phase 3 Architecture                        │
├────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ┌─────────────┐                                               │
│   │   Client    │                                               │
│   │   Request   │                                               │
│   └──────┬──────┘                                               │
│          │                                                       │
│          ▼                                                       │
│   ┌─────────────────────────────────┐                          │
│   │  Query Result Cache (NEW)       │                          │
│   │  - Adaptive TTL from redb       │                          │
│   │  - Query pattern matching       │                          │
│   └──────┬──────────────────────────┘                          │
│          │ Cache Miss                                           │
│          ▼                                                       │
│   ┌─────────────────────────────────┐                          │
│   │  Prepared Statement Cache (NEW) │                          │
│   │  - SQL parsing optimization     │                          │
│   └──────┬──────────────────────────┘                          │
│          │                                                       │
│          ▼                                                       │
│   ┌─────────────────────────────────┐                          │
│   │  Keep-Alive Pool (Phase 2)      │                          │
│   │  + Adaptive Sizing (Phase 2)    │                          │
│   └──────┬──────────────────────────┘                          │
│          │                                                       │
│          ▼                                                       │
│   ┌─────────────────────────────────┐                          │
│   │  Turso DB with Compression      │                          │
│   │  (Phase 2)                      │                          │
│   └─────────────────────────────────┘                          │
│                                                                 │
│   ┌─────────────────────────────────┐                          │
│   │  Performance Metrics (NEW)      │◀─── All Layers          │
│   │  - Latency tracking             │                          │
│   │  - Cache statistics             │                          │
│   │  - Query patterns               │                          │
│   └─────────────────────────────────┘                          │
│                                                                 │
└────────────────────────────────────────────────────────────────┘
```

---

## Implementation Components

### 3.1 Adaptive Cache Integration 🔴 P0

**Priority**: P0 - Critical Path
**Files**: 
- `memory-storage-turso/src/cache/mod.rs` (NEW)
- `memory-storage-turso/src/cache/query_cache.rs` (NEW)
- `memory-storage-turso/src/lib.rs` (modification)

**Status**: Leverage existing `memory-storage-redb::cache::adaptive`

#### Problem Statement
Turso storage currently has no caching layer. Every query hits the database, even for frequently accessed data.

#### Implementation Design

```rust
use memory_storage_redb::cache::{AdaptiveCache, AdaptiveCacheConfig};
use std::sync::Arc;

/// Turso storage with integrated adaptive caching
pub struct CachedTursoStorage {
    /// Underlying Turso storage
    storage: Arc<TursoStorage>,
    
    /// Adaptive cache for episodes
    episode_cache: AdaptiveCache<Uuid, Episode>,
    
    /// Adaptive cache for patterns
    pattern_cache: AdaptiveCache<PatternId, Pattern>,
    
    /// Query result cache (for complex queries)
    query_cache: AdaptiveCache<QueryKey, QueryResult>,
    
    /// Cache configuration
    config: CacheConfig,
}

/// Key for query result caching
#[derive(Clone, Hash, Eq, PartialEq)]
enum QueryKey {
    EpisodesSince(DateTime<Utc>),
    EpisodesByMetadata { key: String, value: String },
    PatternsByType(String),
}

impl CachedTursoStorage {
    pub fn new(storage: TursoStorage, config: CacheConfig) -> Self {
        let episode_cache_config = AdaptiveCacheConfig {
            base_ttl: Duration::from_secs(300), // 5 minutes base
            max_ttl: Duration::from_secs(3600), // 1 hour max
            min_ttl: Duration::from_secs(60),   // 1 minute min
            max_entries: 10_000,
            access_count_weight: 0.6,
            recency_weight: 0.4,
        };
        
        Self {
            storage: Arc::new(storage),
            episode_cache: AdaptiveCache::new(episode_cache_config.clone()),
            pattern_cache: AdaptiveCache::new(episode_cache_config.clone()),
            query_cache: AdaptiveCache::new(episode_cache_config),
            config,
        }
    }
    
    /// Get episode with caching
    pub async fn get_episode_cached(&self, id: Uuid) -> Result<Option<Episode>> {
        // Check cache first
        if let Some(episode) = self.episode_cache.get(&id) {
            return Ok(Some(episode));
        }
        
        // Cache miss - fetch from storage
        if let Some(episode) = self.storage.get_episode(id).await? {
            self.episode_cache.insert(id, episode.clone());
            Ok(Some(episode))
        } else {
            Ok(None)
        }
    }
    
    /// Query episodes with result caching
    pub async fn query_episodes_since_cached(
        &self,
        since: DateTime<Utc>,
    ) -> Result<Vec<Episode>> {
        let key = QueryKey::EpisodesSince(since);
        
        // Check query cache
        if let Some(result) = self.query_cache.get(&key) {
            return Ok(result);
        }
        
        // Execute query
        let episodes = self.storage.query_episodes_since(since).await?;
        
        // Cache result
        self.query_cache.insert(key, episodes.clone());
        
        Ok(episodes)
    }
}
```

#### Integration Points
- **TursoStorage**: Wrap with `CachedTursoStorage`
- **StorageBackend trait**: Implement for `CachedTursoStorage`
- **Configuration**: Add cache settings to `TursoConfig`

#### Testing Strategy
| Test Type | Coverage |
|-----------|----------|
| Unit tests | Cache hit/miss logic |
| Integration tests | Storage + cache integration |
| Performance tests | Cache effectiveness |
| Load tests | Cache under pressure |

#### Effort & Risk
- **Effort**: 8-12 hours
- **Risk**: Low (reusing existing implementation)
- **Dependencies**: None (redb cache already exists)

---

### 3.2 Prepared Statement Cache 🟡 P1

**Priority**: P1 - High Value
**Files**: 
- `memory-storage-turso/src/prepared/mod.rs` (NEW)
- `memory-storage-turso/src/prepared/cache.rs` (NEW)

#### Problem Statement
SQL queries are parsed on every execution, adding ~2-5ms overhead per query. For repeated queries, this is wasteful.

#### Implementation Design

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use libsql::Statement;

/// Cache for prepared SQL statements
pub struct PreparedStatementCache {
    /// Cached prepared statements
    statements: RwLock<HashMap<String, Arc<Statement>>>,
    
    /// Cache statistics
    stats: RwLock<PreparedCacheStats>,
}

#[derive(Default)]
struct PreparedCacheStats {
    hits: u64,
    misses: u64,
    evictions: u64,
}

impl PreparedStatementCache {
    pub fn new() -> Self {
        Self {
            statements: RwLock::new(HashMap::new()),
            stats: RwLock::new(PreparedCacheStats::default()),
        }
    }
    
    /// Get or prepare a statement
    pub async fn get_or_prepare(
        &self,
        conn: &Connection,
        sql: &str,
    ) -> Result<Arc<Statement>> {
        // Check cache first
        {
            let cache = self.statements.read().unwrap();
            if let Some(stmt) = cache.get(sql) {
                self.stats.write().unwrap().hits += 1;
                return Ok(Arc::clone(stmt));
            }
        }
        
        // Cache miss - prepare statement
        self.stats.write().unwrap().misses += 1;
        let stmt = conn.prepare(sql).await?;
        let stmt_arc = Arc::new(stmt);
        
        // Store in cache
        {
            let mut cache = self.statements.write().unwrap();
            cache.insert(sql.to_string(), Arc::clone(&stmt_arc));
        }
        
        Ok(stmt_arc)
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> PreparedCacheStats {
        *self.stats.read().unwrap()
    }
}
```

#### Integration
Modify storage operations to use prepared statement cache:

```rust
impl TursoStorage {
    async fn store_episode_with_prepared(&self, episode: &Episode) -> Result<()> {
        let conn = self.get_connection().await?;
        
        // Use prepared statement cache
        let stmt = self.prepared_cache
            .get_or_prepare(&conn, INSERT_EPISODE_SQL)
            .await?;
        
        stmt.execute(params![...]).await?;
        Ok(())
    }
}
```

#### Effort & Risk
- **Effort**: 6-10 hours
- **Risk**: Low
- **Expected Impact**: 2-5ms reduction per query

---

### 3.3 Batch Operations Optimization 🟡 P1

**Priority**: P1 - High Value
**Files**: 
- `memory-storage-turso/src/storage/batch.rs` (NEW)
- `memory-storage-turso/src/lib.rs` (add batch methods)

#### Problem Statement
Storing multiple episodes/patterns requires multiple round trips to the database. Bulk operations should use transactions.

#### Implementation Design

```rust
impl TursoStorage {
    /// Store multiple episodes in a single transaction
    pub async fn store_episodes_batch(&self, episodes: Vec<Episode>) -> Result<()> {
        let conn = self.get_connection().await?;
        
        // Begin transaction
        conn.execute("BEGIN TRANSACTION", ()).await?;
        
        // Use prepared statement
        let stmt = self.prepared_cache
            .get_or_prepare(&conn, INSERT_EPISODE_SQL)
            .await?;
        
        for episode in episodes {
            stmt.execute(params![/* ... */]).await.map_err(|e| {
                // Rollback on error
                let _ = conn.execute("ROLLBACK", ());
                e
            })?;
        }
        
        // Commit transaction
        conn.execute("COMMIT", ()).await?;
        
        Ok(())
    }
}
```

#### Effort & Risk
- **Effort**: 8-12 hours
- **Risk**: Low
- **Expected Impact**: 4-6x throughput for bulk operations

---

### 3.4 Performance Metrics & Observability 🟢 P2

**Priority**: P2 - Nice to Have
**Files**: 
- `memory-storage-turso/src/metrics/mod.rs` (NEW)
- `memory-storage-turso/src/metrics/collector.rs` (NEW)

#### Implementation Design

```rust
use std::time::Instant;

pub struct TursoMetrics {
    /// Query latency histogram
    query_latencies: RwLock<HashMap<String, Vec<Duration>>>,
    
    /// Cache statistics
    cache_stats: RwLock<CacheStats>,
    
    /// Connection pool statistics
    pool_stats: RwLock<PoolStats>,
}

#[derive(Default)]
pub struct CacheStats {
    pub episode_hits: u64,
    pub episode_misses: u64,
    pub pattern_hits: u64,
    pub pattern_misses: u64,
    pub query_hits: u64,
    pub query_misses: u64,
}

impl TursoMetrics {
    /// Record query execution
    pub fn record_query(&self, operation: &str, duration: Duration) {
        let mut latencies = self.query_latencies.write().unwrap();
        latencies
            .entry(operation.to_string())
            .or_insert_with(Vec::new)
            .push(duration);
    }
    
    /// Get P50, P95, P99 latencies for an operation
    pub fn latency_percentiles(&self, operation: &str) -> Option<(Duration, Duration, Duration)> {
        let latencies = self.query_latencies.read().unwrap();
        let mut durations = latencies.get(operation)?.clone();
        
        if durations.is_empty() {
            return None;
        }
        
        durations.sort();
        let len = durations.len();
        
        let p50 = durations[len / 2];
        let p95 = durations[(len * 95) / 100];
        let p99 = durations[(len * 99) / 100];
        
        Some((p50, p95, p99))
    }
}
```

#### Effort & Risk
- **Effort**: 8-12 hours
- **Risk**: Low
- **Expected Impact**: Better operational visibility

---

## Implementation Phases

### Week 1: Caching Foundation
- **Days 1-2**: Integrate adaptive cache from redb
- **Days 3-4**: Implement query result caching
- **Day 5**: Testing and validation

### Week 2: Query Optimization
- **Days 1-2**: Implement prepared statement cache
- **Days 3-4**: Optimize batch operations
- **Day 5**: Integration testing

### Week 3: Observability & Polish
- **Days 1-2**: Add performance metrics
- **Days 3-4**: Comprehensive testing and benchmarks
- **Day 5**: Documentation and reporting

---

## Success Metrics

### Primary Metrics
1. **Cache Hit Rate**: Target 85-90% (from 70%)
2. **Query Latency (cached)**: Target 5-10ms (from 45ms)
3. **Bulk Insert Throughput**: Target 200-300/sec (from 50/sec)
4. **Statement Preparation Overhead**: Target <1ms (from ~5ms)

### Secondary Metrics
1. **Memory Usage**: Keep under 500MB for cache
2. **Cache Eviction Rate**: Target <10% per hour
3. **P99 Latency**: Keep under 100ms for all operations

---

## Risk Assessment

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Cache invalidation bugs | High | Low | Comprehensive testing, TTL safety net |
| Memory pressure from cache | Medium | Medium | Configurable cache limits, monitoring |
| Prepared statement staleness | Low | Low | Statement versioning, periodic refresh |
| Complexity increase | Medium | High | Good documentation, clear interfaces |

---

## Testing Strategy

### Unit Tests
- Cache hit/miss logic
- Prepared statement lifecycle
- Batch operation transactions
- Metrics collection

### Integration Tests
- Full storage + cache flow
- Cache invalidation scenarios
- Transaction rollback handling
- Multi-threaded access

### Performance Tests
- Before/after benchmarks for each feature
- Load testing under various patterns
- Cache effectiveness measurements
- Memory profiling

### Chaos Tests
- Cache eviction under pressure
- Connection pool exhaustion
- Transaction failures
- Concurrent access patterns

---

## Dependencies

### External Dependencies
- ✅ `memory-storage-redb::cache` (already exists)
- ✅ `libsql` (already integrated)
- ⚠️ May need `lru` crate for additional caching

### Internal Dependencies
- ✅ Phase 2 connection pooling (complete)
- ✅ Phase 2 compression (complete)
- ⚠️ Benchmark infrastructure (partially complete)

---

## Documentation Updates Required

1. **README.md**: Add Phase 3 features and configuration
2. **ARCHITECTURE.md**: Document caching layers
3. **CONFIGURATION.md**: Cache and metrics settings
4. **BENCHMARKS.md**: Phase 3 performance results

---

## Post-Implementation

### Immediate Actions
1. Run comprehensive benchmarks
2. Update all documentation
3. Create migration guide for existing users
4. Performance comparison report

### Future Considerations (Phase 4)
1. Read replica support
2. Predictive connection scaling
3. Advanced query optimization
4. Distributed caching

---

## Effort Summary

| Component | Effort | Priority |
|-----------|--------|----------|
| 3.1 Adaptive Cache Integration | 8-12h | 🔴 P0 |
| 3.2 Prepared Statement Cache | 6-10h | 🟡 P1 |
| 3.3 Batch Operations | 8-12h | 🟡 P1 |
| 3.4 Metrics & Observability | 8-12h | 🟢 P2 |
| Testing & Validation | 8-12h | - |
| Documentation | 2-4h | - |
| **Total** | **40-62h** | - |

**Conservative Estimate**: 50 hours over 2-3 weeks

---

## Conclusion

Phase 3 builds intelligently on existing infrastructure (redb's adaptive cache) and Phase 2's improvements. The focus on caching and query optimization will deliver significant performance gains with manageable complexity and risk.

**Expected Cumulative Impact**:
- Phase 1: 10-20x improvements (quick wins)
- Phase 2: 1.5-2x additional improvements (infrastructure)
- Phase 3: 1.5-2x additional improvements (caching)
- **Total**: ~20-80x overall improvement from baseline

**Key Innovation**: Leveraging existing adaptive cache implementation reduces risk and accelerates delivery.

---

*Document Version*: 1.0
*Created*: 2026-01-23
*Next Review*: After Phase 2 benchmarks
*Status*: Ready for Implementation
