# ADR-032: Disk Space Optimization Strategy

- **Status**: Partially Implemented (Phase 1 & 2 Complete)
- **Date**: 2026-02-21
- **Deciders**: Project maintainers
- **Supersedes**: None
- **Related**: ADR-029 (GitHub Actions Modernization), ADR-030 (Test Optimization)

## Context

The project's `target/` directory consumes **5.2 GB** for debug builds alone:
- `target/debug/deps/`: 3.9 GB
- `target/debug/incremental/`: 1.1 GB
- `target/debug/build/`: 395 MB

Root causes:
1. **Full debug info** (`debug = true`) in dev profile generates ~2x artifact size
2. **No debug info stripping** for dependency crates
3. **No split-debuginfo** configured (monolithic DWARF)
4. **No fast linker** (using default `ld`, not `mold` or `lld`)
5. **120 duplicate dependency roots** across 863 packages inflating compiled output
6. **Orphaned `node_modules/`** (89 MB, only `@openai`, not used by Rust build)
7. **No incremental build GC** strategy

Additionally, CI builds waste disk space and time rebuilding artifacts unnecessarily.

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

- Remove orphaned `node_modules/` directory (add to `.gitignore` if not already)
- Add `scripts/clean-artifacts.sh` for periodic local cleanup
- Document `cargo clean-all` usage for developers

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

| Metric | Before | Target | Measurement |
|--------|--------|--------|-------------|
| target/ size (clean build) | 5.2 GB | < 2 GB | `du -sh target/` |
| Incremental rebuild time | baseline | -30% | `cargo build` timing |
| CI disk usage | baseline | -40% | workflow logs |
| Link time | baseline | -60% | `cargo build` timing |

## References

- [Cargo Build Performance Guide](https://doc.rust-lang.org/cargo/guide/build-performance.html)
- [Tips for Faster Rust Compile Times](https://corrode.dev/blog/tips-for-faster-rust-compile-times/)
- [Freeing Disk Space from Cargo Builds](https://thisdavej.com/freeing-up-gigabytes-reclaiming-disk-space-from-rust-cargo-builds/)
- [Cargo Build Dir Rework (Rust Project Goal 2025H2)](https://rust-lang.github.io/rust-project-goals/2025h2/goals.html)
