# Advanced GitHub Actions Features for Rust

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

### Using cargo-tarpaulin

```yaml
- name: Install tarpaulin
  run: cargo install cargo-tarpaulin

- name: Generate coverage
  run: |
    cargo tarpaulin \
      --out xml \
      --output-dir ./coverage \
      --all-features \
      --workspace \
      --timeout 300

- name: Upload to Codecov
  uses: codecov/codecov-action@v4
  with:
    files: ./coverage/cobertura.xml
    token: ${{ secrets.CODECOV_TOKEN }}
```

### Using cargo-llvm-cov

```yaml
- name: Install llvm-cov
  run: cargo install cargo-llvm-cov

- name: Generate coverage
  run: cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

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
