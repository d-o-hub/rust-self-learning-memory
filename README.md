# Rust Self-Learning Memory System

<div align="center">

![Rust](https://img.shields.io/badge/Rust-000000?logo=Rust&logoColor=white)
![License](https://img.shields.io/badge/License-MIT-blue.svg)
![Rust Version](https://img.shields.io/badge/Rust-1.75%2B-yellow)
![Last Updated](https://img.shields.io/badge/last%20updated-February%202026-blue)
![Coverage](https://img.shields.io/badge/coverage-92.5%25-brightgreen)
![Clippy](https://img.shields.io/badge/clippy-0%20warnings-success)
![Security Audit](https://img.shields.io/badge/security-audit%20passed-brightgreen)
![Open Issues](https://img.shields.io/github/issues/d-o-hub/rust-self-learning-memory)

**Production Ready** • 99.3% Test Pass Rate • 92.5% Coverage • Zero Clippy Warnings

**NEW:** Semantic Pattern Search & Recommendations 🔍

A self-learning episodic memory system with semantic pattern search, embeddings, MCP server, and secure code execution sandbox.

[Overview](#overview) • [Features](#features) • [Quick Start](#quick-start) • [Documentation](#documentation) • [Contributing](#contributing) • [Quality Gates](#quality-gates) • [License](#license)

</div>

## Overview

The Rust Self-Learning Memory System provides persistent memory across agent interactions through a comprehensive MCP (Model Context Protocol) server with secure code execution. It captures, stores, and learns from episodic experiences to improve future performance.

**Current Status (v0.1.14):**
- **Production-ready** episodic memory management system for AI agents
- **9 workspace members**: memory-core, memory-storage-turso, memory-storage-redb, memory-mcp, memory-cli, test-utils, benches, examples, tests
- **632 Rust source files** with ~140,000 lines of code
- **811+ lib tests** with 99.5% pass rate
- **92.5% test coverage** across all modules
- **10-100x performance improvements** over baseline measurements
- **Zero clippy warnings** with strict linting rules
- **Multi-provider semantic embeddings** with OpenAI, Mistral, and local backends
- **Dual storage backends**: Turso for durability, redb for cache
- **6-layer security sandbox** in Wasmtime for safe code execution
- **Phase 2 Turso Optimization**: 100% complete (connection pooling, adaptive sizing, compression, adaptive TTL)
- **CI Status**: ALL PASSING (Nightly Full Tests FIXED in #283)
- **MCP Token Optimization**: 98% token reduction for tool discovery

**Architecture:**
- **memory-core**: Core memory operations, pattern extraction, and reward scoring
- **memory-storage-turso**: Primary database storage (libSQL)
- **memory-storage-redb**: Fast embedded cache layer
- **memory-mcp**: MCP server with secure WASM sandbox
- **memory-cli**: Full-featured command-line interface (9 commands, 9 aliases)
- **test-utils**: Shared testing utilities
- **benches**: Comprehensive benchmark suite
- **examples**: Usage examples and demonstrations

**Tech Stack:** Rust/Tokio + Turso/libSQL + redb cache + Wasmtime WASM + optional embeddings (OpenAI, Mistral, local)

## Features

### 🧠 Episodic Memory
- Complete episode lifecycle (start → execute → score → learn → retrieve)
- Detailed execution step logging with tool usage tracking
- Intelligent reward scoring with efficiency and quality bonuses
- Automatic reflection generation for learning

### 📚 Multiple Storage Backends
- **Turso Cloud**: Remote libSQL database (default)
- **redb Cache**: Fast embedded key-value storage
- **Local SQLite**: Local file-based database (fallback)
- Automatic caching with TTL-based invalidation

### 🎯 Pattern Recognition & Semantic Search
- Four pattern types: ToolSequence, DecisionPoint, ErrorRecovery, ContextPattern
- **NEW: Semantic pattern search** with natural language queries
- **NEW: Intelligent pattern recommendations** for tasks using multi-signal ranking
- **NEW: Cross-domain pattern discovery** to find analogous patterns
- Async pattern extraction with queue-based workers
- Pattern effectiveness tracking with decay over time
- Multi-signal ranking: semantic similarity, context match, effectiveness, recency, success rate
- Minimum success rate filtering (default 70%)

### 🔒 Secure Code Sandbox
- Wasmtime WASM sandbox for safe code execution
- Resource limits (timeout, memory, CPU)
- Defense-in-depth security with access controls
- Support for concurrent executions (20 parallel by default)

### 📊 Advanced Analysis
- Statistical analysis (ETS forecasting, MSTL decomposition)
- Anomaly detection and changepoint analysis
- Time series forecasting with configurable horizon
- Causal inference for pattern relationships

### 🔍 MCP Server
- Standard MCP protocol implementation (v2024-11)
- **8 MCP tools** for memory operations, pattern search, and code execution
- **NEW: `search_patterns`** - Semantic pattern search with configurable ranking
- **NEW: `recommend_patterns`** - Task-specific pattern recommendations
- Progressive tool disclosure based on usage
- Execution monitoring and metrics tracking
- Wasmtime-based WASM sandbox for secure code execution

### 🛠️ Full-Featured CLI
- 9 main commands for episode, pattern, and storage management
- 9 command aliases for rapid development workflow
- Episode management (create, list, search, complete)
- Pattern analysis and effectiveness tracking
- Storage operations (sync, vacuum, health checks)
- Backup and restore capabilities
- Monitoring and metrics export
- Multiple output formats (human, JSON, YAML)

### 🌐 Multi-Provider Embeddings
- OpenAI embeddings integration (text-embedding-3-small, text-embedding-3-large, ada-002)
- Mistral AI embeddings integration
- Local CPU-based embeddings
- Semantic search with cosine similarity
- Automatic embedding caching and batch processing

### 🛡️ Quality Assurance
- Automated quality gates (>90% coverage)
- Comprehensive test suite across all crates (811+ lib tests)
- Security auditing for sandbox operations
- Performance benchmarks with regression detection
- Zero clippy warnings policy
- Pre-commit and post-commit hooks for code quality

## Quick Start

### 🔍 Pattern Search Example (NEW in v0.1.13)

```rust
use memory_core::{SelfLearningMemory, TaskContext, ComplexityLevel};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let memory = SelfLearningMemory::new();
    
    // Search for patterns using natural language
    let context = TaskContext {
        domain: "web-api".to_string(),
        language: Some("rust".to_string()),
        framework: None,
        complexity: ComplexityLevel::Moderate,
        tags: vec!["rest".to_string(), "async".to_string()],
    };
    
    let results = memory.search_patterns_semantic(
        "How to handle API rate limiting with retries",
        context,
        5  // limit
    ).await?;
    
    for result in results {
        println!("Pattern: {:?}", result.pattern);
        println!("Relevance: {:.2}", result.relevance_score);
        println!("Success Rate: {:.1}%", result.pattern.success_rate() * 100.0);
    }
    
    // Get task-specific recommendations
    let recommendations = memory.recommend_patterns_for_task(
        "Build async HTTP client with connection pooling",
        context,
        3
    ).await?;
    
    for rec in recommendations {
        println!("Recommended: {:?}", rec.pattern);
    }
    
    Ok(())
}
```

**Try it yourself:** `cargo run --example pattern_search_demo`

**Documentation:** See `memory-core/PATTERN_SEARCH_FEATURE.md` for complete API reference and examples.

### Prerequisites
- Rust (latest stable)
- SQLite (for local development)
- Optional: Turso CLI (for cloud database)
- Optional: Node.js (for JavaScript sandbox features)

### Installation

```bash
# Clone the repository
git clone https://github.com/d-o-hub/rust-self-learning-memory.git
cd rust-self-learning-memory

# Build the project
cargo build --release

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

#### Setup Configuration

```bash
# Run interactive configuration wizard
memory config wizard

# Follow the prompts to configure:
# - Database (local SQLite or remote Turso)
# - Storage (cache size, TTL, connection pool)
# - CLI (output format, progress bars, batch size)

# Validate configuration
memory config validate

# Check configuration status
memory config check
```

**NEW:** Configuration Wizard - Interactive step-by-step setup with sensible defaults and validation.

#### CLI Interaction

```bash
# Create an episode
memory-cli episode create --task "Implement user authentication" --context '{"language": "rust", "domain": "auth"}'

# List episodes
memory-cli episode list --limit 10

# Retrieve relevant context
memory-cli episode search "authentication" --limit 5

# NEW: Search patterns semantically
memory-cli pattern search --query "How to build REST API" --domain web-api --limit 5

# NEW: Get pattern recommendations
memory-cli pattern recommend --task "Build async HTTP client" --domain web-api --limit 3

# Analyze patterns
memory-cli pattern list --min-confidence 0.8
```

#### MCP Server

```bash
# Start the MCP server
cargo run --bin memory-mcp-server

# Or run with custom config
cargo run --bin memory-mcp-server -- --config mcp-config-memory.json
```

#### Programmatic Usage

```rust
use memory_core::{SelfLearningMemory, TaskContext, TaskType};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let memory = SelfLearningMemory::new(Default::default()).await?;

    let context = TaskContext {
        language: "rust".to_string(),
        domain: "web".to_string(),
        tags: vec!["api".to_string()],
    };

    let episode_id = memory.start_episode(
        "Build REST API endpoint".to_string(),
        context,
        TaskType::CodeGeneration,
    ).await;

    Ok(())
}
```

## Documentation

| Document | Description |
|----------|-------------|
| [Configuration Wizard](docs/CONFIG_WIZARD.md) | Interactive setup guide |
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

### Crate Documentation

| Crate | Description |
|-------|-------------|
| [memory-core](memory-core/README.md) | Core episodic learning system |
| [memory-mcp](memory-mcp/README.md) | MCP server with secure sandbox |
| [memory-cli](memory-cli/README.md) | Command-line interface |
| [memory-storage-turso](memory-storage-turso/README.md) | Turso/libSQL storage backend |
| [memory-storage-redb](memory-storage-redb/README.md) | redb cache backend |

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

## Feature Flags

Enable optional features via Cargo:

```bash
# Basic features (default)
cargo build

# All features
cargo build --all-features

# Specific features
cargo build --features openai
cargo build --features mistral
cargo build --features local-embeddings
cargo build --features embeddings-full
```

**Available Features:**
- `openai`: OpenAI API embeddings support
- `mistral`: Mistral AI embeddings support
- `local-embeddings`: CPU-based local embeddings
- `embeddings-full`: All embedding providers (openai + mistral)
- `mcp`: MCP server tools and protocol support
- `sandbox`: Wasmtime sandbox for code execution

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

# Sandbox settings
MCP_USE_WASM=true
JAVY_PLUGIN=./memory-mcp/javy-plugin.wasm
```

### TOML Configuration

```toml
[database]
turso_url = "libsql://your-db.turso.io"
turso_token = "your-auth-token"
redb_path = "memory.redb"

[storage]
max_episodes_cache = 1000
cache_ttl_seconds = 3600
pool_size = 10

[sandbox]
max_execution_time_ms = 5000
max_memory_mb = 128
max_cpu_percent = 50
allow_network = false
allow_filesystem = false

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
│  │   Episode   │  │  Pattern    │  │   Storage   │         │
│  │ Management  │  │  Analysis   │  │ Operations  │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
└─────────────────────────────────────────────────────────────┘
                               │
┌─────────────────────────────────────────────────────────────┐
│                     Memory MCP Server                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │  MCP Tools  │  │  WASM       │  │  Advanced   │         │
│  │  Interface  │  │  Sandbox    │  │  Analysis   │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
└─────────────────────────────────────────────────────────────┘
                               │
┌─────────────────────────────────────────────────────────────┐
│                     Memory Core                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │   Episode   │  │   Pattern   │  │   Reward    │         │
│  │ Management  │  │ Extraction  │  │   Scoring   │         │
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

## MCP Server Tools

The MCP server exposes the following tools:

### query_memory
Query episodic memory for relevant past experiences based on task type, domain, and query text.

### analyze_patterns
Analyze patterns from past episodes to identify successful strategies and recommendations.

### execute_agent_code
Execute TypeScript/JavaScript code in a secure WASM sandbox with resource limits.

### Advanced Pattern Analysis
Statistical analysis, forecasting, anomaly detection, and causal inference on time series data from memory episodes.

## Performance

All operations meet or exceed performance targets:

| Operation | Target (P95) | Typical Performance |
|-----------|-------------|---------------------|
| Episode Creation | < 50ms | ~2.5 µs (19,531x faster) |
| Step Logging | < 20ms | ~1.1 µs (17,699x faster) |
| Episode Completion | < 500ms | ~3.8 µs (130,890x faster) |
| Pattern Extraction | < 1000ms | ~10.4 µs (95,880x faster) |
| Memory Retrieval | < 100ms | ~721 µs (138x faster) |
| WASM Execution | < 200ms | ~50-200ms (typical) |

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

## Acknowledgments

- [libSQL](https://github.com/libsql/libsql) for the embedded database
- [redb](https://github.com/cberner/redb) for the embedded key-value store
- [tokio](https://github.com/tokio-rs/tokio) for asynchronous runtime
- [Turso](https://turso.tech/) for the cloud database service
- [Wasmtime](https://github.com/bytecodealliance/wasmtime) for the secure WASM runtime
- [Javy](https://github.com/bytecodealliance/javy) for JavaScript compilation

---

<div align="center">

[Documentation](docs/) • [GitHub](https://github.com/d-o-hub/rust-self-learning-memory)

</div>
