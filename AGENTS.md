# Agent Coding Guidelines

## Project Overview
This is a memory management system with episodic memory capabilities, semantic embeddings, and multiple storage backends. The system provides persistent memory across agent interactions through an MCP (Model Context Protocol) server.

**Stack**: Rust/Tokio + Turso/libSQL + redb cache + optional embeddings
**Crates**: 
- `memory-core`: Core memory operations and embeddings
- `memory-storage-turso`: Primary database storage
- `memory-storage-redb`: Cache layer
- `memory-mcp`: MCP server implementation

## Quick Reference
- **Build & Test**: `cargo build --all && cargo test --all`
- **Quality Check**: `./scripts/quality-gates.sh` (maintains >90% coverage)
- **Debug Tests**: `RUST_LOG=debug cargo test`

## File Organization
- Plans/analysis stored in `@plans/` folder only
- Maximum 500 lines per file

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

## Development Workflow
- Follow existing code patterns from examples in the codebase
- Use `cargo fmt` and `cargo clippy` for automatic formatting/linting
- Write tests for new functionality
- Run quality gates before committing

## Commit Format
`[module] description` or `fix(module): description`