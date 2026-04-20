# Rust Self-Learning Memory System

<div align="center">

![Rust](https://img.shields.io/badge/Rust-000000?logo=Rust&logoColor=white)
![License](https://img.shields.io/badge/License-MIT-blue.svg)
![Rust Version](https://img.shields.io/badge/Rust-stable%20(2024%20edition)-yellow)
![Last Updated](https://img.shields.io/badge/last%20updated-March%202026-blue)
![Quick Check](https://github.com/d-o-hub/rust-self-learning-memory/actions/workflows/quick-check.yml/badge.svg)
![Security](https://github.com/d-o-hub/rust-self-learning-memory/actions/workflows/security.yml/badge.svg)
[![Coverage](https://codecov.io/gh/d-o-hub/rust-self-learning-memory/branch/main/graph/badge.svg)](https://codecov.io/gh/d-o-hub/rust-self-learning-memory)
![Open Issues](https://img.shields.io/github/issues/d-o-hub/rust-self-learning-memory)

A self-learning episodic memory system with semantic pattern search, embeddings, MCP server, and optional sandboxed code execution.

[Overview](#overview) • [Features](#features) • [Quick Start](#quick-start) • [Documentation](#documentation) • [Contributing](#contributing) • [Quality Gates](#quality-gates) • [License](#license)

</div>

## Overview

The Rust Self-Learning Memory System provides persistent memory across agent interactions through a comprehensive MCP (Model Context Protocol) server. It captures, stores, and learns from episodic experiences to improve future performance.

**Architecture:**
- **do-memory-core**: Core memory operations, pattern extraction, and reward scoring
- **do-memory-storage-turso**: Primary database storage (libSQL)
- **do-memory-storage-redb**: Fast embedded cache layer
- **do-memory-mcp**: MCP server with secure WASM sandbox
- **do-memory-cli**: Full-featured command-line interface (episode, pattern, storage, playbook, feedback, and more)
- **do-memory-test-utils**: Shared testing utilities
- **do-memory-benches**: Comprehensive benchmark suite
- **do-memory-examples**: Usage examples and demonstrations

**Tech Stack:** Rust 2024 edition / Tokio + Turso/libSQL + redb cache + Wasmtime WASM + optional embeddings (OpenAI, Mistral, local)

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
- Semantic pattern search with natural language queries
- Intelligent pattern recommendations for tasks using multi-signal ranking
- Cross-domain pattern discovery to find analogous patterns
- Async pattern extraction with queue-based workers
- Pattern effectiveness tracking with decay over time
- Multi-signal ranking: semantic similarity, context match, effectiveness, recency, success rate
- Minimum success rate filtering (default 70%)

### 🔒 Secure Code Sandbox (conditional)
- Wasmtime WASM sandbox for safe code execution when enabled/available
- Resource limits (timeout, memory, CPU)
- Defense-in-depth security with access controls
- Support for concurrent executions (20 parallel by default)

### 📊 Advanced Analysis
- Statistical analysis (ETS forecasting, MSTL decomposition)
- Anomaly detection and changepoint analysis
- Time series forecasting with configurable horizon
- Causal inference for pattern relationships

### 🔍 MCP Server
- MCP protocol implementation (v2025-11-25) with lazy tool loading
- **MCP tools** for memory operations, pattern search, and code execution
- **`search_patterns`** - Semantic pattern search with configurable ranking
- **`recommend_patterns`** - Task-specific pattern recommendations
- **`recommend_playbook`** - Actionable step-by-step guidance (ADR-044)
- **`checkpoint_episode`** - Mid-task progress snapshots (ADR-044)
- Embedding tools: configure, test, generate, search, provider-status
- Progressive tool disclosure based on usage
- Execution monitoring and metrics tracking

### 🛠️ Full-Featured CLI
- Top-level command groups include: episode, pattern, storage, config, health, backup, monitor, logs, eval, embedding, completion, tag, relationship, playbook, feedback
- Episode management (create, list, view, search, complete, delete, update, bulk, filter)
- Pattern analysis and effectiveness tracking (list, view, analyze, decay, batch)
- Tag management (add, remove, search, rename, stats)
- Relationship management (add, remove, list, graph, validate)
- Storage operations (stats, sync, vacuum, health, connections)
- Backup and restore capabilities
- Multiple output formats (human, JSON, YAML)

### 🌐 Multi-Provider Embeddings
- OpenAI embeddings integration (text-embedding-3-small, text-embedding-3-large, ada-002)
- Mistral AI embeddings integration
- Local CPU-based embeddings
- Semantic search with cosine similarity
- Automatic embedding caching and batch processing

### 🛡️ Quality Assurance
- Automated quality gates (`./scripts/quality-gates.sh`)
- ~2,900 test functions across all crates (cargo-nextest)
- Property-based testing (proptest) and snapshot testing (insta)
- Mutation testing (cargo-mutants) in nightly CI
- Security auditing and semver checking in CI
- Zero clippy warnings policy
- Pre-commit hooks for code quality

## Quick Start

### 🔍 Pattern Search Example

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

    // NEW (ADR-044): Generate an actionable playbook
    let playbooks = memory.retrieve_playbooks(
        "Implement user authentication",
        "security",
        TaskType::CodeGeneration,
        context,
        1, // max playbooks
        5  // max steps
    ).await;

    if let Some(playbook) = playbooks.first() {
        println!("Playbook ID: {}", playbook.playbook_id);
        for step in &playbook.ordered_steps {
            println!("{}. {}", step.order, step.action);
        }
    }
    
    Ok(())
}
```

**Documentation:** See `do-memory-core/PATTERN_SEARCH_FEATURE.md` for complete API reference and examples.

### Prerequisites
- Rust stable (2024 edition)
- SQLite (for local development)
- Optional: Turso CLI (for cloud database)

### Installation

```bash
# Clone the repository
git clone https://github.com/d-o-hub/rust-self-learning-memory.git
cd rust-self-learning-memory

# Build the project
cargo build --release

# Run tests (nextest recommended)
cargo nextest run --all
# Doctests separately
cargo test --doc

# Run quality gates
./scripts/quality-gates.sh
```

### Local Database Setup

```bash
# Quick setup with the provided script
./scripts/setup-local-db.sh

# Or manual setup
cp do-memory-cli/.env.example .env
mkdir -p ./data ./backups
```

### Basic Usage

#### Setup Configuration

```bash
# Run interactive configuration wizard
do-memory-cli config wizard

# Follow the prompts to configure:
# - Database (local SQLite or remote Turso)
# - Storage (cache size, TTL, connection pool)
# - CLI (output format, progress bars, batch size)

# Validate configuration
do-memory-cli config validate

# Check configuration status
do-memory-cli config check
```

Configuration Wizard provides interactive step-by-step setup with sensible defaults and validation.

#### CLI Interaction

```bash
# Create an episode
do-memory-cli episode create --task "Implement user authentication" --context '{"language": "rust", "domain": "auth"}'

# List episodes
do-memory-cli episode list --limit 10

# Search episodes
do-memory-cli episode search "authentication" --limit 5

# Search patterns semantically
do-memory-cli pattern search --query "How to build REST API" --limit 5

# Analyze patterns
do-memory-cli pattern list --min-confidence 0.8

# Tag management
do-memory-cli tag add <episode-id> "important"
do-memory-cli tag search "important"

# Health check
do-memory-cli health check

# Playbook recommendation
do-memory-cli playbook recommend "Implement JWT auth" --domain security
```

#### MCP Server

```bash
# Start the MCP server
cargo run --bin do-memory-mcp-server

# Or run with custom config
cargo run --bin do-memory-mcp-server -- --config mcp-config-memory.json
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
| [API Reference](docs/API_REFERENCE.md) | Current MCP tool contract index |
| [Configuration Guide](do-memory-cli/CONFIGURATION_GUIDE.md) | Complete configuration options |
| [Database Setup](docs/LOCAL_DATABASE_SETUP.md) | Local database configuration |
| [Quality Gates](docs/QUALITY_GATES.md) | Automated quality standards |
| [YAML Validation](docs/YAML_VALIDATION.md) | Configuration validation strategy |
| [Testing Guide](agent_docs/running_tests.md) | Testing infrastructure and strategies |
| [Contributing](CONTRIBUTING.md) | Development workflow |
| [Security](SECURITY.md) | Security policies and practices |
| [Deployment](DEPLOYMENT.md) | Deployment strategies |
| [Release Engineering](plans/adr/ADR-034-Release-Engineering-Modernization.md) | Release workflow and automation |
| [Playbooks & Checkpoints](docs/PLAYBOOKS_AND_CHECKPOINTS.md) | Actionable memory and handoff |

### Agent Documentation

| Document | Description |
|----------|-------------|
| [Building the Project](agent_docs/building_the_project.md) | Build commands and setup |
| [Running Tests](agent_docs/running_tests.md) | Testing strategies and coverage |
| [Code Conventions](agent_docs/code_conventions.md) | Rust idioms and patterns |
| [Service Architecture](agent_docs/service_architecture.md) | System design and components |
| [Database Schema](agent_docs/database_schema.md) | Data structures and relationships |
| [Communication Patterns](agent_docs/service_communication_patterns.md) | Inter-service communication |
| [Agent Docs Index](agent_docs/README.md) | Workflow docs and high-impact reference files |

### Crate Documentation

| Crate | Description |
|-------|-------------|
| [do-memory-core](do-memory-core/README.md) | Core episodic learning system |
| [do-memory-mcp](do-memory-mcp/README.md) | MCP server with secure sandbox |
| [do-memory-cli](do-memory-cli/README.md) | Command-line interface |
| [do-memory-storage-turso](do-memory-storage-turso/README.md) | Turso/libSQL storage backend |
| [do-memory-storage-redb](do-memory-storage-redb/README.md) | redb cache backend |

## Quality Gates

The project maintains high quality standards through automated quality gates:

| Gate | Threshold | Description |
|------|-----------|-------------|
| **Build** | 0 errors | `cargo build --all` |
| **Linting** | 0 warnings | `./scripts/code-quality.sh clippy --workspace` |
| **Formatting** | 100% | `./scripts/code-quality.sh fmt` |
| **Tests** | All pass | `cargo nextest run --all` |
| **File Size** | ≤500 LOC | Production source files only |
| **Security** | 0 vulns | `cargo audit` in CI |
| **Semver** | No breaks | `cargo semver-checks` in CI |

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
- `openai`: OpenAI API embeddings support (do-memory-core)
- `mistral`: Mistral AI embeddings support (do-memory-core)
- `local-embeddings`: CPU-based local embeddings (do-memory-core)
- `embeddings-full`: All embedding providers (do-memory-core)
- `turso`: Turso cloud storage with keepalive pool (do-memory-cli)
- `redb`: redb local cache layer (do-memory-cli, default)
- `full`: All features combined (do-memory-cli)
- `wasmtime-backend`: Wasmtime WASM sandbox (do-memory-mcp, default)
- `compression`: Network compression — lz4, zstd, gzip (do-memory-storage-turso)
- `hybrid_search`: FTS5 hybrid search (do-memory-storage-turso)

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

# CLI config
MEMORY_CLI_CONFIG=./do-memory-cli.toml

# Embeddings (CLI/MCP)
EMBEDDING_PROVIDER=openai|mistral|azure|local
OPENAI_API_KEY=sk-your-key
MISTRAL_API_KEY=your-mistral-key
AZURE_OPENAI_API_KEY=your-azure-key
OPENAI_API_KEY_ENV=OPENAI_API_KEY
EMBEDDING_MODEL=text-embedding-3-small
EMBEDDING_SIMILARITY_THRESHOLD=0.7
EMBEDDING_BATCH_SIZE=32

# Sandbox settings
MCP_USE_WASM=true
JAVY_PLUGIN=./do-memory-mcp/javy-plugin.wasm
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
│                      Memory CLI                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐          │
│  │   Episode   │  │  Pattern    │  │   Storage   │          │
│  │ Management  │  │  Analysis   │  │ Operations  │          │
│  └─────────────┘  └─────────────┘  └─────────────┘          │
└─────────────────────────────────────────────────────────────┘
                               │
┌─────────────────────────────────────────────────────────────┐
│                     Memory MCP Server                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐          │
│  │  MCP Tools  │  │  WASM       │  │  Advanced   │          │
│  │  Interface  │  │  Sandbox    │  │  Analysis   │          │
│  └─────────────┘  └─────────────┘  └─────────────┘          │
└─────────────────────────────────────────────────────────────┘
                               │
┌─────────────────────────────────────────────────────────────┐
│                     Memory Core                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐          │
│  │   Episode   │  │   Pattern   │  │   Reward    │          │
│  │ Management  │  │ Extraction  │  │   Scoring   │          │
│  └─────────────┘  └─────────────┘  └─────────────┘          │
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

The MCP server exposes tools via lazy loading (ADR-024):

- **query_memory** — Query episodic memory for relevant past experiences
- **analyze_patterns** — Identify successful strategies and recommendations
- **search_patterns** — Semantic pattern search with multi-signal ranking
- **recommend_patterns** — Task-specific pattern recommendations
- **configure_embeddings** / **test_embeddings** / **generate_embedding** — Embedding management
- **search_by_embedding** / **embedding_provider_status** — Semantic search and provider monitoring
- **Episode lifecycle tools** — create, complete, log steps, get, timeline
- **Advanced analysis** — Statistical analysis, forecasting, anomaly detection, causal inference

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

Typical performance numbers are from internal benchmarks on a warm cache; results vary by hardware and configuration. Run `cargo bench` for local measurements and see `docs/QUALITY_GATES.md` for the performance regression gate.

## Benchmarks

Run `cargo bench` for workspace benchmarks. CLI benchmarks live in `do-memory-cli/benches/cli_benchmarks.rs`; quality gate expectations and regression checks are documented in `docs/QUALITY_GATES.md`.

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
