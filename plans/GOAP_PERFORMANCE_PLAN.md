# GOAP: Performance Improvement Plan

**Date**: 2026-05-16
**Type**: Performance Optimization Plan
**Priority**: P2 - System performance
**WG**: WG-146

---

## Executive Summary

**Goal**: Systematically identify and resolve performance bottlenecks across the memory storage system.

**Current State**: Sub-optimal performance in specific storage operations (episode creation, memory retrieval).
**Target**: Meet or exceed all performance targets defined in AGENTS.md.

---

## Phase 1: ANALYZE

### Performance Targets (from AGENTS.md)

| Operation | Target | Current (Estimated) | Gap |
|-----------|--------|--------------------|-----|
| Episode Creation | < 50ms | ~45-55ms | Near target |
| Step Logging | < 20ms | ~15-25ms | Near target |
| Episode Completion | < 500ms | ~400-600ms | Near target |
| Memory Retrieval | < 100ms | ~80-150ms | Variable |

### Bottleneck Candidates

| Area | Impact | Investigation Priority |
|------|--------|----------------------|
| Turso connection pooling | HIGH | P1 |
| Embedding generation latency | HIGH | P1 |
| Postcard serialization overhead | MEDIUM | P2 |
| Redb cache eviction cost | MEDIUM | P2 |
| Pattern extraction CPU usage | LOW | P3 |

---

## Phase 2: DECOMPOSE

### WG Tasks

| WG | Task | Priority | Dependencies |
|----|------|----------|--------------|
| WG-146.1 | Benchmark current performance baseline | HIGH | None |
| WG-146.2 | Profile Turso query latency | HIGH | WG-146.1 |
| WG-146.3 | Analyze embedding generation pipeline | HIGH | WG-146.1 |
| WG-146.4 | Review postcard serialization hotspots | MEDIUM | WG-146.1 |
| WG-146.5 | Optimize redb cache eviction path | MEDIUM | WG-146.4 |
| WG-146.6 | Report results and update targets | MEDIUM | All above |

---

## Phase 3: EXECUTE

### Sprint 1: Measurement

```text
Week 1: WG-146.1 (baseline benchmarks) + WG-146.2 (Turso profiling)
Week 2: WG-146.3 (embedding analysis) + WG-146.4 (postcard review)
Week 3: WG-146.5 (optimizations) + WG-146.6 (reporting)
```

### Benchmark Commands

```bash
# Run performance benchmarks
cargo bench --package do-memory-benches

# Run specific benchmark
cargo bench --package do-memory-benches -- "episode_lifecycle"

# Check against targets
bash scripts/check_performance_regression.sh
```

---

## Quality Gates

| Milestone | Check | Target |
|-----------|-------|--------|
| Baseline measured | Benchmark report in plans/ | All metrics collected |
| Bottleneck identified | Profiling report | Top 3 bottlenecks |
| Optimizations applied | Re-benchmark | All targets met |
| Regression check | CI benchmark comparison | No regression vs baseline |

---

## Cross-References

- AGENTS.md: Performance targets
- benches/: Criterion benchmarks
- scripts/check_performance_regression.sh: Regression detection
- GOAP_STATE.md: Current GOAP state tracking
