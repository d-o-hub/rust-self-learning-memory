# AGENTS.md

## Essential Commands
```bash
# Build & test all
cargo build --all && cargo test --all

# Run single test
cargo test test_name -- --nocapture

# Quality checks
cargo fmt -- --check
cargo clippy -- -D warnings
cargo security-check  # alias for: audit && deny check

# Coverage
cargo coverage  # alias for: llvm-cov --all-features --workspace --lcov --output-path lcov.info
```

## Code Style
- **500 LOC per file** — split when exceeded
- **Formatting**: `rustfmt.toml` (max_width=100, tab_spaces=4, reorder_imports=true)
- **Error handling**: `anyhow::Result` for public APIs, `thiserror` for libraries
- **Logging**: Use `tracing` instead of `println!`
- **Async**: `tokio` with full features
- **Naming**: snake_case functions/modules, CamelCase types/traits
- **Imports**: Group std→external→local, alphabetical within groups
- **Security**: Never hardcode tokens, sanitize logs, validate inputs

## Environment
- `TURSO_URL`, `TURSO_TOKEN` for database
- `MAX_EPISODES_CACHE=1000`, `BATCH_SIZE=50`

## Architecture
**Crates**: `memory-core`, `memory-storage-turso`, `memory-storage-redb`, `memory-mcp`
**Stack**: Rust/async-Tokio, Turso/libSQL (durable), redb (cache), optional embeddings