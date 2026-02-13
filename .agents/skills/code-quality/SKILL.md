---
name: code-quality
description: Maintain high code quality through formatting, linting, and static analysis. Use code-quality skill and scripts for rustfmt, clippy, or cargo audit.
---

# Code Quality

Use the code-quality scripts for all operations.

## Usage

```bash
# Format check (fast)
./scripts/code-quality.sh fmt

# Lint with clippy
./scripts/code-quality.sh clippy

# Security audit
./scripts/code-quality.sh audit

# Run all quality gates
./scripts/code-quality.sh check

# Auto-fix common issues
./scripts/code-quality.sh clippy --fix
```

## Quality Gates

| Check | Command |
|-------|---------|
| Format | `cargo fmt --all -- --check` |
| Lint | `cargo clippy --all -- -D warnings` |
| Audit | `cargo audit` |
| Full | `./scripts/quality-gates.sh` |
