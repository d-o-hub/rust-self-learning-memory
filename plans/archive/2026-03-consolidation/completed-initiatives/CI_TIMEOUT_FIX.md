# CI Cross-Platform Timeout Fix

**Date**: 2026-01-20
**Commit**: f8849a3
**Issue**: CI failures on macOS and timeout protection issues

## Root Cause Analysis

### Issue 1: macOS Timeout Command (Exit Code 127)
The original CI workflow used a perl-based timeout fallback for macOS:
```yaml
perl -e 'alarm 600; exec @ARGV' cargo test --lib --all || exit $?
```

**Problem**: The `@ARGV` in perl doesn't receive shell arguments because there's no `--` to separate perl arguments from command arguments. This caused "command not found" (exit code 127).

### Issue 2: Invalid nextest Arguments (Exit Code 2)
The test job used `--timeout` flags directly with cargo nextest:
```yaml
cargo nextest run --profile ci --lib --all --timeout 300
```

**Problem**: `--timeout` is not a valid cargo nextest CLI argument. The timeout should be configured in `.config/nextest.toml` (which it was).

### Issue 3: Missing junit Output Directory
The test job tried to upload artifacts from `target/nextest/ci/junit.xml` without ensuring the directory existed.

## Fixes Applied

### Fix 1: Cross-Platform Timeout Wrapper
```yaml
- name: Run tests on ${{ matrix.os }}
  run: |
    # Cross-platform timeout wrapper
    # macOS has `gtimeout` from coreutils, or use perl as fallback
    if command -v timeout >/dev/null 2>&1; then
      # Linux with GNU timeout
      timeout 600s cargo test --lib --all
    elif command -v gtimeout >/dev/null 2>&1; then
      # macOS with GNU timeout (coreutils)
      gtimeout 600s cargo test --lib --all
    else
      # macOS fallback: use perl for portable timeout
      # Use -- to separate perl args from command args
      perl -e 'alarm 600; exec @ARGV' -- cargo test --lib --all
    fi
```

**Changes**:
1. Added `gtimeout` detection for macOS with coreutils installed
2. Fixed perl command with `--` to properly pass arguments
3. Added `s` suffix for timeout values (POSIX compliance)

### Fix 2: Removed Invalid nextest Arguments
```yaml
- name: Run tests with timeout protection
  run: |
    # Ensure junit output directory exists
    mkdir -p target/nextest/ci
    # Run unit tests first (faster feedback) - nextest uses slow-timeout from config
    cargo nextest run --profile ci --lib --all
    # Run integration tests if unit tests pass
    cargo nextest run --profile ci --tests --all
```

**Changes**:
1. Removed invalid `--timeout` CLI arguments
2. Added `mkdir -p target/nextest/ci` to ensure output directory exists
3. The timeout is now handled by `.config/nextest.toml` profile configuration

### Fix 3: MCP Build Timeout Handling
```yaml
timeout 300s cargo build -p memory-mcp || exit 1
timeout 300s cargo test -p memory-mcp --lib || exit 1
```

**Changes**:
1. Added `s` suffix for timeout values
2. Added explicit `|| exit 1` for proper error propagation

## Nextest Configuration

The timeout is now properly configured in `.config/nextest.toml`:

```toml
[profile.ci]
retries = 2
fail-fast = false
slow-timeout = { period = "60s", terminate-after = 5 }
test-threads = 4
```

## Verification Steps

1. Push changes to trigger CI
2. Verify all jobs pass:
   - Essential Checks (format, clippy, doctest)
   - Tests (with timeout protection)
   - MCP Build (default, wasm-rquickjs)
   - Multi-Platform Test (ubuntu-latest, macos-latest)
   - Quality Gates
3. Check junit.xml artifacts are uploaded correctly

## Future Considerations

1. Consider using GitHub Actions `timeout-minutes` at job level as primary timeout
2. Add explicit platform detection in workflow for better maintainability
3. Consider using `taiki-e/timeout-action` for more reliable cross-platform timeouts
4. Document timeout strategy in CI documentation
