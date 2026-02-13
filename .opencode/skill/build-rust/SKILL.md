---
name: build-rust
description: Optimized Rust build operations with timing, profiling, and workspace support
---

# Rust Build Operations

Efficiently build Rust workspaces with the build-rust CLI.

## Usage

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
./scripts/build-rust.sh release memory-core
```

## Modes

| Mode | Purpose | Flags |
|------|---------|--------|
| `dev` | Development build | `--workspace` |
| `release` | Production optimized | `--release --workspace` |
| `profile` | Performance timing | `--release --timings` |
| `check` | Fast type-check | `--workspace` |
| `clean` | Clean artifacts | `--clean` |

## Common Issues

**Timeouts**
- Use `dev` mode for faster iteration
- Reduce parallel jobs: `CARGO_BUILD_JOBS=4 ./scripts/build-rust.sh release`

**Memory errors**
- Build with fewer jobs: `cargo build -j 4`
- Use `check` instead of full build

**Dependency conflicts**
- Update: `cargo update`
- Check tree: `cargo tree -e features`

**Platform-specific**
- Install targets: `rustup target add <triple>`
- Conditional compilation: `#[cfg(target_os = "linux")]`
