# GOAP Execution Plan: Remaining Test Fixes & v0.1.16 Prep
**Date**: 2026-02-16
**Status**: In Progress
**Branch**: `fix/remaining-tests-cli-api-2026-02-16`
**Previous**: PR #296 merged (31 of 39 tests fixed)

## Problem Statement

After successfully fixing Nightly CI and 31 of 39 flaky tests, **8 tests remain failing**:

### Remaining Failures (8 tests - 21%)

**CLI Workflow Tests (7 tests)** - HIGHEST PRIORITY
- **Root Cause**: Tests use old CLI API (flag arguments like `--description`, `--domain`)
- **Current CLI**: Uses positional arguments and simplified interface
- **Impact**: Blocks CLI workflow validation
- **Files**: `tests/e2e/cli_workflows.rs`

**memory-mcp Test (1 test)** - MEDIUM PRIORITY
- **Test**: `test_mcp_server_tools`
- **Status**: Needs investigation
- **File**: `memory-mcp/tests/simple_integration_tests.rs`

## GOAP Decomposition

### Goals (Ordered by Priority)

1. **CRITICAL**: Fix 7 CLI workflow tests to validate CLI functionality
2. **HIGH**: Investigate and fix 1 remaining memory-mcp test
3. **MEDIUM**: Prepare for v0.1.16 development (Code Quality + Pattern Algorithms)
4. **LOW**: Update ROADMAP with remaining work

### Actions (Atomic Tasks)

#### Phase 1: CLI Test Fixes (CRITICAL - Sequential with handoff)

- [ ] **A1**: Analyze current CLI API vs test expectations
  - Document current CLI interface (positional args)
  - Document test expectations (flag args)
  - Identify mapping between old and new API
  - Target: Analysis Agent

- [ ] **A2**: Update CLI workflow tests to use current API
  - Rewrite test commands to use positional arguments
  - Update assertion logic for new output format
  - Test each workflow individually
  - Target: Test Fix Agent

- [ ] **A3**: Verify all 7 CLI tests passing
  - Run tests individually and as suite
  - Validate no regressions in other tests
  - Target: Quality Agent

#### Phase 2: memory-mcp Investigation (HIGH - Parallel with A3)

- [ ] **A4**: Investigate test_mcp_server_tools failure
  - Run test with debug logging
  - Identify root cause (timeout, assertion, environment)
  - Apply appropriate fix
  - Target: Debug Agent

#### Phase 3: v0.1.16 Preparation (MEDIUM - Sequential)

- [ ] **A5**: Review v0.1.16 roadmap items
  - Code Quality improvements
  - Pattern Algorithms enhancements
  - Target: Architecture Agent

- [ ] **A6**: Create v0.1.16 GOAP plan
  - Decompose into atomic tasks
  - Identify dependencies
  - Set success criteria
  - Target: Plan Agent

- [ ] **A7**: Update ROADMAPS
  - Mark CLI tests as in progress
  - Document v0.1.16 preparation
  - Target: Documentation Agent

## Execution Strategy: Sequential with Handoff

### Group 1: Analysis → Test Fix (Sequential)
1. **Analysis Agent** (A1) → Handoff to Test Fix Agent
2. **Test Fix Agent** (A2) → Handoff to Quality Agent

### Group 2: Parallel Investigation (During A2)
3. **Debug Agent** (A4) → Investigate memory-mcp test

### Group 3: Sequential (After A1-A4 complete)
4. **Architecture Agent** (A5) → Handoff to Plan Agent
5. **Plan Agent** (A6) → Handoff to Documentation Agent
6. **Documentation Agent** (A7)

## Quality Gates

### Pre-Commit Checks
- [ ] All CLI workflow tests pass (7/7)
- [ ] test_mcp_server_tools passes
- [ ] cargo fmt --all -- --check
- [ ] cargo clippy --all -- -D warnings
- [ ] cargo build --all
- [ ] cargo test --all (quick check, no --ignored)

### Atomic Commit Strategy
```
[analysis] Document CLI API changes and test mapping
[test] Update CLI workflow tests to current API
[fix] Fix remaining memory-mcp test
[docs] Update ROADMAPS for v0.1.16 preparation
```

## Success Criteria

### Must Have (Blocking)
1. ✅ All 7 CLI workflow tests passing
2. ✅ test_mcp_server_tools passing
3. ✅ Zero regression in other tests
4. ✅ v0.1.16 GOAP plan created

### Nice to Have
1. CLI test helper functions created
2. Documentation for CLI API changes
3. Automated CI checks for CLI tests

## ADR References

- **ADR-030**: Test Optimization and CI Stability Patterns
- **ADR-022**: GOAP Agent System - Execution methodology
- **ADR-028**: Feature Enhancement Roadmap

## Progress Tracking

| Task | Agent | Status | Notes |
|------|-------|--------|-------|
| A1 | Analysis | 🟡 Pending | CLI API analysis |
| A2 | Test Fix | ⚪ Blocked | Waiting on A1 |
| A3 | Quality | ⚪ Blocked | Waiting on A2 |
| A4 | Debug | 🟡 Pending | memory-mcp investigation |
| A5 | Architecture | ⚪ Blocked | Waiting on A1-A4 |
| A6 | Plan | ⚪ Blocked | Waiting on A5 |
| A7 | Documentation | ⚪ Blocked | Waiting on A6 |

## Execution Log

### 2026-02-16 Initial Planning
- Created comprehensive GOAP plan for remaining work
- Identified 8 remaining test failures (7 CLI + 1 memory-mcp)
- Planned sequential execution with handoffs
- Branch: `fix/remaining-tests-cli-api-2026-02-16`
