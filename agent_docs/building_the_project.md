# Building the Project

## Core Build Commands

### Quick Build & Test
```bash
# Build all workspace members
cargo build --all

# Run all tests
cargo test --all

# Combined build and test
cargo build --all && cargo test --all
```

### Feature Flags

The project supports the following optional features:

**memory-core features:**
- `openai` - Enable OpenAI embedding provider (requires `reqwest`)
- `local-embeddings` - Enable local embedding models with ONNX Runtime
- `embeddings-full` - Enable all embedding providers (implies `openai`)

**memory-cli features:**
- `turso` - Enable Turso storage backend (implies redb)
- `redb` - Enable redb storage backend (default)
- `full` - Enable all storage backends (implies both turso and redb)

**memory-mcp features:**
- `wasmtime-backend` - Wasmtime WASM sandbox (default)
- `wasm-rquickjs` - QuickJS JavaScript runtime (deprecated)
- `javy-backend` - Javy JavaScript to WASM compilation

```bash
# Build with specific features
cargo build --all --features "openai,local-embeddings"

# Build with all features
cargo build --all --all-features

# Build specific crate with features
cd memory-core && cargo build --features openai
```

### Release Build
```bash
# Optimized release build (LTO enabled, single codegen unit)
cargo build --release --workspace

# Release with specific features
cargo build --release --features "openai"

# Check release build
cargo build --release --workspace --all-targets
```

### Development Build
```bash
# Build with debug info and all features
cargo build --all-features --workspace

# Build specific crate
cd memory-core && cargo build
cd memory-storage-turso && cargo build
cd memory-storage-redb && cargo build
cd memory-mcp && cargo build
cd memory-cli && cargo build
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

### Quality Gates
```bash
# Run full quality check (coverage >90%)
./scripts/quality-gates.sh

# Individual quality checks
cargo fmt --all -- --check
cargo clippy --all -- -D warnings
cargo test --all

# Fix clippy suggestions automatically
cargo clippy --fix --allow-dirty --all-targets --all-features

# Fix formatting automatically
cargo fmt --all
```

### Docker Setup (Optional)
```bash
# Build memory-cli Docker image
cd memory-cli
docker build -t memory-cli .

# Run with docker-compose
cd memory-cli/docker
docker-compose up -d
```

## Testing

### Run All Tests
```bash
# Run entire test suite
cargo test --all

# With verbose output
cargo test --all -- --nocapture

# With debug logging
RUST_LOG=debug cargo test

# Specific test
cargo test test_episode_creation
```

### Coverage Testing
```bash
# Install coverage tool (once)
cargo install cargo-llvm-cov

# Generate HTML coverage report
cargo llvm-cov --html --output-dir coverage

# Generate multiple formats
cargo llvm-cov --html --lcov --json --output-dir coverage

# View coverage
open coverage/html/index.html  # macOS
xdg-open coverage/html/index.html  # Linux
```

### Benchmarking
```bash
# Run all benchmarks
cd benches && cargo bench

# Specific benchmark
cd benches && cargo bench --bench episode_lifecycle

# Benchmark with criterion output
cd benches && cargo bench -- --output-format html
```

## Build Targets

- **Debug**: `cargo build`
- **Release**: `cargo build --release` (optimized with LTO)
- **All Features**: `cargo build --all-features`
- **Check Only**: `cargo check` (faster, no binaries)
- **Doc Only**: `cargo doc --no-deps`

## Troubleshooting

### Common Build Issues
1. **Missing dependencies**: Run `cargo update`
2. **Format errors**: Run `cargo fmt --all`
3. **Clippy warnings**:
   - Run `cargo clippy --all -- -D warnings` to see warnings
   - Apply fixes: `cargo clippy --fix --allow-dirty`
   - For intentional violations, add `#[allow(...)]` with justification
   - See [CLAUDE.md](../../.claude/CLAUDE.md) for recent changes and best practices
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
cargo fmt -- --check

# Clippy check (zero warnings required)
cargo clippy --all -- -D warnings

# Build check
cargo build --all

# Test check
cargo test --all

# Documentation check
cargo doc --no-deps
```