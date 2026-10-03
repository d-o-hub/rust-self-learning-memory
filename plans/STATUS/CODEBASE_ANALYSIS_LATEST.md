# Codebase Analysis Latest — 2026-10-02 (Architecture & Status Refresh)

**Branch**: `main` @ `4f4f4ba8d06a82429a2e539c65c3cb37624291e9`
**Workspace Version**: `0.1.45` · **Released Tag**: `v0.1.44` (tagged 2026-10-02)
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
| Workspace crates | 9 crates, all synchronized at version `0.1.45` |
| Code execution | `execute_agent_code` is fail-closed (no production Wasmtime sandbox) |
| Serialization | Postcard in production (`do-memory-storage-redb`) |
| Validation harness | `./scripts/validate-plans.sh --all` exit 0 |
