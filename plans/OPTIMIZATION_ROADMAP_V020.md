# Optimization Roadmap - v0.2.0

**Target Release**: Q1 2026
**Duration**: 8-10 weeks
**Priority**: P1 - High Value Improvements

---

## 🎯 Goals

1. **Code Quality**: Achieve 100% compliance with codebase standards
2. **Performance**: Improve throughput by 15-30% across key operations
3. **Maintainability**: Reduce file sizes, improve modularity
4. **Scalability**: Optimize for high-concurrency workloads

---

## 📋 Phase-by-Phase Breakdown

### Phase 1: Code Quality & Standards Compliance (3 weeks)
**Goal**: Fix all codebase standard violations
**Effort**: 70-95 hours
**Impact**: HIGH - Required for codebase health

#### Tasks

**Week 1: Large File Splitting (40-50 hours)**

Priority Order:

1. **`memory-mcp/src/patterns/predictive.rs`** (2,435 LOC)
   - Target: 5 modules (~450 LOC each)
   - Modules:
     - `predictive/mod.rs` - Core types and traits
     - `predictive/forecasting.rs` - ETS forecasting logic
     - `predictive/anomaly.rs` - DBSCAN anomaly detection
     - `predictive/analysis.rs` - Analysis algorithms
     - `predictive/tests.rs` - Test suite
   - Effort: 10-12 hours

2. **`memory-storage-turso/src/storage.rs`** (2,243 LOC)
   - Target: 5 modules (~450 LOC each)
   - Modules:
     - `storage/mod.rs` - Core TursoStorage impl
     - `storage/episodes.rs` - Episode operations
     - `storage/patterns.rs` - Pattern operations
     - `storage/embeddings.rs` - Embedding operations
     - `storage/monitoring.rs` - Monitoring operations
   - Effort: 10-12 hours

3. **`memory-storage-redb/src/storage.rs`** (1,514 LOC)
   - Target: 3 modules (~500 LOC each)
   - Modules:
     - `storage/mod.rs` - Core RedbStorage impl
     - `storage/operations.rs` - CRUD operations
     - `storage/queries.rs` - Query operations
   - Effort: 6-8 hours

4. **`memory-core/src/memory/mod.rs`** (1,461 LOC)
   - Target: 3 modules (~480 LOC each)
   - Modules:
     - `memory/core.rs` - Core SelfLearningMemory
     - `memory/operations.rs` - Episode operations
     - `memory/retrieval.rs` - Already exists, integrate better
   - Effort: 6-8 hours

5. **`memory-mcp/src/server.rs`** (1,414 LOC)
   - Target: 3 modules (~470 LOC each)
   - Modules:
     - `server/mod.rs` - Core server implementation
     - `server/handlers.rs` - Request handlers
     - `server/tools.rs` - MCP tool implementations
   - Effort: 6-8 hours

**Validation:**
- [ ] All files ≤ 500 LOC
- [ ] All tests passing
- [ ] No clippy warnings
- [ ] Documentation updated

---

**Week 2: Error Handling Audit (20-30 hours)**

**Objective**: Replace 356 unwrap/expect calls with proper error handling

**Strategy**:
```rust
// Before: Potential panic
let value = some_option.unwrap();

// After: Proper error handling
let value = some_option
    .ok_or_else(|| Error::MissingValue("Expected value not found"))?;

// Or with context
let value = some_option
    .context("Failed to get required value")?;
```

**Tasks**:
- [ ] Audit all unwrap() calls (est. 200)
- [ ] Audit all expect() calls (est. 156)
- [ ] Convert to proper error handling
- [ ] Add context to errors
- [ ] Update tests
- [ ] Document error cases

**Validation:**
- [ ] <50 unwrap/expect calls remaining (only in tests)
- [ ] All error paths tested
- [ ] Error messages clear and actionable

---

**Week 3: Dependency Cleanup (10-15 hours)**

**Objective**: Consolidate duplicate dependencies, reduce binary size

**Duplicate Dependencies to Resolve**:
```toml
# Target: Single version of each
approx = "0.5.1"         # Remove v0.4.0
nalgebra = "0.34.1"      # Remove v0.32.6
changepoint = "0.15.0"   # Remove v0.14.2
argmin = "0.11.0"        # Remove v0.8.1
rv = "0.19.1"            # Remove v0.16.5
```

**Tasks**:
- [ ] Update dependency tree
- [ ] Test with consolidated versions
- [ ] Audit unused dependencies
- [ ] Optimize feature flags
- [ ] Measure binary size improvement

**Success Metrics**:
- [ ] Zero duplicate dependencies
- [ ] Binary size < 1.5 GB (from 2.1 GB)
- [ ] Build time < 5 min (clean build)

---

### Phase 2: Performance Optimization (3 weeks)
**Goal**: Achieve 15-30% performance improvement
**Effort**: 60-90 hours
**Impact**: HIGH - Better user experience

#### Week 1: Clone Reduction (20-30 hours)

**Objective**: Reduce 298 clone operations to <200

**Hot Paths to Optimize**:

1. **Episode Handling** (Est. 50 clones)
```rust
// Before: Expensive clone in hot path
pub async fn store_episode(&self, episode: Episode) -> Result<()> {
    let episode_clone = episode.clone(); // Unnecessary
    // ...
}

// After: Use reference
pub async fn store_episode(&self, episode: &Episode) -> Result<()> {
    // No clone needed
}
```

2. **Pattern Retrieval** (Est. 40 clones)
```rust
// Before: Clone in loop
for pattern in patterns.iter() {
    let p = pattern.clone();
    results.push(process(p));
}

// After: Use Arc
let patterns: Vec<Arc<Pattern>> = // ...
for pattern in patterns.iter() {
    results.push(process(Arc::clone(pattern)));
}
```

3. **Context Propagation** (Est. 30 clones)
```rust
// Before: Clone context everywhere
async fn operation(&self, ctx: TaskContext) {
    let ctx_copy = ctx.clone();
    inner_operation(ctx_copy).await;
}

// After: Use Cow or reference
async fn operation(&self, ctx: &TaskContext) {
    inner_operation(ctx).await;
}
```

**Tasks**:
- [ ] Profile clone hotspots
- [ ] Convert to Arc/Cow/references
- [ ] Benchmark improvements
- [ ] Update API signatures
- [ ] Fix breaking changes

**Success Metrics**:
- [ ] <200 clone operations
- [ ] 5-15% throughput improvement
- [ ] No performance regressions

---

#### Week 2: Database Optimization (25-35 hours)

**Objective**: Improve database operation performance by 10-20%

**1. Query Result Caching** (12-15 hours)

Implementation:
```rust
// memory-storage-turso/src/query_cache.rs
use lru::LruCache;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct QueryCache {
    cache: Arc<RwLock<LruCache<String, CachedResult>>>,
    ttl: Duration,
}

impl QueryCache {
    pub async fn get_or_query<F, Fut, T>(
        &self,
        key: String,
        query_fn: F,
    ) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T>>,
        T: Clone,
    {
        // Check cache
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.peek(&key) {
                if !cached.is_expired() {
                    return Ok(cached.value.clone());
                }
            }
        }
        
        // Execute query
        let result = query_fn().await?;
        
        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.put(key, CachedResult::new(result.clone()));
        }
        
        Ok(result)
    }
}
```

**2. Batch Query Operations** (8-10 hours)

```rust
// Batch multiple queries into single round-trip
pub async fn batch_get_episodes(
    &self,
    episode_ids: &[Uuid],
) -> Result<Vec<Episode>> {
    let placeholders = episode_ids
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(",");
    
    let sql = format!(
        "SELECT * FROM episodes WHERE episode_id IN ({})",
        placeholders
    );
    
    // Execute single query for all IDs
    // ...
}
```

**3. Helper Macros for Row Conversion** (5-10 hours)

```rust
macro_rules! extract_field {
    ($row:expr, $idx:expr, $type:ty, $field:expr) => {
        $row.get::<$type>($idx).map_err(|e| {
            Error::Storage(format!("Failed to get {}: {}", $field, e))
        })?
    };
}

fn row_to_episode(&self, row: &Row) -> Result<Episode> {
    Ok(Episode {
        episode_id: extract_field!(row, 0, String, "episode_id").parse()?,
        task_type: extract_field!(row, 1, String, "task_type").into(),
        // Much cleaner!
    })
}
```

**Success Metrics**:
- [ ] 50% cache hit rate for common queries
- [ ] 10-20% faster query operations
- [ ] Reduced database load

---

#### Week 3: Memory Optimization (15-25 hours)

**Objective**: Reduce memory allocations by 10-20%

**1. String Allocation Reduction** (8-12 hours)

```rust
// Before: Multiple allocations
let json = serde_json::to_string(&data)?;
let bytes = json.as_bytes();

// After: Direct serialization
let mut buffer = Vec::with_capacity(512);
serde_json::to_writer(&mut buffer, &data)?;
```

**2. Buffer Reuse** (4-8 hours)

```rust
pub struct BufferPool {
    buffers: Arc<Mutex<Vec<Vec<u8>>>>,
}

impl BufferPool {
    pub fn acquire(&self) -> Vec<u8> {
        self.buffers.lock().unwrap()
            .pop()
            .unwrap_or_else(|| Vec::with_capacity(1024))
    }
    
    pub fn release(&self, mut buffer: Vec<u8>) {
        buffer.clear();
        let mut buffers = self.buffers.lock().unwrap();
        if buffers.len() < 100 {
            buffers.push(buffer);
        }
    }
}
```

**3. Embedding Storage Optimization** (3-5 hours)

Ensure all embedding paths use postcard or native F32_BLOB:

```rust
// Consistent postcard usage
pub async fn store_embedding(&self, embedding: &[f32]) -> Result<()> {
    let bytes = postcard::to_allocvec(embedding)?;
    // Store binary data
}
```

**Success Metrics**:
- [ ] 10-20% reduction in allocations
- [ ] Faster serialization/deserialization
- [ ] Lower memory footprint

---

### Phase 3: Enhancements & Polish (3-4 weeks)
**Goal**: Add production-ready features
**Effort**: 85-125 hours
**Impact**: MEDIUM - Better observability and scalability

#### Week 1: Benchmarking Framework (20-30 hours)

**Objective**: Comprehensive performance tracking

```rust
// benches/comprehensive.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn episode_lifecycle_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("episode_lifecycle");
    
    for size in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("full_cycle", size),
            size,
            |b, &size| {
                b.iter(|| async {
                    // Create episode
                    let id = memory.start_episode(/* ... */).await;
                    
                    // Log steps
                    for i in 0..*size {
                        memory.log_step(id, create_step(i)).await;
                    }
                    
                    // Complete episode
                    memory.complete_episode(id, outcome).await;
                });
            }
        );
    }
    
    group.finish();
}

criterion_group!(benches, episode_lifecycle_benchmark);
criterion_main!(benches);
```

**Tasks**:
- [ ] Add benchmarks for all core operations
- [ ] Set up criterion for continuous benchmarking
- [ ] Create performance baselines
- [ ] Add regression detection
- [ ] CI integration

---

#### Week 2: Observability (25-35 hours)

**Objective**: Production-ready monitoring

**1. Structured Tracing** (12-18 hours)

```rust
use tracing::{instrument, info_span};
use tracing_subscriber::prelude::*;

#[instrument(skip(self), fields(episode_id = %id))]
pub async fn complete_episode(&self, id: Uuid) -> Result<()> {
    let span = info_span!("complete_episode", episode_id = %id);
    let _enter = span.enter();
    
    // Automatically traced
    self.extract_patterns(id).await?;
    self.calculate_reward(id).await?;
    
    Ok(())
}
```

**2. Metrics Collection** (8-12 hours)

```rust
use metrics::{counter, histogram, gauge};

pub async fn store_episode(&self, episode: &Episode) -> Result<()> {
    counter!("episodes_stored_total").increment(1);
    histogram!("episode_size_bytes").record(episode.size() as f64);
    gauge!("active_episodes").set(self.active_count() as f64);
    
    // ... existing logic
}
```

**3. Health Check Endpoints** (5-5 hours)

```rust
pub struct HealthCheck {
    pub database: HealthStatus,
    pub cache: HealthStatus,
    pub memory: MemoryStats,
}

pub async fn health_check(&self) -> HealthCheck {
    HealthCheck {
        database: self.check_database().await,
        cache: self.check_cache().await,
        memory: self.memory_stats(),
    }
}
```

---

#### Week 3: Advanced Features (30-40 hours)

**1. Query Result Caching** (Already covered in Phase 2)

**2. Batch Processing** (15-20 hours)

```rust
pub struct BatchProcessor {
    queue: Vec<Operation>,
    max_batch_size: usize,
    flush_interval: Duration,
}

impl BatchProcessor {
    pub async fn add(&mut self, op: Operation) {
        self.queue.push(op);
        
        if self.queue.len() >= self.max_batch_size {
            self.flush().await;
        }
    }
    
    pub async fn flush(&mut self) -> Result<Vec<Result<()>>> {
        let batch = std::mem::take(&mut self.queue);
        
        // Process all in parallel
        let results = futures::future::join_all(
            batch.into_iter().map(|op| self.process(op))
        ).await;
        
        results
    }
}
```

**3. Advanced Indexing** (15-20 hours)

```rust
// Add composite indexes for common queries
CREATE INDEX idx_episodes_task_domain 
    ON episodes(task_type, domain, created_at DESC);

CREATE INDEX idx_patterns_effectiveness 
    ON patterns(success_rate DESC, occurrence_count DESC);
```

---

#### Week 4: Testing & Documentation (15-20 hours)

**Tasks**:
- [ ] Add performance tests
- [ ] Update all documentation
- [ ] Create optimization guide
- [ ] Write migration guide
- [ ] Performance comparison report

---

## 🎯 Success Metrics

### Code Quality

| Metric | Before | Target | Status |
|--------|--------|--------|--------|
| Files > 500 LOC | 15 | 0 | ⏳ |
| Unwrap/Expect | 356 | <50 | ⏳ |
| Binary Size | 2.1 GB | <1.5 GB | ⏳ |
| Duplicate Deps | 5+ | 0 | ⏳ |

### Performance

| Operation | Before | Target | Improvement |
|-----------|--------|--------|-------------|
| Episode Creation | 2.5 µs | 2.0 µs | 20% |
| Step Logging | 1.1 µs | 0.9 µs | 18% |
| Episode Completion | 3.8 µs | 3.0 µs | 21% |
| Pattern Extraction | 10.4 µs | 8.0 µs | 23% |
| Memory Retrieval | 721 µs | 500 µs | 31% |
| Clone Operations | 298 | <200 | 33% |

### Observability

| Feature | Before | Target | Status |
|---------|--------|--------|--------|
| Structured Tracing | Partial | Complete | ⏳ |
| Metrics Collection | Basic | Comprehensive | ⏳ |
| Health Checks | Basic | Detailed | ⏳ |
| Benchmarking | Partial | Continuous | ⏳ |

---

## 📅 Timeline

```
Week 1-3:  Phase 1 - Code Quality
Week 4-6:  Phase 2 - Performance
Week 7-10: Phase 3 - Enhancements
```

**Total Duration**: 10 weeks (with buffer)
**Target Release**: Q1 2026

---

## 🚀 Quick Wins (Can do immediately)

1. **Add Pre-commit Hooks** (1 hour)
2. **Remove Debug Statements** (2 hours)
3. **Consolidate String Constants** (2 hours)
4. **Add Linting Rules** (1 hour)
5. **Update Documentation** (4 hours)

**Total**: ~10 hours for immediate improvements

---

## 📊 Risk Assessment

### High Risk
- **File splitting may break tests** → Mitigation: Incremental changes, full test suite
- **API changes from clone reduction** → Mitigation: Deprecation warnings, migration guide

### Medium Risk
- **Performance regressions** → Mitigation: Continuous benchmarking
- **Dependency conflicts** → Mitigation: Careful version selection, thorough testing

### Low Risk
- **Documentation drift** → Mitigation: Update docs with code changes
- **Build time increase** → Mitigation: Optimize feature flags

---

## 🎓 Team Requirements

### Skills Needed
- Rust performance optimization
- Database query optimization
- Async/await patterns
- Benchmarking and profiling

### Tools Required
- Criterion for benchmarking
- flamegraph for profiling
- cargo-bloat for binary analysis
- cargo-audit for security

---

## 📚 Related Documents

- `plans/OPTIMIZATION_ANALYSIS_2025-12-29.md` - Detailed analysis
- `plans/PHASE3_ADVANCED_OPTIMIZATIONS_PLAN.md` - Advanced features
- `plans/ROADMAPS/ROADMAP_ACTIVE.md` - Active development
- `AGENTS.md` - Coding standards

---

**Created**: 2025-12-29
**Version**: v0.2.0 Roadmap
**Status**: Ready for Review
