# GOAP Execution Plan: Fix 7 GitHub Issues

## Objective
Fix 7 GitHub issues in the Rust memory management system, prioritizing P0 critical issues (file size refactoring) that block code reviews, followed by P1 high-priority issues (dependency cleanup, clone reduction, error handling).

## Task Analysis

### Current State
- **Project**: Rust/Tokio memory management system with MCP server
- **Quality**: 92.5% test coverage, 76.7% test pass rate (7 failing tests)
- **Issues**: 7 issues requiring comprehensive fixes
- **Direct Dependencies**: 1,184 unique packages
- **Binary Sizes**: 365MB (memory-mcp-server), 110MB (memory-cli)

### Issue Breakdown

**P0 Issues (Critical - Block Code Reviews):**

1. **Issue #216**: Refactor 11 files to ≤500 LOC
   - memory-mcp/src/patterns/statistical/analysis.rs (811 LOC)
   - memory-mcp/src/bin/server/mcp.rs (796 LOC)
   - memory-mcp/src/mcp/tools/advanced_pattern_analysis.rs (741 LOC)
   - memory-mcp/src/unified_sandbox.rs (733 LOC)
   - memory-mcp/src/mcp/tools/embeddings.rs (713 LOC)
   - memory-mcp/src/mcp/tools/quality_metrics.rs (694 LOC)
   - memory-mcp/src/sandbox.rs (690 LOC)
   - memory-mcp/src/wasm_sandbox.rs (683 LOC)
   - memory-mcp/src/javy_compiler.rs (679 LOC)
   - memory-mcp/src/wasmtime_sandbox.rs (595 LOC)
   - memory-storage-redb/src/cache.rs (654 LOC)
   - memory-storage-turso/src/lib.rs (710 LOC)
   - memory-storage-turso/src/pool.rs (589 LOC)
   - memory-storage-turso/src/storage/episodes.rs (469 LOC) - Close to limit

2. **Issue #215**: Already addressed - memory-mcp/src/patterns/predictive.rs (497 LOC) is compliant

3. **Issue #214**: Already addressed - memory-storage-turso/src/storage.rs (348 LOC) is compliant

**P1 Issues (High Priority - Production Quality):**

4. **Issue #219**: Dependency Cleanup
   - Reduce 1,184 direct dependencies to <900
   - Remove unused/dev-only dependencies from release builds
   - Reduce binary size from ~2.1 GB (build artifacts) to <1.5 GB

5. **Issue #218**: Clone Reduction
   - Current: Unknown (need accurate count)
   - Target: <200 clone operations
   - Strategy: Replace clone with references, use Cow, implement Copy where appropriate

6. **Issue #217**: Error Handling Audit
   - Current: Unknown (need accurate count)
   - Target: <50 unwrap/expect calls
   - Strategy: Replace with proper error handling, use ?

## Dependencies

```
P0 Issues (Independent - Can Run in Parallel):
├─ [Parallel Group 1: memory-mcp server files]
│  ├─ mcp.rs refactoring (796 LOC → ≤500)
│  ├─ advanced_pattern_analysis.rs refactoring (741 LOC → ≤500)
│  ├─ embeddings.rs refactoring (713 LOC → ≤500)
│  └─ quality_metrics.rs refactoring (694 LOC → ≤500)
│
├─ [Parallel Group 2: memory-mcp sandbox files]
│  ├─ unified_sandbox.rs refactoring (733 LOC → ≤500)
│  ├─ sandbox.rs refactoring (690 LOC → ≤500)
│  ├─ wasm_sandbox.rs refactoring (683 LOC → ≤500)
│  ├─ javy_compiler.rs refactoring (679 LOC → ≤500)
│  └─ wasmtime_sandbox.rs refactoring (595 LOC → ≤500)
│
└─ [Parallel Group 3: storage files]
   ├─ statistical/analysis.rs refactoring (811 LOC → ≤500)
   ├─ cache.rs refactoring (654 LOC → ≤500)
   ├─ lib.rs refactoring (710 LOC → ≤500)
   ├─ pool.rs refactoring (589 LOC → ≤500)
   └─ episodes.rs refactoring (469 LOC → ≤500)

P1 Issues (Sequential):
├─ Issue #219: Dependency Cleanup (must be first)
│  └─ Remove unused dependencies
│  └─ Reduce binary size
│
├─ [Parallel: After #219 completes]
│  ├─ Issue #218: Clone Reduction
│  └─ Issue #217: Error Handling Audit
```

## Execution Strategy

### Phase 1: P0 File Size Refactoring (Parallel Execution)

**Strategy**: Split into 3 parallel groups for maximum throughput

#### Group 1: memory-mcp Server Tools (4 files)
- Agent: clean-code-developer
- Files:
  - src/patterns/statistical/analysis.rs (811 LOC)
  - src/bin/server/mcp.rs (796 LOC)
  - src/mcp/tools/advanced_pattern_analysis.rs (741 LOC)
  - src/mcp/tools/embeddings.rs (713 LOC)
  - src/mcp/tools/quality_metrics.rs (694 LOC)

#### Group 2: memory-mcp Sandbox Implementation (5 files)
- Agent: clean-code-developer
- Files:
  - src/unified_sandbox.rs (733 LOC)
  - src/sandbox.rs (690 LOC)
  - src/wasm_sandbox.rs (683 LOC)
  - src/javy_compiler.rs (679 LOC)
  - src/wasmtime_sandbox.rs (595 LOC)

#### Group 3: Storage Layer Files (4 files)
- Agent: clean-code-developer
- Files:
  - memory-storage-redb/src/cache.rs (654 LOC)
  - memory-storage-turso/src/lib.rs (710 LOC)
  - memory-storage-turso/src/pool.rs (589 LOC)
  - memory-storage-turso/src/storage/episodes.rs (469 LOC - optimization opportunity)

### Phase 2: P1 Dependency Cleanup (Sequential)

**Agent**: rust-specialist (deep Rust expertise required)

#### Task 1: Dependency Analysis
- Analyze all 1,184 dependencies
- Identify unused/dead dependencies
- Identify dev-only dependencies in release builds
- Identify duplicate functionality dependencies

#### Task 2: Dependency Removal
- Remove unused dependencies from Cargo.toml files
- Update Cargo.lock
- Verify no breaking changes

#### Task 3: Binary Size Optimization
- Enable LTO (Link Time Optimization)
- Strip debug symbols in release
- Enable codegen-units = 1 for release
- Optimize feature flags

### Phase 3: P1 Code Quality Improvements (Parallel)

#### Parallel Task A: Clone Reduction
**Agent**: clean-code-developer
- Find all clone() operations
- Analyze necessity
- Replace with references where possible
- Use Cow for conditional cloning
- Implement Copy trait where appropriate

#### Parallel Task B: Error Handling Audit
**Agent**: clean-code-developer
- Find all unwrap() and expect() calls
- Replace with proper error handling (? operator)
- Create custom error types where needed
- Ensure all error paths are tested

## Quality Gates

### After Each Phase:
1. **Compilation**: `cargo build --all` must succeed
2. **Tests**: `cargo test --all` must pass (all 7 failing tests fixed)
3. **Clippy**: `cargo clippy --all -- -D warnings` must show 0 warnings
4. **Formatting**: `cargo fmt --all -- --check` must pass
5. **Coverage**: Maintain >90% coverage

### Final Validation:
- All files ≤500 LOC
- All dependencies cleaned
- Binary size <1.5 GB
- Clone count <200
- Unwrap/expect count <50
- All tests passing
- Zero clippy warnings

## Agent Assignment

### Phase 1: File Size Refactoring (3 Parallel Groups)
- **clean-code-developer** (x3): Refactor files following SOLID principles
- **test-runner**: Run tests after each file refactored
- **code-reviewer**: Quality checks after each group

### Phase 2: Dependency Cleanup (Sequential)
- **rust-specialist**: Deep dependency analysis and cleanup
- **test-runner**: Verify no regressions
- **code-reviewer**: Review dependency changes

### Phase 3: Code Quality Improvements (2 Parallel Tasks)
- **clean-code-developer** (x2): Clone reduction & error handling
- **test-runner**: Continuous testing
- **code-reviewer**: Final quality audit

## Success Criteria

### Issue #216 (P0):
- [ ] All 14 files ≤500 LOC
- [ ] No functionality broken
- [ ] All tests passing

### Issue #219 (P1):
- [ ] Dependencies reduced to <900
- [ ] Binary size <1.5 GB
- [ ] No functionality regression

### Issue #218 (P1):
- [ ] Clone operations <200
- [ ] Performance maintained or improved
- [ ] All tests passing

### Issue #217 (P1):
- [ ] unwrap/expect calls <50
- [ ] Proper error handling throughout
- [ ] All error paths tested

### Overall:
- [ ] Zero clippy warnings
- [ ] >90% test coverage maintained
- [ ] All tests passing (7 failing tests fixed)
- [ ] Code quality standards met

## Risks & Mitigations

### Risk 1: Refactoring breaks existing functionality
- **Mitigation**: Run tests after each file refactor, use git for easy rollback
- **Fallback**: Use git bisect to identify breaking change

### Risk 2: Dependency cleanup causes compilation failures
- **Mitigation**: Analyze dependencies thoroughly before removal, keep unused dependencies commented out initially
- **Fallback**: Revert dependencies one by one to find issue

### Risk 3: Clone reduction changes semantics
- **Mitigation**: Review each clone removal carefully, add tests for behavior verification
- **Fallback**: Keep clones in critical paths, only optimize non-critical code

### Risk 4: Error handling changes break caller contracts
- **Mitigation**: Review all callers before changing error types, maintain error compatibility
- **Fallback**: Use wrapper errors to preserve existing error types

## Timeline Estimate

### Phase 1: P0 File Size Refactoring
- Group 1: 4 files × 30 min = 2 hours
- Group 2: 5 files × 30 min = 2.5 hours
- Group 3: 4 files × 30 min = 2 hours
- **Total (parallel): 2.5 hours**

### Phase 2: P1 Dependency Cleanup
- Analysis: 1 hour
- Removal & Optimization: 1.5 hours
- **Total: 2.5 hours**

### Phase 3: P1 Code Quality Improvements
- Clone Reduction: 1.5 hours
- Error Handling: 1.5 hours
- **Total (parallel): 1.5 hours**

### Quality Validation: 1 hour

**Total Estimated Time: 7.5 hours**

## Rollback Plan

### If Phase 1 fails:
1. `git checkout --memory-mcp/src` or specific files
2. Revert commits one by one
3. Use `git bisect` to identify problematic commit

### If Phase 2 fails:
1. Restore Cargo.toml and Cargo.lock from git
2. Revert dependency changes
3. Incrementally re-add dependencies

### If Phase 3 fails:
1. Use `git diff` to identify changes
2. Revert specific error handling or clone reductions
3. Keep working improvements

## Commit Strategy

### Phase 1 Commits:
```
refactor(memory-mcp): split analysis.rs into multiple modules
refactor(memory-mcp): split mcp.rs into mcp and mcp handlers
refactor(memory-mcp): split embeddings.rs into modules
refactor(memory-mcp): split sandbox implementation
refactor(storage-turso): split lib.rs into focused modules
refactor(storage-redb): split cache.rs into modules
```

### Phase 2 Commits:
```
refactor(deps): remove unused dependencies from memory-mcp
refactor(deps): optimize dependency graph across workspace
perf(deps): enable LTO and optimize binary size
```

### Phase 3 Commits:
```
refactor(performance): reduce clone operations with references
refactor(error): replace unwrap with proper error handling
```

## Execution Protocol

### Phase 1 Execution:
1. Start 3 parallel clean-code-developer agents
2. Each agent refactors their assigned file group
3. After each file, test-runner validates no regressions
4. After each group, code-reviewer performs quality check
5. Commit all changes for the group

### Phase 2 Execution:
1. rust-specialist analyzes dependencies
2. Create dependency removal plan
3. Incrementally remove dependencies with testing
4. Optimize build settings
5. Final validation and commit

### Phase 3 Execution:
1. Start 2 parallel clean-code-developer agents
2. One works on clone reduction, one on error handling
3. Continuous testing throughout
4. Final code review and commit

### Final Validation:
1. Run full quality gates
2. Generate comprehensive report
3. Create summary commit if needed

## Next Steps

**Immediate Actions**:
1. ✅ Load episode-start skill for learning tracking
2. 🔄 Begin Phase 1: Start 3 parallel clean-code-developer agents for P0 file refactoring
3. 🔄 Coordinate agents with test-runner and code-reviewer
4. 🔄 Track progress and log results
5. 🔄 Move to Phase 2 after all P0 issues resolved
6. 🔄 Complete Phase 3 with parallel code quality improvements
7. 🔄 Final validation and reporting

**Status**: Ready to execute Phase 1
