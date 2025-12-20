# Service Architecture

## System Overview

The memory management system provides persistent memory across agent interactions through an MCP (Model Context Protocol) server.

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Client App    │    │   Memory MCP     │    │  Storage Layer  │
│                 │◄──►│     Server       │◄──►│                 │
│ - Claude Code   │    │                  │    │ - Turso (SQL)   │
│ - OpenCode      │    │ - Memory Core    │    │ - redb Cache    │
│ - Other MCP     │    │ - Episode Mgmt   │    │ - Embeddings    │
│   Clients       │    │ - Pattern Extract│    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## Core Components

### Memory Core (`memory-core/`)
**Purpose**: Core memory operations and embeddings
- Episode lifecycle management
- Pattern extraction algorithms
- Semantic embeddings for similarity search
- Memory retrieval and filtering

**Key Files**:
- `src/episode.rs` - Episode data structures and operations
- `src/pattern.rs` - Pattern recognition and storage
- `src/embeddings/` - Vector embeddings for semantic search

### Storage Layer

#### Turso Storage (`memory-storage-turso/`)
**Purpose**: Primary persistent database storage
- SQLite-based with libSQL
- Episode storage and retrieval
- Query optimization
- Concurrent write handling

**Key Files**:
- `src/lib.rs` - Main storage interface
- Database schema in `tests/integration_test.rs`

#### Redb Cache (`memory-storage-redb/`)
**Purpose**: High-performance cache layer
- Embedded key-value store
- Fast episode retrieval
- Cache invalidation strategies

**Key Files**:
- `src/cache.rs` - Cache implementation
- Performance benchmarks in `tests/`

### MCP Server (`memory-mcp/`)
**Purpose**: Model Context Protocol server implementation
- Tool definitions and handlers
- Episode management tools
- Pattern analysis tools
- Health monitoring

**Key Files**:
- `src/server.rs` - Main server implementation
- `src/tools/` - MCP tool implementations
- `examples/memory_mcp_integration.rs` - Usage examples

### CLI Interface (`memory-cli/`)
**Purpose**: Command-line interface for memory operations
- Direct memory operations
- Configuration management
- Monitoring and diagnostics

## Data Flow

### Episode Lifecycle
1. **Creation**: `episode::Episode::new()`
2. **Step Logging**: `episode.add_step()`
3. **Completion**: `episode.complete()`
4. **Storage**: Concurrent write to Turso + cache
5. **Pattern Extraction**: Async background processing

### Memory Retrieval
1. **Query Input**: Natural language or structured query
2. **Semantic Search**: Vector similarity in embeddings
3. **Cache Check**: Fast lookup in redb
4. **Database Query**: Fallback to Turso if needed
5. **Result Filtering**: Apply relevance and time filters

## Configuration

### Environment Variables
```bash
# Database URLs
DATABASE_URL=file:./memory.db
REDIS_URL=redis://localhost:6379

# MCP Configuration
MCP_HOST=localhost
MCP_PORT=3000

# Embeddings
OPENAI_API_KEY=sk-...
EMBEDDING_MODEL=text-embedding-3-small
```

### Configuration Files
- `memory-cli.toml` - CLI configuration
- `memory-cli/config/` - User-specific settings
- `.env` - Environment variables

## Scalability

### Performance Characteristics
- **Episodes**: 10K+ concurrent episodes supported
- **Retrieval**: Sub-100ms P95 latency for 10K episodes
- **Storage**: Linear scaling with Turso partitioning
- **Cache**: Sub-ms lookup for hot data

### Horizontal Scaling
- Multiple MCP server instances
- Database sharding by tenant/domain
- Cache warming strategies
- Load balancing across instances

## Security

### Data Protection
- Encrypted storage for sensitive episodes
- API key rotation for embeddings
- Access control through MCP authentication
- Audit logging for all operations

### Privacy
- Episode data isolation by domain
- Configurable data retention policies
- GDPR compliance through data deletion
- Secure communication (TLS)

## Monitoring

### Health Checks
- `memory-mcp_health_check` - System status
- `memory-mcp_get_metrics` - Performance metrics
- Database connectivity monitoring
- Cache hit rate tracking

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
cargo run --bin memory-mcp
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