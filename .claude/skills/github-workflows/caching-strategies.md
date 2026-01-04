# GitHub Actions Caching Strategies for Rust

## Caching Methods (2025)

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
- uses: actions/checkout@v5

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

## Performance Tips

1. **Use `save-always: true`** (actions/cache@v4) - Saves cache even on job failure
2. **Split large caches** - Avoid single caches >2GB (GitHub limit)
3. **Use restore-keys** - Provides fallback cache restoration
4. **Cache per job** - Include job name in cache key for isolation
5. **Clean incrementally** - Use rust-cache for automatic cleanup
