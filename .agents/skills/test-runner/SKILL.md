---
name: test-runner
description: Execute Rust tests (unit, integration, doc). Use cargo nextest for fast parallel execution.
---

# Test Runner

Execute and manage Rust tests for the self-learning memory project.

## Commands

```bash
# Unit tests (fast, <30s)
cargo nextest run --lib

# Integration tests
cargo nextest run --test '*'

# Full suite
cargo nextest run --all
cargo test --doc  # doctests (nextest unsupported)

# Quality gates (coverage threshold 90%)
./scripts/quality-gates.sh
```

## Test Categories

| Category | Command | Scope |
|----------|---------|-------|
| Unit | `cargo nextest run --lib` | Individual functions |
| Integration | `cargo nextest run --test '*'` | End-to-end workflows |
| Doc | `cargo test --doc` | Documentation examples |
| Mutation | `cargo mutants -p do-memory-core` | Test effectiveness |

## Best Practices

- **Isolation**: Each test independent
- **AAA pattern**: Arrange-Act-Assert
- **Single responsibility**: One behavior per test
- **Naming**: `test_<function>_<scenario>_<expected>`
- **Speed**: <1s per unit test

## Async Testing

```rust
#[tokio::test]
async fn test_episode() { let result = async_fn().await; }

#[tokio::test(start_paused = true)]
async fn test_timeout() { /* paused clock */ }

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent() { /* parallel ops */ }
```

| Bad | Good |
|-----|------|
| `std::thread::sleep()` | `tokio::time::sleep().await` |
| Missing `.await` | Always await async |

## Troubleshooting

| Issue | Fix |
|-------|-----|
| Race conditions | `cargo test -- --test-threads=1` |
| redb lock errors | Separate DB per test |
| DB connection refused | Check TURSO_URL/TURSO_TOKEN |

## Coverage

```bash
cargo llvm-cov --html --output-dir coverage
```

## References

- ADR-033: Modern Testing Strategy

Related skills: `test-patterns` for test design, `test-optimization` for performance-focused test work, and `test-fix` for failing-suite diagnosis.
