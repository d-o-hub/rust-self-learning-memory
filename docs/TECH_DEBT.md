# Structured Tech Debt Registry

This document tracks identified technical debt across the Rust Self-Learning Memory System.

## Architecture

- **Dual Backend Synchronization**: The current `StorageBackend` trait lacks methods for native vacuum and advanced statistics, leading to approximations in the CLI.
- **Lock Contention**: Mixed usage of `parking_lot` and `tokio` locks in `memory-mcp` could lead to performance bottlenecks under heavy load.

## Testing

- **Integration Test Flakiness**: Some tests depend on environment variables or network access and may be flaky in constrained CI environments.
- **Turso Integration Tests**: 70 tests are currently ignored due to an upstream `libsql` bug (see ADR-027).

## Documentation

- **Broken Internal Links**: Historical plan files contain many broken links to archived documents.
- **API Reference Drift**: Some doc examples lag behind the latest trait implementations (fixed in v0.1.22 but requires monitoring).

## Maintenance

- **Dependency Bloat**: High number of duplicate transitive dependencies due to wasmtime and libsql ecosystem split.
- **Dead Code**: Target ≤40 `#[allow(dead_code)]` annotations; requires continuous pruning.
