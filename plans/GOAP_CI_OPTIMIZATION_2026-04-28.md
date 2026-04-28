# GOAP Plan: CI Optimization - Reduce PR Build Time

**Goal**: Reduce PR build time from ~50+ minutes to under 20 minutes
**Constraint**: Preserve quality gates and performance regression detection for main branch
**Date**: 2026-04-28
**ADR Reference**: ADR-026, ADR-029

## Current State Analysis

### CI Timing Breakdown (PRs)

| Job | Timeout | Actual Time | Impact |
|-----|---------|-------------|--------|
| Run Benchmarks | 55 min | ~54 min | **BLOCKER** |
| Tests | 30 min | ~12 min | OK |
| MCP Build | 15 min | ~10 min | OK |
| Multi-Platform (ubuntu) | 45 min | ~12 min | OK |
| Multi-Platform (macos) | 45 min | ~15 min | OK |
| Quality Gates | 45 min | ~12 min | OK |
| Semver Check | 30 min | ~5 min | OK |

**Bottleneck**: `Run Benchmarks` job takes ~54 min and runs on ALL PRs.

### Current Benchmark Workflow (benchmarks.yml)

```yaml
"on":
  pull_request:
    branches: [main]
    paths-ignore: ['docs/**', 'plans/**', '**/*.md', '.agents/**', 'agent_docs/**']
```

- Runs on ALL PRs (except docs-only)
- Full benchmark suite: storage_operations, concurrent_operations, phase3_retrieval_accuracy, phase3_cache_performance, etc.
- Individual benchmark timeouts: 5-10 min each
- Total suite timeout: 55 min

## Optimization Strategy

### Recommended Approach: Paths-Based + Label Skip

| Optimization | Implementation | Impact | Risk |
|--------------|----------------|--------|------|
| **Paths filter for benchmarks** | Only run when perf-critical paths change | ~50 min saved on docs/agent PRs | Low |
| **Label-based skip** | `skip-benchmarks` label opt-out | ~50 min saved when used | Low |
| **Make informational** | Benchmark not required for PR merge | Maintains check visibility | Low |
| **Quick subset** | Run 3 key benchmarks on PRs | ~40 min saved | Medium |

### Perf-Critical Paths

Files that affect performance and require benchmarks:

```yaml
paths:
  - 'memory-core/src/**'
  - 'memory-storage-turso/src/**'
  - 'memory-storage-redb/src/**'
  - 'benches/**'
  - 'Cargo.toml'
  - 'Cargo.lock'
```

## Task Decomposition

### Phase 1: Paths-Based Benchmark Trigger (Sequential)

```
Task 1: Update benchmarks.yml paths trigger
  ↓
Task 2: Add skip-benchmarks label support
  ↓
Task 3: Make benchmark informational (not required)
  ↓
Task 4: Update AGENTS.md with CI guidelines
  ↓
Task 5: Create ci-optimization skill if needed
```

### Phase 2: Verification (Parallel)

```
Task 6: Verify benchmark workflow still runs on main
  ─┐
Task 7: Verify benchmark skip works with label
  ─┼─> Task 8: Document results in plans/
```

## Success Criteria

- [ ] PR build time reduced to <20 min (excluding benchmark-dependent PRs)
- [ ] Benchmark workflow runs on perf-critical file changes
- [ ] Benchmark workflow runs on main branch (unchanged)
- [ ] `skip-benchmarks` label allows manual skip
- [ ] AGENTS.md updated with CI optimization guidelines
- [ ] No regression in quality gate coverage

## Execution Plan

### WG-150: Update Benchmark Paths Trigger

**File**: `.github/workflows/benchmarks.yml`
**Action**: Add explicit `paths` filter for perf-critical files

```yaml
"on":
  pull_request:
    branches: [main]
    paths:  # Only run when these files change
      - 'memory-core/src/**'
      - 'memory-storage-turso/src/**'
      - 'memory-storage-redb/src/**'
      - 'benches/**'
      - 'Cargo.toml'
      - 'Cargo.lock'
    paths-ignore:  # Still ignore docs
      - 'docs/**'
      - 'plans/**'
      - '**/*.md'
      - '.agents/**'
      - 'agent_docs/**'
```

### WG-151: Add skip-benchmarks Label Support

**File**: `.github/workflows/benchmarks.yml`
**Action**: Add job condition to skip when label present

```yaml
benchmark:
  if: >-
    ${{
      always() &&
      github.actor != 'dependabot[bot]' &&
      !contains(github.event.pull_request.labels.*.name, 'skip-benchmarks') &&
      ...
    }}
```

### WG-152: Make Benchmark Informational

**File**: `.github/workflows/benchmarks.yml`
**Action**: The regression-check job already uses `continue-on-error` pattern
**Verify**: Ensure benchmark is not in required checks

### WG-153: Update AGENTS.md

**Section**: Add CI Optimization Guidelines

### WG-154: Create ci-optimization Skill (Optional)

**File**: `.claude/skills/ci-optimization.md` (if needed)

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Performance regression missed on PR | Benchmarks still run on main; manual benchmark trigger available |
| Label abuse | Only repo maintainers can add labels |
| Path filter too narrow | Include all storage/core/embedding paths |

## Metrics Target

- **Before**: PR CI ~50+ min (full benchmark suite)
- **After**: PR CI ~15-18 min (non-perf PRs), ~50 min (perf PRs)

## References

- ADR-026: Performance Benchmark CI Failures
- ADR-029: GitHub Actions Modernization
- benchmarks.yml: Current workflow configuration
- ci.yml: Main CI workflow