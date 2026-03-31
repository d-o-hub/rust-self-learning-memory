# Building the Project

## Core Build Commands (Script-First)

### Quick Build & Test
```bash
# Build all workspace members
./scripts/build-rust.sh dev

# Run all tests (nextest + doctests)
cargo nextest run --all
cargo test --doc

# Combined quality flow
./scripts/code-quality.sh fmt
./scripts/code-quality.sh clippy --workspace
./scripts/build-rust.sh check
```

> Prefer wrappers in `./scripts/` for CI parity. Use raw `cargo` commands only when wrappers do not cover a specific case.

### Feature Flags

The project supports the following optional features:

**do-memory-core features:**
- `openai` - Enable OpenAI embedding provider (requires `reqwest`)
- `local-embeddings` - Enable local embedding models with ONNX Runtime
- `embeddings-full` - Enable all embedding providers (implies `openai`)

**do-memory-cli features:**
- `turso` - Enable Turso storage backend (implies redb)
- `redb` - Enable redb storage backend (default)
- `full` - Enable all storage backends (implies both turso and redb)

**do-memory-mcp features:**
- `wasmtime-backend` - Wasmtime WASM sandbox (default)
- `wasm-rquickjs` - QuickJS JavaScript runtime (deprecated)
- `javy-backend` - Javy JavaScript to WASM compilation

```bash
# Build with specific features
cargo build --all --features "openai,local-embeddings"

# Build with all features
cargo build --all --all-features

# Build specific crate with features
cd do-memory-core && cargo build --features openai
```

### Release Build
```bash
# Optimized release build (LTO enabled, single codegen unit)
./scripts/build-rust.sh release

# Release with specific features
cargo build --release --features "openai"

# Check release build
./scripts/build-rust.sh check
```

### Development Build
```bash
# Build with debug info and all features
./scripts/build-rust.sh dev

# Build specific crate
cd do-memory-core && cargo build
cd do-memory-storage-turso && cargo build
cd do-memory-storage-redb && cargo build
cd do-memory-mcp && cargo build
cd do-memory-cli && cargo build
```

## Development Setup

### Prerequisites
- Rust toolchain (see `rust-toolchain.toml`)
- SQLite (for local development)
- Docker (optional, for Turso)
- `cargo-llvm-cov` for coverage (install with `cargo install cargo-llvm-cov`)

### Environment Setup
```bash
# Copy environment template
cp .env.example .env

# Setup local database
./scripts/setup-local-db.sh

# Install git hooks (required)
git config core.hooksPath .githooks
chmod +x .githooks/*
```

### Turso Local Development

For local development without cloud Turso:

```bash
# Install Turso CLI
curl -sSfL https://get.tur.so/install.sh | bash

# Start local Turso server
turso dev --db-file ./data/memory.db --port 8080

# Set environment variables
export TURSO_DATABASE_URL="http://127.0.0.1:8080"
export TURSO_AUTH_TOKEN=""  # No auth required for local
```

No cloud account or auth token required for local development. See [LOCAL_DATABASE_SETUP.md](../docs/LOCAL_DATABASE_SETUP.md) for more details.

> For quality gates commands, see [running_tests.md](running_tests.md) or run `./scripts/quality-gates.sh`.

### Docker Setup (Optional)
```bash
# Build do-memory-cli Docker image
cd do-memory-cli
docker build -t do-memory-cli .

# Run with docker-compose
cd do-memory-cli/docker
docker-compose up -d
```

## Testing

> For testing commands, see [running_tests.md](running_tests.md).

```bash
# Quick test
cargo nextest run --all

# Run doctests
cargo test --doc

# Full local gates (includes coverage threshold checks)
./scripts/quality-gates.sh
```

## Build Targets

- **Debug**: `./scripts/build-rust.sh dev`
- **Release**: `./scripts/build-rust.sh release` (optimized with LTO)
- **All Features**: use raw cargo when needed (`cargo build --all-features`)
- **Check Only**: `./scripts/build-rust.sh check` (faster, no binaries)
- **Doc Only**: `cargo doc --no-deps`

## Troubleshooting

### Common Build Issues
1. **Missing dependencies**: Run `cargo update`
2. **Format errors**: Run `./scripts/code-quality.sh fmt`
3. **Clippy warnings**:
   - Run `./scripts/code-quality.sh clippy --workspace` to see warnings
   - Apply fixes: `./scripts/code-quality.sh clippy --fix`
   - For intentional violations, add `#[allow(...)]` with justification
   - See [CLAUDE.md](../CLAUDE.md) for recent changes and best practices
4. **Test failures**: Check [TESTING.md](../TESTING.md) for debugging
5. **Build errors**: Ensure Rust toolchain is up to date: `rustup update`

### Performance Issues
- Use `cargo check` for faster compile checks
- Use `--lib` flag for library-only builds
- Enable incremental compilation in `.cargo/config.toml`
- Use `--offline` flag to avoid network operations

### Feature Flag Issues
- Check that optional features are enabled: `cargo build --all-features`
- Verify dependencies in `Cargo.toml` have proper `optional = true`
- For embeddings, ensure API keys are set in `.env`

## Verification Commands

After building, verify with:
```bash
# Format check
./scripts/code-quality.sh fmt

# Clippy check (zero warnings required)
./scripts/code-quality.sh clippy --workspace

# Build check
./scripts/build-rust.sh check

# Test check
cargo nextest run --all
cargo test --doc

# Coverage + integrated quality gates (default threshold: 90%)
./scripts/quality-gates.sh

# Documentation check
cargo doc --no-deps
```

## Disk Space Optimization

### Dev Profile Settings
```toml
# .cargo/config.toml
[profile.dev]
debug = "line-tables-only"  # Faster builds, smaller binaries

[profile.dev.package."*"]
debug = false  # Don't debug info for dependencies
```

### Cleanup Commands
```bash
# Routine cleanup (recommended)
./scripts/clean-artifacts.sh standard

# Fast cleanup for daily iteration
./scripts/clean-artifacts.sh quick

# Full reset (includes cargo clean)
./scripts/clean-artifacts.sh full

# Optional: include JS dependency cleanup
./scripts/clean-artifacts.sh standard --node-modules
```

### Offloading Artifacts with `CARGO_TARGET_DIR`

```bash
# Build using external target directory
CARGO_TARGET_DIR=/mnt/fastssd/rslm-target ./scripts/build-rust.sh dev

# Clean that same target directory
CARGO_TARGET_DIR=/mnt/fastssd/rslm-target ./scripts/clean-artifacts.sh standard
```
