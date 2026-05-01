---
name: build-rust
description: Build Rust code with proper error handling, optimization, and workspace support for development, testing, and production
---

# Rust Build Operations

Build Rust workspaces efficiently with the build-rust CLI.

## CLI Usage

```bash
./scripts/build-rust.sh dev        # Development (fast, debug symbols)
./scripts/build-rust.sh release   # Production (optimized, stripped)
./scripts/build-rust.sh profile   # Performance analysis (--timings)
./scripts/build-rust.sh check     # Fast type-check only
./scripts/build-rust.sh clean     # Artifact cleanup
./scripts/build-rust.sh release do-memory-core  # Build specific crate
```

## Build Modes

| Mode | Use Case | Flags |
|------|----------|-------|
| `dev` | Development iteration | `--workspace` |
| `release` | Production deployment | `--release --workspace` |
| `profile` | Performance analysis | `--release --timings` |
| `check` | Fast validation | `--workspace` (type-check only) |

## Disk Space Optimization (ADR-032)

Dev profile reduces target/ size (~5.2 GB to ~2 GB):

```toml
[profile.dev]
debug = "line-tables-only"    # ~60% smaller
[profile.dev.package."*"]
debug = false                 # No debug for deps
[profile.dev.build-override]
opt-level = 3                 # Faster proc-macros
```

Cleanup: `./scripts/clean-artifacts.sh quick|standard|full`
Offload: `CARGO_TARGET_DIR=/mnt/fastssd/rslm-target ./scripts/build-rust.sh dev`

## Error Handling

| Issue | Fix |
|-------|-----|
| Timeout | `CARGO_BUILD_JOBS=4 cargo build` or use `check` |
| Memory | `cargo build -j 1` or monitor with `/usr/bin/time -v` |
| Dep conflicts | `cargo update` then `cargo tree -d | grep -cE "^[a-z]"` |
| Cross-compilation | `rustup target add x86_64-unknown-linux-musl` |

## Common Workflows

```bash
# Full CI Pipeline
./scripts/code-quality.sh fmt && ./scripts/code-quality.sh clippy --workspace
cargo build --release --workspace && cargo nextest run --all && cargo test --doc

# Quick Development
cargo check -p do-memory-core && cargo test -p do-memory-core --lib

# Production Release
cargo build --release --workspace && strip target/release/do-memory-mcp
```

## Troubleshooting

| Issue | Fix |
|-------|-----|
| Incremental cache corruption | `cargo clean && cargo build` |
| Stale lock file | `rm Cargo.lock && cargo generate-lockfile` |
| Rust version mismatch | `rustup update stable && rustup default stable` |

## Verification Checklist

- [ ] Build completes, no clippy warnings (`cargo clippy -- -D warnings`)
- [ ] Tests compile, binary < 10MB stripped, startup < 100ms

## References

ADR-032 (Disk Space), ADR-036 (Dependency Deduplication)
