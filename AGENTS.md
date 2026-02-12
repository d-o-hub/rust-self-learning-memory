# Agent Coding Guidelines

## Project Overview
This is a memory management system with episodic memory capabilities, semantic embeddings, and multiple storage backends. The system provides persistent memory across agent interactions through an MCP (Model Context Protocol) server.

**Last Updated**: 2026-02-10 (v0.1.14)\n\n**Codebase Stats**: 791 Rust files, ~198K LOC, 147+ test files, unknown workspace members

**Stack**: Rust/Tokio + Turso/libSQL + redb cache + optional embeddings (OpenAI, Cohere, Ollama, local)

**Crates**:
- `memory-core`: Core memory operations and embeddings
- `memory-storage-turso`: Primary database storage (libSQL) with connection pooling & compression
- `memory-storage-redb`: Cache layer (postcard serialization)
- `memory-mcp`: MCP server with 6-layer security sandbox
- `memory-cli`: Full-featured CLI (9 commands, 9 aliases)
- `test-utils`: Shared testing utilities
- `benches`: Comprehensive benchmark suite
- `examples`: Usage examples and demonstrations
- `tests`: Integration tests

## Quick Reference
- **Build & Test**: `cargo build --all && cargo test --all`
- **Quality Check**: `./scripts/quality-gates.sh` (maintains >90% coverage, current: 92.5%)
- **Debug Tests**: `RUST_LOG=debug cargo test`
- **Release Build**: `cargo build --release --workspace`
- **Clippy**: `cargo clippy --all -- -D warnings` (zero warnings enforced)

## File Organization
- Plans/analysis/validation/reports and all other non-permanent documentation files (.md, .txt, .rst, etc.) stored in `plans/` folder only
- Maximum 500 lines per file for source code (all 9/9 modules compliant after splitting 17 oversized files)
- **Benchmark files** (`benches/*.rs`) are exempt from the 500 LOC limit - they contain comprehensive performance tests that require extensive setup and measurement code
- Module structure follows single responsibility principle
- Each module should be self-contained and testable

## Agent Documentation
For specific tasks, refer to these focused documentation files:

- `agent_docs/building_the_project.md` - Build commands and setup
- `agent_docs/running_tests.md` - Testing strategies and coverage
- `agent_docs/code_conventions.md` - Rust idioms and patterns
- `agent_docs/service_architecture.md` - System design and components
- `agent_docs/database_schema.md` - Data structures and relationships
- `agent_docs/service_communication_patterns.md` - Inter-service communication

## General Documentation
- `docs/LOCAL_DATABASE_SETUP.md` - Database configuration
- `docs/YAML_VALIDATION.md` - Configuration validation
- `docs/QUALITY_GATES.md` - Testing and quality standards
- `TESTING.md` - Comprehensive testing guide
- `CONTRIBUTING.md` - Development workflow
- `SECURITY.md` - Security policies and practices
- `DEPLOYMENT.md` - Production deployment guide

## Development Workflow
- Follow existing code patterns from examples in the codebase
- Use `cargo fmt` and `cargo clippy` for automatic formatting/linting
- Write tests for new functionality (maintain >90% coverage)
- Run quality gates before committing
- Use postcard for serialization in storage layers
- Use parameterized queries to prevent SQL injection
- **Module patterns used**: async traits for storage operations, `thiserror` for domain errors, `anyhow::Result` for public APIs, builder pattern for complex types, newtype pattern for type safety, Arc/Mutex for shared state

## Feature Flags
Enable optional features via Cargo:
- `openai`: OpenAI API embeddings
- `local-embeddings`: CPU-based local embeddings
- `turso`: Enable Turso cloud storage (requires memory-storage-turso)
- `redb`: Enable redb cache storage (requires memory-storage-redb)
- `embeddings-full`: Enable all embedding providers
- `full`: All features enabled (turso, redb, embeddings-full)

## Performance Targets
Performance targets are configured in [performance-config.yaml](performance-config.yaml):

| Operation | Target (P95) | Status |
|-----------|-------------|--------|
| Episode Creation | < 50ms | ✅ |
| Step Logging | < 20ms | ✅ |
| Episode Completion | < 500ms | ✅ |
| Pattern Extraction | < 1000ms | ✅ |
| Memory Retrieval | < 100ms | ✅ |

## Quality Standards
See [docs/QUALITY_GATES.md](docs/QUALITY_GATES.md) for comprehensive quality thresholds and configuration options.

### Key Requirements
- **Test Coverage**: >90% ([configured via](docs/QUALITY_GATES.md#configuration) `QUALITY_GATE_COVERAGE_THRESHOLD`)
- **File Size**: <500 LOC ([enforced by](docs/QUALITY_GATES.md#code-complexity-gate) quality gates)
- **Zero Warnings**: Strict clippy compliance ([see](agent_docs/code_conventions.md#zero-warnings-policy))
- **Performance**: <10% regression threshold ([monitored by](performance-config.yaml) performance targets)

## Commit Format
`[module] description` or `fix(module): description`

Examples:
- `feat(episode): add reflection generation after episode completion`
- `fix(storage): resolve redb cache corruption on concurrent writes`
- `docs(readme): update performance metrics for v0.1.7`
- `refactor(embeddings): simplify multi-provider interface`

**Field Renaming**: When renaming fields, include the old and new field names in commit messages to help reviewers:
- `refactor(episode): rename field_name to new_field_name for clarity`
- `feat(storage): rename old_id to external_id to match API spec`

## Security Guidelines
- Use environment variables for all secrets (never hardcode)
- Use parameterized queries for all SQL operations
- Validate all inputs at API boundaries
- Use postcard for serialization (safer than bincode)
- Document any unsafe code blocks
- Run security checks before committing
- Never edit `.env` or credential files

## CI/CD Quality Standards (CRITICAL)

### Pre-existing Issue Resolution Policy
**NEVER skip or ignore pre-existing issues. All warnings and errors must be resolved.**

When running lint, build, test, or CI checks:
1. **First Pass**: Run checks and identify ALL issues (new AND pre-existing)
2. **Categorize**: Separate issues by type (lint, test, security, performance)
3. **Prioritize**: Fix critical errors first, then warnings
4. **Loop Until Clean**: Use iterative approach with @loop-agent until ALL issues resolved
5. **Web Research**: If unable to resolve after first loop, use @perplexity-researcher-reasoning-pro for solutions
6. **Handoff Coordination**: Spawn 1-12 agents with grouping and skills with GOAP orchestration for complex fixes
7. **Verify**: Re-run checks after each fix iteration
8. **Never Skip**: Do not commit or push until ALL checks pass

### Required Checks Before Any Commit
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all -- -D warnings` passes (zero warnings)
- [ ] `cargo build --all` succeeds
- [ ] `cargo test --all` passes (or identify pre-existing failures to fix)
- [ ] `./scripts/quality-gates.sh` passes
- [ ] All GitHub Actions workflows that can pass, DO pass
- [ ] Security audit passes (or secrets properly excluded)
