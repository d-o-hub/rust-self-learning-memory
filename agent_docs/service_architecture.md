# Service Architecture

## System Overview

The memory management system provides persistent memory across agent interactions through an MCP (Model Context Protocol) server.

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Client App    │    │   Memory MCP     │    │  Storage Layer  │
│                 │◄──►│     Server       │◄──►│                 │
│ - Claude Code   │    │                  │    │ - Turso (SQL)   │
│ - OpenCode      │    │ - Memory Core    │    │ - redb Cache    │
│ - Other MCP     │    │ - Episode Mgmt   │    │ - Postcard Ser  │
│   Clients       │    │ - Pattern Extract│    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## Current Status

- **7 workspace members**: do-memory-core, do-memory-storage-turso, do-memory-storage-redb, do-memory-mcp, do-memory-cli, do-memory-test-utils, benches
- **Test coverage**: exceeds 90% target
- **File size compliance**: 100% (≤500 LOC per file)

## Workspace Members

## Hybrid Storage Model
The system employs a hybrid storage architecture: `do-memory-storage-turso` provides persistent, durable, and distributed storage via libSQL, while `do-memory-storage-redb` acts as a high-performance, embedded, local cache layer to dramatically reduce read latency and API calls.

### 1. Memory Core (`memory-core/`)
**Purpose**: Core memory operations and embeddings (~44,250 LOC)

**Module Breakdown** (by size):
- `memory/` - Core memory operations (~6,300 LOC)
  - `retrieval/` - Context retrieval and search
  - `completion.rs` - Episode completion logic
  - `learning_ops.rs` - Learning operations
  - `playbook/` - Playbook generation and recommendation (ADR-044)
  - `attribution/` - Recommendation tracking and feedback (ADR-044)
  - `checkpoint/` - Episode checkpoints and handoff (ADR-044)
- `patterns/` - Pattern extraction and validation (~4,230 LOC)
  - `validation/` - Split validation module (mod.rs + metrics.rs)
  - `optimized_validator/` - Risk and compatibility (mod.rs, context.rs, applicator.rs)
  - `dbscan/` - DBSCAN clustering (detector.rs)
  - Clustering algorithms and quality scoring
- `embeddings/` - Vector embeddings for semantic search (~3,600 LOC)
  - `openai/` - OpenAI provider (split: client.rs)
  - `circuit_breaker.rs` - Resilience for API calls
  - Multi-provider support (OpenAI, Cohere, Ollama, local)
- `spatiotemporal/` - Spatial-temporal indexing (~2,400 LOC)
  - `retriever/` - Temporal retrieval (split: mod.rs, types.rs, scoring.rs, tests.rs)
  - `diversity/` - Diversity ranking
  - `embeddings/` - Spatiotemporal embeddings
  - `index/` - Spatial indexing
- `retrieval/` - Episode retrieval
  - `cache/` - LRU cache (split: mod.rs, lru.rs, types.rs, tests.rs)
- `semantic/` - Semantic processing
  - `summary/` - Summarization (summarizer.rs, types.rs, extractors.rs, helpers.rs)
- `reflection/` - Reflection generation (~1,950 LOC)
  - Episode and pattern reflection
  - Learning summaries
- `pre_storage/` - Pre-storage validation (~1,618 LOC)
  - Quality assessment and validation pipeline
- `monitoring/` - System monitoring (~1,358 LOC)
  - Metrics collection, health checks, performance tracking
- `types/` - Core data structures (split: config.rs, constants.rs, enums.rs, structs.rs, tests.rs)

**Key Files**:
- `src/lib.rs` - Main library entry point
- `src/types.rs` - Core data structures re-export
- `src/episode.rs` - Episode lifecycle
- `src/storage/` - Storage abstraction layer

### 2. Turso Storage (`memory-storage-turso/`)
**Purpose**: Primary persistent database storage (libSQL)

**Features**:
- SQLite-based with libSQL for cloud-native database
- Episode, step, pattern, and embedding storage
- Parameterized queries (SQL injection prevention)
- Concurrent write handling with connection pooling
- Synchronization with redb cache

**Key Files**:
- `src/lib.rs` - Main storage interface
- `src/client.rs` - Turso client with connection pooling
- `tests/integration_test.rs` - Schema definition

**Connection Pooling**:
- Semaphore-based, default 10 concurrent connections
- Configurable via environment variables
- Automatic connection reuse

### 3. Redb Cache (`memory-storage-redb/`)
**Purpose**: High-performance cache layer

**Features**:
- Embedded key-value store
- Postcard serialization (v0.1.7+)
- LRU cache with TTL
- Fast sub-millisecond lookups
- Automatic synchronization with Turso

**Key Files**:
- `src/lib.rs` - Cache interface
- `src/cache.rs` - Cache implementation
- `tests/` - Performance benchmarks

**Serialization**: Uses Postcard (NOT bincode) for safety and performance

### 4. MCP Server (do-memory-mcp/)
**Purpose**: Model Context Protocol server for episodic memory, pattern analysis, and monitoring over JSON-RPC on stdio

**Architecture**:
- JSON-RPC protocol implementation over stdio
- Progressive tool disclosure via a shared tool registry
- OAuth 2.1 authentication (opt-in `oauth` feature), rate limiting, audit logging
- Advanced pattern analysis tools
- **Agent code execution is fail-closed**: `execute_agent_code` is unavailable and direct calls are rejected

**Components**:
- `src/server/` - Main server implementation and tool registry
- `src/mcp/tools/` - MCP tool implementations
- `src/patterns/` - Advanced pattern analysis
- `src/monitoring/` - Health and metrics
- `src/sandbox/` - Legacy Node executor, compiled only with the non-default `sandbox-dev` feature (trusted local experimentation only; not a production sandbox)

**MCP Tools**:
1. `query_memory` - Retrieve episodes and patterns
2. `create_episode` - Create new episodes
3. `add_episode_step` - Log execution steps
4. `complete_episode` - Complete and score episodes
5. `recommend_playbook` - Get actionable playbooks for tasks
6. `health_check` - System health status
7. `record_recommendation_feedback` - Track recommendation outcomes
8. `checkpoint_episode` - Create mid-task checkpoints
9. `get_handoff_pack` - Generate handoff for multi-agent workflows
10. `execute_agent_code` - **Unavailable / fail-closed** (no working execution backend)

**Features**:
- Tool usage tracking
- Performance metrics
- Error handling and recovery
- Security boundaries enforcement

### 5. CLI Interface (`memory-cli/`)
**Purpose**: Command-line interface for memory operations (~13,690 LOC)

**Commands** (9 main commands + 9 aliases):
- `episode` - Episode management (create, list, search, complete, checkpoint)
- `pattern` - Pattern analysis and tracking
- `playbook` - Playbook recommendation (ADR-044)
- `feedback` - Record recommendation feedback (ADR-044)
- `storage` - Storage operations (sync, vacuum, health)
- `eval` - Evaluation and calibration (retrieval quality, domain calibration, benchmarks)
- `health` - System health checks
- `monitor` - Metrics and monitoring
- `logs` - View and filter logs
- `config` - Configuration management
- `backup` - Backup and restore

**Components**:
- `src/commands/` - Command implementations
- `src/config/` - Configuration management
- `src/main.rs` - CLI entry point

**Output Formats**:
- Human-readable (default)
- JSON
- YAML

### 6. Test Utils (`do-memory-test-utils/`)
**Purpose**: Shared testing utilities

**Features**:
- Test episode creation
- Test storage setup
- Common test helpers
- Mock data generators

### 7. Benchmarks (`benches/`)
**Purpose**: Performance benchmarking suite

**Benchmarks**:
- `episode_lifecycle` - Episode creation, completion, pattern extraction
- `phase3_retrieval_accuracy` - Retrieval accuracy and precision
- `spatiotemporal_benchmark` - Spatial-temporal indexing performance
- `storage_operations` - Storage backend performance
- `multi_backend_comparison` - Turso vs redb comparison

**Results** (10-100x improvements over baseline):
- Episode creation: ~2.5 µs (19,531x faster)
- Step logging: ~1.1 µs (17,699x faster)
- Episode completion: ~3.8 µs (130,890x faster)
- Pattern extraction: ~10.4 µs (95,880x faster)
- Memory retrieval: ~721 µs (138x faster)

### 8. Examples (`examples/`)
**Purpose**: Usage examples and demonstrations

**Examples**:
- `memory_mcp_integration.rs` - MCP server integration
- Local database usage
- Embedding provider setup
- Pattern extraction workflows

## Data Flow

### Episode Lifecycle
1. **Creation**: `episode::Episode::new()`
2. **Step Logging**: `episode.add_step()`
3. **Completion**: `episode.complete()` with reward scoring
4. **Reflection**: Automatic reflection generation
5. **Storage**: Concurrent write to Turso + redb cache
6. **Pattern Extraction**: Async background processing via queue

### Memory Retrieval
1. **Query Input**: Natural language or structured query
2. **Semantic Search**: Vector similarity in embeddings
3. **Cache Check**: Fast lookup in redb (postcard deserialization)
4. **Database Query**: Fallback to Turso if cache miss
5. **Result Filtering**: Apply relevance and time filters
6. **Spatiotemporal Ranking**: Diversity-aware ranking

### Pattern Extraction
1. **Queue Processing**: Episodes queued for pattern extraction
2. **Async Workers**: Parallel pattern extraction workers
3. **Validation**: Pattern quality and success rate validation
4. **Storage**: High-success patterns stored in Turso
5. **Cache**: Hot patterns cached in redb

## Configuration

### Environment Variables
```bash
# Database URLs
DATABASE_URL=file:./memory.db
TURSO_DATABASE_URL=libsql://...
REDB_CACHE_PATH=./cache.redb

# MCP Configuration
MCP_HOST=localhost
MCP_PORT=3000

# Embeddings
OPENAI_API_KEY=sk-...
COHERE_API_KEY=...
OLLAMA_BASE_URL=http://localhost:11434
EMBEDDING_MODEL=text-embedding-3-small

# Connection Pooling
TURSO_POOL_SIZE=10
```

### Configuration Files
- `do-memory-cli.toml` - CLI configuration
- `memory-cli/config/` - User-specific settings
- `.env` - Environment variables (NOT in git)
- `rust-toolchain.toml` - Rust version

## Scalability

### Performance Characteristics
- **Episodes**: 10K+ concurrent episodes supported
- **Retrieval**: Sub-100ms P95 latency for 10K episodes (actual: ~721 µs)
- **Storage**: Linear scaling with Turso partitioning
- **Cache**: Sub-ms lookup for hot data (postcard deserialization)
- **Pattern Extraction**: Async queue-based processing
- **Code Execution**: fail-closed (`execute_agent_code` unavailable; no execution backend)

### Horizontal Scaling
- Multiple MCP server instances
- Database sharding by tenant/domain
- Cache warming strategies
- Load balancing across instances

## Security

### Data Protection
- Postcard serialization (safer than bincode)
- Encrypted storage for sensitive episodes
- API key rotation for embeddings
- Access control through MCP authentication
- Audit logging for all operations

### Privacy
- Episode data isolation by domain
- Configurable data retention policies
- GDPR compliance through data deletion
- Secure communication (TLS)

### Security Boundaries
- JSON-RPC over stdio (no network listener by default)
- OAuth 2.1 authentication (opt-in `oauth` feature; `MCP_OAUTH_*` env vars)
- Per-client token-bucket rate limiting
- Structured, redacted audit logging
- Parameterized SQL for all storage access
- Agent code execution is fail-closed; no production sandbox is provided
  (the in-tree Node executor is non-default `sandbox-dev`, trusted local use only)

## Monitoring

### Health Checks
- `do-memory-mcp_health_check` - System status
- `do-memory-mcp_get_metrics` - Performance metrics
- Database connectivity monitoring
- Cache hit rate tracking

### Metrics Tracked
- Episode creation/completion rates
- Pattern extraction performance
- Cache hit/miss ratios
- Query latency (P50, P95, P99)
- Storage operation times

### Logging
- Structured logging with `tracing`
- Episode lifecycle events
- Performance metrics
- Error tracking and alerting

## Deployment

### Development
```bash
# Local development setup
./scripts/setup-local-db.sh
cargo run --bin do-memory-mcp
```

### Production
- Docker containerization
- Kubernetes deployment
- Database migrations
- Rolling updates

### Configuration Management
- Environment-specific configs
- Secret management (Kubernetes secrets)
- Feature flags for gradual rollouts

## Performance Optimizations

### Storage Optimizations
- Connection pooling (semaphore-based)
- Postcard serialization (faster than bincode)
- Dual storage (Turso + redb)
- Async operations throughout
- Batch operations support

### Retrieval Optimizations
- LRU cache with TTL
- Spatial-temporal indexing
- Diversity ranking
- Parallel query execution
- Cached embeddings

### Pattern Extraction Optimizations
- Queue-based async processing
- Parallel pattern extraction
- Incremental pattern learning
- Quality-based filtering
- Success rate decay