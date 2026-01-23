# Quality Standards

## Code Quality Checklist

- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] >90% test coverage
- [ ] Documentation updated
- [ ] Examples included
- [ ] No unwrap() in production code
- [ ] Error messages are descriptive

## Rust Conventions

- Use `anyhow::Result` for public APIs
- Use `thiserror` for domain errors
- Use `Arc<Mutex<T>>` for shared state
- Use `tokio::spawn_blocking` for CPU-intensive work
- Use `instrument` for tracing

## File Organization

- Single responsibility per module
- Max 500 LOC per file
- Public APIs documented
- Tests colocated or in tests/ dir

## Pre-commit Checklist

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo audit
```

## Documentation Requirements

- Module-level docs ( //! )
- Public function docs ( /// )
- Examples in doc comments
- Update relevant .md files
