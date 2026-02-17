# GOAP Execution Plan: Fix GitHub Actions CI Failures

## Phase 1: ANALYZE

### Task Understanding
- **Goal**: Fix all GitHub Actions CI failures and create a pull request
- **Complexity**: Medium (build error + potential additional issues)
- **Domain**: CI/CD, Rust compilation, feature flags
- **Quality Requirements**: All CI checks must pass (format, clippy, build, tests)

### Context
- Current branch: `feature/v0.1.16-c2-embeddings-integration-2026-02-17`
- Working tree: Clean (no uncommitted changes)
- CI Status: Multiple workflows in progress, cannot access failure logs yet

### Root Cause Identified
**Build Error**: `memory-mcp-server.rs` calls `load_oauth_config()` but:
- Function is gated behind `#[cfg(feature = "oauth")]`
- The `oauth` feature is NOT enabled by default
- Cargo.toml shows: `oauth = []` (opt-in feature)
- Binary calls function unconditionally → compilation failure

## Phase 2: DECOMPOSE

### Atomic Tasks

#### Task 1: Fix OAuth Feature Compilation Error
**Type**: Build Fix
**Priority**: P0 (Blocks all CI)
**Dependencies**: None

**Problem**: Binary calls `server_impl::load_oauth_config()` but function doesn't exist without `oauth` feature.

**Solution Options**:
1. **Option A**: Enable `oauth` feature by default
   - Pros: Simple, one-line change
   - Cons: Forces OAuth on all users, marked as "future" feature

2. **Option B**: Make `load_oauth_config` always available (ungated)
   - Pros: Allows conditional OAuth at runtime
   - Cons: Requires modifying oauth.rs to remove cfg attribute

3. **Option C**: Add feature-gated main function
   - Pros: Clean separation, feature-specific code
   - Cons: Complex, requires cfg attributes in binary

**Decision**: **Option B** - Make `load_oauth_config` always available
- Aligns with binary design (already checks `oauth_config.enabled`)
- OAuth can be disabled at runtime via environment variables
- Minimal code changes
- Maintains backward compatibility

**Implementation**:
1. Remove `#![cfg(feature = "oauth")]` from `oauth.rs` (line 12)
2. Make function return disabled config when feature not compiled
3. Keep validation functions gated behind feature flag

#### Task 2: Verify No Other Build Errors
**Type**: Validation
**Priority**: P0
**Dependencies**: Task 1

**Steps**:
1. Run `cargo build --all`
2. Check for any remaining compilation errors
3. Document any additional issues found

#### Task 3: Run Quality Checks
**Type**: Quality Assurance
**Priority**: P1
**Dependencies**: Task 2

**Checks**:
1. `cargo fmt --all -- --check` (format)
2. `cargo clippy --all -- -D warnings` (lint)
3. `cargo test --all` (tests)
4. `./scripts/quality-gates.sh` (comprehensive)

#### Task 4: Check for Additional CI Issues
**Type**: Investigation
**Priority**: P1
**Dependencies**: Task 3

**Steps**:
1. Review all workflow files for potential issues
2. Check ADR-023 for any outstanding CI items
3. Verify workflow configurations match best practices

#### Task 5: Create Feature Branch
**Type**: Git Operations
**Priority**: P2
**Dependencies**: None (can run in parallel with Task 1)

**Steps**:
1. Verify current branch status
2. Create new branch: `fix/ci-oauth-compilation-error`
3. Document branch purpose

#### Task 6: Commit Changes
**Type**: Git Operations
**Priority**: P2
**Dependencies**: Tasks 1-4

**Steps**:
1. Stage all changes
2. Create conventional commit
3. Follow commit message format from AGENTS.md

#### Task 7: Create Pull Request
**Type**: Git Operations
**Priority**: P2
**Dependencies**: Task 6

**Steps**:
1. Push branch to remote
2. Create PR with comprehensive description
3. Reference relevant ADRs
4. Link to any related issues

## Phase 3: STRATEGIZE

### Execution Strategy: **Sequential with Parallel Validation**

**Rationale**:
- Tasks have strong dependencies (build → quality → commit)
- Some tasks can run in parallel (branch creation, code fixes)
- Sequential ensures we don't commit broken code

### Execution Flow

```
┌─────────────────────────────────────────┐
│ Phase 1: Fix & Branch (Parallel)       │
├─────────────────────────────────────────┤
│ Task 1: Fix OAuth compilation          │
│ Task 5: Create feature branch          │
└─────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────┐
│ Phase 2: Validate (Sequential)         │
├─────────────────────────────────────────┤
│ Task 2: Verify build                   │
│ Task 3: Run quality checks             │
│ Task 4: Check additional CI issues     │
└─────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────┐
│ Phase 3: Git Operations (Sequential)   │
├─────────────────────────────────────────┤
│ Task 6: Commit changes                 │
│ Task 7: Create PR                      │
└─────────────────────────────────────────┘
```

### Coordination Approach

**Phase 1 (Mixed)**:
- Load required skills: `build-rust`, `conventional-commits`
- Execute code fix (direct edit)
- Create branch (bash)

**Phase 2 (Sequential)**:
- Run build validation (bash)
- Run quality checks (bash)
- Each step validates before proceeding

**Phase 3 (Sequential)**:
- Stage and commit changes
- Push and create PR

## Phase 4: COORDINATE

### Agent Assignment

**GOAP Agent (Current)**:
- Overall coordination
- Plan execution
- Quality gates
- Final synthesis

**No Additional Agents Needed**:
- This is a focused, single-fix task
- Build error is clearly identified
- Fix is straightforward (remove cfg attribute)
- Quality checks are standard bash commands

### Skill Loading

Load skills via `skill` tool (NOT `task` tool):
1. **build-rust**: For compilation validation
2. **code-quality**: For formatting/linting checks
3. **conventional-commits**: For proper commit messages
4. **test-runner**: For test execution (if needed)

## Phase 5: EXECUTE

### Quality Gates

**Gate 1**: After Task 1 (Code Fix)
- [ ] File `oauth.rs` modified correctly
- [ ] No cfg attribute on module level
- [ ] Function returns disabled config by default

**Gate 2**: After Task 2 (Build)
- [ ] `cargo build --all` succeeds
- [ ] No compilation errors
- [ ] No new warnings

**Gate 3**: After Task 3 (Quality)
- [ ] Format check passes
- [ ] Clippy passes (zero warnings)
- [ ] Tests pass
- [ ] Quality gates script passes

**Gate 4**: After Task 6 (Commit)
- [ ] Commit follows conventional format
- [ ] Commit message is descriptive
- [ ] Changes are staged correctly

**Gate 5**: After Task 7 (PR)
- [ ] PR created successfully
- [ ] PR description is comprehensive
- [ ] All CI checks triggered

## Phase 6: SYNTHESIZE

### Expected Outcomes

**Deliverables**:
1. Fixed `oauth.rs` file (feature-ungated)
2. New git branch with fix
3. Conventional commit
4. Pull request with description

**Metrics**:
- Compilation errors: 1 → 0
- CI workflows failing: Unknown → Will verify
- Code quality: Maintained (fmt, clippy pass)
- Tests: All passing

### Success Criteria

✅ Build succeeds: `cargo build --all`
✅ Format passes: `cargo fmt --check`
✅ Clippy passes: `cargo clippy -- -D warnings`
✅ Tests pass: `cargo test --all`
✅ PR created with proper description
✅ All changes committed atomically

## Risk Mitigation

**Risk 1**: Additional compilation errors after fix
- **Mitigation**: Task 2 validates complete build
- **Fallback**: Investigate and fix additional errors

**Risk 2**: Quality checks fail
- **Mitigation**: Run checks sequentially, fix issues as found
- **Fallback**: Use `test-fix` skill if test failures occur

**Risk 3**: Conflicts when pushing branch
- **Mitigation**: Start from clean branch state
- **Fallback**: Rebase or merge as needed

## Timeline Estimate

- Task 1 (Fix): 5 minutes
- Task 2 (Build): 10 minutes
- Task 3 (Quality): 15 minutes
- Task 4 (Additional): 10 minutes
- Task 5-7 (Git): 5 minutes

**Total**: ~45 minutes

---

**Status**: Ready to execute
**Next Action**: Begin Phase 1 - Fix OAuth compilation error
