# Rust Self-Learning Memory

[![CI](https://github.com/d-o-hub/rust-self-learning-memory/actions/workflows/ci.yml/badge.svg)](https://github.com/d-o-hub/rust-self-learning-memory/actions/workflows/ci.yml)
[![Coverage](https://codecov.io/gh/d-o-hub/rust-self-learning-memory/branch/main/graph/badge.svg)](https://codecov.io/gh/d-o-hub/rust-self-learning-memory)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-stable-orange)](https://www.rust-lang.org/)

A zero-trust episodic memory backend for AI agents, written in Rust. This system maintains a durable, verifiable record of agent execution while extracting and learning from patterns to improve future decision-making.

## Overview

This project provides a production-grade memory system designed for AI agents that need to:
- **Record episodes**: Start → Execute → Score → Learn → Retrieve lifecycle
- **Store durably**: Leverage Turso/libSQL for distributed SQL persistence
- **Cache efficiently**: Use redb for hot-path key-value access
- **Extract patterns**: Learn decision points, tool sequences, and recovery heuristics
- **Retrieve intelligently**: Semantic and context-based pattern retrieval
- **Verify security**: Zero-trust validation with comprehensive security checks

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

## Quick Start

### Build

```bash
# Install Rust (if not already installed)
rustup override set stable

# Build all crates
cargo build --all

# Run tests
cargo test --all
```

### Basic Usage

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

## Security

See [SECURITY.md](SECURITY.md) for detailed security guidelines including:
- Zero-trust validation principles
- Credential handling (environment variables only)
- Input sanitization
- Parameterized SQL queries
- RBAC and access control

## Development

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
  - unsafe-code:      Unsafe code detection (cargo-geiger)
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

## Contact

For questions or issues, please open a GitHub issue or check the project's issue tracker.
