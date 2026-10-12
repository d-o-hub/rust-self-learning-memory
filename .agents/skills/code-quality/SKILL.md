---
name: code-quality
description: "Maintain high code quality through formatting, linting, static analysis, and clean code principles. Use for rustfmt, clippy, cargo audit, code reviews, refactoring, and quality gates."
---

# Code Quality

Unified skill for Rust code quality and clean code development.

## Quick Commands

```bash
./scripts/code-quality.sh fmt                    # Format check
./scripts/code-quality.sh clippy --workspace     # Lint with clippy
./scripts/code-quality.sh audit                  # Security audit
./scripts/code-quality.sh check                  # Run all quality gates
./scripts/quality-gates.sh                       # Full gates (coverage floor 70%, target 90%)
```

## Quality Gates

| Check | Command | Target |
|-------|---------|--------|
| Format | `./scripts/code-quality.sh fmt` | 100% compliant |
| Lint | `./scripts/code-quality.sh clippy --workspace` | Zero warnings |
| Audit | `cargo audit` | No known vulnerabilities |
| Coverage | `cargo llvm-cov --html` | >=70% floor (90% target) |
| Docs | `cargo doc --no-deps` | All public APIs |

## Rust Quality Dimensions

| Dimension | Focus |
|-----------|-------|
| Structure | Files <500 LOC, clear modules |
| Error Handling | Custom Error, Result<T>, `?` operator (no unwrap) |
| Async | spawn_blocking for CPU work, no blocking in async |
| Testing | >=70% coverage floor (90% target), AAA pattern, cargo nextest + doctests |
| Security | Parameterized SQL, env vars, no hardcoded secrets |

## Clean Code Principles

**SOLID**: Single Responsibility, Open-Closed, Liskov Substitution, Interface Segregation, Dependency Inversion

**DRY/KISS/YAGNI**: Extract common code, simple solutions, build what's needed now

## Before Every Push

- Run `cargo fmt --all -- --check` as the **last** step, including after conflict
  resolution and after hand-indenting merged blocks. `tests/quality_gates.rs::quality_gate_formatting`
  re-runs the same check inside the Storage Matrix *and* Coverage jobs, so one
  formatting miss reds three jobs (LESSON-039).
- A crate-root `#[expect(lint, reason = "...")]` must be **fulfilled** in every
  configuration that compiles the crate; an expectation that never fires fails
  `cargo clippy --workspace --tests -- -D warnings` as
  `unfulfilled_lint_expectations`. Dead `cfg(test)` suppressions are the usual
  cause — delete them or narrow the `cfg_attr` (LESSON-037).
- A new defaulted `StorageBackend` method also needs a classification in
  `scripts/storage-backend-capabilities.toml` (see `feature-implement`).

## Anti-Patterns

| Bad | Fix |
|-----|-----|
| Excessive clone | Use borrowing or Arc |
| Unnecessary unwrap | Use `?` operator |
| Deep nesting | Extract methods |
| Large functions | Split (<50 LOC) |
| Deadlocks | Release locks before `.await` |
| `.py`/`.sh` in root | Move to `scripts/` or delete if one-off |

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
