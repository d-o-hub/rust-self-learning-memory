# GOAP: 80% Coverage Sprint - Comprehensive Plan

**Date**: 2026-04-29
**Type**: Quality Improvement Sprint
**Priority**: P1 - User requirement
**WG**: WG-148

---

## Executive Summary

**Goal**: Reach 80% code coverage (codecov.yml project target)

**Current State**: 61.22% coverage
**Gap**: 18.78% (need ~1,500 more lines covered)

---

## Phase 1: ANALYZE

### 0% Coverage Modules (Critical)

| Module | LOC | Crate | Priority |
|--------|-----|-------|----------|
| `storage/monitoring.rs` | 427 | turso | HIGH |
| `storage/capacity.rs` | 217 | turso | HIGH |
| `storage/recommendations.rs` | 315 | turso | HIGH |
| `storage/search/episodes.rs` | 154 | turso | MEDIUM |
| `storage/search/patterns.rs` | 137 | turso | MEDIUM |
| `storage/embedding_backend.rs` | 12 | turso | LOW |
| `storage/embedding_tables.rs` | 16 | turso | LOW |

**Total 0%**: 1,262 LOC

### Low Coverage Modules (<50%)

| Module | LOC | Current Coverage | Priority |
|--------|-----|------------------|----------|
| `resilient.rs` | 369 | 38.21% | HIGH |
| `pool/keepalive/monitoring.rs` | 81 | 29.63% | MEDIUM |
| `pool/keepalive/connection.rs` | 41 | 41.46% | MEDIUM |
| `transport/decompression.rs` | 79 | 17.72% | LOW |
| `trait_impls/mod.rs` | 66 | 21.21% | LOW |

**Total Low Coverage**: 536 LOC (need ~270 more lines covered)

---

## Phase 2: DECOMPOSE

### Swarm Execution Pattern

**Spawn 4 parallel agents**:
- Agent A: monitoring.rs + capacity.rs (644 LOC)
- Agent B: recommendations.rs + search/* (606 LOC)
- Agent C: resilient.rs + pool/keepalive/* (491 LOC)
- Agent D: transport/decompression.rs + trait_impls/* (145 LOC)

### Task Breakdown

| Agent | Module | Target Coverage | Lines to Cover |
|-------|--------|----------------|----------------|
| A | monitoring.rs (427) | 50% | ~214 |
| A | capacity.rs (217) | 50% | ~109 |
| B | recommendations.rs (315) | 50% | ~158 |
| B | search/episodes.rs (154) | 50% | ~77 |
| B | search/patterns.rs (137) | 50% | ~69 |
| C | resilient.rs (369) | 70% | ~147 |
| C | pool/keepalive/* (122) | 50% | ~61 |
| D | transport/decompression.rs (79) | 50% | ~40 |
| D | trait_impls/mod.rs (66) | 50% | ~33 |

**Total Target**: ~748 additional lines covered
**Projected Coverage**: 61.22% + ~8.8% = ~70%

### Additional Coverage for 80%

After Phase 1 achieves 70%, Phase 2 targets:

| Module | Current | Target | Additional Lines |
|--------|---------|--------|------------------|
| All Phase 1 modules | 50% | 70% | ~374 |
| pattern_helpers.rs (82 LOC) | 28% | 50% | ~18 |
| clustering/tests.rs | existing | verify | maintain |

**Total Phase 2**: ~392 additional lines
**Projected Coverage**: ~74%

### Phase 3: Core Crate Coverage

| Module | LOC | Current | Target |
|--------|-----|---------|--------|
| reward/*.rs | ~200 | ~65% | 80% |
| spatiotemporal/*.rs | ~300 | ~60% | 75% |
| sync/*.rs | ~150 | ~55% | 70% |

**Projected Coverage**: ~78-80%

---

## Phase 3: STRATEGIZE

**Pattern**: Swarm + Iterative Refinement

1. **Swarm Phase 1**: 4 parallel agents on 0% modules
2. **Iterative Phase 2**: Sequential improvement on 50% → 70%
3. **Sequential Phase 3**: Core crate modules

---

## Phase 4: COORDINATE

### Agent Assignments

| Agent Type | Modules | Expected Contribution |
|------------|---------|----------------------|
| feature-implementer A | monitoring.rs, capacity.rs | +3.7% |
| feature-implementer B | recommendations.rs, search/* | +3.5% |
| feature-implementer C | resilient.rs, keepalive/* | +2.5% |
| feature-implementer D | transport/*, trait_impls/* | +0.9% |

---

## Phase 5: EXECUTE

### Step 1: Spawn Swarm Agents

```
4 agents in parallel:
- Agent A: Add tests for monitoring.rs + capacity.rs
- Agent B: Add tests for recommendations.rs + search/*
- Agent C: Add tests for resilient.rs + keepalive/*
- Agent D: Add tests for transport/decompression.rs + trait_impls/*
```

### Step 2: Verify Coverage Progress

After each agent completes:
```bash
cargo llvm-cov --workspace --summary-only --exclude e2e-tests --exclude memory-benches | grep TOTAL
```

### Step 3: Iterate Until 80%

Repeat Phase 2 improvements until target reached.

---

## Quality Gates

| Milestone | Coverage | Gate Status |
|-----------|----------|-------------|
| Phase 1 Complete | ≥70% | PASS (threshold aligned) |
| Phase 2 Complete | ≥75% | Progress check |
| Phase 3 Complete | ≥80% | PASS (codecov.yml target) |

---

## README Updates Required

Current README states "90%+ test coverage" - needs update:

```markdown
# Current (line 543, 552):
Maintain 90%+ test coverage

# Updated (aligned with ADR-042):
Test coverage targets per ADR-042:
- Phase 1: 70% (current focus)
- Phase 2: 75%
- Phase 3: 80% (codecov.yml project target)
```

---

## Monitoring

### Coverage Check Command

```bash
cargo llvm-cov --workspace --summary-only --exclude e2e-tests --exclude memory-benches 2>&1 | grep TOTAL
```

### Weekly Tracking

Track in `plans/STATUS/COVERAGE_PROGRESS.md`:
- Current coverage percentage
- Modules added this week
- Remaining gap

---

## References

- ADR-042: Code Coverage Improvement Plan
- codecov.yml: Project target 80%
- GOAP_COVERAGE_GATE_FIX.md: Threshold alignment
- README.md: Lines 543, 552 (outdated coverage claims)