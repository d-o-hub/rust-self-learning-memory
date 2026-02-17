# Phase 2 Storage Optimization - Execution Plan

**Date**: 2026-02-12
**Orchestrator**: GOAP Agent
**Phase**: Storage Optimization (Parallel Execution)
**Dependencies**: Phase 1 Complete (2.2x episode completion speedup achieved)

---

## Executive Summary

Phase 2 targets storage layer optimization with 4 high-priority improvements:

1. **Zstd Compression Enablement** - 40% bandwidth reduction
2. **Turso Batch Writes** - 2-3x write improvement
3. **redb Batch Writes** - 2-3x write improvement
4. **redb Read-Through Cache** - 5-10x read improvement

**Overall Target**: 2-3x storage operations improvement

---

## Current State Analysis

### Compression Module
- **Location**: `memory-storage-turso/src/compression/`
- **Status**: ✅ Fully implemented
- **Features**: LZ4, Zstd, Gzip
- **Issue**: Not enabled by default in Cargo.toml

### Turso Transport
- **Location**: `memory-storage-turso/src/transport/`
- **Status**: ✅ Compression support exists
- **Issue**: Not actively used in storage operations

### Batch Writes
- **Turso**: ❌ Not implemented
- **redb**: ❌ Not implemented (has batch_embedding methods only)

### Read-Through Cache
- **redb**: ❌ Not implemented (has LRU but no read-through)

---

## Task Decomposition

### Goal 1: Enable Zstd Compression for Turso (Priority: P0)

**Success Criteria**:
- [ ] Compression enabled by default
- [ ] 40% bandwidth reduction measured
- [ ] <1MB payloads automatically compressed
- [ ] Integration with transport layer
- [ ] Performance validated

**Tasks**:
1. Update Cargo.toml to enable compression-zstd by default
2. Integrate compression into TursoStorage write operations
3. Add compression statistics tracking
4. Write integration tests
5. Validate bandwidth reduction

**Estimated Time**: 2-3 hours

**Agent Assignment**: rust-specialist

---

### Goal 2: Implement Turso Batch Writes (Priority: P0)

**Success Criteria**:
- [ ] Batch size: 100 operations
- [ ] Flush interval: 100ms or on demand
- [ ] 2-3x write improvement measured
- [ ] Prepared statements used
- [ ] Tests passing

**Tasks**:
1. Design batch write API
2. Implement in-memory batch buffer
3. Add flush trigger (size-based + time-based)
4. Use prepared statements for batch operations
5. Write integration tests
6. Validate performance improvement

**Estimated Time**: 3-4 hours

**Agent Assignment**: rust-specialist

---

### Goal 3: Implement redb Batch Writes (Priority: P0)

**Success Criteria**:
- [ ] Batch size: 100 operations
- [ ] Flush interval: 100ms or on demand
- [ ] 2-3x write improvement measured
- [ ] Transaction batching
- [ ] Tests passing

**Tasks**:
1. Design batch write API for redb
2. Implement transaction batch buffer
3. Add flush triggers
4. Write integration tests
5. Validate performance improvement

**Estimated Time**: 3-4 hours

**Agent Assignment**: rust-specialist

---

### Goal 4: Add Read-Through Cache for redb (Priority: P1)

**Success Criteria**:
- [ ] LRU cache configured for hot episodes
- [ ] Lazy loading from storage
- [ ] 5-10x read improvement for cache hits
- [ ] Cache invalidation on writes
- [ ] Tests passing

**Tasks**:
1. Design read-through cache API
2. Implement lazy loading on cache miss
3. Add cache population on read
4. Implement cache invalidation on write
5. Write integration tests
6. Validate read performance

**Estimated Time**: 2-3 hours

**Agent Assignment**: rust-specialist

---

## Execution Strategy

### Phase 2.1: Compression Enablement (Sequential)
**Duration**: 2-3 hours
**Agent**: rust-specialist (1 agent)

**Tasks**:
- Task 1.1: Enable compression-zstd in Cargo.toml
- Task 1.2: Integrate compression into transport layer
- Task 1.3: Add compression statistics
- Task 1.4: Write tests
- Task 1.5: Validate performance

**Quality Gate**: Compression active, 40% bandwidth reduction achieved

---

### Phase 2.2: Parallel Storage Optimization (Parallel)
**Duration**: 3-4 hours
**Agents**: rust-specialist (3-4 agents in parallel)

**Parallel Track 1 (Turso Batch)**:
- Task 2.1: Design batch API
- Task 2.2: Implement batch buffer
- Task 2.3: Add flush logic
- Task 2.4: Write tests

**Parallel Track 2 (redb Batch)**:
- Task 3.1: Design batch API
- Task 3.2: Implement transaction batching
- Task 3.3: Add flush logic
- Task 3.4: Write tests

**Parallel Track 3 (redb Read-Through)**:
- Task 4.1: Design cache API
- Task 4.2: Implement lazy loading
- Task 4.3: Add invalidation logic
- Task 4.4: Write tests

**Quality Gate**: All batch operations functional, 2-3x improvement measured

---

### Phase 2.3: Integration & Validation (Sequential)
**Duration**: 1-2 hours
**Agents**: build-compile, code-quality, test-runner

**Tasks**:
- Task 5.1: Build all workspace (build-compile)
- Task 5.2: Run quality gates (code-quality)
- Task 5.3: Run full test suite (test-runner)
- Task 5.4: Performance benchmarking
- Task 5.5: Documentation updates

**Quality Gate**:
- [ ] All code compiles
- [ ] Zero clippy warnings
- [ ] All tests passing
- [ ] Performance validated
- [ ] Documentation complete

---

## Agent Coordination Plan

### Phase 2.1: Compression Enablement
**Mode**: Sequential
**Agent**: rust-specialist

```markdown
Task: Enable Zstd compression for Turso transport

Context:
- Compression module already implemented in memory-storage-turso/src/compression/
- Transport layer exists in memory-storage-turso/src/transport/
- Need to enable compression-zstd feature by default
- Integrate into storage operations

Actions:
1. Enable compression-zstd in Cargo.toml
2. Integrate compression into TursoStorage::store_episode
3. Add compression statistics tracking
4. Write tests for compression behavior
5. Validate 40% bandwidth reduction

Success Criteria:
- Compression enabled by default
- 40% bandwidth reduction measured
- Tests passing
- Performance validated
```

---

### Phase 2.2: Parallel Storage Optimization
**Mode**: Parallel (3 agents)
**Agents**: 3x rust-specialist

**Agent 1 - Turso Batch**:
```markdown
Task: Implement batch writes for Turso backend

Actions:
1. Design batch write API (BatchWrite struct)
2. Implement in-memory batch buffer
3. Add flush triggers (100 operations OR 100ms)
4. Use prepared statements
5. Write integration tests
6. Validate 2-3x improvement

Deliverables:
- memory-storage-turso/src/storage/batch/mod.rs
- Tests in memory-storage-turso/tests/
- Performance validation report
```

**Agent 2 - redb Batch**:
```markdown
Task: Implement batch writes for redb backend

Actions:
1. Design batch write API
2. Implement transaction batching
3. Add flush triggers
4. Write integration tests
5. Validate 2-3x improvement

Deliverables:
- memory-storage-redb/src/storage/batch.rs
- Tests in memory-storage-redb/tests/
- Performance validation report
```

**Agent 3 - redb Read-Through**:
```markdown
Task: Implement read-through cache for redb

Actions:
1. Design read-through cache API
2. Implement lazy loading
3. Add cache invalidation
4. Write integration tests
5. Validate 5-10x improvement

Deliverables:
- memory-storage-redb/src/cache/read_through.rs
- Tests in memory-storage-redb/tests/
- Performance validation report
```

---

### Phase 2.3: Integration & Validation
**Mode**: Sequential
**Agents**: build-compile → code-quality → test-runner

```markdown
Phase 2.3.1: Build Validation (build-compile)
Actions:
1. cargo build --all --release
2. Validate no compilation errors
3. Check binary sizes

Phase 2.3.2: Quality Validation (code-quality)
Actions:
1. cargo fmt --all -- --check
2. cargo clippy --all -- -D warnings
3. Validate zero warnings
4. Check documentation coverage

Phase 2.3.3: Test Validation (test-runner)
Actions:
1. cargo test --all
2. Validate all tests passing
3. Check test coverage
4. Run performance benchmarks

Phase 2.3.4: Final Validation
Actions:
1. Aggregate performance metrics
2. Compare vs baseline
3. Generate validation report
4. Update documentation
```

---

## Dependency Graph

```
Phase 2.1 (Compression)
└─> Single agent sequential

Phase 2.2 (Parallel Storage)
├─> Agent 1: Turso Batch (independent)
├─> Agent 2: redb Batch (independent)
└─> Agent 3: redb Read-Through (independent)

Phase 2.3 (Integration)
└─> Sequential validation (depends on Phase 2.2)
```

---

## Success Criteria

### Functional
- [ ] Zstd compression enabled and functional
- [ ] Turso batch writes working
- [ ] redb batch writes working
- [ ] redb read-through cache working
- [ ] All features backward compatible

### Performance
- [ ] 40% bandwidth reduction (compression)
- [ ] 2-3x write improvement (Turso batch)
- [ ] 2-3x write improvement (redb batch)
- [ ] 5-10x read improvement (redb cache)
- [ ] No performance regressions

### Quality
- [ ] All code compiles
- [ ] Zero clippy warnings
- [ ] All tests passing
- [ ] >90% coverage maintained
- [ ] Documentation complete

---

## Risk Assessment

### High Risk
1. **Batch Write Corruption**: Concurrent batch operations
   - **Mitigation**: Use proper locking and transactions
   - **Validation**: Concurrent operation tests

2. **Cache Invalidation**: Stale data in read-through cache
   - **Mitigation**: Aggressive invalidation on writes
   - **Validation**: Cache consistency tests

### Medium Risk
3. **Compression Overhead**: CPU cost vs bandwidth savings
   - **Mitigation**: Benchmark and tune threshold
   - **Validation**: Performance profiling

4. **Batch Flush Latency**: Delayed writes
   - **Mitigation**: Configurable flush intervals
   - **Validation**: Latency benchmarks

### Low Risk
5. **API Compatibility**: Breaking changes
   - **Mitigation**: Maintain backward compatibility
   - **Validation**: Integration tests

---

## Quality Gates

### Gate 1: Compression Enablement
- [ ] Compression enabled in Cargo.toml
- [ ] 40% bandwidth reduction measured
- [ ] Integration tests passing
- [ ] No performance regression

### Gate 2: Batch Operations
- [ ] All batch implementations working
- [ ] 2-3x improvement measured
- [ ] Concurrent operations safe
- [ ] Tests comprehensive

### Gate 3: Read-Through Cache
- [ ] Cache hit rate >80% (hot data)
- [ ] 5-10x improvement on cache hits
- [ ] Invalidation correct
- [ ] No memory leaks

### Gate 4: Final Validation
- [ ] All code compiles
- [ ] Zero clippy warnings
- [ ] All tests passing
- [ ] Performance targets met
- [ ] Documentation complete

---

## Performance Targets

| Operation | Baseline | Target | Measurement |
|-----------|----------|--------|-------------|
| Network Transfer | 100% | 60% (-40%) | Bytes transferred |
| Turso Write | 50ms | 17-25ms (2-3x) | Batch write latency |
| redb Write | 10ms | 3-5ms (2-3x) | Batch write latency |
| redb Read (cache hit) | 5ms | 0.5-1ms (5-10x) | Cache read latency |
| redb Read (cache miss) | 5ms | 5ms (no change) | Direct read latency |

---

## Testing Plan

### Compression Tests
- [ ] Small payloads (<1KB) not compressed
- [ ] Large payloads (>1MB) compressed
- [ ] 40% reduction achieved
- [ ] Round-trip (compress/decompress) works
- [ ] Statistics tracking correct

### Batch Write Tests
- [ ] Batch size 100 operations
- [ ] Flush on size limit
- [ ] Flush on timeout (100ms)
- [ ] Manual flush works
- [ ] Concurrent batches safe
- [ ] 2-3x improvement measured

### Read-Through Cache Tests
- [ ] Cache miss triggers storage load
- [ ] Cache hit returns cached data
- [ ] Cache invalidation on writes
- [ ] LRU eviction works
- [ ] 5-10x improvement on hits
- [ ] No memory leaks

---

## Rollback Plan

If performance targets not met:

1. **Compression**: Disable via feature flag
2. **Batch Writes**: Make optional, add config toggle
3. **Read-Through Cache**: Disable by default, opt-in only

---

## Next Steps

### Immediate (Start Now)
1. [ ] Create execution plan ✅ (this document)
2. [ ] Start Phase 2.1 (Compression)
3. [ ] Spawn rust-specialist agent

### Today
4. [ ] Complete Phase 2.1
5. [ ] Start Phase 2.2 (parallel agents)

### Tomorrow
6. [ ] Complete Phase 2.2
7. [ ] Run Phase 2.3 (integration)
8. [ ] Generate final report

---

**Document Status**: ✅ COMPLETE
**Ready for Execution**: Yes
**Next Action**: Spawn rust-specialist for Phase 2.1
