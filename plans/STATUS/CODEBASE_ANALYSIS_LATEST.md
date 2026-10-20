# Codebase Analysis Latest — 2026-10-20 (Architecture & Status Refresh)

**Branch**: `main` @ `7411a33644cd59b9d796ebd349bd50969e4237a9`
**Workspace Version**: `0.1.47` · **Released Tag**: `v0.1.46` (tagged 2026-10-18)
**Active Track**: Architecture, status, and serialization evidence synchronization (DOC02)

## Architecture (as implemented)

| Crate | Role |
|-------|------|
| `do-memory-core` | Episodes, patterns, rewards, retrieval (CSM cascade), embeddings, F4 provenance/journal |
| `do-memory-storage-turso` | Durable libSQL / Turso |
| `do-memory-storage-redb` | Embedded cache (Postcard serialization) |
| `do-memory-mcp` | MCP server, lazy tools, audit, fail-closed code exec |
| `do-memory-cli` | Operator CLI |
| `do-memory-test-utils` / `do-memory-benches` / `do-memory-examples` / `e2e-tests` | Support, benchmarking, examples, and integration test suite |

**Stack**: Rust 2024, Tokio, Turso/libSQL, redb, Postcard, optional embeddings, `csm` cascade.

## Health Summary

| Check | Result |
|-------|--------|
| Workspace crates | 9 crates, all synchronized at version `0.1.47` |
| Code execution | `execute_agent_code` is fail-closed (no production Wasmtime sandbox) |
| Serialization | Postcard in production (`do-memory-storage-redb`) |
| Validation harness | `./scripts/validate-plans.sh --all` exit 0 |
