---
name: code-quality
description: Maintain high code quality through formatting, linting, static analysis, and clean code principles. Use for rustfmt, clippy, cargo audit, code reviews, refactoring, and quality gates.
---

# Code Quality

Unified skill for Rust code quality and clean code development. Use scripts as primary interface.

## Quick Commands

```bash
# Format check
./scripts/code-quality.sh fmt

# Lint with clippy
./scripts/code-quality.sh clippy --workspace

# Security audit
./scripts/code-quality.sh audit

# Run all quality gates
./scripts/code-quality.sh check

# Full quality gates (coverage threshold 90%)
./scripts/quality-gates.sh
```

## Quality Gates

| Check | Command | Target |
|-------|---------|--------|
| Format | `./scripts/code-quality.sh fmt` | 100% compliant |
| Lint | `./scripts/code-quality.sh clippy --workspace` | Zero warnings |
| Audit | `cargo audit` | No known vulnerabilities |
| Coverage | `cargo llvm-cov --html` | >=90% |
| Docs | `cargo doc --no-deps` | All public APIs |

## Rust Quality Dimensions

| Dimension | Focus | Target |
|-----------|-------|--------|
| Structure | Files <500 LOC, clear modules | <500 LOC per file |
| Error Handling | Custom Error, Result<T>, no unwrap | `?` operator, thiserror |
| Async Patterns | spawn_blocking for CPU work | No blocking in async |
| Testing | >=90% coverage, AAA pattern | cargo nextest + doctests |
| Documentation | Public APIs documented | cargo doc passes |
| Security | Parameterized SQL, env vars | No hardcoded secrets |

## Clean Code Principles

### SOLID
- **Single Responsibility**: One reason to change per module
- **Open-Closed**: Extend via traits, not modification
- **Liskov Substitution**: Subtypes substitutable for base types
- **Interface Segregation**: Small, focused interfaces
- **Dependency Inversion**: Depend on abstractions, inject deps

### DRY/KISS/YAGNI
- **DRY**: Extract common code, single source of truth
- **KISS**: Simple solutions, avoid over-engineering
- **YAGNI**: Build what's needed now, defer complexity

## Anti-Patterns

### Rust-Specific
- Excessive clone -> Use borrowing or Arc
- Unnecessary unwrap -> Use `?` operator
- Deep nesting -> Extract methods
- Large functions -> Split (<50 LOC)
- Deadlocks -> Release locks before `.await`

### General Code Smells
- Long methods -> Break into smaller
- Duplicate code -> Extract reusable functions
- God class -> Reduce responsibilities
- Feature envy -> Move methods to appropriate classes

## Best Practices Checklist

### DO
- Files <500 LOC, clear module hierarchy
- Custom Error enum with Result<T>
- No unwrap() in production (tests only)
- async fn for IO, spawn_blocking for CPU
- >=90% test coverage
- Public APIs documented
- SOLID principles applied
- No code duplication (DRY)

### DON'T
- Violate SOLID principles
- Duplicate code "for now"
- Functions >50 lines without justification
- Skip static analysis warnings
- Over-engineer simple problems
- Create tight coupling

## Code Review Output Format

```markdown
# Code Quality Report

## Summary
- Score: X/100
- Critical Issues: N
- Warnings: M

## By Dimension
- Structure: [Status]
- Error Handling: [Status]
- Async Patterns: [Status]
- Testing: [Status]
- Documentation: [Status]

## Critical Issues
1. [Issue] - File:line - Fix: [Recommendation]

## Action Items
- [ ] High: Fix critical issues
- [ ] Medium: Address warnings
```

## Dependency Monitoring

```bash
# Count duplicate dependencies (target: <100)
cargo tree -d | grep -cE "^[a-z]"

# Find unused dependencies
cargo machete && cargo shear
```

## References

- Detailed dimensions: `quality-dimensions.md` (in this directory)
- ADR-036: Dependency Deduplication
- ADR-032: Disk Space Optimization