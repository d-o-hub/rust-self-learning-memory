# GitHub Actions Troubleshooting for Rust

## Common Issues and Fixes

### Issue 1: Cache Not Saved on Failure

**Problem**: Cache is not saved when job fails.

**Solution**: Use `save-always: true` (actions/cache@v4)

```yaml
- uses: actions/cache@v4
  with:
    path: target
    key: ${{ runner.os }}-target-${{ hashFiles('**/Cargo.lock') }}
    save-always: true  # Save even on job failure
```

### Issue 2: Cache Key Mismatch

**Problem**: Cache keys don't match, no restoration.

**Solution**: Use `hashFiles()` and `restore-keys`

```yaml
- uses: actions/cache@v4
  with:
    path: ~/.cargo
    key: cargo-${{ hashFiles('**/Cargo.lock') }}
    restore-keys: |
      cargo-${{ hashFiles('**/Cargo.lock') }}-
      cargo-
```

### Issue 3: actions/cache v3 vs v4 Compatibility

**Problem**: Cache created with v3 might not restore in v4.

**Solution**: Use consistent version across all jobs

```yaml
# All jobs use same version
- uses: actions/cache@v4
```

### Issue 4: Deprecated actions-rs

**Problem**: `actions-rs/toolchain` and `actions-rs/cargo` are deprecated.

**Solution**: Use modern alternatives

```yaml
# OLD (deprecated)
- uses: actions-rs/toolchain@v1
  with:
    toolchain: stable

# NEW (recommended)
- uses: dtolnay/rust-toolchain@stable

# Or with specific version
- uses: dtolnay/rust-toolchain@master
  with:
    toolchain: 1.75.0
```

### Issue 5: tar Creation Errors on ubuntu-latest

**Problem**:
```
tar: target/debug/deps: file changed as we read it
```

**Solution**: Exclude build artifacts or use rust-cache

```yaml
# Option 1: Exclude problematic paths
- uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry/index
      ~/.cargo/registry/cache
      ~/.cargo/git/db
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

# Option 2: Use rust-cache (handles this automatically)
- uses: Swatinem/rust-cache@v2
```

### Issue 6: Large Files (>2GB) Download Failure

**Problem**: Cache restoration fails for files >2GB.

**Solution**: Split cache or exclude large files

```yaml
# Split into smaller caches
- uses: actions/cache@v4
  with:
    path: ~/.cargo/registry/index
    key: cargo-index-${{ hashFiles('**/Cargo.lock') }}

- uses: actions/cache@v4
  with:
    path: ~/.cargo/registry/cache
    key: cargo-cache-${{ hashFiles('**/Cargo.lock') }}

- uses: actions/cache@v4
  with:
    path: target
    key: target-${{ hashFiles('**/Cargo.lock') }}
```

### Issue 7: Workflow Permissions

**Problem**: Workflow can't write to cache or create releases.

**Solution**: Set permissions

```yaml
permissions:
  contents: write  # For releases
  packages: write  # For container registry
  actions: write   # For cache

jobs:
  build:
    # ...
```

### Issue 8: Flaky Tests in CI

**Problem**: Tests pass locally but fail in CI.

**Solution**: Add retries and debugging

```yaml
- name: Run tests with retry
  uses: nick-fields/retry@v2
  with:
    timeout_minutes: 10
    max_attempts: 3
    command: cargo test --all

- name: Run tests with backtrace
  run: RUST_BACKTRACE=full cargo test --all -- --nocapture
```

## Debugging Workflows

### Enable Debug Logging

```yaml
env:
  ACTIONS_STEP_DEBUG: true
  ACTIONS_RUNNER_DEBUG: true
```

### Workflow Debug Commands

```yaml
- name: Debug info
  run: |
    echo "Event: ${{ github.event_name }}"
    echo "Ref: ${{ github.ref }}"
    echo "SHA: ${{ github.sha }}"
    echo "Actor: ${{ github.actor }}"
    rustc --version
    cargo --version
```

### Interactive Debugging

```yaml
- name: Setup tmate session
  if: failure()
  uses: mxschmitt/action-tmate@v3
  timeout-minutes: 30
```

## Troubleshooting Checklist

When workflow fails:
1. Check Actions tab for error messages
2. Look for cache restoration logs
3. Verify cache key matches between save/restore
4. Check `hashFiles()` is evaluating correctly
5. Ensure Rust version compatibility
6. Review recent GitHub Actions updates
7. Test locally with `act` tool
8. Enable debug logging if needed
9. Check for concurrent workflow limits
10. Verify permissions are sufficient

## Monitoring and Optimization

### Track Build Times

```yaml
- name: Build with timing
  run: cargo build --release --timings

- name: Upload timing report
  uses: actions/upload-artifact@v4
  with:
    name: cargo-timing
    path: target/cargo-timings/
```

### Cache Hit Rate

Check workflow logs for:
```
Cache restored from key: cargo-ubuntu-latest-abc123
Cache hit: true
```

### Optimize Based on Metrics

- High cache miss rate → improve cache keys
- Long build times → add sccache or split jobs
- Frequent failures → add retries or fix flaky tests
- Large cache size → split into smaller caches
