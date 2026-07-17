# Rust Self-Learning Memory System

<div align="center">

![Rust](https://img.shields.io/badge/Rust-000000?logo=Rust&logoColor=white)
![Version](https://img.shields.io/badge/version-v0.1.35-orange)
![License](https://img.shields.io/badge/License-MIT-blue.svg)
![Quick Check](https://github.com/d-o-hub/rust-self-learning-memory/actions/workflows/quick-check.yml/badge.svg)
![Security](https://github.com/d-o-hub/rust-self-learning-memory/actions/workflows/security.yml/badge.svg)

A self-learning episodic memory system with semantic pattern search, embeddings, MCP server, and a full-featured CLI.

[Overview](#overview) • [Features](#features) • [Quick Start](#quick-start) • [Documentation](#documentation) • [Contributing](#contributing) • [Quality Gates](#quality-gates) • [License](#license)

</div>

## Overview

The Rust Self-Learning Memory System provides persistent memory across agent interactions through a comprehensive MCP (Model Context Protocol) server. It captures, stores, and learns from episodic experiences to improve future performance.

**Architecture:**
- **do-memory-core**: Core memory operations, pattern extraction, and reward scoring
- **do-memory-storage-turso**: Primary database storage (libSQL)
- **do-memory-storage-redb**: Fast embedded cache layer
- **do-memory-mcp**: MCP server (lazy tool loading; code-exec fail-closed)
- **do-memory-cli**: Full-featured command-line interface (episode, pattern, storage, playbook, feedback, and more)
- **do-memory-test-utils**: Shared testing utilities
- **do-memory-benches**: Comprehensive benchmark suite
- **do-memory-examples**: Usage examples and demonstrations

**Tech Stack:** Rust 2024 edition / Tokio + Turso/libSQL + redb cache + optional embeddings (OpenAI, Mistral, local)  
**Latest release:** [v0.1.35](https://github.com/d-o-hub/rust-self-learning-memory/releases/tag/v0.1.35) (workspace version `0.1.35`)

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

### 🔄 Episode Checkpoints and Handoff
- Checkpoint episodes mid-task for long-running workflows
- Generate handoff packs to transfer state between agents or sessions
- Resume execution from saved checkpoints
- State preservation with findings and pending actions

### 🌊 CSM Cascading Retrieval
- 100% CPU-local retrieval via Chaotic Semantic Memory (CSM)
- 4-tier cascade (BM25 -> HDC -> ConceptGraph -> API Embeddings)
- Hyperdimensional computing (HDC) binary vectors for zero-API similarity search
- 50-70% reduction in external API embedding calls
- Semantic pattern search with natural language queries
- Intelligent pattern recommendations for tasks using multi-signal ranking
- Cross-domain pattern discovery to find analogous patterns
- Async pattern extraction with queue-based workers
- Pattern effectiveness tracking with decay over time
- Multi-signal ranking: semantic similarity, context match, effectiveness, recency, success rate
- Minimum success rate filtering (default 70%)

### 🔒 Agent Code Execution (fail-closed)
- `execute_agent_code` is **not** a working execution backend; calls fail closed
- Prefer episode lifecycle tools and external runners for agent code
- Historical WASM/Javy sandbox paths are removed from the supported feature set

### 📊 Advanced Analysis
- Statistical analysis (ETS forecasting, MSTL decomposition)
- Anomaly detection and changepoint analysis
- Time series forecasting with configurable horizon
- Causal inference for pattern relationships

### 🔍 MCP Server
- MCP protocol implementation (v2025-11-25) with lazy tool loading
- **MCP tools** for memory operations, pattern search, episodes, and embeddings
- **`search_patterns`** - Semantic pattern search with configurable ranking
- **`recommend_patterns`** - Task-specific pattern recommendations
- **`recommend_playbook`** - Actionable step-by-step guidance
- **`checkpoint_episode`** - Mid-task progress snapshots
- Embedding tools: configure, test, generate, search, provider-status
- Progressive tool disclosure based on usage
- Execution monitoring and metrics tracking

### 🛠️ Full-Featured CLI
- Top-level command groups include: episode, pattern, storage, config, health, backup, monitor, logs, eval, embedding, completion, tag, relationship, playbook, feedback
- Episode management (create, list, view, search, log-step, complete, **fail**, delete, update, bulk, filter)
- Pattern analysis and effectiveness tracking (list, view, search, analyze, decay, batch)
- Discoverable config (`config init` / `config show-template`) and documented precedence (flags → env → config → defaults)
- Tag management (add, remove, search, rename, stats)
- Relationship management (add, remove, list, graph, validate)
- Storage operations (stats, sync, vacuum, health, connections) — **sync** is Turso↔redb only, not pattern extraction
- Backup and restore capabilities
- Multiple output formats (human, JSON, YAML)

### 🌐 Multi-Provider Embeddings
- OpenAI embeddings integration (text-embedding-3-small, text-embedding-3-large, ada-002)
- Mistral AI embeddings integration
- Local CPU-based embeddings
- Semantic search with cosine similarity
- Automatic embedding caching and batch processing

### 🛡️ Quality Assurance
- Automated quality gates (`./scripts/quality-gates.sh`; see `plans/GATE_CONTRACT.md`)
- Large nextest suite across crates; doctests via `cargo test --doc`
- Property-based testing (proptest) and snapshot testing (insta)
- Mutation testing (cargo-mutants) in nightly CI
- Blocking advisory gate via `cargo deny`; semver checks in CI
- Zero clippy warnings policy (`-D warnings`)
- Skill eval contract: `./scripts/run-evals.sh` (strict non-noop tests)

## Quick Start

### 🔍 Pattern Search Example

```rust
use do_memory_core::{SelfLearningMemory, TaskContext, ComplexityLevel};

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

    // Generate an actionable playbook
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

**Documentation:** See `memory-core/PATTERN_SEARCH_FEATURE.md` for complete API reference and examples.

### Prerequisites
- Rust toolchain via [rustup](https://rustup.rs/) (stable, 2024 edition) — only needed if building from source
- SQLite (for local development)
- Optional: Turso CLI (for cloud database)

### Install Pre-Built Binary

Pre-built binaries are available for all major platforms from [GitHub Releases](https://github.com/d-o-hub/rust-self-learning-memory/releases). No Rust toolchain required.

| Platform | Download |
|----------|----------|
| Linux x64 | [do-memory-cli-x86_64-unknown-linux-gnu.tar.xz](https://github.com/d-o-hub/rust-self-learning-memory/releases/latest/download/do-memory-cli-x86_64-unknown-linux-gnu.tar.xz) |
| Linux ARM64 | [do-memory-cli-aarch64-unknown-linux-gnu.tar.xz](https://github.com/d-o-hub/rust-self-learning-memory/releases/latest/download/do-memory-cli-aarch64-unknown-linux-gnu.tar.xz) |
| macOS Intel | [do-memory-cli-x86_64-apple-darwin.tar.xz](https://github.com/d-o-hub/rust-self-learning-memory/releases/latest/download/do-memory-cli-x86_64-apple-darwin.tar.xz) |
| macOS Apple Silicon | [do-memory-cli-aarch64-apple-darwin.tar.xz](https://github.com/d-o-hub/rust-self-learning-memory/releases/latest/download/do-memory-cli-aarch64-apple-darwin.tar.xz) |
| Windows x64 | [do-memory-cli-x86_64-pc-windows-msvc.zip](https://github.com/d-o-hub/rust-self-learning-memory/releases/latest/download/do-memory-cli-x86_64-pc-windows-msvc.zip) |

```bash
# Example: Linux x64
curl -sSL https://github.com/d-o-hub/rust-self-learning-memory/releases/latest/download/do-memory-cli-x86_64-unknown-linux-gnu.tar.xz | tar -xJ
# Add to PATH
export PATH="$PWD:$PATH"
```

### Build from Source

The following system packages are required to compile from source:

| Platform | Command |
|----------|---------|
| Ubuntu/Debian | `sudo apt-get install libssl-dev pkg-config` |
| Fedora/RHEL | `sudo dnf install openssl-devel pkg-config` |
| Arch Linux | `sudo pacman -S openssl pkg-config` |
| macOS | `brew install openssl pkg-config` (usually pre-installed with Xcode Command Line Tools) |

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

### Local Database Setup (Zero Credentials)

The system supports a first-class local-only mode that requires no Turso account or cloud credentials.

```bash
# Via CLI (using auto-detected OS data directory)
do-memory-cli --storage-mode local episode list

# Via CLI (specifying a custom database path)
do-memory-cli --storage-mode local --db-path ./data/memory.db episode list

# Via Environment Variables
export MEMORY_STORAGE_MODE=local
export MEMORY_DB_PATH=./data/memory.db
do-memory-cli episode list
```

> **Note on `--db-path` / `MEMORY_DB_PATH`**: these set the local database
> file path. For the default `local` (redb) backend they set the redb cache
> file; for Turso `remote` mode with a local file they set the Turso SQLite
> path. The config-file equivalent is `[database].redb_path` /
> `[database].db_path`. See [TOML Configuration](#toml-configuration).

For programmatic usage, see the [Local Development example](#local-offline-development).

### Basic Usage

#### Setup Configuration

```bash
# Run interactive configuration wizard
do-memory-cli config wizard

# Follow the prompts to configure:
# - Database (local SQLite or remote Turso)
# - Storage (cache size, TTL, connection pool)
# - CLI (output format, progress bars, batch size)

# Generate a starter config file
do-memory-cli config init

# Print a starter config template
do-memory-cli config show-template

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

# Log steps (needed for tool-sequence pattern extraction)
do-memory-cli episode log-step <episode-id> --tool cargo --action "run tests" --success true

# Complete with outcome (success | partial-success | failure)
do-memory-cli episode complete <episode-id> success

# Force-fail abandoned in_progress rows (operator path; ADR-075)
do-memory-cli episode fail <episode-id>

# List / search episodes
do-memory-cli episode list --limit 10
do-memory-cli episode search "authentication" --limit 5

# Patterns (populated after complete with steps; empty list explains why)
do-memory-cli pattern list --min-confidence 0.8
do-memory-cli pattern search --query "How to build REST API" --limit 5

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

##### Local Offline Development

```rust
use do_memory_core::SelfLearningMemory;
use do_memory_storage_turso::SelfLearningMemoryExt; // Required for convenience constructors

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Local file-based storage
    let memory = SelfLearningMemory::with_local_storage("./data/memory.db").await?;

    // 2. Or in-memory storage (ephemeral)
    // let memory = SelfLearningMemory::with_in_memory_storage().await?;

    Ok(())
}
```

**Episode Recording**

```rust
use do_memory_core::{SelfLearningMemory, TaskContext, TaskType, ComplexityLevel};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let memory = SelfLearningMemory::new();

    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: None,
        complexity: ComplexityLevel::Moderate,
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
| [Configuration Guide](memory-cli/CONFIGURATION_GUIDE.md) | Complete configuration options |
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
| [do-memory-core](memory-core/README.md) | Core episodic learning system |
| [do-memory-mcp](memory-mcp/README.md) | MCP server with secure sandbox |
| [do-memory-cli](memory-cli/README.md) | Command-line interface |
| [do-memory-storage-turso](memory-storage-turso/README.md) | Turso/libSQL storage backend |
| [do-memory-storage-redb](memory-storage-redb/README.md) | redb cache backend |

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
- `csm`: Chaotic Semantic Memory integration for cascading retrieval (do-memory-core)
- `embeddings-full`: All embedding providers (do-memory-core)
- `turso`: Turso cloud storage with keepalive pool (do-memory-cli)
- `redb`: redb local cache layer (do-memory-cli, default)
- `full`: All features combined (do-memory-cli)
- `oauth`: Optional OAuth authorization support (do-memory-mcp)
- `embeddings`: Optional embeddings configuration support (do-memory-mcp)
- `compression`: Network compression — lz4, zstd, gzip (do-memory-storage-turso)
- `hybrid_search`: FTS5 hybrid search (do-memory-storage-turso)

### Cascading Retrieval (with `csm` feature)

When the `csm` feature is enabled, semantic search uses a 4-tier cascade to minimize API calls:

| Tier | Method | CPU Cost | API Calls | When Used |
|------|--------|----------|-----------|-----------|
| 1 | BM25 exact match | O(n) Rayon scan | 0 | Keyword-heavy queries |
| 2 | HDC similarity | 10,240-bit SIMD | 0 | Semantic fallback |
| 3 | ConceptGraph expansion | Graph BFS | 0 | Known-domain synonyms |
| 4 | API embedding | Network call | 1 | Final fallback |

**Target**: 50-70% API call reduction for typical query workloads. See `agent_docs/csm_integration.md` for details.

## Configuration

### Configuration Precedence

Settings resolve highest-wins in this order (issue #846):

| Priority | Source | Examples |
|----------|--------|----------|
| 1 (highest) | CLI flags | `--db-path`, `--storage-mode` |
| 2 | Environment variables | `MEMORY_DB_PATH`, `MEMORY_STORAGE_MODE` |
| 3 | Explicit config file | `--config /path/to.toml` |
| 4 | Auto-discovered config in CWD | `do-memory-cli.toml`, `memory-cli.toml` (and `.memory-cli.toml` / JSON / YAML variants) |
| 5 (lowest) | Built-in defaults | XDG paths, local storage defaults |

`--db-path` / `MEMORY_DB_PATH` and `--storage-mode` / `MEMORY_STORAGE_MODE` are clap options with env fallbacks: a CLI flag always beats the env var for that option, and both beat values loaded from a config file. See [memory-cli/CONFIGURATION_GUIDE.md](memory-cli/CONFIGURATION_GUIDE.md) for the full reference.

### Environment Variables

```bash
# Turso Cloud (default)
TURSO_URL=libsql://your-db.turso.io
TURSO_TOKEN=your-auth-token

# Local redb cache (default local backend)
LOCAL_DATABASE_URL=sqlite:./data/memory.db
REDB_PATH=./data/memory.redb

# CLI path / mode overrides (precedence levels 1–2 above)
MEMORY_DB_PATH=./data/memory.db
MEMORY_STORAGE_MODE=local

# CLI config
MEMORY_CLI_CONFIG=./memory-cli.toml

# Embeddings (CLI/MCP)
EMBEDDING_PROVIDER=openai|mistral|azure|local
OPENAI_API_KEY=sk-your-key
MISTRAL_API_KEY=your-mistral-key
AZURE_OPENAI_API_KEY=your-azure-key
OPENAI_API_KEY_ENV=OPENAI_API_KEY
EMBEDDING_MODEL=text-embedding-3-small
EMBEDDING_SIMILARITY_THRESHOLD=0.7
EMBEDDING_BATCH_SIZE=32

# MCP OAuth (production HMAC verification)
# MCP_OAUTH_TOKEN_SECRET=...
```

### TOML Configuration

```toml
[database]
turso_url = "libsql://your-db.turso.io"
turso_token = "your-auth-token"
redb_path = "memory.redb"          # local (redb) cache file path
storage_mode = "local"             # "remote" | "local" | "memory"
db_path = "./data/memory.db"       # Turso local SQLite path (storage_mode = "local")

[storage]
max_episodes_cache = 1000
cache_ttl_seconds = 3600
pool_size = 10

[cli]
default_format = "human"
progress_bars = true
batch_size = 100
```

> Tip: generate a valid starter config with `do-memory-cli config init`
> (writes `do-memory-cli.toml`) or print one with `do-memory-cli config show-template`.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                 do-memory-cli                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐          │
│  │   Episode   │  │  Pattern    │  │   Storage   │          │
│  │ Management  │  │  Analysis   │  │ Operations  │          │
│  └─────────────┘  └─────────────┘  └─────────────┘          │
└─────────────────────────────────────────────────────────────┘
                               │
┌─────────────────────────────────────────────────────────────┐
│                 do-memory-mcp                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐          │
│  │  MCP Tools  │  │  Lazy Tool  │  │  Advanced   │          │
│  │  Interface  │  │  Loading    │  │  Analysis   │          │
│  └─────────────┘  └─────────────┘  └─────────────┘          │
└─────────────────────────────────────────────────────────────┘
                               │
┌─────────────────────────────────────────────────────────────┐
│                 do-memory-core                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐          │
│  │   Episode   │  │   Pattern   │  │   Reward    │          │
│  │ Management  │  │ Extraction  │  │   Scoring   │          │
│  └─────────────┘  └─────────────┘  └─────────────┘          │
└─────────────────────────────────────────────────────────────┘
                               │
            ┌──────────────────┴──────┬────────────────────────┐
            │                         │                        │
┌───────────▼───────────┐ ┌───────────▼───────────┐ ┌───────────▼───────────┐
│do-memory-storage-turso│ │do-memory-storage-redb │ │do-memory-storage-redb │
│                       │ │                       │ │                       │
│     libSQL/Remote     │ │      Fast Access      │ │  In-Memory/Temporary  │
└───────────────────────┘ └───────────────────────┘ └───────────────────────┘
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
- **Unavailable / fail-closed** — `execute_agent_code` (no working WASM execution backend)

## Performance

All operations meet or exceed performance targets:

| Operation | Target (P95) | Typical Performance |
|-----------|-------------|---------------------|
| Episode Creation | < 50ms | ~2.5 µs (19,531x faster) |
| Step Logging | < 20ms | ~1.1 µs (17,699x faster) |
| Episode Completion | < 500ms | ~3.8 µs (130,890x faster) |
| Pattern Extraction | < 1000ms | ~10.4 µs (95,880x faster) |
| Memory Retrieval | < 100ms | ~721 µs (138x faster) |

Typical performance numbers are from internal benchmarks on a warm cache; results vary by hardware and configuration. Run `cargo bench` for local measurements and see `docs/QUALITY_GATES.md` for the performance regression gate.

## Benchmarks

Run `cargo bench` for workspace benchmarks. CLI benchmarks live in `memory-cli/benches/cli_benchmarks.rs`; quality gate expectations and regression checks are documented in `docs/QUALITY_GATES.md`.

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
- Test coverage targets per ADR-042 phases (70% → 75% → 80%)
- Run `cargo fmt` and `cargo clippy` before committing
- Document public APIs
- Write descriptive commit messages

### Quality Requirements

- All tests must pass
- No clippy warnings
- Test coverage targets per ADR-042:
  - Phase 1: 70% (current focus - actual: 61.22%)
  - Phase 2: 75%
  - Phase 3: 80% (codecov.yml project target)
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

---

<div align="center">

[Documentation](docs/) • [GitHub](https://github.com/d-o-hub/rust-self-learning-memory)

</div>
