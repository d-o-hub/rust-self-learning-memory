---
name: github-workflows
description: "Diagnose, fix, and optimize GitHub Actions workflows for Rust projects. Use when setting up CI/CD, troubleshooting workflow failures, optimizing build times, or ensuring best practices."
---

# GitHub Workflows

Diagnose, fix, and optimize GitHub Actions workflows for Rust projects.

## Quick Reference

- **[Caching Strategies](caching-strategies.md)** - Manual cache, rust-cache, sccache
- **[Troubleshooting](troubleshooting.md)** - Common issues, debugging, fixes
- **[Advanced Features](advanced-features.md)** - Coverage, security, benchmarking
- **[Release Management](release-management.md)** - Automated releases, versioning

## When to Use

- Setting up CI/CD for Rust projects
- Troubleshooting workflow failures
- Optimizing build times with caching
- Ensuring best practices for testing, linting, releases

## Before Making Changes

**ALWAYS verify current state first:**

```bash
# Get repo info
gh repo view --json nameWithOwner,owner,name

# List existing workflows
gh workflow list

# Check recent runs
gh run list --limit 10

# View workflow files
ls -la .github/workflows/
```

## Complete Rust CI Workflow

See the full workflow template with:
- Check job (format, clippy, check)
- Test job (unit, integration, doc tests)
- Coverage job (tarpaulin, codecov)
- Audit job (security, licenses)

See linked files for caching strategies, troubleshooting, and release management.

## Core Workflow Components

| Job | Purpose | Tools |
|-----|---------|-------|
| check | Code quality | rustfmt, clippy, cargo check |
| test | Verification | cargo test |
| coverage | Test metrics | cargo tarpaulin |
| audit | Security | cargo audit, deny |

## Common Patterns

- **Caching**: Dependencies, target directory, sccache
- **Matrix builds**: Multiple Rust versions, targets
- **Conditional jobs**: Skip on docs-only changes
- **Quality gates**: Block merge on failures

## Toolchain Pinning Guardrail

- Never float a toolchain in a workflow that gates work or runs on a schedule.
  `nightly` broke a transitive dependency (`branches 0.4.5`, `core::intrinsics`)
  with no repository change and failed every fuzz target as a startup failure;
  floating `stable` similarly adds deny-by-default lints between releases.
- Pin a dated channel (`nightly-2026-09-26`) in **both** the job env and the
  `dtolnay/rust-toolchain` action input, comment why, and validate the pin by
  building the affected crate with that toolchain before pushing (LESSON-040).

## PR Check Attachment Guardrail

- After pushing fixes, validate checks are attached to the PR head SHA:
  - `gh pr view --json statusCheckRollup,mergeStateStatus`
  - `gh run list --commit <head_sha>`
- If rollup is empty for required checks, treat as blocked and investigate trigger/path conditions.
