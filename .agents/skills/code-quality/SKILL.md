---
name: code-quality
description: Maintain high code quality through formatting, linting, static analysis, and clean code principles. Use for rustfmt, clippy, cargo audit, code reviews, refactoring, and quality gates.
---

# Code Quality

Unified skill for Rust code quality and clean code development.

## Quick Commands

```bash
./scripts/code-quality.sh fmt                    # Format check
./scripts/code-quality.sh clippy --workspace     # Lint with clippy
./scripts/code-quality.sh audit                  # Security audit
./scripts/code-quality.sh check                  # Run all quality gates
./scripts/quality-gates.sh                       # Full gates (coverage 90%)
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

| Dimension | Focus |
|-----------|-------|
| Structure | Files <500 LOC, clear modules |
| Error Handling | Custom Error, Result<T>, `?` operator (no unwrap) |
| Async | spawn_blocking for CPU work, no blocking in async |
| Testing | >=90% coverage, AAA pattern, cargo nextest + doctests |
| Security | Parameterized SQL, env vars, no hardcoded secrets |

## Clean Code Principles

**SOLID**: Single Responsibility, Open-Closed, Liskov Substitution, Interface Segregation, Dependency Inversion

**DRY/KISS/YAGNI**: Extract common code, simple solutions, build what's needed now

## Anti-Patterns

| Bad | Fix |
|-----|-----|
| Excessive clone | Use borrowing or Arc |
| Unnecessary unwrap | Use `?` operator |
| Deep nesting | Extract methods |
| Large functions | Split (<50 LOC) |
| Deadlocks | Release locks before `.await` |

## Dependency Monitoring

```bash
cargo tree -d | grep -cE "^[a-z]"   # Count duplicates (target: <100)
cargo machete && cargo shear       # Find unused deps
```

## Code Review Output Format

```markdown
# Code Quality Report
Score: X/100 | Critical Issues: N | Warnings: M
- Structure: [Status] | Error Handling: [Status] | Testing: [Status]
1. [Issue] - File:line - Fix: [Recommendation]
```

## References

quality-dimensions.md, ADR-036 (Dependency Deduplication), ADR-032 (Disk Space)
