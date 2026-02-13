# ADR-023: CI/CD GitHub Actions Remediation Plan

**Status**: Proposed
**Date**: 2026-02-12
**Context**: GitHub Actions CI/CD pipeline has multiple failures requiring architectural decisions for remediation. Coverage workflow fails due to disk space exhaustion, 5 Dependabot PRs blocked by clippy errors, benchmark workflow has reliability issues, and a stale workflow file exists.

**Decision**: Implement multi-pronged CI/CD remediation strategy addressing disk space, dependency management, and workflow cleanup.

## Alternatives Considered

1. **Do Nothing (Status Quo)**
   - Pros: No effort required
   - Cons: Coverage permanently broken on main, Dependabot PRs pile up, no regression detection
   - **REJECTED**: Violates CI/CD quality standards in AGENTS.md

2. **Single Monolithic Fix PR**
   - Pros: One PR to review
   - Cons: High risk, complex review, hard to rollback individual changes
   - **REJECTED**: Too risky for CI infrastructure

3. **Incremental Fix Strategy (Chosen)**
   - Pros: Low risk per change, easy rollback, can prioritize by severity
   - Cons: Multiple PRs, more review cycles
   - **ACCEPTED**: Best risk/reward balance

## Decision Details

### D1: Coverage Disk Space Fix (P0)
- **Problem**: `cargo llvm-cov --workspace` exhausts runner disk on main branch
- **Decision**: Add `jlumbroso/free-disk-space` action before coverage step AND exclude benchmark/example crates from full workspace coverage
- **Rationale**: GitHub Actions runners have ~14GB free; full workspace Rust build + coverage instrumentation exceeds this
- **Impact**: Coverage workflow will pass, Codecov uploads resume
- **Files Affected**: `.github/workflows/coverage.yml`

### D2: Dependabot PR Triage Strategy (P1)
- **Problem**: 5 Dependabot PRs (#266-271) all failing CI with clippy errors
- **Decision**: Fix underlying clippy warnings that surface with new dependency versions, then merge compatible PRs. Close PRs for major version bumps (criterion 0.5→0.8) that require API changes.
- **Rationale**: Dependabot PRs for minor/patch versions should work; major versions need dedicated migration effort
- **Impact**: Reduces PR backlog, keeps dependencies current
- **Files Affected**: Various source files with clippy warnings, `Cargo.toml`

### D3: Benchmark Workflow Reliability (P1)
- **Problem**: Benchmark runs getting stuck or cancelled, two runs in_progress simultaneously
- **Decision**: Add explicit timeout, improve concurrency controls, add disk space check before benchmark execution
- **Rationale**: Benchmarks are resource-intensive and need guardrails
- **Impact**: More reliable performance regression detection
- **Files Affected**: `.github/workflows/benchmarks.yml`

### D4: Stale Workflow Cleanup (P2)
- **Problem**: `ci-old.yml` workflow still active but appears to be a legacy duplicate
- **Decision**: Remove `ci-old.yml` from the repository
- **Rationale**: Stale workflows waste CI minutes and create confusion
- **Impact**: Cleaner workflow list, reduced CI cost
- **Files Affected**: `.github/workflows/ci-old.yml` (delete)

### D5: CI Issue Tracking (P2)
- **Problem**: Zero GitHub Issues tracking CI/CD problems
- **Decision**: Create GitHub Issues for each identified CI problem with appropriate labels
- **Rationale**: Issues provide visibility, assignment, and tracking for CI maintenance
- **Impact**: Better CI maintenance culture

## Tradeoffs
- **Positive**: CI pipeline reliability restored, dependencies kept current
- **Positive**: Coverage metrics available for quality gates
- **Positive**: Clear tracking of CI technical debt
- **Negative**: Multiple PRs needed for full remediation
- **Negative**: Some Dependabot PRs may need to be closed (criterion major bump)

## Consequences
- Coverage workflow passes on main → badge and Codecov data restored
- Dependabot backlog cleared → security updates applied
- Benchmark runs complete reliably → regression detection works
- Stale workflow removed → cleaner CI configuration
- Issues created → visible tracking of CI health

## Implementation Status
⏳ **PROPOSED** - Awaiting implementation

**Execution Strategy**: GOAP sequential with parallel sub-tasks
- Phase 1: Coverage fix (P0, independent)
- Phase 2: Dependabot triage (P1, can parallel with Phase 1)
- Phase 3: Benchmark reliability (P1, after Phase 2)
- Phase 4: Cleanup (P2, after Phase 3)

## Related
- **ADR-022**: GOAP Agent System (orchestration methodology used)
- **Plans**: `plans/CI_GITHUB_ACTIONS_STATUS_2026-02-12.md` (detailed status report)
- **Ref**: `plans/CI_STATUS_REPORT_2026-02-03.md` (previous CI status)
