# ADR-025: Comprehensive Project Health and Missing Tasks Remediation

**Status**: Accepted
**Date**: 2026-02-13
**Context**: Multiple areas of project health have degraded across CI, releases, code quality, and feature completion — requiring a coordinated remediation effort.

**Decision**: Adopt a phased remediation approach (A→D) prioritized by blast radius and dependency order, addressing CI stabilization, dependency updates, code quality, and feature completion.

---

## Alternatives Considered

### 1. Ad-Hoc Fixes as Discovered
- **Pros**: No planning overhead, immediate action on each issue
- **Cons**: No prioritization, risk of conflicting fixes, no visibility into overall progress
- **REJECTED**: Unsustainable with 8+ distinct problem areas; leads to whack-a-mole remediation

### 2. Full Stop and Rewrite (Big Bang)
- **Pros**: Clean slate, address everything at once
- **Cons**: Blocks all feature work, high risk, massive single PR, no incremental value delivery
- **REJECTED**: Disproportionate risk for the scope of issues; most problems are incremental not architectural

### 3. Phased Remediation with Priority Matrix (Chosen)
- **Pros**: Clear prioritization, incremental delivery, each phase unblocks the next, measurable progress
- **Cons**: Requires upfront planning, multiple PRs, coordination across phases
- **ACCEPTED**: Best balance of risk, velocity, and thoroughness

---

## Problem Inventory

| # | Problem | Severity | Blast Radius |
|---|---------|----------|--------------|
| 1 | v0.1.14 on main but no GitHub release (last release: v0.1.13) | High | Users, changelog |
| 2 | Nightly Tests workflow failing on main | High | Quality gate |
| 3 | Coverage workflow failing on main | High | Quality metrics |
| 4 | YAML Lint workflow failing on main | Medium | CI hygiene |
| 5 | Benchmarks workflow failing/stuck on main | Medium | Perf regression detection |
| 6 | 5 Dependabot PRs (#267-271) blocked by clippy warnings | Medium | Security updates |
| 7 | MCP batch module disabled with TODOs (non-existent jsonrpsee/ServerState types) | Medium | Feature completeness |
| 8 | ~100 plan files in plans/, many stale/conflicting | Low | Developer onboarding |
| 9 | Phase 2 Turso optimization at 75% (Adaptive TTL pending) | Medium | Performance |
| 10 | MCP Token Optimization planned but not started | Low | Token efficiency |
| 11 | Error handling audit: 73 production unwraps needing conversion | Medium | Reliability |
| 12 | No v0.1.15 release despite significant work merged | Medium | Users, changelog |

---

## Decision

### Phase A — CI/CD Stabilization (P0)

**Goal**: All workflows green on main.

| Task | Target | Files Affected |
|------|--------|----------------|
| Fix Nightly Tests workflow | Green on main | `.github/workflows/nightly-tests.yml`, source files |
| Fix Coverage workflow (disk space) | Green on main | `.github/workflows/coverage.yml` |
| Fix YAML Lint workflow | Green on main | `.yamllint.yml`, YAML files with violations |
| Fix Benchmarks workflow (timeout/stuck) | Green on main | `.github/workflows/benchmarks.yml` |

**Rationale**: CI must be green before merging any other changes. Broken CI masks regressions and blocks Dependabot PRs.
**Exit Criteria**: All 4 workflows pass on main branch.

### Phase B — Dependency Updates (P1)

**Goal**: Clear Dependabot backlog and migrate major version bumps.

| Task | Target | Files Affected |
|------|--------|----------------|
| Fix clippy warnings blocking Dependabot | Unblock PRs #267-271 | Source files with warnings |
| Merge compatible Dependabot PRs | Dependencies current | `Cargo.toml`, `Cargo.lock` |
| Migrate criterion 0.5.1 → 0.8.x | Benchmarks on modern API | `benches/`, `Cargo.toml` |

**Rationale**: Dependency updates carry security implications. Clippy warnings must be fixed first (Phase A unblocks this).
**Exit Criteria**: All Dependabot PRs resolved (merged or closed with tracking issue). Criterion migration complete.
**Depends On**: Phase A (CI green to validate merges).

### Phase C — Code Quality (P1)

**Goal**: Fix disabled code, reduce production unwraps, clean up stale plans.

| Task | Target | Files Affected |
|------|--------|----------------|
| Fix MCP batch module (disabled TODOs) | Module compiles and passes tests | `memory-mcp/src/server/tools/batch.rs` |
| Unwrap audit: convert 73 production unwraps | All production paths use proper error handling | Multiple source files |
| Plans directory cleanup | Remove stale/conflicting docs, consolidate | `plans/` (~100 files) |

**Rationale**: Disabled modules and production unwraps are latent defects. Stale plans create confusion for contributors.
**Exit Criteria**: Batch module enabled and tested. Production unwrap count reduced to ≤10. Plans directory under 40 files.
**Depends On**: Phase A (CI green to validate changes).

### Phase D — Feature Completion and Release (P2)

**Goal**: Complete pending features, cut releases.

| Task | Target | Files Affected |
|------|--------|----------------|
| Complete Adaptive TTL (Phase 2 Turso optimization) | Feature implemented and tested | `memory-storage-turso/` |
| MCP Token Optimization implementation | Token-efficient MCP responses | `memory-mcp/` |
| Cut v0.1.14 GitHub release | Release published with changelog | GitHub Releases |
| Cut v0.1.15 GitHub release | Release published with changelog | GitHub Releases |

**Rationale**: Features and releases should only ship after CI is stable and code quality is addressed.
**Exit Criteria**: Adaptive TTL merged. MCP token optimization merged. Both releases published on GitHub.
**Depends On**: Phases A, B, C.

---

## Decision Matrix

| Phase | Priority | Risk | Effort | Value | Dependencies |
|-------|----------|------|--------|-------|--------------|
| **A: CI/CD Stabilization** | P0 | Low | Medium | Very High | None |
| **B: Dependency Updates** | P1 | Medium | Medium | High | Phase A |
| **C: Code Quality** | P1 | Low | High | High | Phase A |
| **D: Feature Completion** | P2 | Medium | High | Medium | Phases A, B, C |

**Execution Order**: A → (B ∥ C) → D

Phases B and C can execute in parallel once Phase A is complete. Phase D is sequential after B and C.

---

## Tradeoffs

### Positive
- CI reliability restored before any feature work proceeds
- Dependency security updates unblocked systematically
- Production reliability improved (unwrap audit)
- Clean plans directory improves contributor experience
- GitHub releases catch up to actual shipped code
- Each phase delivers standalone value

### Negative
- Feature work (Phase D) is deferred until quality is restored
- Multiple PRs across phases increases review burden
- Plans cleanup may lose historical context if not archived properly
- Phased approach takes longer than a single heroic effort (but with less risk)

---

## Consequences

- **Positive**: All CI workflows green on main — quality gates enforceable again
- **Positive**: Dependabot PRs unblocked — security updates flow automatically
- **Positive**: Production unwraps eliminated — fewer panics in production
- **Positive**: MCP batch module functional — full feature set available
- **Positive**: GitHub releases match actual shipped code — users can track changes
- **Positive**: Plans directory maintainable — reduced onboarding confusion
- **Negative**: 2-4 weeks of remediation work before new features ship
- **Negative**: Some stale plan files may be deleted that had historical value
- **Negative**: Criterion migration (0.5→0.8) may require benchmark rebaselining

---

## Implementation Status

⬚ **NOT STARTED**

| Phase | Status | Notes |
|-------|--------|-------|
| Phase A: CI/CD Stabilization | ⬚ Not Started | ADR-023 addressed some items; remaining failures need work |
| Phase B: Dependency Updates | ⬚ Not Started | Blocked by Phase A |
| Phase C: Code Quality | ⬚ Not Started | Blocked by Phase A |
| Phase D: Feature Completion | ⬚ Not Started | Blocked by Phases B, C |

---

## Related ADRs

- **ADR-023**: CI/CD GitHub Actions Remediation (subset of Phase A; partially complete)
- **ADR-024**: MCP Lazy Tool Loading (related to Phase D MCP optimization)
- **ADR-022**: GOAP Agent System (orchestration methodology for multi-phase execution)

---

## References

- `plans/CI_GITHUB_ACTIONS_STATUS_2026-02-12.md` — CI status audit
- `plans/DEPENDABOT_TRIAGE_REPORT_2026-02-13.md` — Dependabot PR analysis
- GitHub Issues #276, #277 — Clippy and criterion migration tracking
- `plans/research/MCP_TOKEN_OPTIMIZATION_RESEARCH.md` — MCP optimization research

---

**Individual ADR**: `plans/adr/ADR-025-Project-Health-Remediation.md`
**Supersedes**: None
**Superseded By**: None
