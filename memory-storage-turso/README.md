# memory-storage-turso

[![Crates.io](https://img.shields.io/crates/v/memory-storage-turso.svg)](https://crates.io/crates/memory-storage-turso)
[![Documentation](https://docs.rs/memory-storage-turso/badge.svg)](https://docs.rs/memory-storage-turso)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Purpose**: Primary durable storage using Turso/libSQL

## Overview

`memory-storage-turso` implements the durable storage layer for AI agent episodic memory using [Turso](https://turso.tech) and libSQL. It provides production-ready distributed SQL storage with connection pooling, circuit breaking, and resilient error handling.

## Features

- **Distributed Storage**: Deploy to Turso's edge network for global low-latency access
- **Local Development**: Use local libSQL files for offline development
- **Production Ready**: Connection pooling, circuit breakers, and resilient error handling
- **Secure**: Parameterized queries, TLS enforcement, credential validation
- **Analytics**: Complex SQL queries for pattern analysis and insights
- **Conflict Resolution**: Turso as source of truth with automatic reconciliation
- **Vector Indexing**: Native DiskANN vector index for semantic search

## Key Modules

| Module | Purpose |
|--------|---------|
| `schema` | Database schema initialization and migrations |
| `storage` | Core storage operations and CRUD functionality |
| `pool` | Semaphore-based connection pooling |
| `resilient` | Circuit breaker and retry logic |

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
memory-storage-turso = "0.1"
```

## Quick Start

### With Turso (Production)

```rust
use memory_storage_turso::TursoStorage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Set environment variables
    // TURSO_DATABASE_URL=libsql://your-database.turso.io
    // TURSO_AUTH_TOKEN=your-auth-token

    let storage = TursoStorage::new(
        std::env::var("TURSO_DATABASE_URL")?,
        std::env::var("TURSO_AUTH_TOKEN")?,
    ).await?;

    // Storage is ready to use
    Ok(())
}
```

### With Local libSQL (Development)

```rust
use memory_storage_turso::TursoStorage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let storage = TursoStorage::new_local("./data/memory.db").await?;

    // Use for development and testing
    Ok(())
}
```

## Configuration

### Environment Variables

```bash
# Production (Turso)
export TURSO_DATABASE_URL="libsql://your-database.turso.io"
export TURSO_AUTH_TOKEN="your-auth-token-here"

# Development (Local libSQL)
export LIBSQL_DATABASE_PATH="./data/memory.db"
```

### Getting Turso Credentials

1. Sign up at [turso.tech](https://turso.tech)
2. Install Turso CLI: `brew install tursodatabase/tap/turso` (macOS)
3. Create a database: `turso db create my-memory-db`
4. Get the URL: `turso db show my-memory-db --url`
5. Create a token: `turso db tokens create my-memory-db`

## Database Schema

The storage layer maintains five main tables:

### Tables

- **episodes**: Stores complete episode data with JSON fields for steps, context, and artifacts
- **steps**: Individual execution steps with timestamps, tool usage, and results
- **patterns**: Extracted patterns with metadata for retrieval
- **embeddings**: Vector embeddings for semantic search (if enabled)
- **spatiotemporal_index**: Spatiotemporal metadata for location/time-aware queries

### Indexes

- **Native DiskANN Vector Index**: Built-in vector similarity search for embeddings
- **Secondary Indexes**: Optimized queries for episode retrieval, pattern matching, and temporal queries
- **Spatial Indexes**: Geospatial queries for spatiotemporal features

Schema is automatically initialized on first connection.

## Performance

Connection pooling and query optimization ensure excellent performance:

- **Concurrent operations**: 100+ ops/second
- **Connection pool**: Configurable size (default: 10)
- **Circuit breaker**: Automatic failure detection with exponential backoff
- **Query caching**: Prepared statements for frequently used queries
- **Vector similarity**: Sub-millisecond search with DiskANN index

## Resilience Features

### Connection Pooling
- **Semaphore-based pool management** for efficient resource usage
- **Default pool size**: 10 connections
- **Configurable limits**: Adjust based on workload
- **Automatic cleanup**: Close idle connections

### Circuit Breaker
- **Automatic failure detection** with configurable thresholds
- **Exponential backoff** for retry attempts
- **Half-open state**: Probes for recovery before full restoration
- **Timeout handling**: Prevents cascade failures

### Retry Logic
- **Configurable retry strategies** for transient failures
- **Max retry attempts**: Prevent infinite loops
- **Jitter**: Adds randomness to avoid thundering herd

### Health Checks
- **Endpoint verification**: Test database connectivity
- **Pool statistics**: Monitor connection health
- **Metrics tracking**: Query performance and error rates

## Security

### SQL Injection Prevention
- **Parameterized queries**: All SQL queries use parameter binding
- **No string concatenation**: Never build SQL queries with user input
- **Input validation**: Sanitize all data before storage

### TLS Enforcement
- **Encrypted connections**: TLS required for Turso connections
- **Certificate validation**: Automatic certificate verification
- **No plaintext fallback**: Rejects unencrypted connections

### Credential Validation
- **Environment variable validation**: Check credentials on startup
- **Token expiration handling**: Detect and handle expired tokens
- **Secure storage**: Never log or expose credentials

### Input Sanitization
- **JSON serialization validation**: Validate before storage
- **Size limits**: Prevent oversized data insertion
- **Type checking**: Ensure data integrity

## Performance Characteristics

| Operation | Latency (P95) | Throughput |
|-----------|---------------|------------|
| Episode Insert | < 10ms | 100+ ops/s |
| Episode Query | < 5ms | 200+ ops/s |
| Pattern Insert | < 5ms | 200+ ops/s |
| Vector Search | < 50ms | 50+ ops/s |
| Spatiotemporal Query | < 20ms | 100+ ops/s |

## Usage with memory-core

```rust
use memory_core::SelfLearningMemory;
use memory_storage_turso::TursoStorage;
use memory_storage_redb::RedbStorage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let turso = TursoStorage::new(
        std::env::var("TURSO_DATABASE_URL")?,
        std::env::var("TURSO_AUTH_TOKEN")?,
    ).await?;

    let redb = RedbStorage::new("./data/cache.redb")?;

    let memory = SelfLearningMemory::with_storage(turso, redb).await?;

    // Use memory system
    Ok(())
}
```

## Advanced Configuration

### Connection Pool Configuration

```rust
use memory_storage_turso::{TursoStorage, PoolConfig};

let pool_config = PoolConfig {
    max_connections: 20,      // Max concurrent connections
    min_idle: 5,             // Min idle connections
    connect_timeout: 30,     // Seconds
    idle_timeout: 600,       // Seconds
};

let storage = TursoStorage::with_config(
    db_url,
    auth_token,
    pool_config,
).await?;
```

### Circuit Breaker Configuration

```rust
use memory_storage_turso::{CircuitBreakerConfig, RetryPolicy};

let breaker_config = CircuitBreakerConfig {
    failure_threshold: 5,        // Failures before opening
    success_threshold: 2,       // Successes to close
    timeout: 60,                // Seconds to wait before retry
    half_open_max_calls: 3,     // Max calls in half-open state
};

let retry_policy = RetryPolicy {
    max_attempts: 3,
    initial_backoff_ms: 100,
    max_backoff_ms: 5000,
    multiplier: 2.0,
};
```

## Monitoring

### Pool Statistics

```rust
let stats = storage.get_pool_stats()?;
println!("Active connections: {}", stats.active);
println!("Idle connections: {}", stats.idle);
println!("Total acquired: {}", stats.total_acquired);
```

### Circuit Breaker Status

```rust
let status = storage.get_circuit_breaker_status()?;
println!("State: {:?}", status.state);
println!("Failure count: {}", status.failure_count);
println!("Last failure: {:?}", status.last_failure);
```

## Testing

Run integration tests with a local libSQL database:

```bash
cargo test -p memory-storage-turso
```

For production testing against Turso, set environment variables first:

```bash
export TURSO_DATABASE_URL="libsql://your-test-db.turso.io"
export TURSO_AUTH_TOKEN="your-test-token"
cargo test -p memory-storage-turso
```

## Dependencies

### Core Dependencies
- **libsql**: Turso/libSQL client library
- **tokio**: Async runtime
- **async-trait**: Async trait support
- **anyhow**: Error handling
- **serde**: Serialization framework
- **serde_json**: JSON serialization for stored data
- **uuid**: Unique identifiers
- **chrono**: Date/time handling

## Documentation

Full API documentation: [docs.rs/memory-storage-turso](https://docs.rs/memory-storage-turso)

### Additional Documentation
- [LOCAL_DATABASE_SETUP.md](../docs/LOCAL_DATABASE_SETUP.md) - Local database configuration
- [database_schema.md](../agent_docs/database_schema.md) - Detailed schema documentation

## License

Licensed under the MIT License. See [LICENSE](../LICENSE) for details.

## Project

Part of the [rust-self-learning-memory](https://github.com/d-o-hub/rust-self-learning-memory) project.

## Best Practices

### Production Deployment
- Use Turso for distributed, globally accessible storage
- Enable TLS encryption (default)
- Configure connection pool based on expected load
- Monitor circuit breaker status and adjust thresholds
- Set up alerts for high failure rates

### Development
- Use local libSQL for offline development
- Lower connection pool size to conserve resources
- Disable circuit breaker for easier debugging
- Use separate database for testing

### Security
- Never commit database credentials
- Rotate auth tokens regularly
- Use environment variables for secrets
- Enable query logging for audit trails
- Validate all user inputs

## Migration Guide

See [database_schema.md](../agent_docs/database_schema.md) for schema changes and migration instructions between versions.
