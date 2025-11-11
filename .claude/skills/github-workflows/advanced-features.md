# Advanced GitHub Actions Features for Rust

## Release Automation

**For comprehensive release management, see [release-management.md](release-management.md).**

### Quick Release Example

```yaml
name: Release

on:
  push:
    tags:
      - 'v*.*.*'

permissions:
  contents: write

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Build release
        run: cargo build --release --all

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          generate_release_notes: true
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

**See [release-management.md](release-management.md) for:**
- Multi-platform builds
- Changelog generation
- Version bumping
- Crates.io publishing
- Pre-releases and rollbacks

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

## Performance Benchmarking

### Criterion Benchmarks with Regression Detection

```yaml
name: Performance Benchmarks

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  schedule:
    - cron: '0 0 * * 1'  # Weekly on Monday
  workflow_dispatch:

permissions:
  contents: write
  pull-requests: write

jobs:
  benchmark:
    name: Run Benchmarks
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
        with:
          fetch-depth: 0  # Need history for comparison

      - uses: dtolnay/rust-toolchain@stable

      - uses: Swatinem/rust-cache@v2

      - name: Run benchmarks
        run: |
          if timeout 300 cargo bench --quiet 2>/dev/null; then
            echo "Benchmarks completed"
          else
            echo "Benchmarks timed out"
          fi

      - name: Convert Criterion output
        run: |
          # Parse Criterion JSON and convert to bencher format
          find target/criterion -type f -path "*/new/estimates.json" | while read json; do
            bench_name=$(echo "$json" | sed 's|target/criterion/||' | sed 's|/new/estimates.json||')
            mean_ns=$(grep -A 3 '"mean"' "$json" | grep '"point_estimate"' | grep -o '[0-9.]*' | head -1 | cut -d. -f1)
            std_dev=$(grep -A 3 '"std_dev"' "$json" | grep '"point_estimate"' | grep -o '[0-9.]*' | head -1 | cut -d. -f1)
            if [ -n "$mean_ns" ]; then
              echo "test $bench_name ... bench: $mean_ns ns/iter (+/- $std_dev)"
            fi
          done | sort > bench_results.txt

      - name: Store benchmark result
        uses: benchmark-action/github-action-benchmark@v1
        if: github.ref == 'refs/heads/main'
        with:
          name: Rust Benchmarks
          tool: 'cargo'
          output-file-path: bench_results.txt
          github-token: ${{ secrets.GITHUB_TOKEN }}
          auto-push: true
          alert-threshold: '110%'  # Alert if 10% slower
          comment-on-alert: true
          fail-on-alert: false

      - name: Comment PR with results
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const results = fs.readFileSync('bench_results.txt', 'utf8');
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: \`## Performance Benchmark Results\n\n\`\`\`\n\${results}\n\`\`\`\`
            });

      - name: Upload benchmark artifacts
        uses: actions/upload-artifact@v4
        with:
          name: benchmark-results-${{ github.sha }}
          path: |
            bench_results.txt
            target/criterion/
          retention-days: 90
```

### Benchmark Best Practices

- Run on consistent hardware (same runner type)
- Use `timeout` to prevent hangs
- Store historical results for trend analysis
- Alert on regressions (>10% slower)
- Test on representative workloads
- Isolate benchmarks from tests

## Quality Gates

### Comprehensive Quality Enforcement

```yaml
name: Quality Gates

on:
  push:
    branches: [main, develop]
  pull_request:

env:
  RUST_BACKTRACE: 1
  CARGO_TERM_COLOR: always

jobs:
  quality-gates:
    name: Quality Gates
    runs-on: ubuntu-latest
    needs: [format, clippy, test, coverage, security-audit]
    steps:
      - uses: actions/checkout@v5

      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt, llvm-tools-preview

      - uses: Swatinem/rust-cache@v2

      - name: Install quality tools
        run: |
          cargo install cargo-llvm-cov
          cargo install cargo-audit

      - name: Run Quality Gates
        run: cargo test --test quality_gates -- --nocapture
        env:
          RUST_LOG: info
          QUALITY_GATE_COVERAGE_THRESHOLD: "90"
          QUALITY_GATE_PATTERN_ACCURACY_THRESHOLD: "70"
          QUALITY_GATE_COMPLEXITY_THRESHOLD: "10"
          QUALITY_GATE_SECURITY_THRESHOLD: "0"

      - name: Generate Quality Report
        if: always()
        run: |
          echo "# Quality Gates Report" > quality-report.md
          echo "" >> quality-report.md
          echo "**Date:** $(date)" >> quality-report.md
          echo "" >> quality-report.md
          echo "## Thresholds" >> quality-report.md
          echo "- Code Coverage: ≥90%" >> quality-report.md
          echo "- Pattern Accuracy: ≥70%" >> quality-report.md
          echo "- Max Complexity: ≤10" >> quality-report.md
          echo "- Security Vulnerabilities: 0" >> quality-report.md

      - name: Upload quality report
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: quality-gates-report
          path: quality-report.md
          retention-days: 30
```

### Quality Gate Metrics

Common quality metrics to enforce:

1. **Code Coverage**
   - Threshold: ≥90%
   - Tool: cargo-llvm-cov
   - Fail build if below threshold

2. **Code Complexity**
   - Threshold: Cyclomatic complexity ≤10
   - Tool: cargo-clippy (cognitive_complexity lint)
   - Warn on high complexity

3. **Security Vulnerabilities**
   - Threshold: 0 known vulnerabilities
   - Tool: cargo-audit, cargo-deny
   - Fail on any vulnerability

4. **Pattern Accuracy** (Project-specific)
   - Custom business logic metrics
   - Project-specific thresholds
   - Integration test validation

5. **Build Time**
   - Monitor build duration
   - Alert on significant increases
   - Track with `--timings`

### Quality Gate Implementation

```rust
// tests/quality_gates.rs
use std::env;

#[test]
fn coverage_threshold_check() {
    let threshold: f64 = env::var("QUALITY_GATE_COVERAGE_THRESHOLD")
        .unwrap_or_else(|_| "90".to_string())
        .parse()
        .expect("Invalid threshold");

    let coverage = get_current_coverage();
    assert!(
        coverage >= threshold,
        "Coverage {}% below threshold {}%",
        coverage,
        threshold
    );
}

#[test]
fn security_vulnerability_check() {
    let output = std::process::Command::new("cargo")
        .args(&["audit", "--json"])
        .output()
        .expect("Failed to run cargo audit");

    let vulnerabilities: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("Invalid JSON");

    let vuln_count = vulnerabilities["vulnerabilities"]["count"].as_u64().unwrap_or(0);
    assert_eq!(vuln_count, 0, "Found {} security vulnerabilities", vuln_count);
}
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
