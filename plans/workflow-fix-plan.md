# GitHub Actions Workflow Fix Plan

## Current State Analysis

### Failing Workflows
1. **Quick Check** (21542226486) - Formatting issues
2. **CI** (21542226482) - Formatting + Test failures
3. **Performance Benchmarks** (21542226481) - Depends on Quick Check

### Root Causes
1. **Formatting Issues**:
   - `memory-core/src/episode/relationships.rs:309` - formatting
   - `memory-storage-turso/src/relationships.rs:3` - imports formatting
   - `memory-storage-turso/src/relationships.rs:208-231` - error handling formatting

2. **Test Failures**:
   - `relationships::tests::test_add_relationship` - "no such table: episodes"
   - `relationships::tests::test_get_dependencies` - "no such table: episodes"
   - `relationships::tests::test_get_relationships` - "no such table: episodes"
   - `relationships::tests::test_relationship_exists` - "no such table: episodes"
   - `relationships::tests::test_remove_relationship` - "no such table: episodes"

## Execution Plan

### Phase 1: Fix Formatting Issues (Parallel)
- Agent: code-quality
- Task: Run cargo fmt to fix all formatting issues
- Priority: P0

### Phase 2: Fix Test Failures (Sequential)
- Agent: rust-specialist
- Task: Fix relationship tests - ensure episodes table is created before running relationship tests
- Priority: P0

### Phase 3: Verify All Checks Pass (Sequential)
- Agent: test-runner
- Task: Run full test suite and verify all checks pass
- Priority: P0

### Phase 4: Re-trigger GitHub Actions (Final)
- Task: Verify workflows pass on GitHub
- Priority: P1

## Success Criteria
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all -- -D warnings` passes
- [ ] `cargo build --all` succeeds
- [ ] `cargo test --all` passes
- [ ] All GitHub Actions workflows green

## Quality Gates
1. After Phase 1: Formatting check passes
2. After Phase 2: Test failures resolved
3. After Phase 3: All local checks pass
4. After Phase 4: GitHub Actions all green
