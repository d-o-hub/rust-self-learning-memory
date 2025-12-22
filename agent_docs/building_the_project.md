# Building the Project

## Core Build Commands

### Quick Build & Test
```bash
cargo build --all && cargo test --all
```

### Individual Crate Builds
```bash
# Build specific crate
cd memory-core && cargo build
cd memory-storage-turso && cargo build
cd memory-storage-redb && cargo build
cd memory-mcp && cargo build
```

### Development Build
```bash
# Build with debug info and all features
cargo build --all-features --workspace
```

## Development Setup

### Prerequisites
- Rust toolchain (see `rust-toolchain.toml`)
- SQLite (for local development)
- Docker (optional, for Turso)

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
cargo fmt -- --check
cargo clippy -- -D warnings
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

## Build Targets

- **Debug**: `cargo build`
- **Release**: `cargo build --release`
- **All Features**: `cargo build --all-features`

## Troubleshooting

### Common Build Issues
1. **Missing dependencies**: Run `cargo update`
2. **Format errors**: Run `cargo fmt`
3. **Clippy warnings**: Address all warnings with `-D warnings`
4. **Test failures**: Check `TESTING.md` for debugging

### Performance Issues
- Use `cargo check` for faster compile checks
- Use `--lib` flag for library-only builds
- Enable incremental compilation in `.cargo/config.toml`