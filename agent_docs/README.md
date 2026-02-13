# Agent Documentation Index

Quick reference for AI coding agents working on this project. Start with `AGENTS.md` in the project root for the primary guidance.

## Documents (Reading Order)

| # | Document | Purpose |
|---|----------|---------|
| 1 | `building_the_project.md` | Build commands, feature flags, prerequisites, troubleshooting |
| 2 | `code_conventions.md` | Rust idioms, formatting, naming, error handling, serialization |
| 3 | `running_tests.md` | Test categories, coverage, benchmarks, debugging tests |
| 4 | `service_architecture.md` | System design, crate responsibilities, module breakdown |
| 5 | `database_schema.md` | Turso + redb schemas, tables, indexes, relationships |
| 6 | `service_communication_patterns.md` | Inter-service communication, MCP protocol, async patterns |

## Quick Links

- **Primary guidance**: `../AGENTS.md`
- **Architecture decisions**: `../plans/adr/`
- **Active roadmap**: `../plans/ROADMAPS/ROADMAP_ACTIVE.md`
- **Skills reference**: `../.agents/skills/`
- **Scripts**: `../scripts/` (build-rust.sh, code-quality.sh, quality-gates.sh)

## Version Policy

Do not hardcode version numbers in documentation. Reference `Cargo.toml` workspace version instead.
