# ADR-027: Strategy for Ignored Tests (WASI, Streaming, libsql)

**Status**: Accepted (Amended 2026-10-04)
**Date**: 2026-02-13

## Context

A total of 159 tests are `#[ignore]` across the workspace and documented in `plans/ignored_tests_inventory.json`:
- **do-memory-storage-turso**: 91 tests (libsql native library memory corruption and concurrency race conditions)
- **do-memory-core**: 48 tests (slow integration tests, ONNX/ort Send trait requirements, real storage backends)
- **do-memory-mcp**: 11 tests (performance benchmarks and native storage initialization)
- **memory-cli**: 2 tests (config property I/O timeout, external Turso setup)
- **e2e-tests**: 7 tests (subproccess quality gates, soak tests, MCP process integration)

### Safe Security Tests Split and Isolation

Previously, all security tests in `memory-storage-turso/tests/security_tests.rs` were ignored due to native libSQL memory corruption concerns.
In October 2026 (Follow-up Q02), pure URL protocol validation, token enforcement, and security error handling tests (14 tests) were unignored so that parameterization and protocol security checks run in standard CI without invoking native C libSQL connections:
- Command: `cargo test -p do-memory-storage-turso --test security_tests`

For native libSQL tests that require physical database handles or C-FFI interactions, tests run in an isolated subprocess/nightly CI step (`isolated-turso-native-tests` in `.github/workflows/nightly-tests.yml`) to prevent memory corruption crashes from failing standard CI runs silently.

### Machine-Readable Inventory

The machine-readable inventory is stored in `plans/ignored_tests_inventory.json`. It is validated continuously by `./scripts/check-ignored-tests.sh`.

Each inventory entry contains:
- `crate`: The workspace crate.
- `file` / `line` / `test`: Location and name of the ignored test.
- `reason`: Truthful explanation for why the test is ignored.
- `upstream_tracker`: URL to upstream tracker issue (e.g. `https://github.com/tursodatabase/libsql/issues`).
- `owner`: Responsible team or workgroup (e.g. `WG-008`).
- `revalidation_date`: Scheduled date for revalidation.

## Decision

1. **Unignore Safe Parameterization & Security Checks**:
   Keep pure input/URL/token security validation tests enabled in normal CI (`memory-storage-turso/tests/security_tests.rs`).

2. **Isolate Native Turso Tests**:
   Run native Turso integration tests in a separate, isolated nightly job with dedicated artifact collection and crash reporting (`cargo nextest run -p do-memory-storage-turso --test security_tests --run-ignored all`).

3. **Enforce Zero Undocumented Ignores**:
   Enforce inventory validation in `./scripts/check-ignored-tests.sh` so that any new or undocumented `#[ignore]` attribute fails CI.

## Consequences

- ✅ Pure URL/token security checks execute on every PR and standard CI run.
- ✅ Machine-readable inventory `plans/ignored_tests_inventory.json` tracks 100% of ignored tests.
- ✅ `./scripts/check-ignored-tests.sh` guarantees zero undocumented ignores.
- ✅ Isolated execution isolates native C libSQL memory corruption from standard CI pipeline while ensuring nightly tracking.
