# ADR-055: DuckDB Analytics Test Stabilization

## Status

Proposed -> Implemented

## Context

During the integration of the DuckDB storage backend, the integration tests (`memory-storage-duckdb/tests/integration_tests.rs`) were written against a speculative API. Specifically:
1. They assumed the existence of `AgentType::Orchestrator`.
2. They assumed a generic `run_analytics_query` method taking raw SQL.
3. Module paths for `MonitoringStorageBackend` and `Pattern` were slightly off.

As a result, the test suite failed to compile.

## Decision

Instead of expanding the core API with arbitrary raw SQL execution (`run_analytics_query`) just for testing, we will adhere to the typed and structured analytics methods already implemented in `DuckDbStorage` (e.g., `query_pattern_trends`, `query_session_windowing`).

1. Update tests to use `AgentType::GoapAgent` instead of `Orchestrator`.
2. Fix all incorrect import paths.
3. Refactor `test_duckdb_analytics` to test the actual implemented analytics methods.

## Consequences

- **Positive**: Tests will accurately reflect the public API of the DuckDB storage crate. No generic raw SQL injection points are added to the storage backend trait or struct.
- **Negative**: Less flexibility in the test suite to write arbitrary queries without adding new methods to `DuckDbStorage`.
