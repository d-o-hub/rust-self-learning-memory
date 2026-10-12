---
name: feature-implement
description: "Systematic approach to implementing new features in the Rust memory system following project conventions. Use when adding new functionality with proper testing and documentation, maintaining code quality and test coverage."
---

# Feature Implementation

Systematic approach to implementing new features in the Rust memory system.

## Quick Reference

- **[Process](process.md)** - Implementation phases (planning, design, implementation, testing)
- **[Structure](structure.md)** - Module structure and file organization
- **[Patterns](patterns.md)** - Code patterns and conventions
- **[Quality](quality.md)** - Quality standards and checklists

## When to Use

- Adding new functionality to the codebase
- Following project conventions for feature development
- Maintaining code quality and test coverage

## Core Process

1. **Planning** - Understand requirements and constraints
2. **Design** - Architecture and module structure
3. **Implementation** - Write code following patterns
4. **Testing** - Add tests with >90% coverage
5. **Documentation** - Update docs and examples

See **[process.md](process.md)** for detailed phases and **[patterns.md](patterns.md)** for Rust code patterns.

## Project Standards

- File size: ≤ 500 LOC per file
- Async/Tokio patterns for I/O
- Error handling: `anyhow::Result`
- Storage: Turso (durable) + redb (cache)
- Serialization: postcard

## StorageBackend Surface Guardrails

- A **new defaulted `StorageBackend` method** must be classified in
  `scripts/storage-backend-capabilities.toml` in the same PR, or the fail-closed
  checker in `fast-gate` blocks every PR (regenerate
  `docs/generated/storage-backend-capabilities.json` with `--write`).
- Use `fail-closed` when the default returns `Error::CapabilityUnavailable` but
  there is no predicate to name; `capability-gated` requires a real
  `supports_*`/`can_*` predicate in the trait source.
- A new capability predicate belongs on `StorageBackendCapabilities` (default
  `false`), and every cache/durability caller gates on it before treating a write
  as durable (ADR-081 style).
- Every `StorageBackend` test double needs the `StorageBackendCapabilities` impl
  (`impl StorageBackendCapabilities for T {}` is enough when it advertises
  nothing).
