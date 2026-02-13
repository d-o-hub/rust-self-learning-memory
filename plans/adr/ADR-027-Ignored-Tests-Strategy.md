# ADR-027: Strategy for Ignored Tests (WASI, Streaming)

**Status**: Accepted
**Date**: 2026-02-13

## Context

8+ tests are `#[ignore]` in the memory-mcp crate due to:
- WASI implementation gaps (3 tests)
- Streaming feature immaturity (2 tests)
- Binary data handling issues (2 tests)
- Long-running soak tests (1 test)

## Problem

Ignored tests reduce:
- Test coverage visibility
- Regression detection capability
- Technical debt tracking

## Decision

**Feature-gate tests** with clear documentation and relaxed thresholds.

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

| Test | File | Feature Gate | Priority |
|------|------|--------------|----------|
| test_wasi_capture_with_timeout | wasmtime_sandbox/tests.rs | wasi-impl | P3 |
| test_wasi_stdout_stderr_capture | wasmtime_sandbox/tests.rs | wasi-impl | P3 |
| test_unified_sandbox_wasm_backend | unified_sandbox/tests.rs | wasi-impl | P3 |
| test_backend_update | unified_sandbox/tests.rs | streaming-impl | P3 |
| benchmark_streaming_performance | benchmarks.rs | streaming-impl | P2 |
| stability_test | tests/soak/stability_test.rs | soak-tests | P4 |

## Alternatives Considered

- **Option A** (Implement missing features): 40-60h effort, not priority
- **Option B** (Document and keep ignored): No feature gates
- **Option C** (Remove dead tests): Loss of test specification
