# Agent Coding Guidelines

## Project Overview
This is a memory management system with episodic memory capabilities, semantic embeddings, and multiple storage backends. The system provides persistent memory across agent interactions through an MCP (Model Context Protocol) server.

**Last Updated**: 2026-01-31 (v0.1.13, Phase 2 75% complete, Phase 3 planning)

**Stack**: Rust/Tokio + Turso/libSQL + redb cache + optional embeddings (OpenAI, Cohere, Ollama, local)

**Codebase Stats**: 632 Rust files, ~140K LOC, 811+ lib tests, 9 workspace members

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
- Plans/analysis/validation/reports and all other non permanent doc .md files stored in `plans/` folder only
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
- All source code files must be ≤500 LOC (split large files into modules); benchmark files (`benches/*.rs`) are exempt from this limit
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
| Operation | Target (P95) | Actual Performance | Status |
|-----------|-------------|-------------------|--------|
| Episode Creation | < 50ms | ~2.5 µs (19,531x faster) | ✅ |
| Step Logging | < 20ms | ~1.1 µs (17,699x faster) | ✅ |
| Episode Completion | < 500ms | ~3.8 µs (130,890x faster) | ✅ |
| Pattern Extraction | < 1000ms | ~10.4 µs (95,880x faster) | ✅ |
| Memory Retrieval | < 100ms | ~721 µs (138x faster) | ✅ |

## Quality Standards
- **Test Coverage**: >90% (current: 92.5%)
- **Test Pass Rate**: >95% (current: 99.5%)
- **Clippy Warnings**: 0 (strictly enforced)
- **Code Formatting**: 100% rustfmt compliant
- **Security**: Zero known vulnerabilities
- **Performance**: <10% regression threshold
- **File Size Compliance**: <500 LOC (all modules compliant)

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
6. **Handoff Coordination**: Spawn 1-9 agents with GOAP orchestration for complex fixes
7. **Verify**: Re-run checks after each fix iteration
8. **Never Skip**: Do not commit or push until ALL checks pass

### Iterative Fix Process
```
Loop 1: Identify and fix obvious issues
├─ If all resolved → Success
└─ If issues remain → Continue

Loop 2: Research remaining issues
├─ Use @perplexity-researcher-reasoning-pro for unknown errors
├─ Use @web-search-researcher for best practices
└─ Apply researched solutions

Loop 3+: Continue until convergence
├─ Spawn specialized agents for specific issue types
├─ Use parallel execution where possible
└─ Stop only when ALL checks pass
```

### Agent Coordination for CI Fixes
- **1-3 agents**: For simple lint/format issues
- **4-6 agents**: For test failures across multiple modules
- **7-9 agents**: For complex multi-category issues (lint + tests + security)

### Required Checks Before Any Commit
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all -- -D warnings` passes (zero warnings)
- [ ] `cargo build --all` succeeds
- [ ] `cargo test --all` passes (or identify pre-existing failures to fix)
- [ ] `./scripts/quality-gates.sh` passes
- [ ] All GitHub Actions workflows that can pass, DO pass
- [ ] Security audit passes (or secrets properly excluded)