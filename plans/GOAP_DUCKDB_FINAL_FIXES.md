# GOAP Plan: DuckDB Final Fixes and Coverage Improvement

## Goal
Address all CodeRabbit feedback and improve test coverage to >=90% for the DuckDB storage backend.

## Success Criteria
- [ ] Schema initialization uses `execute_batch` for multi-statement SQL.
- [ ] VSS extension loading uses `execute_batch`.
- [ ] Agent metrics retrieval correctly maps `current_streak` and `longest_streak`.
- [ ] JSON deserialization errors are propagated, not swallowed.
- [ ] Pattern storage uses the actual pattern type.
- [ ] Integration tests cover all storage methods (CRUD + Analytics + Relationships + Recs + Monitoring).
- [ ] Tests avoid `.unwrap()` and use proper error handling.
- [ ] Patch coverage meets project requirements.
- [ ] All CI checks pass.

## Tasks

### Phase 1: Implementation Fixes (Parallel)

#### Task 1.1: Schema and Extensions
- **Files**: `memory-storage-duckdb/src/lib.rs`, `memory-storage-duckdb/src/storage/mod.rs`
- **Action**: Switch `conn.execute` to `conn.execute_batch` for multi-statement SQL.
- **Agent**: `codebase_investigator`

#### Task 1.2: Metrics and Patterns
- **Files**: `memory-storage-duckdb/src/storage/monitoring.rs`, `memory-storage-duckdb/src/storage/patterns.rs`
- **Action**: Fix `current_streak`/`longest_streak` mapping and `pattern_type` placeholder.
- **Agent**: `codebase_investigator`

#### Task 1.3: Error Handling
- **Files**: `memory-storage-duckdb/src/storage/episodes.rs`, `memory-storage-duckdb/src/storage/patterns.rs`
- **Action**: Propagate JSON errors instead of using `.ok()`.
- **Agent**: `codebase_investigator`

### Phase 2: Test Expansion (Sequential to Phase 1)

#### Task 2.1: Comprehensive Integration Tests
- **Files**: `memory-storage-duckdb/tests/integration_tests.rs`
- **Action**: Add missing tests for all storage methods. Fix privacy/type issues.
- **Agent**: `generalist`

### Phase 3: Validation

#### Task 3.1: Quality Gates
- **Action**: Run fmt, clippy, and tests.
- **Agent**: `generalist`

## Dependencies
- Phase 2 depends on Phase 1 completion.
- Phase 3 depends on Phase 2 completion.
