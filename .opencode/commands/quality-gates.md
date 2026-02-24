---
description: Run full quality gates validation for Rust workspace. Use before commits to ensure all checks pass.
subtask: true
---

# Quality Gates Command

Run comprehensive quality checks before commits or CI verification.

## Quality Gates Checklist

```bash
echo "=== Quality Gates ==="
echo "[1/4] Formatting check..."
cargo fmt --all -- --check || { echo "FAIL: fmt"; exit 1; }

echo "[2/4] Clippy linting..."
cargo clippy --all -- -D warnings || { echo "FAIL: clippy"; exit 1; }

echo "[3/4] Building..."
cargo build --all || { echo "FAIL: build"; exit 1; }

echo "[4/4] Testing..."
cargo test --all || { echo "FAIL: tests"; exit 1; }

echo "=== All Gates Passed ==="
```

## Gate Details

### 1. Formatting (fmt)
- Runs: `cargo fmt --all -- --check`
- Purpose: Ensure consistent code style
- Auto-fix: `cargo fmt --all`

### 2. Linting (clippy)
- Runs: `cargo clippy --all -- -D warnings`
- Purpose: Catch common mistakes and style issues
- Auto-fix: `cargo clippy --fix`

### 3. Build
- Runs: `cargo build --all`
- Purpose: Verify code compiles
- Note: Uses workspace exclude for optional deps

### 4. Tests
- Runs: `cargo test --all`
- Purpose: Verify all tests pass

## Common Issues

### libclang Optional Dependency
```
FAIL: build
```
**Fix**: Use `--exclude memory-mcp` or build without wasm features

### New Clippy Warnings
```
warning: new lint name
```
**Fix**: Add `#[allow(lint_name)]` or fix the code

### Non-ASCII Formatting
```
error: non-ASCII formatting detected
```
**Fix**: Check file encoding, use ASCII-compatible formatting

## Pre-Commit Integration

Run this command before every commit:

```bash
./.claude/commands/quality-gates
```

Or use the PreCommit hook (if configured in `.claude/hooks/hooks.json`).

## CI Verification

This mirrors the CI quality gates from `.github/workflows/`:
- Format check matches CI `fmt` job
- Clippy check matches CI `clippy` job
- Build check matches CI `check` job
- Test check matches CI `test` job

## Exit Codes

- `0`: All gates passed
- `1`: First failing gate (check output for details)

## Related

- `./scripts/quality-gates.sh` - Full script version with coverage
- `.github/workflows/` - CI configuration
