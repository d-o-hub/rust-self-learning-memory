# Rust Self-Learning Memory

A zero-trust episodic memory backend for AI agents, written in Rust. This system maintains a durable, verifiable record of agent execution while extracting and learning from patterns to improve future decision-making.

## Overview

This project provides a production-grade memory system designed for AI agents that need to:
- **Record episodes**: Start → Execute → Score → Learn → Retrieve lifecycle
- **Store durably**: Leverage Turso/libSQL for distributed SQL persistence
- **Cache efficiently**: Use redb for hot-path key-value access
- **Extract patterns**: Learn decision points, tool sequences, and recovery heuristics
- **Retrieve intelligently**: Semantic and context-based pattern retrieval
- **Verify security**: Zero-trust validation with comprehensive security checks

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Features](#features)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Configuration](#configuration)
- [Quick Start](#quick-start)
- [API Documentation](#api-documentation)
- [Project Structure](#project-structure)
- [Code Coverage](#code-coverage)
- [Storage](#storage)
- [Performance](#performance)
- [Security](#security)
- [Development](#development)
  - [Code Style](#code-style)
  - [CI Pipeline](#ci-pipeline)
  - [Testing](#testing)
  - [Pre-commit Hooks](#pre-commit-hooks)
- [Dependencies](#dependencies)
- [Contributing](#contributing)
- [License](#license)
- [Resources](#resources)
- [Contact](#contact)

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                  SelfLearningMemory                     │
│            (Main orchestration interface)               │
└───────────┬─────────────────────────────┬───────────────┘
            │                             │
    ┌───────▼────────┐          ┌─────────▼─────────┐
    │  memory-core   │          │ Episode Lifecycle │
    │                │          │                   │
    │  • Episodes    │          │ • Start           │
    │  • Patterns    │          │ • Log Steps       │
    │  • Heuristics  │          │ • Complete        │
    │  • Learning    │          │ • Score & Learn   │
    └───────┬────────┘          └───────────────────┘
            │
    ┌───────┴─────────────────────────────┐
    │                                     │
    │         Storage Layer               │
    │                                     │
┌───▼──────────────────┐    ┌────────────▼───┐
│  Turso/libSQL        │    │  redb (cache)  │
│  (durable)           │    │  (hot path)    │
│                      │    │                │
│ • episodes table     │    │ • episodes     │
│ • patterns table     │    │ • patterns     │
│ • heuristics table   │    │ • embeddings   │
│ • SQL persistence    │    │ • metadata     │
└──────────────────────┘    └────────────────┘
```

## Features

- **Episode Management**: Create, log execution steps, complete with scoring
- **Pattern Extraction**: Automatic extraction of ToolSequences, DecisionPoints, ErrorRecovery
- **Learning Queue**: Asynchronous pattern learning with backpressure handling
- **Dual Storage**: Durable Turso/libSQL + fast redb cache
- **Security**: Zero-trust validation, sanitization, parameterized queries
- **MCP Support**: Model Context Protocol integration for Claude integration
- **Comprehensive Testing**: >90% code coverage with unit and integration tests
- **Monitoring**: Tracing support with structured logging

## Prerequisites

Before you begin, ensure you have the following installed:

- **Rust**: Version 1.70 or higher
  ```bash
  rustup --version  # Verify installation
  rustup override set stable
  ```

- **Cargo**: Comes with Rust (verify with `cargo --version`)

- **System Requirements**:
  - Linux, macOS, or Windows
  - Minimum 4GB RAM (8GB recommended)
  - 500MB disk space for dependencies

- **Optional**:
  - **Turso Account**: Required for production deployments with durable storage
    - Sign up at [turso.tech](https://turso.tech)
    - Create a database and obtain credentials
  - **libSQL CLI**: For local Turso/libSQL database testing
    ```bash
    # Install libSQL CLI
    brew install tursodatabase/tap/turso  # macOS
    # or download from https://github.com/tursodatabase/libsql
    ```

## Installation

### As a Library

Add this crate to your `Cargo.toml`:

```toml
[dependencies]
memory-core = "0.1"
memory-storage-turso = "0.1"
memory-storage-redb = "0.1"
```

Or use `cargo add`:

```bash
cargo add memory-core memory-storage-turso memory-storage-redb
```

## Configuration

### Environment Variables

The memory system requires the following environment variables for Turso/libSQL connectivity:

```bash
# Required for production Turso deployments
export TURSO_DATABASE_URL="libsql://your-database.turso.io"
export TURSO_AUTH_TOKEN="your-auth-token-here"

# Optional: Local libSQL file (for development/testing)
export LIBSQL_DATABASE_PATH="./data/memory.db"

# Optional: redb cache configuration
export REDB_CACHE_PATH="./data/cache.redb"
export REDB_MAX_CACHE_SIZE="1000"  # Maximum episodes to cache
```

### Example `.env` File

Create a `.env` file in your project root:

```env
# Turso Configuration (Production)
TURSO_DATABASE_URL=libsql://my-memory-db.turso.io
TURSO_AUTH_TOKEN=eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9...

# Local Development (Alternative to Turso)
# LIBSQL_DATABASE_PATH=./data/memory.db

# redb Cache
REDB_CACHE_PATH=./data/cache.redb
REDB_MAX_CACHE_SIZE=1000

# Logging
RUST_LOG=info,memory_core=debug
```

**Important**: Never commit `.env` files to version control. Add `.env` to your `.gitignore`.

### Configuration Options

When initializing `SelfLearningMemory`, you can configure:

```rust
use memory_core::{SelfLearningMemory, MemoryConfig};

let config = MemoryConfig {
    turso_url: std::env::var("TURSO_DATABASE_URL")?,
    turso_token: std::env::var("TURSO_AUTH_TOKEN")?,
    redb_path: std::env::var("REDB_CACHE_PATH")
        .unwrap_or_else(|_| "./data/cache.redb".to_string()),
    max_cache_size: std::env::var("REDB_MAX_CACHE_SIZE")
        .unwrap_or_else(|_| "1000".to_string())
        .parse()?,
    ..Default::default()
};

let memory = SelfLearningMemory::new(config).await?;
```

## Quick Start

### Basic Usage Example

```rust
use memory_core::SelfLearningMemory;
use memory_core::{TaskContext, ExecutionStep, TaskOutcome};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize memory system
    let memory = SelfLearningMemory::new(Default::default()).await?;

    // Start an episode
    let context = TaskContext {
        language: "rust".to_string(),
        domain: "web".to_string(),
        tags: vec!["api".to_string()],
    };

    let episode = memory.start_episode(
        "Build REST API endpoint".to_string(),
        context.clone(),
    ).await?;

    // Log execution steps
    let step = ExecutionStep {
        tool: "rustc".to_string(),
        action: "compile".to_string(),
        latency_ms: 1250,
        tokens: 2500,
        success: true,
        observation: "Compiled successfully".to_string(),
    };

    memory.log_step(episode.id.clone(), step).await?;

    // Complete and score episode
    let outcome = TaskOutcome {
        success: true,
        result: Some("Endpoint created".to_string()),
        error: None,
        duration_ms: 5000,
    };

    let completed = memory.complete_episode(episode.id, outcome).await?;

    // Retrieve similar past episodes
    let relevant = memory.retrieve_relevant_context(
        "Build REST endpoint".to_string(),
        context,
        5,
    ).await?;

    println!("Found {} relevant episodes", relevant.len());
    Ok(())
}
```

## API Documentation

Comprehensive API documentation is available at:

- **[docs.rs/memory-core](https://docs.rs/memory-core)** - Core memory system API
- **[docs.rs/memory-storage-turso](https://docs.rs/memory-storage-turso)** - Turso storage backend
- **[docs.rs/memory-storage-redb](https://docs.rs/memory-storage-redb)** - redb cache backend

You can also generate and view the documentation locally:

```bash
# Generate and open documentation for all crates
cargo doc --all --open

# Generate documentation with private items
cargo doc --all --document-private-items --open
```

## Project Structure

```
.
├── memory-core/              # Core memory system
│   ├── src/
│   │   ├── episode/          # Episode management
│   │   ├── pattern/          # Pattern types and operations
│   │   ├── patterns/         # Pattern extraction and learning
│   │   ├── learning/         # Learning queue and orchestration
│   │   ├── memory.rs         # Main SelfLearningMemory struct
│   │   └── lib.rs
│   └── tests/
├── memory-storage-turso/     # Turso/libSQL backend
├── memory-storage-redb/      # redb cache backend
├── memory-mcp/               # MCP protocol support
├── test-utils/               # Shared test utilities
├── .github/workflows/
│   ├── ci.yml               # Main CI pipeline with coverage gate
│   ├── security.yml         # Security scanning
│   └── release.yml          # Release automation
├── .codecov.yml             # Codecov configuration
└── AGENTS.md                # Agent guidelines and workflows
```

## Code Coverage

This project enforces **>90% code coverage** on the main branch via cargo-llvm-cov in the CI pipeline.

- Coverage reports generated as HTML and LCOV format
- Uploaded to Codecov for tracking and analysis
- Coverage badge above shows current status
- See `.codecov.yml` for configuration

### Running Coverage Locally

```bash
# Install cargo-llvm-cov
cargo install cargo-llvm-cov

# Generate coverage report (HTML)
cargo llvm-cov --all-features --workspace --html

# Generate LCOV format (for Codecov)
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

# View summary
cargo llvm-cov --all-features --workspace --summary-only
```

## Storage

### Turso/libSQL (Durable)
- Distributed SQL database for durable storage
- Tables: `episodes`, `patterns`, `heuristics`
- Parameterized queries for security
- Supports remote and local deployments

### redb (Cache)
- Embedded key-value store for hot-path access
- Tables: `episodes`, `patterns`, `embeddings`, `metadata`
- Synchronous operations (wrapped in spawn_blocking)
- Reconciliation with Turso via sync_memories()

## Performance

This project maintains strict performance baselines and tracks regressions via automated benchmarks. All operations significantly exceed target performance requirements.

### Baseline Metrics (P95)

| Operation | Actual | Target | Status |
|-----------|--------|--------|--------|
| Episode Creation | 2.56 µs | < 50ms | 19,531x faster ✓ |
| Step Logging | 1.13 µs | < 20ms | 17,699x faster ✓ |
| Episode Completion | 3.82 µs | < 500ms | 130,890x faster ✓ |
| Pattern Extraction | 10.43 µs | < 1000ms | 95,880x faster ✓ |
| Memory Retrieval | 721 µs | < 100ms | 138x faster ✓ |
| Storage (Write) | 13.22 ms | < 50ms | 3.8x faster ✓ |

See [PERFORMANCE_BASELINES.md](PERFORMANCE_BASELINES.md) for detailed metrics and analysis.

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench --package memory-benches

# View detailed reports
open target/criterion/report/index.html

# Check for regressions
./scripts/check_performance_regression.sh
```

### CI Performance Tracking

Benchmarks run automatically on:
- Every push to main branch
- All pull requests
- Weekly schedule (Mondays)

Performance regressions >10% trigger automatic alerts. See `.github/workflows/benchmarks.yml` for configuration.

## Security

See [SECURITY.md](SECURITY.md) for detailed security guidelines including:
- Zero-trust validation principles
- Credential handling (environment variables only)
- Input sanitization
- Parameterized SQL queries
- RBAC and access control

## Development

### Development Setup

To set up the project for development and contribution:

```bash
# Clone the repository
git clone https://github.com/d-o-hub/rust-self-learning-memory.git
cd rust-self-learning-memory

# Ensure you're using stable Rust
rustup override set stable

# Build all crates
cargo build --all

# Run the test suite
cargo test --all

# Run with debug logging
RUST_LOG=debug cargo test --all -- --nocapture
```

### Code Style

- Rust: Follow `rustfmt` and Clippy rules
- Keep files ≤ 500 LOC (split into submodules if needed)
- Use `anyhow::Result` for top-level functions
- Use `thiserror` for typed errors
- Document all public APIs

### CI Pipeline

The project uses GitHub Actions for continuous integration:

```yaml
Jobs:
  - format:           Verify code formatting (cargo fmt)
  - clippy:           Lint checks (cargo clippy -D warnings)
  - test:             Test suite (multiple OS, stable Rust)
  - build:            Release build with timing
  - coverage:         Code coverage with >90% gate
  - security-audit:   Vulnerability scanning (cargo audit)
  - supply-chain:     License & advisory checks (cargo-deny)
```

### Testing

```bash
# Run all tests with debug logging
RUST_LOG=debug cargo test --all -- --nocapture

# Run specific test
cargo test --all memory::tests::test_start_episode

# Run with coverage
cargo llvm-cov --all-features --workspace
```

### Pre-commit Hooks

This project uses Claude Code hooks for validation:
- Code formatting check
- Clippy linting
- Cargo audit (security)
- Cargo deny (licenses & advisories)
- Test execution
- Secret scanning

## Dependencies

Key dependencies:
- **tokio**: Async runtime
- **libsql**: Turso/libSQL client
- **redb**: Embedded key-value store
- **serde**: Serialization/deserialization
- **tracing**: Structured logging
- **anyhow/thiserror**: Error handling
- **uuid/chrono**: Identifiers and timestamps

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines.

Contributions are welcome! Please ensure:
- All tests pass: `cargo test --all`
- Code is formatted: `cargo fmt`
- No clippy warnings: `cargo clippy -- -D warnings`
- Coverage maintained: coverage reports generated

## License

This project is licensed under the MIT License - see [LICENSE](LICENSE) file for details.

## Resources

- [AGENTS.md](AGENTS.md) - Agent responsibilities and task templates
- [TESTING.md](TESTING.md) - Testing infrastructure and best practices
- [SECURITY.md](SECURITY.md) - Security guidelines and threat model
- [CONTRIBUTING.md](CONTRIBUTING.md) - Contribution guidelines
- [ROADMAP.md](ROADMAP.md) - Project roadmap and future features
