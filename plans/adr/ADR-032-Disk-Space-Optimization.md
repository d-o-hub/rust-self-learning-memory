# ADR-032: Disk Space Optimization Strategy

- **Status**: Partially Implemented (Phase 1 & 2 Complete, Phase 7 New)
- **Date**: 2026-02-21 (Updated 2026-03-09)
- **Deciders**: Project maintainers
- **Supersedes**: None
- **Related**: ADR-029 (GitHub Actions Modernization), ADR-030 (Test Optimization)

## Context

The project's `target/` directory consumes **74 GB** (as of 2026-03-09):

| Directory | Size | Issue |
|-----------|------|-------|
| `target/debug/incremental/` | **38 GB** | Incremental cache bloat |
| `target/debug/deps/` | **27 GB** | 384K files (stale artifacts) |
| `target/llvm-cov-target/` | **7.8 GB** | Coverage artifacts not cleaned |
| `target/release/` | **1.8 GB** | Release build |
| `target/debug/examples/` | **1.9 GB** | Example binaries |
| `target/debug/build/` | **302 MB** | Build scripts |

**Previous baseline (2026-02-21)**: 5.2 GB — **14x growth in 2 weeks**

Root causes:
1. **Full debug info** (`debug = true`) in dev profile generates ~2x artifact size
2. **No debug info stripping** for dependency crates
3. **No split-debuginfo** configured (monolithic DWARF)
4. **No fast linker** (using default `ld`, not `mold` or `lld`)
5. **134 duplicate dependency roots** across 863 packages inflating compiled output
6. ~~Orphaned `node_modules/`~~ ✅ Removed
7. **No incremental build GC** strategy
8. **Coverage artifacts not cleaned** after CI runs
9. **384K files in deps/** indicating stale artifact accumulation

## Decision

### Phase 1: Build Profile Optimization (Immediate)

Update `.cargo/config.toml` and `Cargo.toml`:

```toml
# .cargo/config.toml
[profile.dev]
debug = "line-tables-only"    # ~60% smaller debug artifacts
split-debuginfo = "unpacked"  # Separate debug info files

[profile.dev.package."*"]
debug = false                 # No debug info for dependencies

[profile.test]
debug = "line-tables-only"

[profile.debugging]
inherits = "dev"
debug = true                  # Full debug when needed: --profile debugging
```

**Expected savings**: 2-3 GB reduction in target/ size.

### Phase 2: Linker Optimization (Complete)

✅ **mold linker is now installed and configured**

Configuration in `.cargo/config.toml`:

```toml
# .cargo/config.toml
[target.'cfg(target_os = "linux")']
rustflags = ["-C", "link-arg=-fuse-ld=mold", "-C", "link-arg=-Wl,-z,relro,-z,now"]
```

**Achievement**: 2-5x faster link times, indirectly reduces incremental cache size.

### Phase 3: Dependency Deduplication (Medium-term)

See ADR-036 for full dependency audit plan. Key actions:
- Audit and remove unused dependencies with `cargo-machete`/`cargo-shear`
- Remove unused features with `cargo-unused-features`
- Track major duplicate groups: `rand` (4 versions), `hashbrown` (5 versions), `getrandom` (4 versions)

### Phase 4: Proc-Macro Build Optimization

```toml
# Cargo.toml - optimize proc-macro compilation
[profile.dev.build-override]
opt-level = 3    # Proc-macros run at build time; optimizing them speeds builds
```

### Phase 5: CI Disk Management

- Use `CARGO_TARGET_DIR` in tmpfs for CI builds
- Add `cargo clean --release` step after artifact packaging
- Standardize disk-free scripts across all workflows

### Phase 6: Cleanup

- ✅ Remove orphaned `node_modules/` directory (add to `.gitignore` if not already)
- Add `scripts/clean-artifacts.sh` for periodic local cleanup
- Document `cargo clean-all` usage for developers

### Phase 7: Aggressive Target Cleanup (NEW - 2026-03-09)

**Critical**: Current target/ size is **74 GB** — 14x the baseline from 2 weeks ago.

#### Immediate Actions

```bash
# Full clean (recommended - recovers 74GB)
cargo clean

# Or selective cleanup (preserves some build cache):
rm -rf target/debug/incremental/     # 38GB - incremental cache bloat
rm -rf target/llvm-cov-target/       # 7.8GB - coverage artifacts
rm -rf target/release/               # 1.8GB (if not needed)
```

#### Automated Cleanup Script

Create `scripts/clean-artifacts.sh`:

```bash
#!/usr/bin/env bash
# Clean Rust build artifacts with configurable aggressiveness

set -euo pipefail

MODE="${1:-standard}"

case "$MODE" in
  "quick")
    # Quick clean - just incremental cache
    rm -rf target/debug/incremental/ target/release/incremental/
    echo "Cleaned incremental caches"
    ;;
  "standard")
    # Standard clean - incremental + coverage + release
    rm -rf target/debug/incremental/ target/release/incremental/
    rm -rf target/llvm-cov-target/
    rm -rf target/release/
    echo "Cleaned incremental, coverage, and release artifacts"
    ;;
  "full")
    # Full clean - everything
    cargo clean
    echo "Full cargo clean completed"
    ;;
  *)
    echo "Usage: $0 [quick|standard|full]"
    exit 1
    ;;
esac
```

#### Prevention Measures

1. **Add to `.gitignore`**:
   ```
   # Build artifacts
   target/
   !target/.gitkeep
   
   # Coverage artifacts
   *.profraw
   *.profdata
   ```

2. **CI Cleanup**: Add post-build cleanup in CI workflows:
   ```yaml
   - name: Clean up target directory
     run: |
       rm -rf target/debug/incremental/
       rm -rf target/llvm-cov-target/
   ```

3. **Local Development**: Run `cargo clean` weekly or before large branch switches

#### Expected Savings

| Action | Space Recovered |
|--------|-----------------|
| `cargo clean` | **~74 GB** |
| Delete `target/debug/incremental/` | **38 GB** |
| Delete `target/llvm-cov-target/` | **7.8 GB** |
| Delete `target/release/` | **1.8 GB** |

## Consequences

### Positive
- **60-70% reduction** in target/ directory size (5.2 GB → ~1.5-2 GB)
- **2-5x faster link times** with mold linker
- **Faster CI builds** from reduced disk I/O
- **Better developer experience** with faster incremental builds
- **Reduced backup/sync overhead** for local development

### Negative
- `--profile debugging` required for full debugger support
- ~~`mold` linker must be installed separately on dev machines~~ ✅ Now installed
- Initial setup effort for each developer (reduced - mold pre-installed)

### Risks
- Split debuginfo may interact poorly with some debugging tools
- Mold linker may not support all linking scenarios (C/C++ deps)

## Metrics

| Metric | Before (2026-02-21) | Current (2026-03-09) | Target | Measurement |
|--------|---------------------|----------------------|--------|-------------|
| target/ size (clean build) | 5.2 GB | **74 GB** 🔴 | < 2 GB | `du -sh target/` |
| target/debug/incremental/ | 1.1 GB | **38 GB** 🔴 | < 500 MB | `du -sh target/debug/incremental/` |
| target/debug/deps/ | 3.9 GB | **27 GB** 🔴 | < 5 GB | `du -sh target/debug/deps/` |
| target/llvm-cov-target/ | N/A | **7.8 GB** 🔴 | 0 (after CI) | `du -sh target/llvm-cov-target/` |
| Deps file count | N/A | **384,837** 🔴 | < 50,000 | `ls target/debug/deps/ \| wc -l` |
| Incremental rebuild time | baseline | -30% | -30% | `cargo build` timing |
| CI disk usage | baseline | -40% | -40% | workflow logs |
| Link time | baseline | -60% | -60% | `cargo build` timing |

**Status**: 🔴 **CRITICAL** - 14x growth in 2 weeks requires immediate action

## References

- [Cargo Build Performance Guide](https://doc.rust-lang.org/cargo/guide/build-performance.html)
- [Tips for Faster Rust Compile Times](https://corrode.dev/blog/tips-for-faster-rust-compile-times/)
- [Freeing Disk Space from Cargo Builds](https://thisdavej.com/freeing-up-gigabytes-reclaiming-disk-space-from-rust-cargo-builds/)
- [Cargo Build Dir Rework (Rust Project Goal 2025H2)](https://rust-lang.github.io/rust-project-goals/2025h2/goals.html)
