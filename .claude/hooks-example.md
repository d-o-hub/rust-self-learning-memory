# Hooks Configuration Example

This file provides example hook configurations for Claude Code. Hooks automate quality checks and workflows.

## What are Hooks?

Hooks are shell commands that run automatically in response to events like:
- Tool usage (Read, Edit, Write, Bash, etc.)
- User prompt submission
- Agent completion

## Recommended Hooks for This Project

### Pre-Commit Quality Check

**Trigger**: Before creating a git commit
**Purpose**: Ensure code quality before committing

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test --all
```

**What it does**:
1. Formats all code with rustfmt
2. Runs clippy with all warnings treated as errors
3. Runs all tests (unit, integration, doc)

**Exit behavior**:
- If any step fails, commit is blocked
- Fix issues and try again

### Pre-Push Validation

**Trigger**: Before pushing to remote
**Purpose**: Full validation before sharing code

```bash
cargo build --release --all && cargo test --all --release && cargo doc --no-deps
```

**What it does**:
1. Builds release version of all crates
2. Runs all tests in release mode
3. Builds documentation

**Exit behavior**:
- Blocks push if any step fails
- Ensures pushed code is production-ready

### Quick Check After Edit

**Trigger**: After using Edit or Write tool
**Purpose**: Fast feedback on code changes

```bash
cargo check --all
```

**What it does**:
- Type checks without building binaries
- Fast feedback (5-10 seconds)

**Exit behavior**:
- Warning only (doesn't block)
- Shows compile errors immediately

### Format on Write

**Trigger**: After writing Rust files
**Purpose**: Auto-format all Rust code

```bash
cargo fmt
```

**What it does**:
- Automatically formats code
- Ensures consistent style

**Exit behavior**:
- Non-blocking
- Always succeeds

### Test After Implementation

**Trigger**: After significant code changes
**Purpose**: Verify tests still pass

```bash
cargo test --all -- --test-threads=4
```

**What it does**:
- Runs all tests in parallel
- Quick feedback on breakage

**Exit behavior**:
- Warning if tests fail
- Encourages fixing immediately

## How to Configure Hooks

Hooks are configured in Claude Code settings. The exact method depends on your Claude Code version:

### Option 1: Settings File

If using settings file (claude_code_settings.json):

```json
{
  "hooks": {
    "pre-commit": {
      "command": "cargo fmt && cargo clippy --all -- -D warnings && cargo test --all",
      "blocking": true
    },
    "pre-push": {
      "command": "cargo build --release --all && cargo test --all",
      "blocking": true
    },
    "tool-use-edit": {
      "command": "cargo check",
      "blocking": false
    }
  }
}
```

### Option 2: User Settings

If using UI settings:

1. Open Claude Code settings
2. Navigate to "Hooks" section
3. Add hook configurations:
   - **Name**: `pre-commit`
   - **Command**: `cargo fmt && cargo clippy --all -- -D warnings && cargo test --all`
   - **Blocking**: Yes
   - **Trigger**: Before git commit

## Hook Types

### Blocking Hooks
- Must succeed for operation to continue
- Use for: quality gates, validation
- Example: pre-commit, pre-push

### Non-Blocking Hooks
- Run but don't prevent operation
- Use for: notifications, logging
- Example: post-edit check

## Project-Specific Hooks

### Check File Size Limit

Ensure no files exceed 500 LOC:

```bash
find src -name "*.rs" -exec wc -l {} + | awk '$1 > 500 {print "File too large: " $2 " (" $1 " LOC)"; exit 1}'
```

### Check for Unwrap in Lib Code

Prevent unwrap() in library code:

```bash
! grep -r "\.unwrap()" src/lib.rs src/*/lib.rs
```

### Verify Database Migrations

Check migrations are up to date:

```bash
ls migrations/*.sql | sort | md5sum --check .migrations-checksum 2>/dev/null || true
```

### Check Test Coverage

Ensure minimum test coverage:

> **⚠️ DEPRECATION NOTICE**: This project has migrated from `cargo-tarpaulin` to `cargo-llvm-cov` for code coverage.
> If you have existing hooks or scripts using `cargo tarpaulin`, please update them to use `cargo llvm-cov` instead.
> The `tarpaulin.toml` configuration file has been removed as `cargo-llvm-cov` uses CLI flags for configuration.

```bash
# Check that total line coverage is at least 80%
cargo llvm-cov --summary-only 2>/dev/null | grep "TOTAL" | grep -oP '\d+\.\d+%' | head -1 | grep -oP '\d+\.\d+' | awk '{if ($1 < 80.0) exit 1}'
```

Alternative using `--json` output (more reliable):

```bash
# Requires jq installed
cargo llvm-cov --json --summary-only | jq -e '.data[0].totals.lines.percent >= 80'
```

**Migration from cargo-tarpaulin:**
- Replace `cargo tarpaulin` with `cargo llvm-cov`
- Remove references to `tarpaulin.toml` (no longer needed)
- Update `--out Html` to `--html --output-dir coverage`
- HTML report is now at `coverage/index.html` (not `tarpaulin-report.html`)

## Advanced Hooks

### Conditional Hook (Only for Rust Files)

```bash
#!/bin/bash
changed_files=$(git diff --cached --name-only)
if echo "$changed_files" | grep -q "\.rs$"; then
    cargo clippy --all -- -D warnings
fi
```

### Hook with Notifications

```bash
#!/bin/bash
cargo test --all
if [ $? -eq 0 ]; then
    echo "✅ All tests passed"
else
    echo "❌ Tests failed - please fix before committing"
    exit 1
fi
```

### Hook with Timing

```bash
#!/bin/bash
start=$(date +%s)
cargo test --all
end=$(date +%s)
echo "Tests completed in $((end - start)) seconds"
```

## Best Practices

1. **Keep hooks fast**: Slow hooks disrupt workflow
2. **Use blocking sparingly**: Only for critical checks
3. **Provide clear feedback**: Echo what's being checked
4. **Test hooks independently**: Run commands manually first
5. **Document hook purpose**: Clear comments in config

## Debugging Hooks

If a hook fails:

1. **Run command manually**:
   ```bash
   cargo clippy --all -- -D warnings
   ```

2. **Check hook output**:
   - Claude Code shows hook output
   - Look for specific errors

3. **Disable temporarily** if needed:
   - Comment out in settings
   - Fix underlying issue
   - Re-enable

## Hook Workflow Examples

### Example 1: Commit Blocked by Failing Test

```
User: Create commit with changes

Hook: pre-commit running...
  → cargo fmt ✓
  → cargo clippy ✓
  → cargo test ❌

  test_episode_creation ... FAILED

Hook failed! Commit blocked.

Claude: The pre-commit hook found a failing test. Let me fix it.
[Fixes test]
[Retries commit]

Hook: pre-commit running...
  → cargo fmt ✓
  → cargo clippy ✓
  → cargo test ✓

Commit created successfully.
```

### Example 2: Quick Feedback After Edit

```
Claude: [Edits src/storage.rs]

Hook: tool-use-edit running...
  → cargo check ✓

Claude: Changes look good.
```

### Example 3: Push Validation

```
User: Push changes to remote

Hook: pre-push running...
  → cargo build --release ✓
  → cargo test --all ✓
  → cargo doc ✓

Push allowed. All validation passed.
```

## Recommended Hook Set for This Project

Minimal (fast feedback):
```bash
# Edit hook
cargo check

# Commit hook
cargo clippy --all -- -D warnings && cargo test --all
```

Standard (balanced):
```bash
# Edit hook
cargo check

# Commit hook
cargo fmt && cargo clippy --all -- -D warnings && cargo test --all

# Push hook
cargo build --release && cargo test --all
```

Comprehensive (maximum quality):
```bash
# Edit hook
cargo check

# Commit hook
cargo fmt && cargo clippy --all -- -D warnings && cargo test --all

# Push hook
cargo build --release --all && \
cargo test --all --release && \
cargo doc --no-deps && \
cargo audit

# Additional checks
- File size < 500 LOC
- No unwrap() in lib code
- Test coverage > 80%
```

## Integration with CI

Hooks complement CI but don't replace it:

**Hooks**: Fast local checks before commit/push
**CI**: Comprehensive checks on all platforms

Keep hooks fast (<30s) to maintain productivity.

---

For more information on configuring hooks, see:
- Claude Code documentation
- Project AGENTS.md
- .claude/README.md
