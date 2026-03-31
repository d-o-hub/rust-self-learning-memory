# Ignored Tests Analysis Report

**Generated**: 2026-03-07
**Total ignored tests**: 51

## Categorization

### 1. Timing-Dependent Tests (8 tests) - **FIXABLE**

| File | Line | Reason |
|------|------|--------|
| `do-memory-storage-turso/src/pool/adaptive_tests.rs` | 179 | Connection ID uniqueness |
| `do-memory-storage-turso/src/pool/adaptive_tests.rs` | 203 | Cleanup callback timing |
| `do-memory-storage-turso/src/pool/adaptive_tests.rs` | 322 | Connection cache timing |
| `do-memory-storage-turso/src/pool/caching_pool_tests.rs` | 20 | Pool creation timing |
| `do-memory-storage-turso/src/transport/compression.rs` | 90 | Decompression matching |
| `do-memory-storage-turso/src/cache/query_cache_tests.rs` | 51 | Cache expiration timing |
| `do-memory-storage-turso/src/cache/adaptive_ttl_tests.rs` | 80 | TTL adaptation timing |
| `do-memory-storage-turso/src/cache/adaptive_ttl_tests.rs` | 101 | Cache expiration timing |

**Fix Strategy**: Use `tokio::time::pause()` for time-dependent tests, increase timeouts for async operations.

### 2. WASM/WASI Tests (6 tests) - **NEEDS IMPLEMENTATION**

| File | Line | Reason |
|------|------|--------|
| `do-memory-mcp/src/sandbox/tests.rs` | 30 | Sandbox timing issues |
| `do-memory-mcp/src/sandbox/tests.rs` | 49 | Sandbox timing issues |
| `do-memory-mcp/src/sandbox/tests.rs` | 213 | Sandbox timing issues |
| `do-memory-mcp/src/sandbox/tests.rs` | 235 | Sandbox timing issues |
| `do-memory-mcp/src/unified_sandbox/tests.rs` | 47 | Binary data handling |
| `do-memory-mcp/src/unified_sandbox/tests.rs` | 198 | Binary data handling |
| `do-memory-mcp/src/wasmtime_sandbox/tests.rs` | 43 | WASI implementation |
| `do-memory-mcp/src/wasmtime_sandbox/tests.rs` | 120 | WASI timeout handling |
| `do-memory-mcp/src/wasm_sandbox/tests.rs` | 19 | WASM timeout enforcement |

**Fix Strategy**: These are infrastructure tests - may remain ignored until proper WASM/WASI implementation.

### 3. Slow Integration Tests (~30 tests) - **ACCEPTABLE BY DESIGN**

| File | Lines |
|------|-------|
| `do-memory-core/tests/tag_operations_test.rs` | 9 tests |
| `do-memory-core/tests/heuristic_learning.rs` | 8 tests |
| `do-memory-core/tests/performance.rs` | 7 tests |
| `do-memory-core/tests/input_validation.rs` | 2 tests |
| `do-memory-core/tests/learning_cycle.rs` | 1 test |

**Fix Strategy**: These are intentionally slow tests for release CI - keep ignored for normal CI runs.

### 4. Requires Infrastructure (4 tests) - **BLOCKED**

| File | Line | Reason |
|------|------|--------|
| `do-memory-core/tests/relationship_integration.rs` | 432 | Requires real storage backends |
| `do-memory-core/tests/regression.rs` | 319 | Non-deterministic pattern extraction |
| `do-memory-core/tests/regression.rs` | 424 | Long-running performance test |
| `do-memory-core/tests/compliance.rs` | 422 | Requires MCP server implementation |
| `do-memory-core/tests/compliance.rs` | 434 | Requires MCP server implementation |
| `do-memory-core/tests/performance.rs` | 400 | Requires pattern accuracy infrastructure |

**Fix Strategy**: Keep ignored until infrastructure is available.

### 5. Flaky Tests (3 tests) - **NEEDS INVESTIGATION**

| File | Line | Reason |
|------|------|--------|
| `do-memory-core/src/embeddings/local.rs` | 330 | Random mock embeddings |
| `do-memory-mcp/src/unified_sandbox/tests.rs` | 47, 198 | Binary data handling |

## Recommended Actions

1. **Fix timing-dependent tests** (8 tests) - Use `tokio::time::pause()`
2. **Keep slow tests ignored** (~30 tests) - By design
3. **Keep infrastructure tests ignored** (6 tests) - Blocked
4. **Investigate flaky tests** (3 tests) - May be fixable

## Target

- Current: 51 ignored tests
- Target: ≤10 ignored tests
- Fixable: 8-11 tests (timing-dependent + flaky)
- Acceptable: ~30-40 tests (slow, infrastructure, WASM by design)