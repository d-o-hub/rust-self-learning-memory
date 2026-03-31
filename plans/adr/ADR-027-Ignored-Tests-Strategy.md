# ADR-027: Strategy for Ignored Tests (WASI, Streaming, libsql)

**Status**: Accepted (Amended 2026-03-11)
**Date**: 2026-02-13

## Context

121 tests are `#[ignore]` across the workspace:
- **do-memory-storage-turso**: 71 tests (libsql memory corruption bug)
- **do-memory-core**: 37 tests (slow integration tests, real storage backends)
- **do-memory-mcp**: 9 tests (WASI, streaming, binary data)
- **tests/**: 4 tests (e2e tests)

### libsql Memory Corruption Bug (71 Tests)

The majority of ignored tests (71/121) are in `do-memory-storage-turso` and are blocked by an upstream bug:
- **Error**: `malloc_consolidate() unaligned fastbin chunk` in CI environment
- **Root Cause**: libsql native library memory corruption
- **Affected Files**: `sql_injection_tests.rs`, `multi_dimension_routing.rs`, `prepared_cache_integration_test.rs`, `security_tests.rs`
- **Tracking**: Upstream libsql issue

These tests are **legitimately skipped** and cannot be fixed without upstream resolution.

## Problem

Ignored tests reduce:
- Test coverage visibility
- Regression detection capability
- Technical debt tracking

## Decision

**Feature-gate tests** with clear documentation and relaxed thresholds.

**For libsql-blocked tests**: Document with clear `#[ignore]` reason referencing upstream bug.

## Amended Decision (2026-03-11)

WG-008 target of ≤30 ignored tests is **not achievable** because:
- 71 tests are blocked by upstream libsql memory corruption bug
- 37 tests in do-memory-core are intentionally slow integration tests
- Total unavoidable ignored tests: ~108

**Revised target**: Document legitimate skips rather than reduce count.

## libsql Memory Corruption - Affected Tests

| File | Count | Reason |
|------|-------|--------|
| `sql_injection_tests.rs` | 11 | libsql memory corruption |
| `multi_dimension_routing.rs` | 7 | libsql memory corruption |
| `prepared_cache_integration_test.rs` | 1 | libsql memory corruption |
| `security_tests.rs` | 15 | libsql memory corruption |
| `compression.rs` | 1 | Flaky in CI |
| **Total (Turso)** | **71** | Upstream bug |

## Implementation

### 1. Add feature gates to test attributes

```rust
#[cfg_attr(feature = "wasi-impl", ignore)]
#[test]
fn test_wasi_capture_with_timeout() {
    // WASI timeout handling requires proper WASI implementation
}

#[cfg_attr(feature = "streaming-impl", ignore)]
#[test]
fn test_streaming_efficiency() {
    // Streaming performance varies by environment
}
```

### 2. Update Cargo.toml features

```toml
[features]
wasi-impl = []       # Requires WASI sandbox completion
streaming-impl = []   # Requires streaming feature maturity
```

### 3. Add ignore reason documentation

```rust
/// WASI stdin/stdout capture requires proper WASI implementation in sandbox
/// Tracking issue: #XXX
/// Enable with: cargo test --features wasi-impl
#[test]
#[ignore = "WASI implementation not complete - see tracking issue #XXX"]
fn test_wasi_stdout_stderr_capture() {
    // ...
}
```

## Consequences

- ✅ Clear visibility of missing features
- ✅ Tests enable when features are ready
- ⚠️ Still ignored by default

## Test Inventory

### do-memory-mcp (9 tests)

| Test | File | Feature Gate | Priority |
|------|------|--------------|----------|
| test_wasi_capture_with_timeout | wasmtime_sandbox/tests.rs | wasi-impl | P3 |
| test_wasi_stdout_stderr_capture | wasmtime_sandbox/tests.rs | wasi-impl | P3 |
| test_unified_sandbox_wasm_backend | unified_sandbox/tests.rs | wasi-impl | P3 |
| test_backend_update | unified_sandbox/tests.rs | streaming-impl | P3 |
| benchmark_streaming_performance | benchmarks.rs | streaming-impl | P2 |
| stability_test | tests/soak/stability_test.rs | soak-tests | P4 |

### do-memory-core (37 tests)

| Category | Count | Reason |
|----------|-------|--------|
| `tag_operations_test.rs` | 9 | Slow integration test |
| `heuristic_learning.rs` | 7 | Slow integration test |
| `relationship_integration.rs` | 1 | Requires real storage backends |
| `regression.rs` | 2 | Non-deterministic / long-running |
| `embeddings/local.rs` | 1 | Flaky with random mock embeddings |

### do-memory-storage-turso (71 tests)

All blocked by upstream libsql memory corruption bug.

## Alternatives Considered

- **Option A** (Implement missing features): 40-60h effort, not priority
- **Option B** (Document and keep ignored): No feature gates
- **Option C** (Remove dead tests): Loss of test specification
