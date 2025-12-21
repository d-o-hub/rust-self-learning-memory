# Rust Self-Learning Memory System

<div align="center">

![Rust](https://img.shields.io/badge/Rust-000000?logo=Rust&logoColor=white)
![License](https://img.shields.io/badge/License-MIT-blue.svg)
[![Build Status](https://github.com/d-o-hub/rust-self-learning-memory/workflows/CI/badge.svg)](https://github.com/d-o-hub/rust-self-learning-memory/actions)
[![codecov](https://codecov.io/gh/d-o-hub/rust-self-learning-memory/branch/main/graph/badge.svg)](https://codecov.io/gh/d-o-hub/rust-self-learning-memory)

A self-learning episodic memory system with semantic embeddings and multiple storage backends.

[Overview](#overview) • [Features](#features) • [Quick Start](#quick-start) • [Documentation](#documentation) • [Contributing](#contributing) • [Quality Gates](#quality-gates) • [License](#license)

</div>

## Overview

The Rust Self-Learning Memory System provides persistent memory across agent interactions through an MCP (Model Context Protocol) server. It captures, stores, and learns from episodic experiences to improve future performance.

**Architecture:**
- **memory-core**: Core memory operations and embeddings
- **memory-storage-turso**: Primary database storage (libSQL)
- **memory-storage-redb**: Cache layer
- **memory-mcp**: MCP server implementation
- **memory-cli**: Command-line interface for interaction

**Tech Stack:** Rust/Tokio + Turso/libSQL + redb cache + optional embeddings

## Features

### 🧠 Episodic Memory
- Stores task execution records with context and outcomes
- Learns from patterns across episodes
- Provides context-aware retrieval

### 📚 Multiple Storage Backends
- **Turso Cloud**: Remote libSQL database (default)
- **Local SQLite**: Local file-based database (fallback)
- **In-memory**: Temporary storage for testing

### 🎯 Pattern Recognition
- Extracts patterns from episodes
- Identifies successful strategies
- Adaptive learning from past experiences

### 🔍 Semantic Search
- Optional embedding-based similarity search
- Fast retrieval of relevant past episodes
- Context-aware recommendations

### 🛡️ Quality Assurance
- Automated quality gates (90%+ coverage)
- Comprehensive test suite
- Security auditing
- Performance benchmarks

## Quick Start

### Prerequisites
- Rust (latest stable)
- SQLite (for local development)
- Optional: Turso CLI (for cloud database)

### Installation

```bash
# Clone the repository
git clone https://github.com/d-o-hub/rust-self-learning-memory.git
cd rust-self-learning-memory

# Build the project
cargo build --all

# Run tests
cargo test --all

# Run quality gates
./scripts/quality-gates.sh
```

### Local Database Setup

```bash
# Quick setup with the provided script
./scripts/setup-local-db.sh

# Or manual setup
cp .env.example .env
mkdir -p ./data ./backups
```

### Basic Usage

```bash
# CLI interaction
cargo run --bin memory-cli -- episode store \
  --description "Implement user authentication" \
  --context '{"language": "rust", "domain": "auth"}' \
  --outcome "success" \
  --verdict "Auth system implemented with JWT tokens"

# Retrieve relevant context
cargo run --bin memory-cli -- context retrieve \
  --query "add user authorization" \
  --limit 5
```

## Documentation

| Document | Description |
|----------|-------------|
| [Configuration Guide](memory-cli/CONFIGURATION_GUIDE.md) | Complete configuration options |
| [Database Setup](docs/LOCAL_DATABASE_SETUP.md) | Local database configuration |
| [Quality Gates](docs/QUALITY_GATES.md) | Automated quality standards |
| [YAML Validation](docs/YAML_VALIDATION.md) | Configuration validation strategy |
| [Testing Guide](TESTING.md) | Testing infrastructure and strategies |
| [Contributing](CONTRIBUTING.md) | Development workflow |
| [Security](SECURITY.md) | Security policies and practices |
| [Deployment](DEPLOYMENT.md) | Deployment strategies |

### Agent Documentation

| Document | Description |
|----------|-------------|
| [Building the Project](agent_docs/building_the_project.md) | Build commands and setup |
| [Running Tests](agent_docs/running_tests.md) | Testing strategies and coverage |
| [Code Conventions](agent_docs/code_conventions.md) | Rust idioms and patterns |
| [Service Architecture](agent_docs/service_architecture.md) | System design and components |
| [Database Schema](agent_docs/database_schema.md) | Data structures and relationships |
| [Communication Patterns](agent_docs/service_communication_patterns.md) | Inter-service communication |

## Quality Gates

The project maintains high quality standards through automated quality gates:

| Gate | Threshold | Description |
|------|-----------|-------------|
| **Test Coverage** | > 90% | Line coverage across all crates |
| **Pattern Accuracy** | > 70% | Pattern recognition accuracy |
| **Code Complexity** | Avg < 10 | Average cyclomatic complexity |
| **Security** | 0 vulns | Zero critical/high/medium vulnerabilities |
| **Linting** | 0 warnings | Zero clippy warnings |
| **Formatting** | 100% | All code rustfmt compliant |
| **Performance** | < 10% regression | No performance degradation |

Run quality gates locally:
```bash
./scripts/quality-gates.sh
```

For more details, see [Quality Gates Documentation](docs/QUALITY_GATES.md).

## Configuration

### Environment Variables

```bash
# Turso Cloud (default)
TURSO_DATABASE_URL=libsql://your-db.turso.io
TURSO_AUTH_TOKEN=your-auth-token

# Local SQLite (fallback)
LOCAL_DATABASE_URL=sqlite:./data/memory.db
MEMORY_REDB_PATH=./data/memory.redb

# Cache settings
MEMORY_MAX_EPISODES_CACHE=1000
MEMORY_CACHE_TTL_SECONDS=3600
```

### TOML Configuration

```toml
[database]
turso_url = "libsql://your-db.turso.io"
turso_token = "your-auth-token"

[storage]
max_episodes_cache = 1000
cache_ttl_seconds = 3600
pool_size = 10

[cli]
default_format = "human"
progress_bars = true
batch_size = 100
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Memory CLI                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │   Store     │  │  Retrieve   │  │   Analyze   │         │
│  │   Episode   │  │  Context    │  │   Patterns  │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                     Memory Core                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │   Episode   │  │   Pattern   │  │  Embedding  │         │
│  │ Management  │  │ Extraction  │  │  Service    │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
└─────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
┌───────▼────────┐  ┌────────▼────────┐  ┌────────▼────────┐
│ Turso Storage  │  │  Redb Cache     │  │  In-Memory      │
│                │  │                 │  │                 │
│ libSQL/Remote  │  │   Fast Access   │  │  Temporary      │
└────────────────┘  └─────────────────┘  └─────────────────┘
```

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Workflow

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make your changes
4. Run quality gates: `./scripts/quality-gates.sh`
5. Submit a pull request

### Code Standards

- Follow [Rust idioms](agent_docs/code_conventions.md)
- Maintain 90%+ test coverage
- Run `cargo fmt` and `cargo clippy` before committing
- Document public APIs
- Write descriptive commit messages

### Quality Requirements

- All tests must pass
- No clippy warnings
- 90%+ test coverage
- Security audit must pass
- Performance benchmarks must not degrade >10%

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

- 📖 [Documentation](docs/)
- 🐛 [Issue Tracker](https://github.com/d-o-hub/rust-self-learning-memory/issues)
- 💬 [Discussions](https://github.com/d-o-hub/rust-self-learning-memory/discussions)
- 📧 [Email](mailto:your-email@example.com)

## Acknowledgments

- [libSQL](https://github.com/libsql/libsql) for the embedded database
- [redb](https://github.com/cberner/redb) for the embedded key-value store
- [tokio](https://github.com/tokio-rs/tokio) for asynchronous runtime
- [Turso](https://turso.tech/) for the cloud database service

---

<div align="center">

**Built with ❤️ using Rust**

[Documentation](docs/) • [GitHub](https://github.com/d-o-hub/rust-self-learning-memory)

</div>