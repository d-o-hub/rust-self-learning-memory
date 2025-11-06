---
name: github-workflows
description: Diagnose, fix, and optimize GitHub Actions workflows for Rust projects. Use when setting up CI/CD, troubleshooting workflow failures, optimizing build times with caching, or ensuring best practices for testing, linting, and releases.
---

# GitHub Workflows

Diagnose, fix, and optimize GitHub Actions workflows for Rust projects.

## Purpose
Set up robust CI/CD pipelines for Rust projects with proper caching, testing, linting, and release automation.

## Workflow Structure (2025 Best Practices)

### Complete Rust CI Workflow
```yaml
name: Rust CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  check:
    name: Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: ~/.cargo/registry/index
          key: ${{ runner.os }}-cargo-index-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-index-

      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: ~/.cargo/registry/cache
          key: ${{ runner.os }}-cargo-cache-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-cache-

      - name: Cache cargo build
        uses: actions/cache@v4
        with:
          path: target
          key: ${{ runner.os }}-target-check-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-target-check-
            ${{ runner.os }}-target-

      - name: Run cargo check
        run: cargo check --all --verbose

  fmt:
    name: Format
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt

      - name: Check formatting
        run: cargo fmt -- --check

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy

      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: ~/.cargo/registry/index
          key: ${{ runner.os }}-cargo-index-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-index-

      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: ~/.cargo/registry/cache
          key: ${{ runner.os }}-cargo-cache-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-cache-

      - name: Cache cargo build
        uses: actions/cache@v4
        with:
          path: target
          key: ${{ runner.os }}-target-clippy-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-target-clippy-
            ${{ runner.os }}-target-

      - name: Run clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

  test:
    name: Test
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}

      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: ~/.cargo/registry/index
          key: ${{ runner.os }}-cargo-index-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-index-

      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: ~/.cargo/registry/cache
          key: ${{ runner.os }}-cargo-cache-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-cache-

      - name: Cache cargo build
        uses: actions/cache@v4
        with:
          path: target
          key: ${{ matrix.os }}-${{ matrix.rust }}-target-test-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ matrix.os }}-${{ matrix.rust }}-target-test-
            ${{ matrix.os }}-${{ matrix.rust }}-target-

      - name: Run tests
        run: cargo test --all --verbose

      - name: Run tests with all features
        run: cargo test --all-features --verbose

  coverage:
    name: Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install cargo-llvm-cov
        run: cargo install cargo-llvm-cov

      - name: Generate coverage
        run: cargo llvm-cov --lcov --all-features --workspace --output-path lcov.info

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v4
        with:
          file: ./lcov.info
          fail_ci_if_error: false
```

## Caching Strategies (2025)

### Method 1: Manual Cache (Full Control)
```yaml
- name: Cache cargo registry index
  uses: actions/cache@v4
  with:
    path: ~/.cargo/registry/index
    key: ${{ runner.os }}-cargo-index-${{ hashFiles('**/Cargo.lock') }}
    restore-keys: |
      ${{ runner.os }}-cargo-index-
    save-always: true  # Important: save even on failure

- name: Cache cargo registry cache
  uses: actions/cache@v4
  with:
    path: ~/.cargo/registry/cache
    key: ${{ runner.os }}-cargo-cache-${{ hashFiles('**/Cargo.lock') }}
    restore-keys: |
      ${{ runner.os }}-cargo-cache-

- name: Cache cargo build
  uses: actions/cache@v4
  with:
    path: target
    key: ${{ runner.os }}-target-${{ hashFiles('**/Cargo.lock') }}-${{ hashFiles('**/*.rs') }}
    restore-keys: |
      ${{ runner.os }}-target-${{ hashFiles('**/Cargo.lock') }}-
      ${{ runner.os }}-target-
```

### Method 2: Rust-Cache Action (Automatic)
```yaml
- uses: actions/checkout@v4

- name: Install Rust
  uses: dtolnay/rust-toolchain@stable

- name: Cache Rust dependencies
  uses: Swatinem/rust-cache@v2
  with:
    shared-key: "stable"
    save-if: ${{ github.ref == 'refs/heads/main' }}

- name: Build
  run: cargo build --release
```

**rust-cache advantages**:
- Automatic cache key management
- Handles registry, git deps, and target/
- Cleans stale cache entries
- Per-job caching

### Method 3: sccache (Distributed Cache)
```yaml
- name: Install sccache
  run: |
    cargo install sccache --locked
    echo "RUSTC_WRAPPER=sccache" >> $GITHUB_ENV
    echo "SCCACHE_GHA_ENABLED=true" >> $GITHUB_ENV

- name: Run sccache-cache
  uses: mozilla-actions/sccache-action@v0.0.4

- name: Build
  run: cargo build --release

- name: Print sccache stats
  run: sccache --show-stats
```

## Cache Key Strategies

### Best Practices
```yaml
# Primary key: OS + Cargo.lock hash
key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

# With job name for isolation
key: ${{ runner.os }}-${{ github.job }}-${{ hashFiles('**/Cargo.lock') }}

# Include source hash for incremental builds
key: ${{ runner.os }}-target-${{ hashFiles('**/Cargo.lock') }}-${{ hashFiles('**/*.rs') }}

# Restore keys (fallback chain)
restore-keys: |
  ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}-
  ${{ runner.os }}-cargo-
```

### Cache Paths
```yaml
# Registry index
~/.cargo/registry/index

# Registry cache (downloaded .crate files)
~/.cargo/registry/cache

# Git dependencies
~/.cargo/git/db

# Build artifacts
target/

# Binary cache (cargo install)
~/.cargo/bin
```

## Build Matrix for Cross-Platform Testing

### Basic Matrix
```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
    rust: [stable, beta]
runs-on: ${{ matrix.os }}
```

### Advanced Matrix with Exclusions
```yaml
strategy:
  fail-fast: false
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
    rust: [stable, beta, nightly]
    exclude:
      - os: macos-latest
        rust: beta
      - os: windows-latest
        rust: nightly
    include:
      - os: ubuntu-latest
        rust: nightly
        experimental: true
runs-on: ${{ matrix.os }}
continue-on-error: ${{ matrix.experimental || false }}
```

### Platform-Specific Steps
```yaml
- name: Install dependencies (Ubuntu)
  if: runner.os == 'Linux'
  run: sudo apt-get update && sudo apt-get install -y libssl-dev

- name: Install dependencies (macOS)
  if: runner.os == 'macOS'
  run: brew install openssl

- name: Install dependencies (Windows)
  if: runner.os == 'Windows'
  run: choco install openssl
```

## Performance Optimizations

### 1. Parallel Jobs
```yaml
jobs:
  check:
    # Fast checks in parallel
  fmt:
    # Independent
  clippy:
    # Independent
  test:
    needs: [check]  # Only after check passes
```

### 2. Conditional Execution
```yaml
- name: Run expensive task
  if: github.event_name == 'push' && github.ref == 'refs/heads/main'
  run: cargo bench

- name: Skip on draft PRs
  if: github.event.pull_request.draft == false
  run: cargo test
```

### 3. Incremental Compilation
```yaml
env:
  CARGO_INCREMENTAL: 1  # Enable incremental compilation
  CARGO_PROFILE_DEV_DEBUG: 0  # Disable debug info for faster builds
```

### 4. Faster Linker (Linux)
```yaml
- name: Install mold linker
  if: runner.os == 'Linux'
  run: |
    sudo apt-get update
    sudo apt-get install -y mold
    echo 'RUSTFLAGS="-C link-arg=-fuse-ld=mold"' >> $GITHUB_ENV
```

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

## Release Automation

### Semantic Release with Cargo
```yaml
name: Release

on:
  push:
    tags:
      - 'v*.*.*'

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Build release
        run: cargo build --release --all

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            target/release/memory-core
            target/release/memory-storage-turso
          generate_release_notes: true
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### Multi-Platform Releases
```yaml
strategy:
  matrix:
    include:
      - os: ubuntu-latest
        target: x86_64-unknown-linux-gnu
      - os: macos-latest
        target: x86_64-apple-darwin
      - os: windows-latest
        target: x86_64-pc-windows-msvc

steps:
  - uses: actions/checkout@v4

  - name: Install Rust
    uses: dtolnay/rust-toolchain@stable
    with:
      targets: ${{ matrix.target }}

  - name: Build
    run: cargo build --release --target ${{ matrix.target }}

  - name: Package (Unix)
    if: runner.os != 'Windows'
    run: tar czf binary-${{ matrix.target }}.tar.gz -C target/${{ matrix.target }}/release binary

  - name: Package (Windows)
    if: runner.os == 'Windows'
    run: Compress-Archive target/${{ matrix.target }}/release/binary.exe binary-${{ matrix.target }}.zip

  - name: Upload artifact
    uses: actions/upload-artifact@v4
    with:
      name: binary-${{ matrix.target }}
      path: binary-${{ matrix.target }}.*
```

## Code Coverage

### Using cargo-llvm-cov (Recommended)
```yaml
- name: Install llvm-cov
  run: cargo install cargo-llvm-cov

- name: Generate coverage
  run: cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

- name: Upload to Codecov
  uses: codecov/codecov-action@v4
  with:
    files: lcov.info
    token: ${{ secrets.CODECOV_TOKEN }}
```

### Alternative: Generate multiple formats
```yaml
- name: Install llvm-cov
  run: cargo install cargo-llvm-cov

- name: Generate coverage (HTML + LCOV)
  run: |
    cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
    cargo llvm-cov --all-features --workspace --html --output-dir coverage

- name: Upload to Codecov
  uses: codecov/codecov-action@v4
  with:
    files: lcov.info
```

## Security Scanning

### Cargo Audit
```yaml
- name: Security audit
  run: |
    cargo install cargo-audit
    cargo audit
```

### Dependency Review
```yaml
- name: Dependency Review
  uses: actions/dependency-review-action@v4
  if: github.event_name == 'pull_request'
```

## Documentation Deployment

### Deploy to GitHub Pages
```yaml
deploy-docs:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4

    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable

    - name: Build docs
      run: cargo doc --no-deps --all-features

    - name: Add index redirect
      run: echo '<meta http-equiv="refresh" content="0; url=memory_core">' > target/doc/index.html

    - name: Deploy to GitHub Pages
      uses: peaceiris/actions-gh-pages@v3
      with:
        github_token: ${{ secrets.GITHUB_TOKEN }}
        publish_dir: ./target/doc
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

## Workflow Templates

### Minimal CI (Quick Feedback)
```yaml
name: Quick CI

on: [push, pull_request]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo check --all
      - run: cargo fmt -- --check
      - run: cargo clippy -- -D warnings
      - run: cargo test --all
```

### Comprehensive CI (Production)
See "Complete Rust CI Workflow" section above.

## Best Practices Summary

### DO:
- Use `actions/cache@v4` with `save-always: true`
- Use `hashFiles('**/Cargo.lock')` for cache keys
- Implement `restore-keys` for cache fallback
- Use `dtolnay/rust-toolchain` instead of actions-rs
- Split large caches to avoid 2GB limit
- Test on multiple platforms (matrix)
- Use `Swatinem/rust-cache@v2` for simplicity
- Cache both registry and target directory
- Set `CARGO_TERM_COLOR: always` for readable logs
- Use `continue-on-error` for experimental builds

### DON'T:
- Use deprecated `actions-rs/*` actions
- Create monolithic cache entries >2GB
- Cache without `restore-keys`
- Forget `save-always: true` for partial builds
- Cache `target/` across different jobs without unique keys
- Run expensive operations on every PR
- Use `actions/cache@v3` and `@v4` inconsistently
- Hardcode Rust version (use rust-toolchain file)

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

## Integration with Project

For the rust-self-learning-memory project:

```yaml
name: Self-Learning Memory CI

on:
  push:
    branches: [main]
  pull_request:

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo check --all --verbose

  fmt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt -- --check

  clippy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo clippy --all-targets -- -D warnings

  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --all --verbose
      - run: cargo test --all --all-features --verbose
```

This workflow ensures all memory-core, memory-storage-turso, and memory-storage-redb crates are properly tested.
