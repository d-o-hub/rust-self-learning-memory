# Agent Coding Guidelines

## Quick Reference
- **Build**: `./scripts/build-rust.sh dev|release|check|clean`
- **Quality**: `./scripts/code-quality.sh fmt|clippy|audit|check`
- **Tests**: `cargo test --all`
- **Quality Gates**: `./scripts/quality-gates.sh`

## Project Overview
Memory management system with episodic memory, semantic embeddings, Turso/libSQL + redb cache, MCP server.

**Stack**: Rust/Tokio + Turso + redb + embeddings (OpenAI/Cohere/Ollama/local)

**Crates**: memory-core, memory-storage-turso, memory-storage-redb, memory-mcp, memory-cli, test-utils, benches, examples

## Repo Orientation

| Crate | Purpose | Entry Point |
|-------|---------|-------------|
| `memory-core` | Domain types, episode lifecycle, embeddings, patterns | `src/lib.rs` |
| `memory-storage-turso` | Turso/libSQL persistent storage | `src/lib.rs` |
| `memory-storage-redb` | redb local cache layer | `src/lib.rs` |
| `memory-mcp` | MCP server, tool registry, Wasmtime sandbox | `src/bin/server.rs` |
| `memory-cli` | CLI interface, config management | `src/main.rs` |
| `test-utils` | Shared test helpers and fixtures | `src/lib.rs` |
| `benches` | Criterion benchmarks | `src/lib.rs` |

**Version**: Always check `Cargo.toml` workspace version (never hardcode in docs).

## Skill + CLI Pattern (CRITICAL)

**ALWAYS use Skill + CLI first** for high-frequency operations:

| Operation | Skill | Script/CLI | Token Cost |
|-----------|-------|-------------|-------------|
| Build | `build-rust` | `./scripts/build-rust.sh` | Low |
| Format/Lint | `code-quality` | `./scripts/code-quality.sh` | Low |
| Quality Gates | `code-quality` | `./scripts/quality-gates.sh` | Medium |
| CI Issues | `github-workflows` | `gh workflow list` | Low |
| Tests | `test-runner` | `cargo test --all` | Medium |
| Debug | `debug-troubleshoot` | `RUST_LOG=debug cargo test` | Medium |

**Before using task tool:**
1. Is there a skill in `.agents/skills/`? → Use it
2. Is there a script in `scripts/`? → Use it
3. Is this high-frequency? → Use Skill + CLI
4. Is this complex multi-agent? → Use task tool

**Example:**
```bash
# Load skill then run
skill: build-rust, code-quality
./scripts/build-rust.sh check
./scripts/code-quality.sh fmt
```

## Change Workflow (Golden Path)
1. Identify owner crate + relevant module
2. Read existing code patterns before modifying
3. Add/update tests (unit first, integration if cross-crate)
4. `./scripts/code-quality.sh fmt` → fix formatting
5. `cargo clippy --all -- -D warnings` → fix warnings
6. `cargo test -p <crate>` → targeted tests
7. `cargo test --all` → full suite
8. `./scripts/quality-gates.sh` → final validation

## Required Checks Before Commit
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all -- -D warnings`
- [ ] `cargo build --all`
- [ ] `cargo test --all`
- [ ] `./scripts/quality-gates.sh`

## Code Conventions
- **Max 500 LOC per file** (source code)
- Zero warnings policy (clippy)
- Single responsibility per module

## Core Invariants (Never Break)
- **Async**: Tokio runtime everywhere. No blocking in async paths (use `spawn_blocking`)
- **Storage**: Parameterized SQL only. Short transactions. No locks across `.await`
- **Serialization**: Postcard required (not bincode). See `agent_docs/code_conventions.md`
- **Clippy**: Zero warnings enforced (`-D warnings`). Fix, don't suppress
- **Files**: ≤500 LOC per source file. Split into submodules when exceeded
- **Tests**: ≥90% coverage. `#[tokio::test]` for async. AAA pattern (Arrange-Act-Assert)

## Feature Flags
- `openai`: OpenAI embeddings
- `local-embeddings`: CPU embeddings (ort, tokenizers)
- `turso`: Turso cloud storage
- `redb`: Cache layer
- `embeddings-full`: All providers
- `full`: All features

## Security
- Use environment variables (never hardcode)
- Parameterized SQL queries
- Validate inputs at API boundaries
- Use postcard for serialization

## Environment Variables
| Variable | Purpose | Required |
|----------|---------|----------|
| `TURSO_DATABASE_URL` | Turso database URL | For Turso backend |
| `TURSO_AUTH_TOKEN` | Turso authentication | For Turso backend |
| `OPENAI_API_KEY` | OpenAI embeddings | For openai feature |
| `RUST_LOG` | Logging level (debug/info/warn) | No (default: info) |

See `.env.example` for full list. Never commit secrets.

## Commit Format
`[module] description` or `fix(module): description`

## Performance Targets
- Episode Creation: < 50ms
- Step Logging: < 20ms
- Episode Completion: < 500ms
- Memory Retrieval: < 100ms

## Cross-References
| Topic | Document |
|-------|----------|
| Build commands & features | `agent_docs/building_the_project.md` |
| Testing strategies | `agent_docs/running_tests.md` |
| Code style & patterns | `agent_docs/code_conventions.md` |
| System architecture | `agent_docs/service_architecture.md` |
| Database schema | `agent_docs/database_schema.md` |
| Communication patterns | `agent_docs/service_communication_patterns.md` |
| Active roadmap | `plans/ROADMAPS/ROADMAP_ACTIVE.md` |
| Architecture decisions | `plans/adr/` |
