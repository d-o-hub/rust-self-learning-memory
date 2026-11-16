# Codebase Analysis Latest — 2026-11-16

**Branch**: `main` @ `0fe1b89d`
**Workspace Version**: `0.1.51` (development) · **Released Tag**: `v0.1.50` (`0fe1b89d`)
**Active Track**: post-release closeout, plans compaction, harness pin

## Architecture (as implemented)

| Crate | Role |
|-------|------|
| `do-memory-core` | Episodes, patterns, rewards, retrieval (CSM cascade), embeddings, provenance/journal |
| `do-memory-storage-turso` | Durable libSQL / Turso |
| `do-memory-storage-redb` | Embedded cache (Postcard serialization) |
| `do-memory-mcp` | MCP server, lazy tools, audit, fail-closed code execution |
| `do-memory-cli` | Operator CLI |
| `do-memory-test-utils` / `do-memory-benches` / `do-memory-examples` / `e2e-tests` | Support, benchmarking, examples, integration tests |

**Stack**: Rust 2024, Tokio, Turso/libSQL, redb, Postcard, optional embeddings, `csm` cascade.

## Health summary

| Check | Result |
|-------|--------|
| Workspace crates | 9 crates, all synchronized at `0.1.51` |
| Code execution | `execute_agent_code` fail-closed (no production Wasmtime sandbox) |
| Serialization | Postcard in production (`do-memory-storage-redb`) |
| Plan validators | `validate-plans.sh --all` green |
| Harness | `v0.4.6` pin; compat + doctrine probes green |
| Storage backends | Turso + redb capability checks green; no faked durable success |
| Tests | nextest suite green in the verification set; doctests via `cargo test --doc` |
| Open code gaps | 0 — only upstream-blocked advisories and maintainer-external ADR lifecycle |

Evidence for the current run lives in [`VALIDATION_LATEST.md`](VALIDATION_LATEST.md).
Historical analysis snapshots are under [`archive/`](archive/).
