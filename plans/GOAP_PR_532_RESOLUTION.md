# GOAP Plan: PR 532 Resolution

## Goal
Resolve all failing GitHub Actions and address all reviewer comments in PR 532 ("Add DuckDB as optional storage backend").

## Success Criteria
- [ ] All GitHub Actions checks pass (CI, Quick Check, YAML Lint, Security, Performance, Coverage).
- [ ] All CodeRabbit reviewer comments are addressed and resolved.
- [ ] Code quality standards (fmt, clippy, audit) are met.
- [ ] Test coverage meets the 90% threshold.

## Tasks

### Phase 1: Infrastructure & Infrastructure-adjacent Fixes
- [x] **T1.1: Fix `scripts/code-quality.sh`**
- [x] **T1.2: Update `Cargo.toml`**
- [x] **T1.3: Fix Markdown Linting (MD022)**
- [x] **T1.4: Fix `memory-core/src/retrieval/cascade/mod.rs`**

### Phase 2: Storage Backend Refinements
- [x] **T2.1: Refactor `memory-storage-duckdb/src/storage/patterns.rs`**
- [x] **T2.2: Refactor `memory-storage-turso/src/cache/query_cache_types.rs`**

### Phase 3: Test Suite Optimization
- [x] **T3.1: Split `memory-mcp/tests/persistent_storage_tests.rs`**
- [x] **T3.2: Update `memory-storage-turso/src/cache/query_cache_tests.rs`**

### Phase 4: Validation
- [x] **T4.1: Run Formatting & Linting**
- [x] **T4.2: Execute Tests**
- [x] **T4.3: Quality Gates**

## Execution Strategy
- Use parallel agents for Tasks in Phase 1 and Phase 2.
- Phase 3 can be handled by a specialized test-fix agent.
- Phase 4 will be the final validation step.
