# GOAP: DuckDB Test Stabilization

## Goal

Resolve compilation errors in `memory-storage-duckdb` integration tests and align the test suite with the actual codebase implementations.

## Current State

- `memory-storage-duckdb/tests/integration_tests.rs` fails to compile.
- The test uses `MonitoringStorageBackend` and `Pattern` from incorrect module paths.
- The test uses `AgentType::Orchestrator` which does not exist in `memory-core/src/monitoring/types.rs`.
- The test calls `run_analytics_query` on `DuckDbStorage` which is not implemented; instead, specific analytics methods like `query_session_windowing` exist.

## Tasks

- [ ] Write ADR-055 documenting the test stabilization approach.
- [ ] Fix module imports in `integration_tests.rs`.
- [ ] Replace `AgentType::Orchestrator` with `AgentType::GoapAgent` in `integration_tests.rs`.
- [ ] Update `test_duckdb_analytics` to use existing analytics methods (e.g., `query_session_windowing` or `query_pattern_trends`) instead of `run_analytics_query`.
- [ ] Run `cargo nextest run -p do-memory-storage-duckdb` to verify.
- [ ] Run `scripts/code-quality.sh` to ensure formatting and linting pass.
