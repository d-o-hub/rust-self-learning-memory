# Coverage Improvement Plan — 2026-04-30

**Generated**: 2026-04-30
**Goal**: Achieve ≥90% code coverage threshold
**Current State**: Coverage baseline measured

---

## Baseline Coverage Metrics (2026-04-30)

| Crate | Coverage | Target | Gap |
|-------|----------|--------|-----|
| do-memory-core | 73.67% | 90% | -16.33% |
| do-memory-storage-turso | 58.17% | 90% | -31.83% |
| do-memory-storage-redb | 43.09% | 90% | -46.91% |

**Overall**: Significantly below 90% threshold. All crates need improvement.

---

## Low Coverage Modules (Priority Targets)

### memory-core (< 80% coverage)
| Module | Coverage | Priority |
|--------|----------|----------|
| types/structs.rs | 34.38% | P1 |
| types/config.rs | 51.22% | P1 |
| spatiotemporal/retriever/types.rs | 31.25% | P1 |
| spatiotemporal/types.rs | 66.14% | P2 |
| conflict.rs | 0% | P1 |
| sync/synchronizer.rs | 0% | P1 |
| sync/types.rs | 0% | P1 |

### storage-turso (< 60% coverage)
| Module | Coverage | Priority |
|--------|----------|----------|
| trait_impls/mod.rs | 15.15% | P1 |
| storage/stats.rs | 0% | P1 |
| cache/adaptive_ttl.rs | partial | P2 |

### storage-redb (< 50% coverage)
| Module | Coverage | Priority |
|--------|----------|----------|
| storage_ops/clear.rs | 44.79% | P1 |
| storage_ops/stats.rs | 0% | P1 |
| storage_ops/schema.rs | 54.19% | P2 |

---

## Coverage Improvement Strategy

### Phase 1: Fix Zero-Coverage Modules (P1)
- Add basic unit tests for all 0% modules
- Focus on: conflict.rs, sync/synchronizer.rs, sync/types.rs, storage/stats.rs
- **Effort**: Low (basic functionality tests)
- **Expected Gain**: +5-10%

### Phase 2: Improve Low-Coverage Core (< 50%)
- Add comprehensive tests for types/structs.rs, types/config.rs
- Add tests for spatiotemporal/retriever/types.rs
- **Effort**: Medium
- **Expected Gain**: +10-15%

### Phase 3: Improve Storage Coverage
- Add tests for trait_impls in storage-turso
- Add tests for storage_ops in storage-redb
- **Effort**: Medium
- **Expected Gain**: +15-20%

### Phase 4: Integration Tests
- Add more e2e tests for full flow coverage
- **Effort**: High
- **Expected Gain**: +5-10%

---

## Execution Pattern

```
Phase 1 (Parallel): Add tests to 0% modules across all crates
Phase 2 (Sequential): Focus on memory-core low-coverage
Phase 3 (Parallel): Add tests to storage crates
Phase 4 (Sequential): Integration tests → Final validation
```

---

## Quality Gates

- Gate 1: All 0% modules have basic tests (coverage >0%)
- Gate 2: memory-core coverage ≥80%
- Gate 3: storage-turso coverage ≥70%
- Gate 4: storage-redb coverage ≥60%
- Gate 5: Overall coverage ≥90%

---

## Status

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 0: Fix blockers | ✅ Complete | OAuth tests fixed |
| Phase 1: Baseline | ✅ Complete | Metrics captured |
| Phase 2: Improve modules | 🔵 Pending | - |
| Phase 3: Storage tests | 🔵 Pending | - |
| Phase 4: Integration | 🔵 Pending | - |

---

## Cross-References

- `plans/STATUS/GOAP_ANALYSIS_2026-04-29.md` — Priorities
- `agent_docs/test_conventions.md` — Testing patterns
- `AGENTS.md` — Quality gates section