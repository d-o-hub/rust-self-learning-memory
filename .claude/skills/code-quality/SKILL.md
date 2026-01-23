---
name: code-quality
description: Maintain high code quality through formatting, linting, and static analysis using rustfmt, clippy, and cargo audit. Use to ensure consistent code style and catch common mistakes.
---

# Code Quality

Maintain high code quality through formatting, linting, and static analysis.

## Core Tools

### 1. Rustfmt (Formatting)

```bash
# Format all code
cargo fmt

# Check without changing
cargo fmt -- --check
```

### 2. Clippy (Linting)

```bash
# Run all lints
cargo clippy

# Treat warnings as errors (CI)
cargo clippy --all -- -D warnings

# Fix automatically
cargo clippy --fix
```

### 3. Cargo Audit (Security)

```bash
cargo audit
cargo deny check
```

## Quality Checklist

### Before Commit
- [ ] `cargo fmt` - Format code
- [ ] `cargo clippy -- -D warnings` - No lint warnings
- [ ] `cargo test --all` - All tests pass

### Before PR
- [ ] `cargo build --release` - Release build works
- [ ] `cargo doc --no-deps` - Documentation builds
- [ ] `cargo audit` - No security issues

## Common Issues

| Issue | Warning | Fix |
|-------|---------|-----|
| Unused imports | `unused import: HashMap` | Remove import |
| Unnecessary clone | `clone on Copy type` | Remove clone |
| Missing await | `unwrap on Result` | Use `?` |
| Large stack | `size difference variants` | Box large variants |

## Code Organization

- Files ≤ 500 LOC
- Functions < 50 LOC
- Single responsibility per module
