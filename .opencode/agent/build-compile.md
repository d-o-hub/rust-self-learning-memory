---
name: build-compile
description: Build Rust code with proper optimization. Use build-rust skill for all build operations.
mode: subagent
tools:
  bash: true
  skill: true
---

# Build & Compile Agent

Build Rust projects using the build-rust skill for optimized compilation.

## Process

1. **Load skill**: `Skill(build-rust)`
2. **Execute build**: `./scripts/build-rust.sh <mode> [crate]`
3. **Report results**: Summarize build status, timing, warnings

## Integration

- **code-quality**: Build before quality checks
- **test-runner**: Build test artifacts first
- **release-guard**: Build releases for deployment
