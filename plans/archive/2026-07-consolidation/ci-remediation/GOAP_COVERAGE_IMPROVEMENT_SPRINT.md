# GOAP: Coverage Improvement Sprint - Fix Root Cause

**Date**: 2026-04-29
**Type**: Quality Improvement
**Priority**: P1 - User requirement
**WG**: WG-148

---

## Problem Statement

**User Requirement**: Improve actual test coverage, not just lower thresholds.

**Current State**: 61.22% coverage vs 90% quality gate target (28.78% gap)

---

## Root Cause: 0% Coverage Modules

| Module | Lines | Coverage | Issue |
|--------|-------|----------|-------|
| `storage/capacity.rs` | 217 | 0% | No tests |
| `storage/monitoring.rs` | 427 | 0% | No tests |
| `storage/recommendations.rs` | 315 | 0% | No tests |
| `storage/search/episodes.rs` | 154 | 0% | No tests |
| `storage/search/patterns.rs` | 137 | 0% | No tests |
| `pool/keepalive/monitoring.rs` | 81 | 29.63% | Minimal tests |
| `resilient.rs` | 369 | 38.21% | Partial tests |

**Total Uncovered**: ~1,700 lines of critical storage logic

---

## Execution Plan

### Phase 1: Quick Wins (<100 LOC modules) - Parallel

| Task | Module | Target Coverage | Status |
|------|--------|----------------|--------|
| WG-148.1 | search/episodes.rs (154 LOC) | 50% | 🔵 Planned |
| WG-148.2 | search/patterns.rs (137 LOC) | 50% | 🔵 Planned |
| WG-148.3 | pool/keepalive/monitoring.rs (81 LOC) | 50% | 🔵 Planned |

### Phase 2: Medium Modules - Sequential

| Task | Module | Target Coverage | Status |
|------|--------|----------------|--------|
| WG-148.4 | capacity.rs (217 LOC) | 50% | 🔵 Planned |
| WG-148.5 | recommendations.rs (315 LOC) | 50% | 🔵 Planned |

### Phase 3: Large Modules - Sequential

| Task | Module | Target Coverage | Status |
|------|--------|----------------|--------|
| WG-148.6 | monitoring.rs (427 LOC) | 50% | 🔵 Planned |
| WG-148.7 | resilient.rs (369 LOC) | 50% | 🔵 Planned |

---

## Execution Strategy

**Use Swarm Pattern**: Spawn multiple test-implementer agents to work on Phase 1 modules in parallel.

---

## Quality Gates

| Gate | Target | Verification |
|------|--------|--------------|
| Phase 1 Complete | +3% coverage | `cargo llvm-cov` shows ≥64% |
| Phase 2 Complete | +5% coverage | `cargo llvm-cov` shows ≥66% |
| Phase 3 Complete | +8% coverage | `cargo llvm-cov` shows ≥69% |
| Final | ≥70% coverage | Quality gates pass with SKIP_OPTIONAL=false |

---

## Threshold Restoration

After coverage reaches 70%:
1. Revert threshold to 90% (aspirational)
2. Keep SKIP_OPTIONAL=true for local dev
3. CI codecov.yml target remains 80%