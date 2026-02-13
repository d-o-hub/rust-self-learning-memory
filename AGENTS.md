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

## Required Checks Before Commit
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all -- -D warnings`
- [ ] `cargo build --all`
- [ ] `cargo test --all`
- [ ] `./scripts/quality-gates.sh`

## Detailed Documentation
For comprehensive guides, see `agent_docs/`:
- `building_the_project.md` - Build commands, features
- `running_tests.md` - Testing strategies, coverage
- `code_conventions.md` - Rust idioms, zero-warnings policy
- `service_architecture.md` - System design, components
- `database_schema.md` - Data structures, relationships

## Code Conventions
- **Max 500 LOC per file** (source code)
- Zero warnings policy (clippy)
- Single responsibility per module

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

## Commit Format
`[module] description` or `fix(module): description`

## Performance Targets
- Episode Creation: < 50ms
- Step Logging: < 20ms
- Episode Completion: < 500ms
- Memory Retrieval: < 100ms
