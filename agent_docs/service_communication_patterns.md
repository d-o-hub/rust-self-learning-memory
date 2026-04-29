# Service Communication Patterns

## Overview

The memory system uses several communication patterns between components:

1. **MCP (Model Context Protocol)**: Primary client-server communication
2. **Internal Rust Communication**: Inter-component messaging with Tokio
3. **Database Communication**: Storage layer interactions (Turso + redb)
4. **Cache Communication**: Performance optimization layer (Postcard-based)

## MCP Protocol Communication

### Client-Server Architecture
```
Client (Claude Code/OpenCode) ↔ MCP Server ↔ Memory Core ↔ Storage (Turso + redb)
```

### MCP Tools Implemented

| Tool | Purpose | Implementation |
|------|---------|----------------|
| `query_memory` | Retrieve episodic context | `memory_handlers.rs` |
| `create_episode` | Start new episode | `memory_handlers.rs` |
| `log_step` | Record execution step | `memory_handlers.rs` |
| `complete_episode` | Finalize and score episode | `memory_handlers.rs` |
| `extract_patterns` | Pattern analysis | `pattern_ops.rs` |
| `embeddings/*` | Embedding operations | `mcp/tools/embeddings/` |
| `checkpoint` | State checkpointing | `mcp/tools/checkpoint/` |
| `recommendation_feedback` | Feedback collection | `mcp/tools/recommendation_feedback/` |

> For MCP specification details, see https://modelcontextprotocol.io/docs/tools/inspector

### Tool Registration Pattern

```rust
// Example from memory_handlers.rs
pub async fn query_memory(
    query: String,
    domain: Option<String>,
    limit: Option<u32>,
) -> Result<String, McpError> {
    let memory = get_memory_instance().await?;
    let results = memory.retrieve(&query, domain, limit).await?;
    Ok(serde_json::to_string(&results)?)
}
```

## Internal Component Communication

### Memory Core Interface

Primary trait for memory operations:

```rust
// See memory-core/src/memory/mod.rs
#[async_trait]
pub trait MemoryStore: Send + Sync {
    async fn create_episode(&self, task: &str) -> Result<EpisodeId>;
    async fn log_step(&self, episode_id: &EpisodeId, step: Step) -> Result<()>;
    async fn complete_episode(&self, episode_id: &EpisodeId) -> Result<f64>;
    async fn retrieve(&self, query: &str, domain: Option<&str>) -> Result<Vec<Episode>>;
}
```

### Storage Communication

> See [database_schema.md](database_schema.md) for detailed schema.

**Turso (libSQL)**: Primary persistent storage
**Redb**: High-performance cache with Postcard serialization

### Postcard Serialization

> For Postcard usage examples, see [code_conventions.md](code_conventions.md#postcard-serialization-v017).

All cache values use Postcard serialization:
```rust
// Serialize
let bytes = postcard::to_allocvec(&episode)?;

// Deserialize
let episode: Episode = postcard::from_bytes(&bytes)?;
```

## Error Handling Patterns

### Circuit Breaker

Used for storage resilience:

```rust
// See memory-storage-turso/src/circuit_breaker.rs
pub struct CircuitBreaker {
    failure_threshold: u32,
    reset_timeout: Duration,
    state: CircuitState,
}
```

### Error Propagation

```rust
// Public APIs: anyhow::Result
pub async fn operation() -> Result<T> { ... }

// Domain errors: thiserror
#[derive(Error, Debug)]
pub enum StorageError { ... }
```

## Performance Patterns

### Connection Pooling

```rust
// Semaphore-based connection limiting
pub struct TursoPool {
    semaphore: Arc<Semaphore>,  // Default: 10 permits
}
```

### Batch Operations

Episodes and patterns support batch operations for efficiency:

```rust
// Batch insert patterns
async fn store_patterns_batch(&self, patterns: &[Pattern]) -> Result<()>;
```

## Security Communication

### Sandbox Isolation

MCP server uses Wasmtime sandbox for untrusted code:

```rust
// See do-memory-mcp/src/wasmtime_sandbox.rs
pub struct WasmtimeSandbox {
    engine: Engine,
    module: Module,
}
```

### Parameterized SQL

All database queries use parameterized statements:

```rust
// Always use parameterized queries
db.query("SELECT * FROM episodes WHERE id = ?1", &[id]).await?;
```

## Cross-References

| Topic | Document |
|-------|----------|
| Database schema | [database_schema.md](database_schema.md) |
| Code conventions | [code_conventions.md](code_conventions.md) |
| Architecture | [service_architecture.md](service_architecture.md) |
| Testing | [running_tests.md](running_tests.md) |