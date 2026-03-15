# Common Friction Points

This document captures recurring friction patterns from Claude Code sessions and provides prevention strategies.

## Session Analysis Summary

Based on analysis of 34 sessions (234 messages, 97 commits):

| Friction Type | Count | Impact |
|---------------|-------|--------|
| wrong_approach | 8 | High - requires rework |
| buggy_code | 6 | Medium - fixes needed |
| excessive_changes | 5 | Medium - review overhead |
| tool_errors | 67 | Low - mostly recoverable |

## Primary Friction Patterns

### 1. Wrong Approach (8 occurrences)

**Symptoms**: Agent implements solution that doesn't fit existing patterns or architecture.

**Root Cause**: Insufficient reading of existing code before implementation.

**Prevention**:
```
1. Read existing patterns in target module
2. Check for similar implementations elsewhere
3. Verify approach aligns with ADRs
4. Consider requesting clarification if uncertain
```

**Quick Check**:
- [ ] Read at least 3 related source files
- [ ] Identified existing patterns to follow
- [ ] Checked ADRs for relevant decisions

### 2. Buggy Code (6 occurrences)

**Symptoms**: Code compiles but fails tests or behaves incorrectly.

**Root Cause**: Insufficient testing before committing.

**Prevention**:
```
1. Write tests first (TDD approach)
2. Run cargo nextest run -p <crate> after changes
3. Run cargo nextest run --all before commit
4. Check edge cases explicitly
```

**Quick Check**:
- [ ] Tests pass for affected crate
- [ ] Tests pass for entire workspace
- [ ] Edge cases considered

### 3. Excessive Changes (5 occurrences)

**Symptoms**: Large commits with multiple unrelated changes.

**Root Cause**: Not separating logical changes into atomic commits.

**Prevention**:
```
1. One logical change per commit
2. Commit message describes exactly what changed
3. git diff --stat before committing
4. Split large changes into series of commits
```

**Quick Check**:
- [ ] Single logical change
- [ ] Commit message is specific
- [ ] git diff --stat shows focused changes

## Tool Usage Patterns

### Current State

| Tool | Usage | Target |
|------|-------|--------|
| Bash | 1892 | Lower |
| Grep | 110 | Higher |
| Ratio | 17:1 | 2:1 |

### Recommended Tool Selection

**Use Grep for**:
- Finding files by pattern: `Grep pattern="pattern"`
- Searching content: `Grep pattern="search_term"`
- Finding definitions: `Grep pattern="fn name"`
- Checking usage: `Grep pattern="use module"`

**Use Bash for**:
- File operations: `cp`, `mv`, `rm`
- Git commands: `git status`, `git diff`
- Running scripts: `./scripts/build-rust.sh`
- Build/test: `cargo build`, `cargo test`

### Tool Error Prevention

Common tool errors and fixes:

| Error | Cause | Fix |
|-------|-------|-----|
| Permission denied | Missing execute permission | `chmod +x script.sh` |
| File not found | Wrong path | Use absolute paths |
| Timeout | Long-running operation | Increase timeout or use background |
| JSON parse error | Malformed output | Use `2>&1` to capture stderr |

## CI/CD Friction Points

### GitHub Actions Version Issues

**Problem**: `wait-on-check-action@v1.5.0` deprecated, causes failures.

**Solution**: Use `fountainhead/action-wait-for-check@v2.0.0`.

```yaml
# Correct
- uses: fountainhead/action-wait-for-check@v2.0.0
  with:
    checkName: ci-check

# Avoid (deprecated)
- uses: fountainhead/action-wait-for-check@v1.5.0
```

### --all-features Libclang Dependency

**Problem**: Building with `--all-features` requires libclang for some crates.

**Solution**: Use feature-specific builds or workspace excludes.

```bash
# Avoid in CI
cargo build --all-features

# Use instead
cargo build --features "turso,redb"

# Or exclude problematic crates
[workspace]
exclude = ["crates/wasm-sandbox"]
```

### Network-Dependent Test Flakiness

**Problem**: Integration tests requiring TURSO_DATABASE_URL fail in CI.

**Solution**: Use serial_test or retry logic.

```rust
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_network_dependent() {
    // Test with retry logic for network issues
}
```

### Clippy Lint Handling

**Problem**: Test-only lints need allow-list propagation to integration test crates.

**Solution**: Add crate-level allow attributes.

```rust
// At top of integration test file
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
```

## Quick Reference

### Before Starting Task

1. [ ] Read AGENTS.md
2. [ ] Check relevant ADRs
3. [ ] Read existing patterns

### During Implementation

1. [ ] Follow existing patterns
2. [ ] Write tests first
3. [ ] Run tests frequently

### Before Commit

1. [ ] `cargo fmt --all`
2. [ ] `cargo clippy --all -- -D warnings`
3. [ ] `cargo nextest run --all`
4. [ ] `git status` verification

### Atomic Commits

1. [ ] Single logical change
2. [ ] Descriptive commit message
3. [ ] All tests pass
4. [ ] No unrelated changes

## Cross-References

- [AGENTS.md](../AGENTS.md) - Primary coding guidelines
- [github_actions_patterns.md](github_actions_patterns.md) - CI/CD patterns
- [git_workflow.md](git_workflow.md) - Git workflow details
- [running_tests.md](running_tests.md) - Testing guidance
- [token_efficiency.md](token_efficiency.md) - Tool selection guidance