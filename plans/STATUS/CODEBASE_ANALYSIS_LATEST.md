# Codebase Analysis Latest — 2026-11-24

**Branch**: `main` @ `eb076391`
**Workspace Version**: `0.1.51` (development) · **Released Tag**: `v0.1.50` (`0fe1b89d`)
**Active Track**: post-`v0.1.50` feature parity (durable episode summaries + retention cleanup across storage, wrappers, sync, CLI, and MCP)

## Architecture (as implemented)

| Crate | Role |
|-------|------|
| `do-memory-core` | Episodes, patterns, rewards, learning orchestrator, retrieval (CSM cascade), embeddings, semantic summaries, retention policies (`WG-075`), provenance/journal, opt-in OTLP tracing (`otel`) |
| `do-memory-storage-turso` | Durable libSQL / Turso (`TursoStorage`, `ResilientStorage`, `CachedTursoStorage`) |
| `do-memory-storage-redb` | Embedded cache (Postcard serialization) |
| `do-memory-mcp` | MCP server, lazy tools, audit, retention cleanup tool, fail-closed code execution |
| `do-memory-cli` | Operator CLI (episodes, cleanup, summaries, patterns, playbooks, storage sync/health/journal) |
| `do-memory-test-utils` / `do-memory-benches` / `do-memory-examples` / `e2e-tests` | Support, benchmarking, adaptive-discovery example, integration tests |

**Stack**: Rust 2024, Tokio, Turso/libSQL, redb, Postcard, optional embeddings, `csm` cascade, opt-in OpenTelemetry OTLP.

## Health summary

| Check | Result |
|-------|--------|
| Workspace crates | 9 crates, all synchronized at `0.1.51` |
| Code execution | `execute_agent_code` fail-closed (no production Wasmtime sandbox) |
| Serialization | Postcard in production (`do-memory-storage-redb`) |
| Plan validators | `validate-plans.sh --all` green |
| Harness | `v0.4.6` pin; compat + doctrine probes green |
| Storage backends | Turso + redb + wrappers (`ResilientStorage`, `CachedTursoStorage`) capability checks green; no faked durable success |
| Summaries & retention | `supports_episode_summaries` and `supports_episode_cleanup` implemented and wired through storage, sync, CLI (`episode view`, `episode cleanup`), and MCP (`get_episode`, `cleanup_episodes`) |
| Tests | nextest suite green in the verification set; doctests via `cargo test --doc` |
| Open code gaps | 0 — only upstream-blocked advisories (ACT-404) |

Evidence for the current run lives in [`VALIDATION_LATEST.md`](VALIDATION_LATEST.md).
Historical analysis snapshots are under [`archive/`](archive/).
