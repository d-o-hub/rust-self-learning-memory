---
name: build-rust
description: Build Rust code with proper error handling, optimization, and workspace support for development, testing, and production
---

# Rust Build Operations

Efficiently build Rust workspaces with the build-rust CLI, with proper error handling and optimization.

## CLI Usage

```bash
# Development (fast, debug symbols)
./scripts/build-rust.sh dev

# Release (optimized, stripped)
./scripts/build-rust.sh release

# Profile with timing information
./scripts/build-rust.sh profile

# Fast type-check only
./scripts/build-rust.sh check

# Clean build artifacts
./scripts/build-rust.sh clean

# Build specific crate
./scripts/build-rust.sh release do-memory-core
```

## Build Modes

| Mode | Use Case | Performance | Flags |
|------|----------|-------------|-------|
| `dev` | Development iteration | Fast | `--workspace` |
| `release` | Production deployment | Optimized | `--release --workspace` |
| `profile` | Performance analysis | Medium | `--release --timings` |
| `check` | Fast validation | Fastest | `--workspace` (type-check only) |
| `clean` | Artifact cleanup | N/A | `--clean` |

## Disk Space Optimization (ADR-032)

Dev profile is optimized to reduce target/ size (~5.2 GB to ~2 GB):

```toml
# .cargo/config.toml
[profile.dev]
debug = "line-tables-only"    # ~60% smaller debug artifacts

[profile.dev.package."*"]
debug = false                 # No debug info for dependencies

[profile.dev.build-override]
opt-level = 3                 # Faster proc-macro execution

[profile.debugging]
inherits = "dev"
debug = true                  # Full debug when needed: --profile debugging
```

**Cleanup**:
```bash
./scripts/clean-artifacts.sh quick
./scripts/clean-artifacts.sh standard
./scripts/clean-artifacts.sh full
```

**Artifact offloading** with `CARGO_TARGET_DIR`:
```bash
CARGO_TARGET_DIR=/mnt/fastssd/rslm-target ./scripts/build-rust.sh dev
```

## Error Handling

**Timeout Errors**
- Reduce concurrency: `CARGO_BUILD_JOBS=4 cargo build`
- Use `check` mode for faster feedback

**Memory Errors**
- Sequential build: `cargo build -j 1`
- Monitor: `/usr/bin/time -v cargo build`
- Use `check` mode (no codegen)

**Dependency Conflicts**
- Update: `cargo update`
- Check tree: `cargo tree -e features`
- Check duplicates: `cargo tree -d | grep -cE "^[a-z]"`

**Platform-Specific**
- Install targets: `rustup target add <triple>`
- Conditional: `#[cfg(target_os = "linux")]`

## Common Workflows

**Full CI Pipeline**
```bash
./scripts/code-quality.sh fmt
./scripts/code-quality.sh clippy --workspace
cargo build --release --workspace
cargo nextest run --all
cargo test --doc
```

**Quick Development Cycle**
```bash
cargo check -p do-memory-core
cargo test -p do-memory-core --lib
```

**Production Release**
```bash
cargo build --release --workspace
strip target/release/do-memory-mcp
./target/release/do-memory-mcp --version
```

## Troubleshooting

| Issue | Fix |
|-------|-----|
| Incremental cache corruption | `cargo clean && cargo build` |
| Stale lock file | `rm Cargo.lock && cargo generate-lockfile` |
| Rust version mismatch | `rustup update stable && rustup default stable` |
| Cross-compilation failures | `rustup target add x86_64-unknown-linux-musl` |

## Verification Checklist

- [ ] Build completes without errors
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)
- [ ] Tests compile (`cargo test --no-run`)
- [ ] Binary size acceptable (< 10MB stripped)
- [ ] Startup time < 100ms

## Related Skills

- **code-quality**: Lint and format checks before builds
- **test-runner**: Execute tests after successful compilation
- **debug-troubleshoot**: Diagnose runtime issues post-build

## References

- [ADR-032: Disk Space Optimization](../../../plans/adr/ADR-032-Disk-Space-Optimization.md)
- [ADR-036: Dependency Deduplication](../../../plans/adr/ADR-036-Dependency-Deduplication.md)