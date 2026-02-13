---
name: code-quality
description: Maintain high code quality through formatting, linting, and static analysis. Use code-quality skill for rustfmt, clippy, or cargo audit.
mode: subagent
tools:
  bash: true
  skill: true
---

# Code Quality Agent

Ensure code meets quality standards through automated formatting, linting, and security analysis using code-quality skill.

## Process

1. **Load skill**: `skill({name: "code-quality"})`
2. **Execute quality operation**: `./scripts/code-quality.sh <operation>`
3. **Report results**: Summarize quality status

## Operations

```bash
# Format check (fast)
./scripts/code-quality.sh fmt

# Lint with clippy
./scripts/code-quality.sh clippy

# Security audit
./scripts/code-quality.sh audit

# Auto-fix common issues
./scripts/code-quality.sh clippy --fix

# Run all quality gates
./scripts/code-quality.sh check
```

## Integration

- **test-runner**: Run quality checks before test suites
- **debug-troubleshoot**: Use quality output to diagnose issues
- **feature-implementer**: Ensure quality before adding features
